"""GPS-DTW (the Rust module gps_dtw) on the duckOSM network of Monaco: a quick sample (every third route, 15 s and 30 s).

Run it from data/:  python ../scripts/gps_dtw_test.py '{}' '{"segment": false}' ...
Each argument is one setting (the keyword arguments of gps_dtw.match_trace); the script prints one line for each."""
import json, sys, time
import pandas as pd
from joblib import Parallel, delayed

_N = {}


def net():
    if not _N:
        import os, duckdb, duckosm, gps_dtw
        e = duckosm.open("monaco.duckdb").driving.edges(crs="utm")
        turns = None
        if os.environ.get("GPSDTW_TURNS", "1") == "1":  # the turns of duckOSM's edge_graph (turn restrictions removed)
            k = {int(x): i for i, x in enumerate(e.edge_id)}
            g = duckdb.connect("monaco.duckdb", read_only=True).sql("select from_edge, to_edge from driving.edge_graph").fetchall()
            turns = [(k[a], k[b]) for a, b in g if a in k and b in k]
        _N["net"] = gps_dtw.Network([list(g.coords) for g in e.geometry], e.source.tolist(), e.target.tolist(), turns)
        _N["ids"] = e.edge_id.astype(int).tolist()
        _N["routes"] = pd.read_parquet("routes.parquet").set_index("route_id")
    return _N


def score(true, found):
    """(recall, precision) of the edge sets."""
    ts, fs = set(true), set(found)
    hit = len(ts & fs)
    return hit / len(ts), (hit / len(fs) if fs else 0.0)


def one(key, g, kw):
    import gps_dtw
    rid, dt, sd = key
    N = net()
    ts = (g.seq.values * dt).astype(int); ts[-1] = int(-(-g.t.values[-1] // 1))
    o = pd.DataFrame({"x": g.ux.values, "y": g.uy.values, "ts": ts}).drop_duplicates("ts", keep="first")
    t0 = time.perf_counter()
    path = gps_dtw.match_trace(N["net"], o.x.tolist(), o.y.tolist(), o.ts.astype(int).tolist(), **kw)
    el = time.perf_counter() - t0
    r, p = score(N["routes"].at[rid, "edges"], [N["ids"][e] for e in path])
    return dict(dt=dt, recall=r, precision=p, time=el)


if __name__ == "__main__":
    T = pd.read_parquet("traces.parquet")
    dts = (15, 30)
    groups = [kg for kg in T.groupby(["route_id", "dt", "noise"]) if kg[0][1] in dts and kg[0][0] % 3 == 0]
    for js in sys.argv[1:]:
        kw = json.loads(js)
        R = pd.DataFrame(Parallel(n_jobs=6)(delayed(one)(k, g, kw) for k, g in groups))
        s = R.groupby("dt")[["recall", "precision"]].mean()
        print(f"{js:40s} " + "  ".join(f"{d} s {s.loc[d].recall:.3f}/{s.loc[d].precision:.3f}" for d in dts)
              + f"  time {R.time.median():.3f} s", flush=True)
