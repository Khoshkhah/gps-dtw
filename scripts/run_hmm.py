"""duckOSM HMM (Newson-Krumm, Viterbi in Rust) on each trace. Two settings: the defaults, and sigma_m = the noise."""
import time

import duckosm
import pandas as pd
from duckosm import mobility as mob

d = duckosm.open("monaco.duckdb").driving
T = pd.read_parquet("traces.parquet")
SETTINGS = {
    "hmm_default": lambda sd: {},
    "hmm_tuned": lambda sd: dict(sigma_m=max(5.0, sd), radius_m=max(50.0, 4.0 * sd)),
}
THIN = "hmm_tuned_thin"   # hmm_tuned on the points that are >= 2 sigma from the last kept point (Newson-Krumm, 4.1)


def thin(g, sd):
    keep, last = [], None
    for i, (x, y) in enumerate(zip(g.ux, g.uy)):
        if last is None or (x - last[0]) ** 2 + (y - last[1]) ** 2 >= (2 * sd) ** 2 or i == len(g) - 1:
            keep.append(i)
            last = (x, y)
    return g.iloc[keep]
out = []
for (rid, dt, sd), g in T.groupby(["route_id", "dt", "noise"]):
    for name in list(SETTINGS) + [THIN]:
        gg = thin(g, max(5.0, sd)) if name == THIN else g
        kw = SETTINGS["hmm_tuned" if name == THIN else name]
        df = pd.DataFrame({"trace_id": 0, "time": gg.t.values, "x": gg.lon.values, "y": gg.lat.values})
        t0 = time.perf_counter()
        try:
            m = mob.match(d, df, **kw(sd))
            found, err = list(map(int, m.edges.edge_id)), ""
        except Exception as e:                                    # a failure counts as no result
            found, err = [], repr(e)[:200]
        out.append(dict(method=name, route_id=rid, dt=dt, noise=sd, found=found,
                        time_s=time.perf_counter() - t0, error=err))
R = pd.DataFrame(out)
R.to_parquet("res_hmm.parquet")
print(R.groupby("method").agg(t=("time_s", "mean"), nofound=("found", lambda s: (s.str.len() == 0).sum())))
