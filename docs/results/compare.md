# Comparison with the HMM

The duckOSM map matcher is a hidden Markov model (Newson and Krumm, with the Viterbi search in Rust). The two methods ran
with their defaults on all 600 traces, in one process and one thread (`scripts/compare_hmm.py`). GPS-DTW matches each
trace with its own call. The HMM matches all traces of one GPS interval in one call, because each call reads the
network from the file, and that read costs more than the matching of one trace.

## By GPS interval

| GPS interval | GPS-DTW recall / precision | HMM recall / precision | GPS-DTW time (150 traces) | HMM time (150 traces) |
|---|---|---|---|---|
| 1 s | 0.945 / **0.974** | **0.962** / 0.656 | **0.5 s** | 2.2 s |
| 5 s | 0.954 / **0.957** | **0.962** / 0.901 | **0.5 s** | 1.0 s |
| 15 s | 0.920 / 0.935 | **0.949** / **0.938** | **0.7 s** | 1.3 s |
| 30 s | 0.891 / 0.911 | **0.939** / **0.951** | **1.0 s** | 1.9 s |

## By GPS noise

| Noise | GPS-DTW recall / precision | HMM recall / precision |
|---|---|---|
| 3 m | 0.958 / 0.974 | **0.980 / 0.983** |
| 10 m | 0.936 / **0.948** | **0.952** / 0.870 |
| 20 m | 0.889 / **0.912** | **0.927** / 0.732 |

## Summary

- **Speed:** GPS-DTW is faster at each interval: 4.7 times at 1 s, 2.3 times at 5 s, 1.8 times at 15 s and 1.9
  times at 30 s.
- **Precision:** GPS-DTW is much better at 1 s and 5 s and with 10 m or 20 m noise. There the HMM adds wrong edges.
- **Recall:** the HMM is 0.01 to 0.04 better at each interval.
