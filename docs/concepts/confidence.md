# The confidence weight

## The formula

```
w = 1 / (1 + (Δt / τ)²)
```

- `Δt` = the seconds from the point to the nearest real GPS point (before or after it).
- `τ` = the time scale (parameter `tau`, default 3 s).

| Δt (s) | 0 | 1 | 3 | 6 | 10 | 15 |
|---|---|---|---|---|---|---|
| w (τ = 3 s) | 1.00 | 0.90 | 0.50 | 0.20 | 0.08 | 0.04 |

A real GPS point has `w = 1`. A point in the middle of a 30 s gap has about `w = 0.04`.

## How it is used

- **Segment mode:** the segment weight is the mean of its two end points. It multiplies the whole cell cost (distance and
  bearing).
- **Node mode:** it multiplies the distance of the point.

## Why it depends on the time to a GPS point

The position error of an interpolated point grows with its time from the nearest real point. A point 5 s from a GPS
point in a short gap is better than a point 15 s from a GPS point in a long gap. A fixed weight in the middle of every
gap does not show this difference.

## Effect

| τ | 15 s | 30 s |
|---|---|---|
| 3 s | 0.917 / 0.957 | 0.823 / 0.885 |
| 10 s | 0.887 / 0.948 | 0.719 / 0.849 |

(node mode, sample). A short time scale is better: the points far from a GPS point must count little.
