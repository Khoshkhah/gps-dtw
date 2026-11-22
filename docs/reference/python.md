# Python API

## `gps_dtw.Network`

```python
Network(coords, source, target, turns=None)
```

| Argument | Type | Description |
|---|---|---|
| `coords` | `list[list[(float, float)]]` | the points of each edge in metres (a projected CRS, for example UTM); two points or more |
| `source` | `list[int]` | the start node of each edge (any integer id) |
| `target` | `list[int]` | the end node of each edge |
| `turns` | `list[(int, int)]` or `None` | the allowed turns as (from edge index, to edge index), for example duckOSM's `driving.edge_graph`, where the turn restrictions are removed. `None`: each edge that starts at the end node of an edge can follow it (all turns and U-turns) |

The edge index in a route is the position of the edge in these lists. A two-way road is two edges, one in each
direction. `len(net)` gives the number of edges.

## `gps_dtw.match_trace`

```python
match_trace(network, x, y, ts, *, speed=12.0, tau=3.0, corridor_min=30.0, corridor_gap=1.0,
            segment=True, bearing_weight=0.0, gps_weight=1.0) -> list[int]
```

| Argument | Type | Description |
|---|---|---|
| `network` | `gps_dtw.Network` | the road network |
| `x`, `y` | `list[float]` | the GPS positions in metres, in the CRS of the network |
| `ts` | `list[int]` | the time of each point in whole seconds, increasing |

Returns the route as edge indexes, in driving order. The keyword arguments are in [Parameters](parameters.md).
`match_trace` releases the Python GIL, so you can match traces in parallel threads with one network.

## `gps_dtw.interpolate`

```python
interpolate(x, y, ts, *, tau=3.0) -> (x, y, ts, origin, weight)
```

The input that the DTW uses for one trace: the cleaned GPS points, interpolated at 1 s with the Bezier curve, without
the stopped repeats. `origin` is `True` for a real GPS point. `weight` is the [confidence weight](../concepts/confidence.md)
of each point. Use it to show or check the interpolation.
