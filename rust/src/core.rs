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

/// The road network: the shape of each edge in metres and its start and end node (0-based indexes).
pub struct Network {
    pub shapes: Vec<Vec<P>>,
    pub from: Vec<usize>,
    pub to: Vec<usize>,
    tree: RTree<GeomWithData<Rectangle<P>, usize>>,
}

impl Network {
    pub fn new(shapes: Vec<Vec<P>>, from: Vec<usize>, to: Vec<usize>) -> Self {
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
        Network { shapes, from, to, tree: RTree::bulk_load(boxes) }
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

/// GPS-DTW on one track (`x`, `y` at `ts`; `origin` = a GPS point). Returns the route (edges).
pub fn gps_dtw(net: &Network, x: &[f64], y: &[f64], ts: &[i64], origin: &[bool], p: &GpsDtwParams) -> Vec<usize> {
    let n = x.len();
    if n == 0 {
        return Vec::new();
    }
    let w = weights(ts, origin, p.tau);
    // the road corridor
    let gps: Vec<usize> = (0..n).filter(|&i| origin[i]).collect();
    let mut gaps: Vec<f64> = gps.windows(2).map(|q| dist([x[q[0]], y[q[0]]], [x[q[1]], y[q[1]]])).collect();
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let med = if gaps.is_empty() { 0.0 } else { gaps[gaps.len() / 2] };
    let r = p.corridor_min.max(p.corridor_gap * med);
    let mut edges: Vec<usize> = Vec::new();
    for i in 0..n {
        edges.extend(net.neighboring_edges(x[i], y[i], r).into_iter().map(|(e, _)| e));
    }
    edges.sort_unstable();
    edges.dedup();
    if edges.is_empty() {
        return Vec::new();
    }
    // the road points: each edge divided into points `speed x 1 s` apart
    let mut vx: Vec<P> = Vec::new();
    let mut vedge: Vec<usize> = Vec::new();
    let mut span: Vec<(usize, usize)> = Vec::new(); // first vertex, count, for each corridor edge
    for &e in &edges {
        let pts = points_each(&net.shapes[e], p.speed.max(0.1));
        span.push((vx.len(), pts.len()));
        for q in pts {
            vx.push(q);
            vedge.push(e);
        }
    }
    let nv = vx.len();
    // arcs: along each edge, and from the last point of an edge to the first point of each edge that
    // starts at its end node (the node table, not the geometry)
    let mut pred: Vec<Vec<usize>> = vec![Vec::new(); nv];
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); nv];
    let mut first_of: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for (k, &e) in edges.iter().enumerate() {
        first_of.entry(net.from[e]).or_default().push(span[k].0);
    }
    for (k, &e) in edges.iter().enumerate() {
        let (f, m) = span[k];
        for t in 0..m - 1 {
            succ[f + t].push(f + t + 1);
            pred[f + t + 1].push(f + t);
        }
        if let Some(starts) = first_of.get(&net.to[e]) {
            for &s0 in starts {
                if s0 != f {
                    succ[f + m - 1].push(s0);
                    pred[s0].push(f + m - 1);
                }
            }
        }
    }
    if p.segment && n >= 2 {
        return segment_dtw(x, y, &w, &vx, &vedge, &edges, &span, net, p.bearing_weight);
    }
    // the DP: d[i][v] = the lowest cost of an alignment that ends with input point i on road point v
    let mut d = vec![f64::INFINITY; n * nv];
    let mut back = vec![(0u8, usize::MAX); n * nv]; // (move, from): 0 start, 1 input only, 2 both, 3 road only
    let mut e = vec![0.0; nv];
    let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<(u64, usize)>> = std::collections::BinaryHeap::new();
    for i in 0..n {
        for v in 0..nv {
            e[v] = w[i] * dist([x[i], y[i]], vx[v]);
        }
        let row = i * nv;
        if i == 0 {
            for v in 0..nv {
                d[v] = e[v];
            }
        } else {
            let prev = (i - 1) * nv;
            for v in 0..nv {
                let mut c = d[prev + v];
                let mut b = (1u8, v);
                for &u in &pred[v] {
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
        // the road-only moves within the row (Dijkstra: the input point stays, the road goes on)
        heap.clear();
        for v in 0..nv {
            if d[row + v].is_finite() {
                heap.push(std::cmp::Reverse((d[row + v].to_bits(), v)));
            }
        }
        while let Some(std::cmp::Reverse((cb, v))) = heap.pop() {
            let c = f64::from_bits(cb);
            if c > d[row + v] {
                continue;
            }
            for &u in &succ[v] {
                let c2 = c + e[u];
                if c2 < d[row + u] {
                    d[row + u] = c2;
                    back[row + u] = (3u8, v);
                    heap.push(std::cmp::Reverse((c2.to_bits(), u)));
                }
            }
        }
    }
    // open end: the best road point of the last input point; back to the start
    let last = (n - 1) * nv;
    let mut v = (0..nv).fold(0, |b, q| if d[last + q] < d[last + b] { q } else { b });
    if !d[last + v].is_finite() {
        return Vec::new();
    }
    let mut i = n - 1;
    let mut seq: Vec<usize> = vec![v];
    loop {
        let (mv, u) = back[i * nv + v];
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
    let mut route: Vec<usize> = Vec::new();
    for v in seq {
        if route.last() != Some(&vedge[v]) {
            route.push(vedge[v]);
        }
    }
    route
}

/// The segment mode of [`gps_dtw`]: states = (input segment s, road segment k). The cost of a state =
/// the segment's confidence weight x (distance between the two middles + `bw` x angle difference).
/// Moves: both on (k follows the previous road segment), input only (same road segment), road only
/// (within a row, along the road). Returns the route (edges).
#[allow(clippy::too_many_arguments)]
fn segment_dtw(
    x: &[f64],
    y: &[f64],
    w: &[f64],
    vx: &[P],
    vedge: &[usize],
    edges: &[usize],
    span: &[(usize, usize)],
    net: &Network,
    bw: f64,
) -> Vec<usize> {
    // the road segments: consecutive road points of one edge (zero-length ones left out)
    let mut au: Vec<usize> = Vec::new();
    let mut av: Vec<usize> = Vec::new();
    let mut first_arc: Vec<Option<usize>> = Vec::new();
    let mut last_arc: Vec<Option<usize>> = Vec::new();
    for &(f, m) in span {
        let (mut fa, mut la) = (None, None);
        for t in 0..m - 1 {
            if dist(vx[f + t], vx[f + t + 1]) > 1e-6 {
                if fa.is_none() {
                    fa = Some(au.len());
                }
                la = Some(au.len());
                au.push(f + t);
                av.push(f + t + 1);
            }
        }
        first_arc.push(fa);
        last_arc.push(la);
    }
    let na = au.len();
    if na == 0 {
        return Vec::new();
    }
    // feeds[k] = the road segments that can come right after road segment k
    let mut feeds: Vec<Vec<usize>> = vec![Vec::new(); na];
    let mut preds: Vec<Vec<usize>> = vec![Vec::new(); na];
    for k in 0..na.saturating_sub(1) {
        if av[k] == au[k + 1] {
            feeds[k].push(k + 1);
            preds[k + 1].push(k);
        }
    }
    let mut first_by_node: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for (ei, &e) in edges.iter().enumerate() {
        if let Some(fa) = first_arc[ei] {
            first_by_node.entry(net.from[e]).or_default().push(fa);
        }
    }
    for (ei, &e) in edges.iter().enumerate() {
        if let (Some(la), Some(nexts)) = (last_arc[ei], first_by_node.get(&net.to[e])) {
            for &fa in nexts {
                if fa != first_arc[ei].unwrap_or(usize::MAX) {
                    feeds[la].push(fa);
                    preds[fa].push(la);
                }
            }
        }
    }
    let mid: Vec<P> = (0..na).map(|k| [(vx[au[k]][0] + vx[av[k]][0]) / 2.0, (vx[au[k]][1] + vx[av[k]][1]) / 2.0]).collect();
    let abear: Vec<f64> = (0..na).map(|k| bearing(vx[au[k]], vx[av[k]])).collect();
    let ns = x.len() - 1;
    let mut d = vec![f64::INFINITY; ns * na];
    let mut back = vec![(0u8, usize::MAX); ns * na];
    let mut e = vec![0.0; na];
    let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<(u64, usize)>> = std::collections::BinaryHeap::new();
    for s_ in 0..ns {
        let m = [(x[s_] + x[s_ + 1]) / 2.0, (y[s_] + y[s_ + 1]) / 2.0];
        let sw = 0.5 * (w[s_] + w[s_ + 1]);
        let sb = bearing([x[s_], y[s_]], [x[s_ + 1], y[s_ + 1]]);
        for k in 0..na {
            let mut c = dist(m, mid[k]);
            if bw > 0.0 {
                let a = (sb - abear[k]).abs() % 360.0;
                c += bw * a.min(360.0 - a);
            }
            e[k] = sw * c;
        }
        let row = s_ * na;
        if s_ == 0 {
            d[..na].copy_from_slice(&e[..na]);
        } else {
            let prev = (s_ - 1) * na;
            for k in 0..na {
                let mut c = d[prev + k];
                let mut b = (1u8, k);
                for &q in &preds[k] {
                    if d[prev + q] < c {
                        c = d[prev + q];
                        b = (2u8, q);
                    }
                }
                if c.is_finite() {
                    d[row + k] = c + e[k];
                    back[row + k] = b;
                }
            }
        }
        heap.clear();
        for k in 0..na {
            if d[row + k].is_finite() {
                heap.push(std::cmp::Reverse((d[row + k].to_bits(), k)));
            }
        }
        while let Some(std::cmp::Reverse((cb, k))) = heap.pop() {
            let c = f64::from_bits(cb);
            if c > d[row + k] {
                continue;
            }
            for &q in &feeds[k] {
                let c2 = c + e[q];
                if c2 < d[row + q] {
                    d[row + q] = c2;
                    back[row + q] = (3u8, k);
                    heap.push(std::cmp::Reverse((c2.to_bits(), q)));
                }
            }
        }
    }
    let last = (ns - 1) * na;
    let mut k = (0..na).fold(0, |b, q| if d[last + q] < d[last + b] { q } else { b });
    if !d[last + k].is_finite() {
        return Vec::new();
    }
    let mut s_ = ns - 1;
    let mut seq = vec![k];
    loop {
        let (mv, q) = back[s_ * na + k];
        match mv {
            1 => s_ -= 1,
            2 => {
                s_ -= 1;
                k = q;
            }
            3 => k = q,
            _ => break,
        }
        seq.push(k);
    }
    seq.reverse();
    let mut route: Vec<usize> = Vec::new();
    for k in seq {
        let ed = vedge[au[k]];
        if route.last() != Some(&ed) {
            route.push(ed);
        }
    }
    route
}

/// The input of the DTW: `richdata`, then the Bezier interpolation at 1 s with the stopped repeats removed.
/// Returns (x, y, ts, origin); origin = a real GPS point.
pub fn prepare(x: &[f64], y: &[f64], ts: &[i64]) -> Result<(Vec<f64>, Vec<f64>, Vec<i64>, Vec<bool>), String> {
    let c = enrich(x, y, ts, 0.7, 1.0, 50.0, 2.0, Some(2.0))?;
    if c.x.len() < 2 {
        let n = c.x.len();
        return Ok((c.x, c.y, c.ts, vec![true; n]));
    }
    let cts: Vec<f64> = c.ts.iter().map(|&t| t as f64).collect();
    let tr = interpolate(&c.x, &c.y, &cts, &c.speed, &c.bearing, &c.stop, 1.0);
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
    let (px, py, pt, po) = prepare(x, y, ts)?;
    Ok(gps_dtw(net, &px, &py, &pt, &po, p))
}
