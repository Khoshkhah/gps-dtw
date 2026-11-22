# One second on both sides

## The idea

GPS data has one point each `n` seconds. After the interpolation, the input has one point each second. GPS-DTW divides
the road in the same unit: a point each `speed × 1 s`. So one DTW step is one second on both sides.

## Why it matters

DTW pays the cost of each cell in the alignment. Over one stretch of road, the input and the road must pair their points:

- If the road has more points than the input, the "road only" moves must cover the extra road points. Each of them pays
  its distance to one input point. A long stretch of correct road then costs much, and DTW prefers a wrong road with
  fewer points.
- If both sides have about one point for each second, the "both on" move covers most of the stretch, and the cost
  measures only how far the input is from the road.

## What we tried before

| Division | Problem |
|---|---|
| A road point each 10 m and the projection of each input point onto each edge (graph-DTW default) | the input also needs added points (projections of the road nodes), and the added points change the input |
| No added input points, road each 10 m | recall fell to 0.49 at 5 s: the road-only moves were too expensive |
| Distance to the input segment and a smaller road-move weight | better (0.91 at 5 s), but two corrections instead of one rule |
| **Road each `speed × 1 s`** | **one rule, no corrections, the best result** |

See [History](../history.md).
