# Install

## 1. Build the module

The code is in the crate `rust/nrt-matcher` of NRTMapMatching, branch `kaveh/nrt-dtw`.

```bash
git clone https://github.com/Khoshkhah/NRTMapMatching.git
cd NRTMapMatching
git checkout kaveh/nrt-dtw
pip install ./rust/nrt-matcher
```

The crate uses the Rust core of network-matching through a local path (`network-matching-core`, branch
`kaveh/dtw-fixed-ends`) for the older per-gap method. GPS-DTW itself does not use it.

!!! tip "Memory"
    On a computer with little memory, set `CARGO_BUILD_JOBS=4` before the build.

## 2. Install duckOSM

```bash
pip install duckosm
```

duckOSM makes the road network of an area (see the [duckOSM manual](https://khoshkhah.github.io/duckOSM/)).

## 3. Check

```python
import nrt_matcher
print(nrt_matcher.match_gps_dtw.__text_signature__)
```
