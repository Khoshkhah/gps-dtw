# One second on both sides

## The idea

GPS data has one point each `n` seconds. After the interpolation, the input has one point each second. GPS-DTW divides
the road in the same unit: a point each `speed × 1 s`. So one DTW step is one second on both sides.

The parameter `step` changes the unit: with `step = 5`, the input has a point each 5 s and the road a point each
`speed × 5 s`. See [Effect of the parameters](../results/parameters.md).

## Why it matters

DTW pays the cost of each cell in the alignment. Over one stretch of road, the input and the road must pair their points:

- If the road has more points than the input, the "road only" moves must cover the extra road points. Each of them pays
  its distance to one input point. A long stretch of correct road then costs much, and DTW prefers a wrong road with
  fewer points.
- If both sides have about one point for each second, the "both on" move covers most of the stretch, and the cost
  measures only how far the input is from the road.
