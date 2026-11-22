# Python API

## `gps_dtw.Network`

```python
Network(coords, source, target)
```

| Argument | Type | Description |
|---|---|---|
| `coords` | `list[list[(float, float)]]` | the points of each edge in metres (a projected CRS, for example UTM); two points or more |
| `source` | `list[int]` | the start node of each edge (any integer id) |
| `target` | `list[int]` | the end node of each edge |

The edge index in a route is the position of the edge in these lists. A two-way road is two edges, one in each
direction. `len(net)` gives the number of edges.

## `gps_dtw.match_trace`

```python
match_trace(network, x, y, ts, *, speed=12.0, tau=3.0, corridor_min=30.0, corridor_gap=1.0,
            segment=True, bearing_weight=0.0) -> list[int]
```

| Argument | Type | Description |
|---|---|---|
| `network` | `gps_dtw.Network` | the road network |
| `x`, `y` | `list[float]` | the GPS positions in metres, in the CRS of the network |
| `ts` | `list[int]` | the time of each point in whole seconds, increasing |

Returns the route as edge indexes, in driving order. The keyword arguments are in [Parameters](parameters.md).
`match_trace` releases the Python GIL, so you can match traces in parallel threads with one network.
