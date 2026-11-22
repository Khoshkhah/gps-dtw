//! GPS-DTW in plain Rust: the GPS cleaning, the Bezier interpolation at 1 s, the road network and the DTW.
//!
//! The cleaning and the interpolation come from NRTMapMatching (`sources/` there); GPS-DTW keeps its own copy.

use rstar::primitives::{GeomWithData, Rectangle};
use rstar::{RTree, AABB};

pub type P = [f64; 2];

/// `geotools.distance2d`: `(dx ** 2 + dy ** 2) ** 0.5`. `powf(0.5)` calls the same C `pow` as Python.
#[inline]
pub fn dist(a: P, b: P) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).powf(0.5)
}

/// `geotools.calculate_bearing_angle`.
#[inline]
pub fn bearing(a: P, b: P) -> f64 {
    let d = (b[0] - a[0]).atan2(b[1] - a[1]).to_degrees();
    if d < 0.0 {
        d + 360.0
    } else {
        d
    }
}

/// `sumolib.geomhelper.distancePointToPolygon` (not perpendicular): the filter of `Net.getNeighboringEdges`.
fn sumo_dist_to_poly(p: P, s: &[P]) -> f64 {
    let sd = |a: P, b: P| ((a[0] - b[0]) * (a[0] - b[0]) + (a[1] - b[1]) * (a[1] - b[1])).sqrt();
    let mut min = f64::INFINITY;
    for w in s.windows(2) {
        let (p1, p2) = (w[0], w[1]);
        let d = sd(p1, p2);
        let u = (p[0] - p1[0]) * (p2[0] - p1[0]) + (p[1] - p1[1]) * (p2[1] - p1[1]);
        let off = if d == 0.0 || u < 0.0 || u > d * d {
            if u < 0.0 {
                0.0
            } else {
                d
            }
        } else {
            u / d
        };
        let dd = if off == 0.0 {
            sd(p, p1)
        } else {
            let f = off / sd(p1, p2);
            sd(p, [p1[0] + f * (p2[0] - p1[0]), p1[1] + f * (p2[1] - p1[1])])
        };
        if dd < min {
            min = dd;
        }
    }
    min
}

// ---------------------------------------------------------------------------------------------
// GPS cleaning, enrichment and stop detection (cleandata.py)
// ---------------------------------------------------------------------------------------------

/// The rows of a cleaned trajectory (columns x, y, timestamp, speed, bearing, stopindex).
#[derive(Default, Debug, Clone)]
pub struct Rows {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub ts: Vec<i64>,
    pub speed: Vec<f64>,
    pub bearing: Vec<f64>,
    pub stop: Vec<i64>,
}

impl Rows {
    fn push(&mut self, p: P, ts: i64, speed: f64, bearing: f64, stop: i64) {
        self.x.push(p[0]);
        self.y.push(p[1]);
        self.ts.push(ts);
        self.speed.push(speed);
        self.bearing.push(bearing);
        self.stop.push(stop);
    }

    /// `xystop_point_editing`: the median x and y of each stop for all points of that stop.
    fn stop_medians(&mut self) {
        let median = |mut v: Vec<f64>| {
            v.sort_by(|a, b| a.total_cmp(b));
            let n = v.len();
            if n % 2 == 1 {
                v[n / 2]
            } else {
                (v[n / 2 - 1] + v[n / 2]) / 2.0
            }
        };
        let mut i = 0;
        while i < self.stop.len() {
            let s = self.stop[i];
            if s <= 0 {
                i += 1;
                continue;
            }
            // the points of one stop index are consecutive
            let idx: Vec<usize> = (i..self.stop.len()).filter(|&k| self.stop[k] == s).collect();
            let mx = median(idx.iter().map(|&k| self.x[k]).collect());
            let my = median(idx.iter().map(|&k| self.y[k]).collect());
            for &k in &idx {
                self.x[k] = mx;
                self.y[k] = my;
            }
            i = idx.last().unwrap() + 1;
        }
    }
}

/// The stop index of a new row: 0 when it moves, else the number of the stop.
fn stop_index(speed: f64, out: &Rows, stops: &mut i64) -> i64 {
    if speed > 0.0 {
        0
    } else {
        if out.stop.last() == Some(&0) {
            *stops += 1;
        }
        *stops
    }
}

/// `removeOutlier` with its default limit (richdata calls it so).
fn remove_outlier(x: &[f64], y: &[f64], ts: &[i64], max_speed: f64) -> (Vec<P>, Vec<i64>) {
    let mut p = vec![[x[0], y[0]]];
    let mut t = vec![ts[0]];
    for i in 1..x.len() {
        let pc = [x[i], y[i]];
        let s = dist(pc, *p.last().unwrap()) / (ts[i] - *t.last().unwrap()) as f64;
        if s < max_speed {
            p.push(pc);
            t.push(ts[i]);
        }
    }
    (p, t)
}

/// `pandas.Series.ewm(alpha=alpha).mean()` (adjust=True, ignore_na=False), the pandas loop.
fn ewm_mean(v: &[f64], alpha: f64) -> Vec<f64> {
    let f = 1.0 - alpha;
    let mut out = Vec::with_capacity(v.len());
    let mut w = v[0];
    let mut old = 1.0;
    out.push(w);
    for &cur in &v[1..] {
        let obs = !cur.is_nan();
        if !w.is_nan() {
            old *= f;
            if obs {
                if w != cur {
                    w = old * w + 1.0 * cur;
                    w /= old + 1.0;
                }
                old += 1.0;
            }
        } else if obs {
            w = cur;
        }
        out.push(w);
    }
    out
}

/// `smoothingPoint(type="local", max_gap)`: a step with a gap over `max_gap` is a group of its own
/// (no smoothing), and the smoothing starts again after it. `None` = smooth all steps.
fn smooth(p: &[P], ts: &[i64], alpha: f64, max_gap: Option<f64>) -> Vec<P> {
    let n = p.len();
    let mut dt = vec![0f64; n];
    let mut vx = vec![f64::NAN; n];
    let mut vy = vec![f64::NAN; n];
    for i in 0..n {
        if i + 1 < n {
            dt[i] = (ts[i + 1] - ts[i]) as f64;
            vx[i] = (p[i + 1][0] - p[i][0]) / dt[i];
            vy[i] = (p[i + 1][1] - p[i][1]) / dt[i];
        } else {
            dt[i] = 1.0;
        }
    }
    let (mut vx2, mut vy2) = (Vec::with_capacity(n), Vec::with_capacity(n));
    let big = |i: usize| max_gap.is_some_and(|g| dt[i] > g);
    let mut start = 0;
    for i in 1..=n {
        if i == n || big(i) || big(i - 1) {
            vx2.extend(ewm_mean(&vx[start..i], alpha));
            vy2.extend(ewm_mean(&vy[start..i], alpha));
            start = i;
        }
    }
    let mut out = vec![p[0]];
    for i in 0..n - 1 {
        out.push([p[i][0] + dt[i] * vx2[i], p[i][1] + dt[i] * vy2[i]]);
    }
    out
}

/// `richdata` after the projection: outliers, smoothing, speed and bearing, stops.
#[allow(clippy::too_many_arguments)]
pub fn enrich(
    x: &[f64],
    y: &[f64],
    ts: &[i64],
    alpha: f64,
    min_speed: f64,
    max_speed_for_outlier: f64,
    min_speed_for_bearing: f64,
    smooth_max_gap: Option<f64>,
) -> Result<Rows, String> {
    let (p1, t1) = remove_outlier(x, y, ts, 50.0);
    let p = smooth(&p1, &t1, alpha, smooth_max_gap);
    let mut out = Rows::default();
    let mut pc = p[0];
    let mut tc = t1[0];
    let mut stops = 1i64;
    let mut speed_current = f64::NAN;
    for i in 1..p.len() {
        let pn = p[i];
        let tn = t1[i];
        speed_current = dist(pc, pn) / (tn - tc) as f64;
        if speed_current < min_speed {
            speed_current = 0.0;
        }
        let mut b = bearing(pc, pn);
        if speed_current < max_speed_for_outlier {
            if speed_current < min_speed_for_bearing && !out.x.is_empty() {
                b = *out.bearing.last().unwrap();
            }
            let s = stop_index(speed_current, &out, &mut stops);
            out.push(pc, tc, speed_current, b, s);
            pc = pn;
            tc = tn;
        }
    }
    if out.x.is_empty() {
        return Err("list index out of range".into());
    }
    let sp = *out.speed.last().unwrap();
    let b = *out.bearing.last().unwrap();
    let s = stop_index(speed_current, &out, &mut stops);
    out.push(pc, tc, sp, b, s);
    out.stop_medians();
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Cubic Bezier interpolation (interpolation.py)
// ---------------------------------------------------------------------------------------------

/// The rows of an interpolated trajectory. `speed` is NaN where Python has `None`.
#[derive(Default, Debug, Clone)]
pub struct Interp {
    /// The index label of the row after `concat(...).reset_index(drop=True).dropna()`.
    pub label: Vec<i64>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub ts: Vec<i64>,
    pub speed: Vec<f64>,
    pub bearing: Vec<f64>,
    pub origin: Vec<bool>,
    pub stop: Vec<i64>,
}

impl Interp {
    fn push(&mut self, p: P, ts: i64, speed: f64, bearing: f64, origin: bool, stop: i64) {
        self.x.push(p[0]);
        self.y.push(p[1]);
        self.ts.push(ts);
        self.speed.push(speed);
        self.bearing.push(bearing);
        self.origin.push(origin);
        self.stop.push(stop);
    }
}

#[derive(Clone, Copy)]
struct Knot {
    p: P,
    ts: f64,
    speed: f64,
    bearing: f64,
    stop: i64,
}

fn bezier3(p0: P, p1: P, p2: P, p3: P, t: f64) -> P {
    let b0 = (1.0 - t).powf(3.0);
    let b1 = 3.0 * (1.0 - t).powf(2.0) * t;
    let b2 = 3.0 * (1.0 - t) * t.powf(2.0);
    let b3 = t.powf(3.0);
    [
        b0 * p0[0] + b1 * p1[0] + b2 * p2[0] + b3 * p3[0],
        b0 * p0[1] + b1 * p1[1] + b2 * p2[1] + b3 * p3[1],
    ]
}

fn bezier2(p0: P, p1: P, p2: P, t: f64) -> P {
    let b0 = (1.0 - t).powf(2.0);
    let b1 = 2.0 * (1.0 - t) * t;
    let b2 = t.powf(2.0);
    [
        b0 * p0[0] + b1 * p1[0] + b2 * p2[0],
        b0 * p0[1] + b1 * p1[1] + b2 * p2[1],
    ]
}

/// `bezierInterpolation(a, b, sample_rate)`, appended to `out`.
fn bezier_segment(a: Knot, b: Knot, sr: f64, out: &mut Interp) {
    let dt = b.ts - a.ts;
    if a.stop > 0 && b.stop > 0 {
        let mut time = a.ts;
        while time <= b.ts {
            out.push(a.p, time as i64, 0.0, a.bearing, time == a.ts || time == b.ts, a.stop);
            time += sr;
            if time > b.ts && time - sr != b.ts {
                time = b.ts;
            }
        }
        return;
    }
    // cubic Hermite curve as a Bezier: control points at gap time x velocity / 3 (see interpolation.py)
    let ra = a.bearing * std::f64::consts::PI / 180.0;
    let rb = b.bearing * std::f64::consts::PI / 180.0;
    let va = [a.speed * ra.sin(), a.speed * ra.cos()];
    let vb = [b.speed * rb.sin(), b.speed * rb.cos()];
    let c0 = [a.p[0] + (dt * va[0]) / 3.0, a.p[1] + (dt * va[1]) / 3.0];
    let c1 = [b.p[0] - (dt * vb[0]) / 3.0, b.p[1] - (dt * vb[1]) / 3.0];
    let mut time = 0.0;
    while time + a.ts <= b.ts {
        let t = time / dt;
        let pt = bezier3(a.p, c0, c1, b.p, t);
        let (origin, speed) = if t == 1.0 && t != 0.0 {
            (true, f64::NAN)
        } else {
            let pn = bezier3(a.p, c0, c1, b.p, t + (sr / dt));
            (t == 0.0, dist(pt, pn) / sr)
        };
        let q0 = [3.0 * (c0[0] - a.p[0]), 3.0 * (c0[1] - a.p[1])];
        let q1 = [3.0 * (c1[0] - c0[0]), 3.0 * (c1[1] - c0[1])];
        let q2 = [3.0 * (b.p[0] - c1[0]), 3.0 * (b.p[1] - c1[1])];
        let v = bezier2(q0, q1, q2, t);
        out.push(pt, (time + a.ts) as i64, speed, bearing([0.0, 0.0], v), origin, 0);
        time += sr;
        if time > dt && time - sr != dt {
            time = dt;
        }
    }
}

/// `interpolateTrajectory(traj, sample_rate)`. The input needs 3 rows or more.
#[allow(clippy::too_many_arguments)]
pub fn interpolate(
    x: &[f64],
    y: &[f64],
    ts: &[f64],
    speed: &[f64],
    brg: &[f64],
    stop: &[i64],
    sr: f64,
) -> Interp {
    let k = |i: usize| Knot { p: [x[i], y[i]], ts: ts[i], speed: speed[i], bearing: brg[i], stop: stop[i] };
    let n = x.len();
    let mut all = Interp::default();
    let mut a = k(0);
    // to the last point (the Python code had traj[1:-1], which left out the last segment)
    for i in 1..n {
        let b = k(i);
        let start = all.x.len();
        bezier_segment(a, b, sr, &mut all);
        a = b;
        if a.stop > 0 && all.x.len() > start {
            let l = all.x.len() - 1;
            a.p = [all.x[l], all.y[l]];
        }
    }
    if let Some(l) = all.speed.last_mut() {
        *l = speed[n - 1];
    }
    // df.dropna()
    let mut out = Interp::default();
    for i in 0..all.x.len() {
        if [all.x[i], all.y[i], all.speed[i], all.bearing[i]].iter().any(|v| v.is_nan()) {
            continue;
        }
        out.label.push(i as i64);
        out.push([all.x[i], all.y[i]], all.ts[i], all.speed[i], all.bearing[i], all.origin[i], all.stop[i]);
    }
    out
}
// ---------------------------------------------------------------------------------------------
// The network and the R-tree (network.py, the part that the matcher reads)
// ---------------------------------------------------------------------------------------------

/// The road network: the shape of each edge in metres and the edges that can follow each edge (the turns).
pub struct Network {
    pub shapes: Vec<Vec<P>>,
    pub next: Vec<Vec<usize>>,
    tree: RTree<GeomWithData<Rectangle<P>, usize>>,
}

impl Network {
    /// `turns`: the allowed (from edge, to edge) pairs, for example duckOSM's `edge_graph` (turn restrictions
    /// removed). `None`: each edge that starts at the end node of an edge can follow it.
    pub fn new(shapes: Vec<Vec<P>>, from: Vec<usize>, to: Vec<usize>, turns: Option<Vec<(usize, usize)>>) -> Self {
        let mut next: Vec<Vec<usize>> = vec![Vec::new(); shapes.len()];
        match turns {
            Some(t) => {
                for (a, b) in t {
                    if a != b {
                        next[a].push(b);
                    }
                }
            }
            None => {
                let mut starts: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
                for (e, &n) in from.iter().enumerate() {
                    starts.entry(n).or_default().push(e);
                }
                for (e, n) in to.iter().enumerate() {
                    next[e] = starts.get(n).map(|v| v.iter().copied().filter(|&b| b != e).collect()).unwrap_or_default();
                }
            }
        }
        let boxes = shapes
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut lo = [f64::INFINITY; 2];
                let mut hi = [f64::NEG_INFINITY; 2];
                for p in s {
                    lo = [lo[0].min(p[0]), lo[1].min(p[1])];
                    hi = [hi[0].max(p[0]), hi[1].max(p[1])];
                }
                GeomWithData::new(Rectangle::from_corners(lo, hi), i)
            })
            .collect();
        Network { shapes, next, tree: RTree::bulk_load(boxes) }
    }

    /// `Net.getNeighboringEdges(x, y, r)`: the edges nearer than `r`, sorted by edge index.
    pub fn neighboring_edges(&self, x: f64, y: f64, r: f64) -> Vec<(usize, f64)> {
        let env = AABB::from_corners([x - r, y - r], [x + r, y + r]);
        let mut v: Vec<(usize, f64)> = self
            .tree
            .locate_in_envelope_intersecting(&env)
            .filter_map(|g| {
                let d = sumo_dist_to_poly([x, y], &self.shapes[g.data]);
                (d < r).then_some((g.data, d))
            })
            .collect();
        v.sort_by_key(|e| e.0);
        v
    }

}

// ---------------------------------------------------------------------------------------------
// GPS-DTW (our own): the input = the 1 s points as they are; the road = each edge near the input
// divided into points `speed x 1 s` apart, joined by the node table. DTW with the three moves; the
// cost of a cell = confidence weight x distance between the input point and the road point.
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct GpsDtwParams {
    /// The road point spacing: an approximate speed (m/s) x 1 s.
    pub speed: f64,
    /// The time scale (s) of the confidence weight 1 / (1 + (dt / tau)^2); dt = seconds to the nearest GPS point.
    pub tau: f64,
    /// The road corridor: the edges within max(corridor_min, corridor_gap x the median GPS spacing) of the input.
    pub corridor_min: f64,
    pub corridor_gap: f64,
    /// true (default): segment mode, an input segment (1 s) against a road segment (speed x 1 s);
    /// false: node mode, an input point against a road point.
    pub segment: bool,
    /// Segment mode: the weight of the angle difference (degrees) between the two segments.
    pub bearing_weight: f64,
    /// The confidence weight of a real GPS point (an interpolated point has at most 1).
    pub gps_weight: f64,
    /// The seconds in one window; 0 = one window for the whole track.
    pub window: usize,
    /// The interpolation interval (s). The road points are `speed x step` apart.
    pub step: f64,
    /// The ends that a window keeps for the next window.
    pub top_k: usize,
}

impl Default for GpsDtwParams {
    /// The defaults of the Python `match_trace`.
    fn default() -> Self {
        GpsDtwParams { speed: 12.0, tau: 3.0, corridor_min: 30.0, corridor_gap: 1.0, segment: true, bearing_weight: 0.0, gps_weight: 5.0, window: 60, step: 1.0, top_k: 20 }
    }
}

/// The seconds from each point to the nearest GPS point (`origin`).
fn seconds_to_gps(ts: &[i64], origin: &[bool]) -> Vec<f64> {
    let o: Vec<i64> = ts.iter().zip(origin).filter(|(_, &g)| g).map(|(&t, _)| t).collect();
    ts.iter().map(|&t| o.iter().map(|&g| (t - g).abs()).min().unwrap_or(0) as f64).collect()
}

/// The points each `spacing` metres along a line (from its start) and its end point.
fn points_each(s: &[P], spacing: f64) -> Vec<P> {
    let mut out = Vec::new();
    let mut at = 0.0; // the next point is at this distance from the start
    for w in s.windows(2) {
        let d = dist(w[0], w[1]);
        while at <= d && d > 0.0 {
            let f = at / d;
            out.push([w[0][0] + f * (w[1][0] - w[0][0]), w[0][1] + f * (w[1][1] - w[0][1])]);
            at += spacing;
        }
        at -= d;
    }
    let e = s[s.len() - 1];
    if out.last() != Some(&e) {
        out.push(e);
    }
    out
}

/// The DTW states of one window: the road points (node mode) or the road segments (segment mode) of the
/// edges near the window's input, and the moves between them.
struct States {
    /// (edge, index on the edge): the same road point or segment has the same key in each window.
    key: Vec<(usize, usize)>,
    edge: Vec<usize>,
    /// the road point, or the middle of the road segment
    pos: Vec<P>,
    /// segment mode: the bearing of the road segment
    bear: Vec<f64>,
    pred: Vec<Vec<usize>>,
    succ: Vec<Vec<usize>>,
}

fn build_states(net: &Network, edges: &[usize], spacing: f64, segment: bool) -> States {
    let mut st = States { key: vec![], edge: vec![], pos: vec![], bear: vec![], pred: vec![], succ: vec![] };
    // the first and the last state of each edge (None: the edge has no state)
    let mut ends: Vec<Option<(usize, usize)>> = Vec::with_capacity(edges.len());
    for &e in edges {
        let pts = points_each(&net.shapes[e], spacing);
        let first = st.key.len();
        let mut prev: Option<(usize, usize)> = None; // (state, index on the edge)
        let add = |st: &mut States, t: usize, pos: P, bear: f64, prev: &mut Option<(usize, usize)>| {
            let k = st.key.len();
            st.key.push((e, t));
            st.edge.push(e);
            st.pos.push(pos);
            st.bear.push(bear);
            st.pred.push(Vec::new());
            st.succ.push(Vec::new());
            if let Some((q, tq)) = *prev {
                if tq + 1 == t {
                    st.succ[q].push(k);
                    st.pred[k].push(q);
                }
            }
            *prev = Some((k, t));
        };
        if segment {
            for t in 0..pts.len().saturating_sub(1) {
                let (a, b) = (pts[t], pts[t + 1]);
                if dist(a, b) > 1e-6 {
                    add(&mut st, t, [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0], bearing(a, b), &mut prev);
                }
            }
        } else {
            for (t, &q) in pts.iter().enumerate() {
                add(&mut st, t, q, 0.0, &mut prev);
            }
        }
        let last = st.key.len();
        ends.push((last > first).then(|| (first, last - 1)));
    }
    // the moves from the last state of an edge to the first state of each edge that can follow it
    let pos: std::collections::HashMap<usize, usize> = edges.iter().enumerate().map(|(k, &e)| (e, k)).collect();
    for (k, &e) in edges.iter().enumerate() {
        let Some((_, la)) = ends[k] else { continue };
        for e2 in &net.next[e] {
            if let Some((fa, _)) = pos.get(e2).and_then(|&k2| ends[k2]) {
                st.succ[la].push(fa);
                st.pred[fa].push(la);
            }
        }
    }
    st
}

/// A kept end of a window: a state key, the cost so far and the route so far.
#[derive(Clone)]
struct Kept {
    key: (usize, usize),
    cost: f64,
    route: Vec<usize>,
}

/// The DTW over the rows `r0..=r1` of one window. `cost(row, state)` is the cell cost. `start`: the kept ends of
/// the window before (at row `r0`), or None for an open start. Returns the best `k` ends at row `r1`, one for
/// each edge, best first.
fn window_dtw(
    st: &States,
    r0: usize,
    r1: usize,
    cost: &dyn Fn(usize, usize) -> f64,
    start: Option<&[Kept]>,
    k: usize,
) -> Vec<Kept> {
    let ns = st.key.len();
    let nr = r1 - r0 + 1;
    if ns == 0 {
        return Vec::new();
    }
    let mut d = vec![f64::INFINITY; nr * ns];
    let mut back = vec![(0u8, usize::MAX); nr * ns]; // (move, from): 0 start, 1 input only, 2 both, 3 road only
    let mut from_kept = vec![usize::MAX; ns]; // row r0: the kept end that a state starts from
    let mut e = vec![0.0; ns];
    let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<(u64, usize)>> = std::collections::BinaryHeap::new();
    for i in 0..nr {
        for v in 0..ns {
            e[v] = cost(r0 + i, v);
        }
        let row = i * ns;
        if i == 0 {
            match start {
                None => d[..ns].copy_from_slice(&e[..ns]),
                Some(kept) => {
                    let at: std::collections::HashMap<(usize, usize), usize> =
                        st.key.iter().enumerate().map(|(v, &q)| (q, v)).collect();
                    for (j, kp) in kept.iter().enumerate() {
                        if let Some(&v) = at.get(&kp.key) {
                            if kp.cost < d[v] {
                                d[v] = kp.cost;
                                from_kept[v] = j;
                            }
                        }
                    }
                }
            }
        } else {
            let prev = (i - 1) * ns;
            for v in 0..ns {
                let mut c = d[prev + v];
                let mut b = (1u8, v);
                for &u in &st.pred[v] {
                    if d[prev + u] < c {
                        c = d[prev + u];
                        b = (2u8, u);
                    }
                }
                if c.is_finite() {
                    d[row + v] = c + e[v];
                    back[row + v] = b;
                }
            }
        }
        // the road-only moves within the row (Dijkstra: the input stays, the road goes on)
        heap.clear();
        for v in 0..ns {
            if d[row + v].is_finite() {
                heap.push(std::cmp::Reverse((d[row + v].to_bits(), v)));
            }
        }
        while let Some(std::cmp::Reverse((cb, v))) = heap.pop() {
            let c = f64::from_bits(cb);
            if c > d[row + v] {
                continue;
            }
            for &u in &st.succ[v] {
                let c2 = c + e[u];
                if c2 < d[row + u] {
                    d[row + u] = c2;
                    back[row + u] = (3u8, v);
                    heap.push(std::cmp::Reverse((c2.to_bits(), u)));
                }
            }
        }
    }
    // the best end state of each edge at the last row, best first (ties: the lower state index)
    let last = (nr - 1) * ns;
    let mut best_of: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for v in 0..ns {
        if d[last + v].is_finite() {
            let b = best_of.entry(st.edge[v]).or_insert(v);
            if d[last + v] < d[last + *b] {
                *b = v;
            }
        }
    }
    let mut ends: Vec<usize> = best_of.into_values().collect();
    ends.sort_by(|&a, &b| d[last + a].total_cmp(&d[last + b]).then(a.cmp(&b)));
    ends.truncate(k.max(1));
    ends.into_iter()
        .map(|v0| {
            // back to row r0
            let (mut i, mut v) = (nr - 1, v0);
            let mut seq = vec![v];
            loop {
                let (mv, u) = back[i * ns + v];
                match mv {
                    1 => i -= 1,
                    2 => {
                        i -= 1;
                        v = u;
                    }
                    3 => v = u,
                    _ => break,
                }
                seq.push(v);
            }
            seq.reverse();
            let mut route = match start {
                Some(kept) => kept[from_kept[seq[0]]].route.clone(),
                None => Vec::new(),
            };
            for v in seq {
                if route.last() != Some(&st.edge[v]) {
                    route.push(st.edge[v]);
                }
            }
            Kept { key: st.key[v0], cost: d[last + v0], route }
        })
        .collect()
}

/// GPS-DTW on one track (`x`, `y` at `ts`; `origin` = a GPS point). Returns the route (edges).
///
/// The rows (input points, or input segments in segment mode) go in windows of `p.window` rows. Each window
/// keeps its best `p.top_k` ends (one for each edge), and the next window starts only from them. `p.window = 0`:
/// one window for the whole track (the full DTW).
pub fn gps_dtw(net: &Network, x: &[f64], y: &[f64], ts: &[i64], origin: &[bool], p: &GpsDtwParams) -> Vec<usize> {
    let n = x.len();
    if n == 0 {
        return Vec::new();
    }
    let mut w = weights(ts, origin, p.tau);
    for (wi, &o) in w.iter_mut().zip(origin) {
        if o {
            *wi = p.gps_weight;
        }
    }
    // the search radius
    let gps: Vec<usize> = (0..n).filter(|&i| origin[i]).collect();
    let mut gaps: Vec<f64> = gps.windows(2).map(|q| dist([x[q[0]], y[q[0]]], [x[q[1]], y[q[1]]])).collect();
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let med = if gaps.is_empty() { 0.0 } else { gaps[gaps.len() / 2] };
    let r = p.corridor_min.max(p.corridor_gap * med);
    let near: Vec<Vec<usize>> = (0..n).map(|i| net.neighboring_edges(x[i], y[i], r).into_iter().map(|(e, _)| e).collect()).collect();
    let segment = p.segment && n >= 2;
    // the rows: input points, or input segments (i, i + 1)
    let nrows = if segment { n - 1 } else { n };
    let seg_mid: Vec<P> = if segment { (0..n - 1).map(|i| [(x[i] + x[i + 1]) / 2.0, (y[i] + y[i + 1]) / 2.0]).collect() } else { vec![] };
    let seg_w: Vec<f64> = if segment { (0..n - 1).map(|i| 0.5 * (w[i] + w[i + 1])).collect() } else { vec![] };
    let seg_b: Vec<f64> = if segment { (0..n - 1).map(|i| bearing([x[i], y[i]], [x[i + 1], y[i + 1]])).collect() } else { vec![] };
    let bw = p.bearing_weight;
    let win = if p.window == 0 { nrows } else { ((p.window as f64 / p.step).round() as usize).max(1) };
    let mut kept: Option<Vec<Kept>> = None;
    let mut r0 = 0;
    loop {
        // a window shares its first row with the last row of the window before
        let r1 = (r0 + win).min(nrows - 1).max(r0);
        let last_pt = if segment { r1 + 1 } else { r1 };
        let mut edges: Vec<usize> = (r0..=last_pt).flat_map(|i| near[i].iter().copied()).collect();
        if let Some(kp) = &kept {
            edges.extend(kp.iter().map(|q| q.key.0));
        }
        edges.sort_unstable();
        edges.dedup();
        let st = build_states(net, &edges, (p.speed * p.step).max(0.1), segment);
        let cost = |row: usize, v: usize| -> f64 {
            if segment {
                let mut c = dist(seg_mid[row], st.pos[v]);
                if bw > 0.0 {
                    let a = (seg_b[row] - st.bear[v]).abs() % 360.0;
                    c += bw * a.min(360.0 - a);
                }
                seg_w[row] * c
            } else {
                w[row] * dist([x[row], y[row]], st.pos[v])
            }
        };
        let k = if r1 == nrows - 1 { 1 } else { p.top_k };
        let out = window_dtw(&st, r0, r1, &cost, kept.as_deref(), k);
        if out.is_empty() {
            // no connected road: give the best route so far
            return kept.and_then(|kp| kp.into_iter().next()).map(|q| q.route).unwrap_or_default();
        }
        if r1 == nrows - 1 {
            return out.into_iter().next().map(|q| q.route).unwrap_or_default();
        }
        kept = Some(out);
        r0 = r1;
    }
}

/// The input of the DTW: `richdata`, then the Bezier interpolation each `step` seconds with the stopped repeats removed.
/// Returns (x, y, ts, origin); origin = a real GPS point.
pub fn prepare(x: &[f64], y: &[f64], ts: &[i64], step: f64) -> Result<(Vec<f64>, Vec<f64>, Vec<i64>, Vec<bool>), String> {
    let c = enrich(x, y, ts, 0.7, 1.0, 50.0, 2.0, Some(2.0))?;
    if c.x.len() < 2 {
        let n = c.x.len();
        return Ok((c.x, c.y, c.ts, vec![true; n]));
    }
    let cts: Vec<f64> = c.ts.iter().map(|&t| t as f64).collect();
    let tr = interpolate(&c.x, &c.y, &cts, &c.speed, &c.bearing, &c.stop, step);
    let (mut px, mut py, mut pt, mut po) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for k in 0..tr.x.len() {
        let same = !px.is_empty() && px[px.len() - 1] == tr.x[k] && py[py.len() - 1] == tr.y[k];
        if same {
            // a stopped repeat: keep the GPS mark on the kept point
            let l = po.len() - 1;
            po[l] = po[l] || tr.origin[k];
            continue;
        }
        px.push(tr.x[k]);
        py.push(tr.y[k]);
        pt.push(tr.ts[k]);
        po.push(tr.origin[k]);
    }
    Ok((px, py, pt, po))
}

/// The confidence weight of each point: 1 / (1 + (dt / tau)^2), dt = seconds to the nearest GPS point.
pub fn weights(ts: &[i64], origin: &[bool], tau: f64) -> Vec<f64> {
    seconds_to_gps(ts, origin).iter().map(|&t| 1.0 / (1.0 + (t / tau).powi(2))).collect()
}

/// GPS-DTW on one trace: [`prepare`], then [`gps_dtw`].
pub fn match_gps_dtw(net: &Network, x: &[f64], y: &[f64], ts: &[i64], p: &GpsDtwParams) -> Result<Vec<usize>, String> {
    let (px, py, pt, po) = prepare(x, y, ts, p.step)?;
    Ok(gps_dtw(net, &px, &py, &pt, &po, p))
}
