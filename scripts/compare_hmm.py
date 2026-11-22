"""GPS-DTW (defaults) and the duckOSM HMM (defaults) on all traces, one after the other in one process and one
thread, so that the times compare. Run it from data/. Prints recall / precision and the median time for each
GPS interval."""
import time
import duckosm
import pandas as pd
from duckosm import mobility as mob
import gps_dtw
import gps_dtw_test as G

d = duckosm.open("monaco.duckdb").driving
T = pd.read_parquet("traces.parquet")
N = G.net()
rows = []
for (rid, dt, sd), g in T.groupby(["route_id", "dt", "noise"]):
    true = N["routes"].at[rid, "edges"]
    ts = (g.seq.values * dt).astype(int); ts[-1] = int(-(-g.t.values[-1] // 1))
    o = pd.DataFrame({"x": g.ux.values, "y": g.uy.values, "ts": ts}).drop_duplicates("ts", keep="first")
    t0 = time.perf_counter()
    path = gps_dtw.match_trace(N["net"], o.x.tolist(), o.y.tolist(), o.ts.astype(int).tolist())
    t1 = time.perf_counter()
    r, p = G.score(true, [N["ids"][e] for e in path])
    rows.append(dict(method="GPS-DTW", dt=dt, noise=sd, recall=r, precision=p, time=t1 - t0))
    df = pd.DataFrame({"trace_id": 0, "time": g.t.values, "x": g.lon.values, "y": g.lat.values})
    t0 = time.perf_counter()
    try:
        found = list(map(int, mob.match(d, df).edges.edge_id))
    except Exception:
        found = []
    t1 = time.perf_counter()
    r, p = G.score(true, found)
    rows.append(dict(method="duckOSM HMM", dt=dt, noise=sd, recall=r, precision=p, time=t1 - t0))
R = pd.DataFrame(rows)
S = R.groupby(["dt", "method"]).agg(recall=("recall", "mean"), precision=("precision", "mean"), time_ms=("time", "median"))
S["time_ms"] *= 1000
print(S.round(3).to_string())
print(R.groupby(["noise", "method"])[["recall", "precision"]].mean().round(3).to_string())
