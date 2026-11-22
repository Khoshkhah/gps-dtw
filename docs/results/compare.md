# Comparison with other methods

Recall / precision. GPS-DTW: the sample of every third route. The other methods: all traces.

| Method | 15 s | 30 s |
|---|---|---|
| **GPS-DTW, segment mode** | 0.93 / 0.95 | 0.83 / 0.87 |
| GPS-DTW, node mode | 0.92 / 0.96 | 0.82 / 0.89 |
| duckOSM HMM (Newson-Krumm, Rust) | 0.93 / 0.96 | 0.90 / 0.96 |
| NRT + DTW for each gap (beam matcher + graph-DTW top 10) | 0.94 / 0.82 | 0.91 / 0.84 |

GPS-DTW needs about 0.02 s for each trace on one CPU core.

## Summary

- At 15 s GPS-DTW is close to the HMM, with about the same precision.
- At 30 s its recall is lower than the HMM (0.83 against 0.90).
- NRT + DTW for each gap has more recall but much less precision.

The older graph-DTW tests are in [History](../history.md).
