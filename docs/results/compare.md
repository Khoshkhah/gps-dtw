# Comparison with other methods

Recall / precision. GPS-DTW: the sample of every third route. The other methods: all traces.

| Method | 15 s | 30 s |
|---|---|---|
| **GPS-DTW, segment mode** | 0.94 / 0.95 | 0.92 / 0.92 |
| GPS-DTW, node mode | 0.95 / 0.96 | 0.92 / 0.93 |
| duckOSM HMM (Newson-Krumm, Rust) | 0.93 / 0.96 | 0.90 / 0.96 |
| NRT + DTW for each gap (beam matcher + graph-DTW top 10) | 0.94 / 0.82 | 0.91 / 0.84 |

GPS-DTW needs 5 to 23 ms (1 s to 30 s GPS) for each trace on one CPU core.

## Summary

- GPS-DTW has the same recall as the duckOSM HMM (tuned) at 15 s and 30 s.
- At 30 s its precision is lower (0.92 against 0.96). At the start of some traces it takes the same street in the
  opposite direction, or a tunnel under the street. The HMM avoids these roads, because they need a detour.
- NRT + DTW for each gap has more recall but much less precision.

The older graph-DTW tests are in [History](../history.md).
