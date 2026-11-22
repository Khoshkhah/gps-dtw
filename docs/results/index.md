# Results

The test data is 600 simulated traces in Monaco: 50 routes, GPS each 1, 5, 15 and 30 s, GPS noise of 3, 10 and 20 m.
See [Test data](data.md).

The scores of one trace:

- **recall** = the share of the true route's edges that the route has;
- **precision** = the share of the route's edges that are on the true route.

## GPS-DTW (segment mode, defaults)

A sample of every third route (51 traces for each GPS interval), with the turn restrictions and the defaults:

| GPS interval | Recall | Precision | Time for each trace |
|---|---|---|---|
| 1 s | 0.962 | 0.982 | 0.04 s |
| 5 s | 0.961 | 0.959 | 0.04 s |
| 15 s | 0.944 | 0.951 | 0.04 s |
| 30 s | 0.918 | 0.921 | 0.04 s |

| Page | Contents |
|---|---|
| [Comparison with other methods](compare.md) | GPS-DTW against the duckOSM HMM, NRT + DTW and graph-DTW |
| [Effect of the parameters](parameters.md) | the mode, the bearing weight, τ, the speed |
| [Test data](data.md) | how the traces were made, and the limits of the test |
