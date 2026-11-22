# Start

GPS-DTW is the Rust module `gps_dtw` in this repository (folder `rust/`). To use it:

1. [Install](install.md) the module.
2. Do [your first match](first_match.md): make the network from duckOSM, then call `gps_dtw.match_trace` for a trace.

## What you need

| Item | Why |
|---|---|
| Rust (stable) and Python 3.9 or later | to build the module `gps_dtw` |
| [duckOSM](https://khoshkhah.github.io/duckOSM/) | to make the road network of an area (a DuckDB file) |
| A GPS trace | points with x, y in metres (a projected CRS, for example UTM) and a time in seconds |
