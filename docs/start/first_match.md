# Your first match

## 1. Make the network

`gps_dtw.Network` takes the shape of each edge in metres and its start node and end node. This example takes duckOSM's
driving edges in UTM.

```python
import duckdb, duckosm, gps_dtw

edges = duckosm.open("monaco.duckdb").driving.edges(crs="utm")
edge_ids = edges.edge_id.astype(int).tolist()

# the allowed turns: duckOSM's edge_graph has no turn that a turn restriction forbids
index = {e: i for i, e in enumerate(edge_ids)}
pairs = duckdb.connect("monaco.duckdb", read_only=True).sql(
    "select from_edge, to_edge from driving.edge_graph").fetchall()
turns = [(index[a], index[b]) for a, b in pairs if a in index and b in index]

net = gps_dtw.Network([list(g.coords) for g in edges.geometry],
                      edges.source.tolist(), edges.target.tolist(), turns)
```

Make the network one time and use it for all traces.

## 2. Match a trace

The trace needs x and y in the same CRS as the network (here UTM) and the time in whole seconds.

```python
path = gps_dtw.match_trace(net, xs, ys, timestamps)   # segment mode, the defaults
route = [edge_ids[e] for e in path]                  # the duckOSM edge ids, in driving order
```

`route` is the sequence of edges that the vehicle drove, each edge once for each pass.

## 3. Change the settings

```python
path = gps_dtw.match_trace(net, xs, ys, timestamps,
                           segment=False,      # node mode
                           speed=10.0,         # road points each 10 m
                           tau=3.0,            # the time scale of the confidence weight
                           bearing_weight=0.1) # segment mode: 180 degrees count as 18 m
```

See [Parameters](../reference/parameters.md).
