# Results

The test data is 600 simulated traces in Monaco: 50 routes, GPS each 1, 5, 15 and 30 s, GPS noise of 3, 10 and 20 m.
See [Test data](data.md).

The scores of one trace:

- **recall** = the share of the true route's edges that the found route has;
- **precision** = the share of the found route's edges that are on the true route.

## GPS-DTW with the defaults

All 600 traces. Recall / precision, and the mean time for each trace on one CPU core.

| GPS interval | Recall | Precision | Time |
|---|---|---|---|
| 1 s | 0.945 | 0.974 | 3 ms |
| 5 s | 0.954 | 0.957 | 3 ms |
| 15 s | 0.920 | 0.935 | 5 ms |
| 30 s | 0.891 | 0.911 | 7 ms |

| Page | Contents |
|---|---|
| [Comparison with the HMM](compare.md) | GPS-DTW against the duckOSM HMM: accuracy and time |
| [Effect of the parameters](parameters.md) | each parameter changed from the defaults |
| [Test data](data.md) | how the traces were made, and the limits of the test |
