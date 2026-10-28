//! [`QuantumCircuit`]: a list of gates, built first and executed later.
//!
//! Building and running are separate steps. Gate calls like `h(0)` only record an
//! [`Instruction`] (validated straight away, so mistakes surface where they were
//! made). [`run`](QuantumCircuit::run) then picks a backend, applies every
//! instruction and samples the result.
//!
//! This split is what keeps the Python layer fast: Python only appends small
//! instructions, and the whole simulation happens in Rust in one call, with the
//! state vector never crossing into Python.

use crate::backend::{QuantumBackend, control_mask};
use crate::gates::{self, Gate};
use crate::{BackendKind, MAX_QUBITS, QsError, Result};
use num_complex::Complex64;
use rand::RngCore;
use std::collections::HashMap;

/// One recorded gate: apply `gate` to `target` where all `control_mask` bits are 1.
/// These are the exact arguments of [`QuantumBackend::apply_gate_kernel`].
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub control_mask: usize,
    pub target: usize,
    pub gate: Gate,
}

/// An ordered list of gates on a fixed number of qubits, all starting in |0...0>.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumCircuit {
    num_qubits: usize,
    instructions: Vec<Instruction>,
}

impl QuantumCircuit {
    /// An empty circuit on `num_qubits` qubits.
    pub fn new(num_qubits: usize) -> Result<Self> {
        if num_qubits > MAX_QUBITS {
            return Err(QsError::TooManyQubits { requested: num_qubits, max: MAX_QUBITS });
        }
        Ok(Self { num_qubits, instructions: Vec::new() })
    }

    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// The recorded gates, in order.
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    /// Record `gate` on `target`, controlled on every qubit in `controls`
    /// (empty for a plain single-qubit gate). Every other gate method calls this.
    pub fn add_gate(&mut self, controls: &[usize], target: usize, gate: Gate) -> Result<()> {
        let control_mask = control_mask(self.num_qubits, controls, target)?;
        self.instructions.push(Instruction { control_mask, target, gate });
        Ok(())
    }

    // Named gates: same names and meaning as on `QuantumBackend`.
    pub fn x(&mut self, q: usize) -> Result<()> { self.add_gate(&[], q, gates::X) }
    pub fn y(&mut self, q: usize) -> Result<()> { self.add_gate(&[], q, gates::Y) }
    pub fn z(&mut self, q: usize) -> Result<()> { self.add_gate(&[], q, gates::Z) }
    pub fn h(&mut self, q: usize) -> Result<()> { self.add_gate(&[], q, gates::H) }
    pub fn s(&mut self, q: usize) -> Result<()> { self.add_gate(&[], q, gates::S) }
    pub fn t(&mut self, q: usize) -> Result<()> { self.add_gate(&[], q, gates::T) }
    pub fn rx(&mut self, q: usize, theta: f64) -> Result<()> { self.add_gate(&[], q, gates::rx(theta)) }
    pub fn ry(&mut self, q: usize, theta: f64) -> Result<()> { self.add_gate(&[], q, gates::ry(theta)) }
    pub fn rz(&mut self, q: usize, theta: f64) -> Result<()> { self.add_gate(&[], q, gates::rz(theta)) }
    pub fn cx(&mut self, control: usize, target: usize) -> Result<()> {
        self.add_gate(&[control], target, gates::X)
    }
    pub fn cz(&mut self, control: usize, target: usize) -> Result<()> {
        self.add_gate(&[control], target, gates::Z)
    }

    /// Apply every instruction to an existing backend, in order.
    pub fn apply_to(&self, backend: &mut dyn QuantumBackend) -> Result<()> {
        if backend.num_qubits() != self.num_qubits {
            return Err(QsError::SizeMismatch { circuit: self.num_qubits, backend: backend.num_qubits() });
        }
        for ins in &self.instructions {
            // Already validated in `add_gate`, so call the unchecked kernel directly.
            backend.apply_gate_kernel(ins.control_mask, ins.target, &ins.gate);
        }
        Ok(())
    }

    /// Create a fresh |0...0> backend of the requested kind and run the circuit on it.
    pub fn simulate(&self, kind: BackendKind) -> Result<Box<dyn QuantumBackend + Send>> {
        let mut backend = kind.create(self.num_qubits)?;
        self.apply_to(backend.as_mut())?;
        Ok(backend)
    }

    /// Final amplitudes after running the circuit.
    pub fn statevector(&self, kind: BackendKind) -> Result<Vec<Complex64>> {
        Ok(self.simulate(kind)?.amplitudes())
    }

    /// Final basis-state probabilities after running the circuit.
    pub fn probabilities(&self, kind: BackendKind) -> Result<Vec<f64>> {
        Ok(self.simulate(kind)?.probabilities())
    }

    /// Run once, then measure all qubits `shots` times.
    /// Returns `basis index -> count`; see [`crate::format_bitstring`].
    pub fn run(&self, shots: usize, kind: BackendKind, rng: &mut dyn RngCore) -> Result<HashMap<usize, usize>> {
        Ok(self.simulate(kind)?.sample(shots, rng))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QuantumState;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn circuit_matches_direct_backend_calls() {
        let mut qc = QuantumCircuit::new(3).unwrap();
        qc.h(0).unwrap();
        qc.cx(0, 1).unwrap();
        qc.rz(2, 0.7).unwrap();
        qc.cz(1, 2).unwrap();

        let mut direct = QuantumState::new(3).unwrap();
        direct.h(0).unwrap();
        direct.cx(0, 1).unwrap();
        direct.rz(2, 0.7).unwrap();
        direct.cz(1, 2).unwrap();

        assert_eq!(qc.statevector(BackendKind::Cpu).unwrap(), direct.amplitudes);
    }

    #[test]
    fn bad_gates_are_rejected_when_added() {
        let mut qc = QuantumCircuit::new(2).unwrap();
        assert_eq!(qc.h(2), Err(QsError::QubitOutOfRange { qubit: 2, num_qubits: 2 }));
        assert_eq!(qc.cx(0, 0), Err(QsError::DuplicateQubit(0)));
        assert!(qc.instructions().is_empty());
    }

    #[test]
    fn run_samples_bell_state() {
        let mut qc = QuantumCircuit::new(2).unwrap();
        qc.h(0).unwrap();
        qc.cx(0, 1).unwrap();
        let counts = qc.run(1000, BackendKind::Cpu, &mut StdRng::seed_from_u64(1)).unwrap();
        assert_eq!(counts.values().sum::<usize>(), 1000);
        assert!(counts.keys().all(|&k| k == 0 || k == 3));
    }

    #[test]
    fn size_mismatch_is_an_error() {
        let qc = QuantumCircuit::new(2).unwrap();
        let mut s = QuantumState::new(3).unwrap();
        assert_eq!(qc.apply_to(&mut s), Err(QsError::SizeMismatch { circuit: 2, backend: 3 }));
    }
}
