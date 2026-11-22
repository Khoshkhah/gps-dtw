"""Full graph-DTW (Rust) on the whole trace, duckOSM network (UTM), no SUMO. A = every GPS point + the Bezier
interpolation (Rust) each 5 s. Versions: with/without new A points (raw_a), with/without confidence weights."""
import os, time
import numpy as np
import pandas as pd
from shapely import STRtree
from shapely.geometry import LineString
from joblib import Parallel, delayed

STEP_S = int(os.environ.get("STEP_S", 5))
_N = {}


def net():
    if not _N:
        import duckosm
        e = duckosm.open("monaco.duckdb").driving.edges(crs="utm")
        _N["geoms"], _N["ids"] = list(e.geometry), [int(i) for i in e.edge_id]
        _N["tree"] = STRtree(_N["geoms"])
        _N["routes"] = pd.read_parquet("routes.parquet").set_index("route_id")
    return _N


def confidence(ts, origin):
    o = np.unique(ts[origin])
    i1 = np.clip(np.searchsorted(o, ts, side="left"), 0, len(o) - 1)
    i0 = np.clip(np.searchsorted(o, ts, side="right") - 1, 0, len(o) - 1)
    t0, t1 = o[i0], o[i1]
    half = (t1 - t0) / 2
    with np.errstate(invalid="ignore", divide="ignore"):
        c = 1 - np.minimum(ts - t0, t1 - ts) / half
    return np.where(half > 0, np.clip(c, 0, 1), 1.0)


def seconds_to_gps(ts, origin):
    """The seconds from each point to the nearest GPS point (origin)."""
    o = np.unique(ts[origin])
    i1 = np.clip(np.searchsorted(o, ts, side="left"), 0, len(o) - 1)
    i0 = np.clip(np.searchsorted(o, ts, side="right") - 1, 0, len(o) - 1)
    return np.minimum(np.abs(ts - o[i0]), np.abs(o[i1] - ts)).astype(float)


def metres_to_gps(x, y, origin):
    """The metres along the track from each point to the nearest GPS point (origin)."""
    s = np.concatenate([[0.0], np.cumsum(np.hypot(np.diff(x), np.diff(y)))])
    o = s[origin]
    i1 = np.clip(np.searchsorted(o, s, side="left"), 0, len(o) - 1)
    i0 = np.clip(np.searchsorted(o, s, side="right") - 1, 0, len(o) - 1)
    return np.minimum(np.abs(s - o[i0]), np.abs(o[i1] - s))


def score(true, found):
    f = [e for i, e in enumerate(found) if i == 0 or e != found[i - 1]]
    ts, fs = set(true), set(f)
    hit = len(ts & fs)
    return hit / len(ts), (hit / len(fs) if fs else 0.0)


def one(key, g, raw_a, wmin, tau=None, lam=None):
    import nrt_matcher
    from network_matching.graph_dtw import match_edge_to_bgraph
    rid, dt, sd = key
    N = net()
    ts = (g.seq.values * dt).astype(int); ts[-1] = int(-(-g.t.values[-1] // 1))
    o = pd.DataFrame({"x": g.ux.values, "y": g.uy.values, "ts": ts}).drop_duplicates("ts")
    t0 = time.perf_counter()
    cx, cy, cts, csp, cbr, cst = nrt_matcher.enrich(o.x.tolist(), o.y.tolist(), o.ts.astype(int).tolist(), 0.7, 1.0, 50.0, 2.0, 2.0)
    if len(cx) >= 2:
        _l, x, y, its, sp, br, orig, st = nrt_matcher.interpolate(cx, cy, [float(v) for v in cts], csp, cbr, cst, 1.0)
    else:
        x, y, its, orig = cx, cy, cts, [True] * len(cx)
    x, y, its, orig = map(np.asarray, (x, y, its, orig))
    conf = confidence(its, orig)
    if tau is not None:          # the weight follows the time to the nearest GPS point: 1 / (1 + (dt / tau)^2)
        conf = 1.0 / (1.0 + (seconds_to_gps(its, orig) / tau) ** 2)
        wmin = 0.0
    if lam is not None:          # the same with the metres along the track to the nearest GPS point
        conf = 1.0 / (1.0 + (metres_to_gps(x.astype(float), y.astype(float), orig) / lam) ** 2)
        wmin = 0.0
    keep = orig | ((its - its[0]) % STEP_S == 0)
    keep[-1] = True
    # drop repeated positions (stop points) so that A has no zero-length steps
    ax, ay, aw = [], [], []
    for xi, yi, ci in zip(x[keep], y[keep], conf[keep]):
        if not ax or (xi, yi) != (ax[-1], ay[-1]):
            ax.append(float(xi)); ay.append(float(yi)); aw.append(wmin + (1 - wmin) * float(ci))
    coords = list(zip(ax, ay))
    line = LineString(coords) if len(coords) > 1 else LineString(coords * 2)
    if os.environ.get("GAP_FROM", "a") == "gps":   # the dtw_wide rule: half the spacing of the GPS points
        gap = float(np.median(np.hypot(np.diff(o.x.values), np.diff(o.y.values)))) if len(o) > 1 else 0.0
    else:
        gap = float(np.median(np.hypot(np.diff(ax), np.diff(ay)))) if len(ax) > 1 else 0.0
    md = max(30.0, 3 * sd, 0.5 * gap)
    step_a = float(np.mean(np.hypot(np.diff(ax), np.diff(ay)))) if len(ax) > 1 else 10.0
    if os.environ.get("TSPACE") == "auto":
        gap_keep = gap
        gap = max(step_a, 1.0)
    b = [(N["ids"][i], N["geoms"][i]) for i in N["tree"].query(line, predicate="dwithin", distance=md)]
    res = match_edge_to_bgraph(coords, b, snap_tolerance_m=0.75, step_meters=10.0, engine="rust",
                               alpha=float(os.environ.get("ALPHA", 1.0)),
                               emission=os.environ.get("EMISSION", "point"),
                               bearing_weight=float(os.environ.get("BEARING_W", 0.0)),
                               time_spacing_m=(gap if os.environ.get("TSPACE") == "auto" else float(os.environ.get("TSPACE", 0))),
                               raw_a=raw_a, a_weights=aw if (wmin < 1 or tau is not None or lam is not None) else None)
    el = time.perf_counter() - t0
    if os.environ.get("TSPACE") == "auto":
        gap = gap_keep
    found = [int(e) for e, _d, _s in res["route"]]
    true = N["routes"].at[rid, "edges"]
    r, p = score(true, found)
    inside = len(set(true) & {e for e, _g in b}) / len(set(true))      # true edges inside the corridor
    return dict(dt=dt, noise=sd, recall=r, precision=p, time=el, n_a=len(coords), inside=inside, md=md)


if __name__ == "__main__":
    T = pd.read_parquet("traces.parquet")
    groups = list(T.groupby(["route_id", "dt", "noise"]))
    pd.set_option("display.width", 220)
    rows = []
    import sys
    VERS = [("DTW adds points, no weights", False, 1.0, None), ("no new points, no weights", True, 1.0, None),
            ("no new points + weights 0.2", True, 0.2, None), ("DTW adds points + weights 0.2", False, 0.2, None)]
    if len(sys.argv) > 1 and sys.argv[1] == "tau":
        VERS = [(f"no new points, time weight, tau {tau:.0f} s", True, 0.0, tau, None) for tau in (3.0, 5.0, 10.0)] + \
               [(f"no new points, distance weight, lambda {lam:.0f} m", True, 0.0, None, lam) for lam in (20.0, 50.0)]
    else:
        VERS = [v + (None,) for v in VERS]
    for name, raw_a, wmin, tau, lam in VERS:
        R = pd.DataFrame(Parallel(n_jobs=6)(delayed(one)(k, g, raw_a, wmin, tau, lam) for k, g in groups))
        s = R.groupby("dt")[["recall", "precision"]].mean()
        rows.append(dict(version=name, **{f"{d} s": f"{s.loc[d].recall:.3f}/{s.loc[d].precision:.3f}" for d in s.index},
                         all=f"{R.recall.mean():.3f}/{R.precision.mean():.3f}", time_med_s=round(R.time.median(), 4)))
    print(pd.DataFrame(rows).to_string(index=False))
