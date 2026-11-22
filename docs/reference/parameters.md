# Parameters

The parameters of `gps_dtw.match_trace`.

| Parameter | Default | Unit | What it does |
|---|---|---|---|
| `segment` | `True` | | `True`: segment mode. `False`: node mode. See [Segment mode and node mode](../concepts/cost.md). |
| `speed` | `12.0` | m/s | The road points are `speed × 1 s` apart. Use an approximate speed of the vehicle. |
| `tau` | `3.0` | s | The time scale of the [confidence weight](../concepts/confidence.md). |
| `bearing_weight` | `0.0` | m per degree | Segment mode: the weight of the angle difference. |
| `corridor_min` | `30.0` | m | The smallest search radius around the input points. |
| `corridor_gap` | `1.0` | | The search radius as a part of the GPS spacing: radius = `max(corridor_min, corridor_gap × median distance between two GPS points)`. With 1.0, the radius equals the GPS spacing (about 270 m at 30 s in Monaco). |

The fixed settings of the steps before DTW: outliers above 50 m/s, smoothing only for steps of 2 s or less, Bezier
interpolation at 1 s.
