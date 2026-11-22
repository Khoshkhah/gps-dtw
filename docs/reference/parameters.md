# Parameters

The parameters of `gps_dtw.match_trace`.

| Parameter | Default | Unit | What it does |
|---|---|---|---|
| `segment` | `True` | | `True`: segment mode. `False`: node mode. See [Segment mode and node mode](../concepts/cost.md). |
| `speed` | `12.0` | m/s | The road points are `speed × 1 s` apart. Use an approximate speed of the vehicle. |
| `tau` | `3.0` | s | The time scale of the [confidence weight](../concepts/confidence.md). |
| `bearing_weight` | `0.0` | m per degree | Segment mode: the weight of the angle difference. |
| `step` | `1.0` | s | The interpolation interval. The road points are `speed × step` apart, so one DTW step is `step` seconds on both sides. |
| `window` | `60` | s | The DTW goes in windows of this many seconds (input rows). `0`: one window for the whole trace (the full DTW). |
| `top_k` | `20` | | The ends (one for each edge) that a window keeps. The next window starts only from them. |
| `gps_weight` | `5.0` | | The confidence weight of a real GPS point. An interpolated point has `1 / (1 + (Δt / τ)²)`, which is less than 1. |
| `corridor_min` | `30.0` | m | The search radius at a real GPS point. At an interpolated point the radius grows with the seconds to the nearest GPS point. |
| `corridor_gap` | `1.0` | | The largest search radius, at the middle of a gap between two GPS points, as a part of the GPS spacing: `max(corridor_min, corridor_gap × median distance between two GPS points)`. With 1.0, it equals the GPS spacing (about 270 m at 30 s in Monaco). |

The fixed settings of the steps before DTW: outliers above 50 m/s, smoothing only for steps of 2 s or less, Bezier
interpolation.
