# Comparison with the HMM

The duckOSM map matcher is a hidden Markov model (Newson and Krumm, with the Viterbi search in Rust). The two methods ran
with their defaults on all 600 traces, one after the other in one process and one thread
(`scripts/compare_hmm.py`).

## By GPS interval

| GPS interval | GPS-DTW recall / precision | HMM recall / precision | GPS-DTW time | HMM time |
|---|---|---|---|---|
| 1 s | 0.945 / **0.974** | **0.962** / 0.656 | **4 ms** | 65 ms |
| 5 s | 0.955 / **0.956** | **0.962** / 0.901 | **6 ms** | 58 ms |
| 15 s | 0.922 / 0.939 | **0.949** / 0.938 | **15 ms** | 60 ms |
| 30 s | 0.898 / 0.912 | **0.939** / **0.951** | **31 ms** | 65 ms |

The time is the median for each trace. The HMM time includes its Python call (the data frame and the projection).

## By GPS noise

| Noise | GPS-DTW recall / precision | HMM recall / precision |
|---|---|---|
| 3 m | 0.958 / 0.974 | **0.980 / 0.983** |
| 10 m | 0.936 / **0.947** | **0.952** / 0.870 |
| 20 m | 0.896 / **0.916** | **0.927** / 0.732 |

## Summary

- **Speed:** GPS-DTW is 2 to 15 times faster.
- **Precision:** GPS-DTW is much better at 1 s and 5 s and with 10 m or 20 m noise. There the HMM adds wrong edges.
- **Recall:** the HMM is 0.01 to 0.04 better at each interval.
- **30 s:** the HMM is better on both scores. The test routes are shortest paths, which favours the HMM (see
  [Test data](data.md)).
