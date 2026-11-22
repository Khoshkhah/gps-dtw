# Start

GPS-DTW is a function of the Rust module `nrt_matcher` (the Rust engine of NRTMapMatching). To use it:

1. [Install](install.md) the module and the tools for the road network.
2. Do [your first match](first_match.md): make the network from duckOSM, then call `match_gps_dtw` for a trace.

## What you need

| Item | Why |
|---|---|
| Rust (stable) and Python 3.9 or later | to build the module `nrt_matcher` |
| [duckOSM](https://khoshkhah.github.io/duckOSM/) | to make the road network of an area (a DuckDB file) |
| A GPS trace | points with x, y in metres (a projected CRS, for example UTM) and a time in seconds |
