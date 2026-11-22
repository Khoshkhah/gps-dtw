# Test data

The files are in `data/` of this repository.

| File | Contents |
|---|---|
| `routes.parquet` | 50 true routes: the duckOSM edge ids in order, the length, the time, the line |
| `traces.parquet` | the GPS traces: route, interval (1, 5, 15, 30 s), noise (3, 10, 20 m), the points in lon/lat and UTM |
| `res_hmm.parquet` | the duckOSM HMM results on all traces (for the comparison) |
| `monaco.duckdb` | the duckOSM network of Monaco (not in git; make it with duckOSM) |

## How the traces were made

`scripts/gen.py`:

1. 50 routes between random points with duckOSM `route()` (the shortest path by time).
2. The vehicle drives each route at its route speed.
3. A GPS point each 1, 5, 15 or 30 s, with a random error of 3, 10 or 20 m.

## Limits of the test

!!! warning "The true routes are shortest paths"
    `route()` gives the shortest path. Any part of a shortest path is also the shortest path between its own points.
    So methods that fill gaps with shortest paths (for example the HMM) are right almost by construction.
    Real drivers do not always take the shortest path. A test on real trips is necessary.

- Only one city (Monaco), with many short edges and tunnels.
- Simulated noise: independent errors, no multipath and no gaps in the trace.
