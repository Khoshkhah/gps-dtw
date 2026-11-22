# Install

## 1. Build the module

```bash
git clone https://github.com/Khoshkhah/gps-dtw.git
cd gps-dtw
pip install ./rust
```

The module has two Rust dependencies: `pyo3` (the Python binding) and `rstar` (the R-tree that finds the edges near the
trace).

!!! tip "Memory"
    On a computer with little memory, set `CARGO_BUILD_JOBS=4` before the build.

## 2. Install duckOSM

```bash
pip install duckosm
```

duckOSM makes the road network of an area (see the [duckOSM manual](https://khoshkhah.github.io/duckOSM/)). You can also
use a network from another source: GPS-DTW needs only the shape, the start node and the end node of each edge.

## 3. Check

```python
import gps_dtw
help(gps_dtw.match_trace)
```
