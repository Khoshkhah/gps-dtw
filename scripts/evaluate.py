"""Scores of each result file against the true routes. Usage: evaluate.py res_a.parquet res_b.parquet ...
Per trace: recall = |true & found| / |true|, precision = |true & found| / |found|, exact = found sequence == true
sequence. 'parallel' = routes where >= 45 % of the length has a same-direction road within 15 m."""
import sys

import pandas as pd

PAR = 0.45
routes = pd.read_parquet("routes.parquet").set_index("route_id")


def score(r):
    t = list(routes.at[r.route_id, "edges"])
    f = [e for i, e in enumerate(r.found) if i == 0 or e != r.found[i - 1]]   # drop repeats in a row
    ts, fs = set(t), set(f)
    hit = len(ts & fs)
    return pd.Series(dict(recall=hit / len(ts), precision=hit / len(fs) if fs else 0.0,
                          exact=float(f == t), exact_set=float(fs == ts), fail=float(not fs)))


def table(paths, subset=None):
    R = pd.concat([pd.read_parquet(p) for p in paths], ignore_index=True)
    R = R.join(R.apply(score, axis=1))
    R["parallel"] = R.route_id.map(routes.parallel) >= PAR
    if subset is not None:
        R = R[R.parallel == subset]
    return R


if __name__ == "__main__":
    pd.set_option("display.width", 200)
    for sub, label in [(None, "ALL ROUTES"), (True, f"PARALLEL ROUTES (share >= {PAR})")]:
        R = table(sys.argv[1:], sub)
        g = R.groupby(["dt", "noise", "method"])[["recall", "precision", "exact", "fail", "time_s"]].mean()
        print(f"\n== {label}: {R.route_id.nunique()} routes ==")
        print(g.unstack("method").round(3).to_string())
        print(R.groupby("method")[["recall", "precision", "exact", "exact_set", "fail", "time_s"]].mean().round(3))
