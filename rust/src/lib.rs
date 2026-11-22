//! The Python module `gps_dtw`: the binding of `core.rs`.

mod core;

use crate::core as c;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// The road network. `coords`: the points of each edge in metres (a projected CRS, for example UTM);
/// `source`, `target`: the start node and the end node of each edge (any integer ids). `turns` (optional): the
/// allowed (from edge index, to edge index) pairs, for example duckOSM's `edge_graph`; without it, each edge
/// that starts at the end node of an edge can follow it. The edge index in a route is the position of the
/// edge in these lists.
#[pyclass(frozen)]
struct Network(Arc<c::Network>);

#[pymethods]
impl Network {
    #[new]
    #[pyo3(signature = (coords, source, target, turns=None))]
    fn new(coords: Vec<Vec<(f64, f64)>>, source: Vec<i64>, target: Vec<i64>, turns: Option<Vec<(usize, usize)>>) -> PyResult<Self> {
        if source.len() != coords.len() || target.len() != coords.len() {
            return Err(PyValueError::new_err("coords, source and target must have the same length"));
        }
        if coords.iter().any(|s| s.len() < 2) {
            return Err(PyValueError::new_err("each edge needs two points or more"));
        }
        if let Some(t) = &turns {
            if t.iter().any(|&(a, b)| a >= coords.len() || b >= coords.len()) {
                return Err(PyValueError::new_err("a turn has an edge index out of range"));
            }
        }
        let mut ids: HashMap<i64, usize> = HashMap::new();
        let mut index = |n: i64| {
            let k = ids.len();
            *ids.entry(n).or_insert(k)
        };
        let from = source.iter().map(|&n| index(n)).collect();
        let to = target.iter().map(|&n| index(n)).collect();
        let shapes = coords.into_iter().map(|s| s.into_iter().map(|(x, y)| [x, y]).collect()).collect();
        Ok(Network(Arc::new(c::Network::new(shapes, from, to, turns))))
    }

    fn __len__(&self) -> usize {
        self.0.shapes.len()
    }
}

/// Match one GPS trace. `x`, `y`: the positions in the CRS of the network; `ts`: the time of each point in
/// whole seconds, increasing. Returns the route as edge indexes, in driving order.
#[pyfunction]
#[pyo3(signature = (network, x, y, ts, *, speed=12.0, tau=3.0, corridor_min=30.0, corridor_gap=1.0, segment=true,
                    bearing_weight=0.0))]
#[allow(clippy::too_many_arguments)]
fn match_trace(
    py: Python<'_>,
    network: &Network,
    x: Vec<f64>,
    y: Vec<f64>,
    ts: Vec<i64>,
    speed: f64,
    tau: f64,
    corridor_min: f64,
    corridor_gap: f64,
    segment: bool,
    bearing_weight: f64,
) -> PyResult<Vec<usize>> {
    if y.len() != x.len() || ts.len() != x.len() {
        return Err(PyValueError::new_err("x, y and ts must have the same length"));
    }
    if x.is_empty() {
        return Ok(Vec::new());
    }
    if speed <= 0.0 || tau <= 0.0 {
        return Err(PyValueError::new_err("speed and tau must be more than 0"));
    }
    let p = c::GpsDtwParams { speed, tau, corridor_min, corridor_gap, segment, bearing_weight };
    let net = Arc::clone(&network.0);
    py.detach(|| c::match_gps_dtw(&net, &x, &y, &ts, &p)).map_err(PyValueError::new_err)
}

/// The input of the DTW for one trace: the cleaned GPS points, interpolated at 1 s (Bezier), with the
/// confidence weight of each point. Returns (x, y, ts, origin, weight); origin = a real GPS point.
#[pyfunction]
#[pyo3(signature = (x, y, ts, *, tau=3.0))]
#[allow(clippy::type_complexity)]
fn interpolate(x: Vec<f64>, y: Vec<f64>, ts: Vec<i64>, tau: f64) -> PyResult<(Vec<f64>, Vec<f64>, Vec<i64>, Vec<bool>, Vec<f64>)> {
    if y.len() != x.len() || ts.len() != x.len() {
        return Err(PyValueError::new_err("x, y and ts must have the same length"));
    }
    let (px, py, pt, po) = c::prepare(&x, &y, &ts).map_err(PyValueError::new_err)?;
    let w = c::weights(&pt, &po, tau);
    Ok((px, py, pt, po, w))
}

#[pymodule]
fn gps_dtw(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(match_trace, m)?)?;
    m.add_function(wrap_pyfunction!(interpolate, m)?)?;
    m.add_class::<Network>()?;
    Ok(())
}
