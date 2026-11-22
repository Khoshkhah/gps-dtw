"""NRT + DTW (Rust, nrt_matcher.match_dtw) on duckOSM's driving network in UTM: no SUMO, no sumolib.
Python only reads the edges, builds the network arrays and calls match_dtw for each trace."""
import time
import numpy as np
import pandas as pd
from joblib import Parallel, delayed

_N = {}


def net():
    if not _N:
        import duckosm
        from engine import rust_network_from_edges
        e = duckosm.open("monaco.duckdb").driving.edges(crs="utm")
        speed = (e.maxspeed_kmh.fillna(30.0) / 3.6).tolist()
        _N["net"], _N["ids"] = rust_network_from_edges(e.edge_id.astype(int).tolist(), e.source.tolist(), e.target.tolist(),
                                                       [list(g.coords) for g in e.geometry], speed)
        _N["routes"] = pd.read_parquet("routes.parquet").set_index("route_id")
    return _N


def score(true, found):
    f = [e for i, e in enumerate(found) if i == 0 or e != found[i - 1]]
    ts, fs = set(true), set(f)
    hit = len(ts & fs)
    return hit / len(ts), (hit / len(fs) if fs else 0.0)


def one(key, g):
    import nrt_matcher
    rid, dt, sd = key
    N = net()
    ts = (g.seq.values * dt).astype(int); ts[-1] = int(-(-g.t.values[-1] // 1))
    obs = pd.DataFrame({"x": g.ux.values, "y": g.uy.values, "ts": ts}).drop_duplicates("ts", keep="first")
    t0 = time.perf_counter()
    path = nrt_matcher.match_dtw(N["net"], obs.x.tolist(), obs.y.tolist(), obs.ts.astype(int).tolist())
    el = time.perf_counter() - t0
    found = [N["ids"][e] for e in path]
    r, p = score(N["routes"].at[rid, "edges"], found)
    return dict(route_id=rid, dt=dt, noise=sd, recall=r, precision=p, time=el, found=found)


if __name__ == "__main__":
    T = pd.read_parquet("traces.parquet")
    groups = list(T.groupby(["route_id", "dt", "noise"]))
    R = pd.DataFrame(Parallel(n_jobs=6)(delayed(one)(k, g) for k, g in groups))
    R.to_parquet("res_nrt_dtw_duckosm.parquet")
    S = pd.read_parquet("res_nrt_rust_dtw.parquet")
    pd.set_option("display.width", 200)
    a = R.groupby(["dt", "noise"])[["recall", "precision"]].mean().round(3)
    b = S.groupby(["dt", "noise"])[["recall", "precision"]].mean().round(3)
    print(a.join(b, rsuffix="_sumo").to_string())
    print("all, duckOSM network:", R[["recall", "precision"]].mean().round(3).to_dict(),
          "| SUMO network:", S[["recall", "precision"]].mean().round(3).to_dict())
    net(); one(*groups[0])
    Q = pd.DataFrame([one(k, g) for k, g in groups[::3]])    # one core
    print("time for each trace, one core (median, s):", Q.groupby("dt").time.median().round(4).to_dict())
