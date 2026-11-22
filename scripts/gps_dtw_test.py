"""GPS-DTW (Rust, nrt_matcher.match_gps_dtw) on the duckOSM network, quick sample.
Usage: gps_dtw_test.py '<json kwargs>' ... (one line of results for each setting)."""
import json, sys, time
import pandas as pd
from joblib import Parallel, delayed
import nrt_dtw_duckosm as N0


def one(key, g, kw):
    import nrt_matcher
    rid, dt, sd = key
    N = N0.net()
    ts = (g.seq.values * dt).astype(int); ts[-1] = int(-(-g.t.values[-1] // 1))
    o = pd.DataFrame({"x": g.ux.values, "y": g.uy.values, "ts": ts}).drop_duplicates("ts", keep="first")
    t0 = time.perf_counter()
    path = nrt_matcher.match_gps_dtw(N["net"], o.x.tolist(), o.y.tolist(), o.ts.astype(int).tolist(), **kw)
    el = time.perf_counter() - t0
    found = [N["ids"][e] for e in path]
    r, p = N0.score(N["routes"].at[rid, "edges"], found)
    return dict(dt=dt, recall=r, precision=p, time=el)


if __name__ == "__main__":
    T = pd.read_parquet("traces.parquet")
    dts = (15, 30)
    groups = [kg for kg in T.groupby(["route_id", "dt", "noise"]) if kg[0][1] in dts and kg[0][0] % 3 == 0]
    for js in sys.argv[1:]:
        kw = json.loads(js)
        R = pd.DataFrame(Parallel(n_jobs=6)(delayed(one)(k, g, kw) for k, g in groups))
        s = R.groupby("dt")[["recall", "precision"]].mean()
        print(f"{js:60s} " + "  ".join(f"{d} s {s.loc[d].recall:.3f}/{s.loc[d].precision:.3f}" for d in dts)
              + f"  time {R.time.median():.3f} s", flush=True)
