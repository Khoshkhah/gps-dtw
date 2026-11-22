# Python API

## `nrt_matcher.match_gps_dtw`

```python
match_gps_dtw(network, x, y, ts, *, speed=12.0, tau=3.0, corridor_min=30.0, corridor_gap=0.5,
              segment=True, bearing_weight=0.0) -> list[int]
```

| Argument | Type | Description |
|---|---|---|
| `network` | `nrt_matcher.Network` | the road network (see below) |
| `x`, `y` | `list[float]` | the GPS positions in metres, in the CRS of the network |
| `ts` | `list[int]` | the time of each point in whole seconds, increasing |

Returns the route as edge indexes of the network, in driving order. The keyword arguments are in
[Parameters](parameters.md).

## `engine.rust_network_from_edges`

```python
rust_network_from_edges(edge_ids, source, target, coords, speed_mps) -> (nrt_matcher.Network, list)
```

In `sources/engine.py` of NRTMapMatching. It makes the network from plain arrays:

| Argument | Description |
|---|---|
| `edge_ids` | the id of each edge |
| `source`, `target` | the start node and the end node of each edge |
| `coords` | the points of each edge in metres |
| `speed_mps` | the speed of each edge in m/s |

Returns the network and the list of edge ids. The edge index in a route is the position in this list. It needs no SUMO
file and no sumolib.

## `nrt_matcher.Network`

The network class of the Rust engine. `rust_network_from_edges` makes it; you do not usually make it directly.
