# Your first match

## 1. Make the network

GPS-DTW reads the network as plain arrays: the shape of each edge in metres, its speed, and its start and end node.
The function `rust_network_from_edges` (in `sources/engine.py` of NRTMapMatching) makes them from duckOSM's driving edges.

```python
import duckosm
from engine import rust_network_from_edges

edges = duckosm.open("monaco.duckdb").driving.edges(crs="utm")
net, edge_ids = rust_network_from_edges(
    edges.edge_id.astype(int).tolist(),
    edges.source.tolist(),
    edges.target.tolist(),
    [list(g.coords) for g in edges.geometry],
    (edges.maxspeed_kmh.fillna(30.0) / 3.6).tolist(),
)
```

Make the network one time and use it for all traces.

## 2. Match a trace

The trace needs x and y in the same CRS as the network (here UTM) and the time in whole seconds.

```python
import nrt_matcher

path = nrt_matcher.match_gps_dtw(net, xs, ys, timestamps)   # segment mode, the defaults
route = [edge_ids[e] for e in path]                         # the duckOSM edge ids, in driving order
```

`route` is the sequence of edges that the vehicle drove, each edge once for each pass.

## 3. Change the settings

```python
path = nrt_matcher.match_gps_dtw(net, xs, ys, timestamps,
                                 segment=False,      # node mode
                                 speed=10.0,         # road points each 10 m
                                 tau=3.0,            # the time scale of the confidence weight
                                 bearing_weight=0.1) # segment mode: 180 degrees count as 18 m
```

See [Parameters](../reference/parameters.md).
