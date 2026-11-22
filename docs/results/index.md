# Results

The test data is 600 simulated traces in Monaco: 50 routes, GPS each 1, 5, 15 and 30 s, GPS noise of 3, 10 and 20 m.
See [Test data](data.md).

The scores of one trace:

- **recall** = the share of the true route's edges that the route has;
- **precision** = the share of the route's edges that are on the true route.

## GPS-DTW (segment mode, defaults)

A sample of every third route at 15 s and 30 s (102 traces):

| GPS interval | Recall | Precision | Time for each trace |
|---|---|---|---|
| 15 s | 0.928 | 0.949 | 0.02 s |
| 30 s | 0.832 | 0.870 | 0.02 s |

| Page | Contents |
|---|---|
| [Comparison with other methods](compare.md) | GPS-DTW against the duckOSM HMM, NRT + DTW and graph-DTW |
| [Effect of the parameters](parameters.md) | the mode, the bearing weight, τ, the speed |
| [Test data](data.md) | how the traces were made, and the limits of the test |
