# The cost

## The moves

| Move | Input | Road |
|---|---|---|
| Both on | the next point (or segment) | the next point (or segment) along a connection |
| Input only | the next | the same (for example a stop) |
| Road only | the same | the next (the road goes on while the input waits) |

Each move pays the cost of the cell that it enters. The alignment can start and end at any road point (open ends).

## Segment mode (default)

One cell is an **input segment** (from point `i` to point `i+1`: one second) against a **road segment** (two consecutive
road points: `speed × 1 s`).

```
cost = w_seg × (distance between the two segment middles + bearing_weight × angle difference)
```

- `w_seg` = the mean [confidence weight](confidence.md) of the two end points of the input segment.
- **The angle difference** is the smaller angle between the bearings of the two segments, 0 to 180 degrees (0 = same
  direction, 180 = opposite). A bearing is in degrees clockwise from north.
- `bearing_weight` converts degrees into metres (default 0). With 0.05, an opposite direction costs the same as 9 m.

## Node mode

One cell is an **input point** against a **road point**.

```
cost = w × distance between the two points
```

## Which mode

| Mode | 15 s | 30 s |
|---|---|---|
| Segment | 0.928 / 0.949 | 0.832 / 0.870 |
| Node | 0.917 / 0.957 | 0.823 / 0.885 |

The segment mode has slightly more recall, the node mode slightly more precision. See
[Effect of the parameters](../results/parameters.md).

!!! warning "The bearing of a short segment"
    A very short input segment (a vehicle that almost stands) has an unreliable bearing. GPS-DTW uses it as it is.
