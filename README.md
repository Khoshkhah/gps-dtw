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

## Results (Monaco, simulated traces, recall / precision)

| GPS interval | GPS-DTW (segment mode) | duckOSM HMM |
|---|---|---|
| 15 s | 0.93 / 0.95 | 0.93 / 0.96 |
| 30 s | 0.83 / 0.87 | 0.90 / 0.96 |

## Folders

| Folder | Contents |
|---|---|
| `rust/` | the Rust crate and the Python module `gps_dtw` |
| `docs/` | the documentation (mkdocs-material) |
| `scripts/` | the test runner `gps_dtw_test.py`, the trace generator, the HMM run; `compare/` has the scripts of the older methods |
| `data/` | the Monaco routes, traces and HMM results (`monaco.duckdb` is not in git; make it with duckOSM) |
| `viz/` | a map page of the 30 s results |

## Run the test

```bash
cd data
python ../scripts/gps_dtw_test.py '{}' '{"segment": false}'
```

## Documentation

```bash
pip install mkdocs-material
mkdocs serve
```
