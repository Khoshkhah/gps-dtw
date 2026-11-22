# GPS-DTW

Map matching of GPS traces with dynamic time warping (DTW), in Rust.

GPS-DTW interpolates a trace to one point each second, divides the roads near the trace into points `speed × 1 s` apart,
and aligns the two sequences with DTW. Each input point has a confidence weight `w = 1 / (1 + (Δt / τ)²)`, where `Δt` is
the time to the nearest real GPS point.

Documentation: <https://khoshkhah.github.io/gps-dtw/>

## Install and use

```bash
pip install ./rust
```

```python
import gps_dtw

net = gps_dtw.Network(coords, source, target)        # the shape (metres), start node and end node of each edge
route = gps_dtw.match_trace(net, xs, ys, timestamps)  # edge indexes in driving order
```

## Results (600 simulated traces in Monaco, recall / precision, median time)

| GPS interval | GPS-DTW | duckOSM HMM | GPS-DTW time | HMM time |
|---|---|---|---|---|
| 1 s | 0.95 / 0.97 | 0.96 / 0.66 | 4 ms | 65 ms |
| 5 s | 0.96 / 0.96 | 0.96 / 0.90 | 6 ms | 58 ms |
| 15 s | 0.92 / 0.94 | 0.95 / 0.94 | 15 ms | 60 ms |
| 30 s | 0.90 / 0.91 | 0.94 / 0.95 | 31 ms | 65 ms |

## Folders

| Folder | Contents |
|---|---|
| `rust/` | the Rust crate and the Python module `gps_dtw` |
| `docs/` | the documentation (mkdocs-material) |
| `scripts/` | `gps_dtw_test.py` (parameter tests on a sample), `compare_hmm.py` (GPS-DTW against the duckOSM HMM), `gen.py` (the test traces), `run_hmm.py` (the HMM results file) |
| `data/` | the Monaco routes, traces and HMM results (`monaco.duckdb` is not in git; make it with duckOSM) |
| `viz/` | a map page of the 30 s results |

## Run the test

```bash
cd data
python ../scripts/gps_dtw_test.py '{}' '{"segment": false}'   # GPSDTW_DTS=1,5,15,30 for all intervals
python ../scripts/compare_hmm.py
```

## Documentation

```bash
pip install mkdocs-material
mkdocs serve
```
