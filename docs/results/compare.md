# Comparison with the HMM

The duckOSM map matcher is a hidden Markov model (Newson and Krumm, with the Viterbi search in Rust). The two methods ran
with their defaults on all 600 traces, in one process and one thread (`scripts/compare_hmm.py`). GPS-DTW matches each
trace with its own call. The HMM matches all traces of one GPS interval in one call, because each call reads the
network from the file, and that read costs more than the matching of one trace.

## By GPS interval

| GPS interval | GPS-DTW recall / precision | HMM recall / precision | GPS-DTW time (150 traces) | HMM time (150 traces) |
|---|---|---|---|---|
| 1 s | 0.945 / **0.974** | **0.962** / 0.656 | **0.6 s** | 2.2 s |
| 5 s | 0.953 / **0.957** | **0.962** / 0.901 | **0.7 s** | 1.0 s |
| 15 s | 0.922 / 0.939 | **0.949** / 0.938 | 1.5 s | **1.2 s** |
| 30 s | 0.898 / 0.912 | **0.939** / **0.951** | 2.9 s | **1.8 s** |

## By GPS noise

| Noise | GPS-DTW recall / precision | HMM recall / precision |
|---|---|---|
| 3 m | 0.958 / 0.974 | **0.980 / 0.983** |
| 10 m | 0.936 / **0.947** | **0.952** / 0.870 |
| 20 m | 0.894 / **0.916** | **0.927** / 0.732 |

## Summary

- **Speed:** GPS-DTW is faster with dense GPS: 3.5 times at 1 s, 1.5 times at 5 s. The HMM is faster at 15 s and
  30 s: it handles each GPS point, and a sparse trace has few points; GPS-DTW handles each second of the trip.
- **Precision:** GPS-DTW is much better at 1 s and 5 s and with 10 m or 20 m noise. There the HMM adds wrong edges.
- **Recall:** the HMM is 0.01 to 0.04 better at each interval.
