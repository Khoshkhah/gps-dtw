"""GPS-DTW (defaults) and the duckOSM HMM (defaults) on all traces, in one process and one thread, so that the times
compare. Run it from data/. GPS-DTW matches each trace with its own call; the HMM matches all traces of one GPS interval
in one call (each call reads the network from the file, which costs more than the matching of one trace). Prints
recall / precision and the total time for each GPS interval, and recall / precision for each noise level."""
import time
import duckosm
import pandas as pd
from duckosm import mobility as mob
import gps_dtw
import gps_dtw_test as G

d = duckosm.open("monaco.duckdb").driving
T = pd.read_parquet("traces.parquet")
N = G.net()
rows, hmm_in, dtw_time = [], [], {}
for k, ((rid, dt, sd), g) in enumerate(T.groupby(["route_id", "dt", "noise"])):
    true = N["routes"].at[rid, "edges"]
    ts = (g.seq.values * dt).astype(int); ts[-1] = int(-(-g.t.values[-1] // 1))
    o = pd.DataFrame({"x": g.ux.values, "y": g.uy.values, "ts": ts}).drop_duplicates("ts", keep="first")
    t0 = time.perf_counter()
    path = gps_dtw.match_trace(N["net"], o.x.tolist(), o.y.tolist(), o.ts.astype(int).tolist())
    dtw_time[dt] = dtw_time.get(dt, 0.0) + time.perf_counter() - t0
    r, p = G.score(true, [N["ids"][e] for e in path])
    rows.append(dict(method="GPS-DTW", dt=dt, noise=sd, recall=r, precision=p))
    hmm_in.append(pd.DataFrame({"trace_id": k, "route_id": rid, "dt": dt, "noise": sd, "time": g.t.values, "x": g.lon.values, "y": g.lat.values}))
H = pd.concat(hmm_in, ignore_index=True)
hmm_time = {}
for dt, Q in H.groupby("dt"):
    t0 = time.perf_counter()
    m = mob.match(d, Q)
    hmm_time[dt] = time.perf_counter() - t0
    found = m.edges.groupby("trace_id").edge_id.apply(lambda s: list(map(int, s)))
    for tid, q in Q.groupby("trace_id"):
        r, p = G.score(N["routes"].at[q.route_id.iat[0], "edges"], found.get(tid, []))
        rows.append(dict(method="duckOSM HMM", dt=dt, noise=q.noise.iat[0], recall=r, precision=p))
R = pd.DataFrame(rows)
S = R.groupby(["dt", "method"]).agg(recall=("recall", "mean"), precision=("precision", "mean")).unstack("method")
S["time GPS-DTW s"] = pd.Series(dtw_time)
S["time HMM s"] = pd.Series(hmm_time)
print(S.round(3).to_string())
print(R.groupby(["noise", "method"])[["recall", "precision"]].mean().round(3).unstack("method").to_string())
