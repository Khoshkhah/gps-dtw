# History

The approaches that we tested before GPS-DTW, and what we learned.

## The NRT matcher and its interpolation

| Change | Result |
|---|---|
| Beam search (keep the best hypotheses) instead of the full backtracking tree | no more traces without a route |
| Road interpolation (a shortest path in each long gap) | much better recall at 15 s and 30 s |
| Fix: the velocity smoothing moved sparse GPS points | the largest single gain |
| Fix: the Bezier curve left out the last segment and made loops | small gain |
| Cost weights, bearing weight, confidence weight in the beam matcher | no gain |

## DTW with network-matching (graph-DTW)

| Approach | Why it did not continue |
|---|---|
| Fixed endpoints between matched GPS points | it cannot be better than the shortest path |
| Open ends, top 10 routes for each gap, Viterbi over the gaps (NRT + DTW) | good recall, low precision at the joins (30 s: 0.91 / 0.84) |
| The whole trace on raw GPS points | the straight line between two points cuts corners |
| The whole trace on the Bezier curve, a corridor from the curve spacing | the corridor was too narrow; part of the true route was outside it |
| No added input points (`raw_a`), road each 10 m | recall fell to 0.49 at 5 s: the road-only moves were too expensive |
| Distance to the input segment and a smaller road-move weight | better, but two corrections instead of one rule |
| Road divided by time (speed × 1 s) | the idea of GPS-DTW |

## GPS-DTW with projections (first version)

Each input point got one projection onto a nearby edge, with a cost for the road distance against the input distance and
a skip move. It made long detours. The time division replaced it.

## Lessons

- Check the output of each step against its input before any tuning. The smoothing fault was found late.
- A fair test needs routes that are not shortest paths.
