"""Make true car routes on Monaco and noisy GPS traces from them.
Out: routes.parquet (one row for each route), traces.parquet (one row for each GPS point)."""
import duckosm
import geopandas as gpd
import numpy as np
import pandas as pd
from shapely import STRtree
from shapely.ops import linemerge

SEED, N_ROUTES = 20261121, 50
INTERVALS, NOISES = [1, 5, 15, 30], [3, 10, 20]

d = duckosm.open("monaco.duckdb").driving
eu = d.edges(crs="utm").set_index("edge_id")
el = d.edges().set_index("edge_id")
rng = np.random.default_rng(SEED)
starts = np.array([g.coords[0] for g in el.geometry])        # the start node of each edge, lon/lat
tree = STRtree(list(eu.geometry))
geoms, ids = list(eu.geometry), np.array(eu.index)


def bearing(p, q):
    return np.degrees(np.arctan2(q[0] - p[0], q[1] - p[1])) % 360


def parallel_share(edges, line):
    """Share of the route length where another edge (not on the route) runs in the same direction within 15 m."""
    on = set(edges)
    hit, n = 0, 0
    for s in np.arange(5, line.length - 5, 10.0):
        p, q = line.interpolate(s), line.interpolate(s + 2)
        b = bearing(p.coords[0], q.coords[0])
        n += 1
        for i in tree.query(p.buffer(15)):
            if ids[i] in on:
                continue
            g = geoms[i]
            f = g.project(p)
            a, c = g.interpolate(max(f - 1, 0)), g.interpolate(min(f + 1, g.length))
            if g.distance(p) <= 15 and abs((bearing(a.coords[0], c.coords[0]) - b + 180) % 360 - 180) < 30:
                hit += 1
                break
    return hit / max(n, 1)


routes = []
while len(routes) < N_ROUTES:
    a, b = starts[rng.choice(len(starts), 2, replace=False)]
    try:
        r = d.route(tuple(a), tuple(b))
    except Exception:
        continue
    if not r or r["length_m"] < 1000 or len(r["edges"]) < 5:
        continue
    line = linemerge(list(eu.loc[r["edges"]].geometry))
    if line.geom_type != "LineString":
        continue
    routes.append(dict(route_id=len(routes), edges=list(map(int, r["edges"])), length_m=float(r["length_m"]),
                       time_s=float(r["time_s"]), parallel=parallel_share(r["edges"], line), line=line.wkt))
R = pd.DataFrame(routes)
R.to_parquet("routes.parquet")
print(R[["length_m", "time_s", "parallel"]].describe().round(2))
print("routes with parallel share >= 0.25:", (R.parallel >= 0.25).sum())

rows = []
for r in R.itertuples():
    line = gpd.GeoSeries.from_wkt([r.line]).iloc[0]
    v = r.length_m / r.time_s                                  # the route speed, m/s
    for dt in INTERVALS:
        s = np.append(np.arange(0, line.length, v * dt), line.length)   # a point every dt seconds, and the end
        xy0 = np.array([line.interpolate(m).coords[0] for m in s])
        for sd in NOISES:
            xy = xy0 + rng.normal(0, sd, xy0.shape)              # sd is the error of each axis
            rows.append(pd.DataFrame(dict(route_id=r.route_id, dt=dt, noise=sd, seq=np.arange(len(s)),
                                          t=s / v, ux=xy[:, 0], uy=xy[:, 1])))
T = pd.concat(rows, ignore_index=True)
ll = gpd.GeoSeries(gpd.points_from_xy(T.ux, T.uy), crs=eu.crs).to_crs(4326)
T["lon"], T["lat"] = ll.x.values, ll.y.values
T.to_parquet("traces.parquet")
print(len(T), "points;", T.groupby(["route_id", "dt", "noise"]).ngroups, "traces")
