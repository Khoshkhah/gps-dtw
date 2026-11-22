# GPS-DTW

Map matching of GPS traces with dynamic time warping (DTW), in Rust.

GPS-DTW interpolates a trace to one point each second, divides the roads near the trace into points `speed × 1 s` apart,
and aligns the two sequences with DTW. Each input point has a confidence weight `w = 1 / (1 + (Δt / τ)²)`, where `Δt` is
the time to the nearest real GPS point.

Documentation: <https://khoshkhah.github.io/gps-dtw/> (source in `docs/`).

## Results (Monaco, simulated traces, recall / precision)

| GPS interval | GPS-DTW (segment mode) | duckOSM HMM |
|---|---|---|
| 15 s | 0.93 / 0.95 | 0.93 / 0.96 |
| 30 s | 0.83 / 0.87 | 0.90 / 0.96 |

## Where the code is

The matcher is the function `match_gps_dtw` of the Rust module `nrt_matcher`, in NRTMapMatching (crate
`rust/nrt-matcher`, branch `kaveh/nrt-dtw`). This repository has the documentation, the test scripts and the test data.

| Folder | Contents |
|---|---|
| `docs/` | the documentation (mkdocs-material) |
| `scripts/` | the test runner `gps_dtw_test.py`, the network builder and the scoring, the trace generator |
| `data/` | the Monaco routes, traces and HMM results (`monaco.duckdb` is not in git) |
| `viz/` | a map page of the 30 s results |

## Run the test

```bash
cd data
python ../scripts/gps_dtw_test.py '{}'                      # defaults
python ../scripts/gps_dtw_test.py '{"segment": false}'      # node mode
```

`PYTHONPATH` must contain the `nrt_matcher` build and the `sources/` folder of NRTMapMatching.

## Documentation

```bash
pip install mkdocs-material
mkdocs serve
```
