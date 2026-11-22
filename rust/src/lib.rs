//! GPS-DTW: map matching of GPS traces with dynamic time warping. `core` is plain Rust (other Rust programs, for
//! example duckOSM, use it with `default-features = false`); `python` is the pyo3 binding, the Python module `gps_dtw`.

pub mod core;
#[cfg(feature = "python")]
mod python;
