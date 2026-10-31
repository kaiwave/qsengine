//! Python bindings (PyO3). Compiled only with the `python` feature.
//!
//! This file is a thin translation layer: it converts Python arguments into
//! Rust calls on [`QuantumCircuit`] and converts results back. No simulation
//! logic lives here. The Python-facing docs are the docstrings below
//! (`///` comments become `__doc__`) and the type stub `qsengine.pyi`.
//!
//! The state vector never crosses into Python while gates are applied. Python
//! appends instructions, and `run` / `statevector` / `probabilities` execute the
//! whole circuit in Rust in one call, with the GIL released.

use crate::{BackendKind, QsError, QuantumCircuit, format_bitstring};
use num_complex::Complex64;
use numpy::{PyArray1, ToPyArray};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashMap;

/// Bad user input becomes `ValueError`; GPU problems become `RuntimeError`.
impl From<QsError> for PyErr {
    fn from(e: QsError) -> PyErr {
        match e {
            QsError::Gpu(_) => PyRuntimeError::new_err(e.to_string()),
            _ => PyValueError::new_err(e.to_string()),
        }
    }
}

/// Parse the `backend=` keyword argument.
fn parse_backend(name: &str) -> PyResult<BackendKind> {
    match name {
        "auto" => Ok(BackendKind::Auto),
        "cpu" => Ok(BackendKind::Cpu),
        "gpu" => Ok(BackendKind::Gpu),
        _ => Err(PyValueError::new_err(format!(
            "unknown backend {name:?}; expected \"auto\", \"cpu\" or \"gpu\""
        ))),
    }
}

/// A quantum circuit on a fixed number of qubits, starting in |0...0>.
///
/// Add gates with methods such as `h(0)` and `cx(0, 1)`, then call `run()`
/// for measurement counts or `statevector()` for the exact final state.
/// Qubit 0 is the rightmost character of every bitstring.
#[pyclass(name = "QuantumCircuit", module = "qsengine")]
struct PyQuantumCircuit {
    inner: QuantumCircuit,
}

#[pymethods]
impl PyQuantumCircuit {
    #[new]
    fn new(num_qubits: usize) -> PyResult<Self> {
        Ok(Self { inner: QuantumCircuit::new(num_qubits)? })
    }

    /// Number of qubits in the circuit.
    #[getter]
    fn num_qubits(&self) -> usize {
        self.inner.num_qubits()
    }

    /// Number of gates recorded so far.
    fn __len__(&self) -> usize {
        self.inner.instructions().len()
    }

    fn __repr__(&self) -> String {
        format!("QuantumCircuit(num_qubits={}, gates={})", self.inner.num_qubits(), self.__len__())
    }

    /// Pauli-X (NOT) on `qubit`.
    fn x(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.x(qubit)?) }
    /// Pauli-Y on `qubit`.
    fn y(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.y(qubit)?) }
    /// Pauli-Z (phase flip) on `qubit`.
    fn z(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.z(qubit)?) }
    /// Hadamard on `qubit`.
    fn h(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.h(qubit)?) }
    /// S gate (sqrt Z) on `qubit`.
    fn s(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.s(qubit)?) }
    /// T gate (fourth root of Z) on `qubit`.
    fn t(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.t(qubit)?) }
    /// Rotate `qubit` about the X axis by `theta` radians.
    fn rx(&mut self, qubit: usize, theta: f64) -> PyResult<()> { Ok(self.inner.rx(qubit, theta)?) }
    /// Rotate `qubit` about the Y axis by `theta` radians.
    fn ry(&mut self, qubit: usize, theta: f64) -> PyResult<()> { Ok(self.inner.ry(qubit, theta)?) }
    /// Rotate `qubit` about the Z axis by `theta` radians.
    fn rz(&mut self, qubit: usize, theta: f64) -> PyResult<()> { Ok(self.inner.rz(qubit, theta)?) }
    /// CNOT: flip `target` when `control` is |1>.
    fn cx(&mut self, control: usize, target: usize) -> PyResult<()> { Ok(self.inner.cx(control, target)?) }
    /// Controlled-Z: flip the phase of |11> on `control`, `target`.
    fn cz(&mut self, control: usize, target: usize) -> PyResult<()> { Ok(self.inner.cz(control, target)?) }
    /// S-dagger (inverse of S) on `qubit`.
    fn sdg(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.sdg(qubit)?) }
    /// T-dagger (inverse of T) on `qubit`.
    fn tdg(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.tdg(qubit)?) }
    /// Phase gate: multiply the |1> amplitude of `qubit` by e^(i*lam).
    fn p(&mut self, qubit: usize, lam: f64) -> PyResult<()> { Ok(self.inner.p(qubit, lam)?) }
    /// General single-qubit gate U(theta, phi, lam), the same convention as Qiskit.
    fn u(&mut self, qubit: usize, theta: f64, phi: f64, lam: f64) -> PyResult<()> {
        Ok(self.inner.u(qubit, theta, phi, lam)?)
    }
    /// Controlled phase: multiply the |11> amplitude of `control`, `target` by e^(i*lam).
    fn cp(&mut self, control: usize, target: usize, lam: f64) -> PyResult<()> { Ok(self.inner.cp(control, target, lam)?) }
    /// Controlled RZ: rotate `target` about Z by `theta` when `control` is |1>.
    fn crz(&mut self, control: usize, target: usize, theta: f64) -> PyResult<()> {
        Ok(self.inner.crz(control, target, theta)?)
    }
    /// Toffoli: flip `target` when `control1` and `control2` are both |1>.
    fn ccx(&mut self, control1: usize, control2: usize, target: usize) -> PyResult<()> {
        Ok(self.inner.ccx(control1, control2, target)?)
    }
    /// Swap the states of `qubit1` and `qubit2` (recorded as three CNOTs).
    fn swap(&mut self, qubit1: usize, qubit2: usize) -> PyResult<()> { Ok(self.inner.swap(qubit1, qubit2)?) }

    /// Apply any 2x2 unitary `matrix` (nested lists or a NumPy array) to `target`,
    /// optionally only where every qubit in `controls` is |1>.
    #[pyo3(signature = (matrix, target, controls = Vec::new()))]
    fn unitary(&mut self, matrix: Vec<Vec<Complex64>>, target: usize, controls: Vec<usize>) -> PyResult<()> {
        let gate = match matrix.as_slice() {
            [r0, r1] if r0.len() == 2 && r1.len() == 2 => [[r0[0], r0[1]], [r1[0], r1[1]]],
            _ => return Err(PyValueError::new_err("matrix must be 2x2")),
        };
        Ok(self.inner.unitary(&controls, target, gate)?)
    }

    /// Simulate the circuit and measure every qubit `shots` times.
    ///
    /// Returns a dict mapping bitstrings (qubit 0 rightmost) to counts, e.g.
    /// `{"00": 507, "11": 517}`. Pass `seed` for reproducible results.
    /// `backend` is "auto" (default), "cpu" or "gpu".
    #[pyo3(signature = (shots = 1024, seed = None, backend = "auto"))]
    fn run(&self, py: Python<'_>, shots: usize, seed: Option<u64>, backend: &str) -> PyResult<HashMap<String, usize>> {
        let kind = parse_backend(backend)?;
        let circuit = self.inner.clone();
        let n = circuit.num_qubits();
        // Release the GIL so other Python threads keep running during the simulation.
        let counts = py.detach(move || {
            let mut rng = match seed {
                Some(s) => StdRng::seed_from_u64(s),
                None => StdRng::from_os_rng(),
            };
            circuit.run(shots, kind, &mut rng)
        })?;
        Ok(counts.into_iter().map(|(i, c)| (format_bitstring(i, n), c)).collect())
    }

    /// Exact final state as a NumPy array of 2**n complex128 amplitudes,
    /// indexed by basis state.
    #[pyo3(signature = (backend = "auto"))]
    fn statevector<'py>(&self, py: Python<'py>, backend: &str) -> PyResult<Bound<'py, PyArray1<Complex64>>> {
        let kind = parse_backend(backend)?;
        let circuit = self.inner.clone();
        let amps = py.detach(move || circuit.statevector(kind))?;
        Ok(amps.to_pyarray(py))
    }

    /// Exact probability of each of the 2**n basis states after the circuit,
    /// as a NumPy float64 array.
    #[pyo3(signature = (backend = "auto"))]
    fn probabilities<'py>(&self, py: Python<'py>, backend: &str) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let kind = parse_backend(backend)?;
        let circuit = self.inner.clone();
        let probs = py.detach(move || circuit.probabilities(kind))?;
        Ok(probs.to_pyarray(py))
    }
}

/// qsengine: a fast state-vector quantum simulator with a Rust core.
#[pymodule]
fn qsengine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyQuantumCircuit>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
