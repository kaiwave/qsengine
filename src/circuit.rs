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

    /// Record a user-supplied 2x2 `gate`, after checking it is unitary.
    /// [`add_gate`](Self::add_gate) trusts its matrix; this is the entry point for untrusted input.
    pub fn unitary(&mut self, controls: &[usize], target: usize, gate: Gate) -> Result<()> {
        if !gates::is_unitary(&gate, 1e-9) {
            return Err(QsError::NotUnitary);
        }
        self.add_gate(controls, target, gate)
    }

    // Named gates. The first eleven match `QuantumBackend`; the rest are circuit-only.
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
    pub fn sdg(&mut self, q: usize) -> Result<()> { self.add_gate(&[], q, gates::SDG) }
    pub fn tdg(&mut self, q: usize) -> Result<()> { self.add_gate(&[], q, gates::TDG) }
    pub fn p(&mut self, q: usize, lambda: f64) -> Result<()> { self.add_gate(&[], q, gates::p(lambda)) }
    pub fn u(&mut self, q: usize, theta: f64, phi: f64, lambda: f64) -> Result<()> {
        self.add_gate(&[], q, gates::u(theta, phi, lambda))
    }
    pub fn cp(&mut self, control: usize, target: usize, lambda: f64) -> Result<()> {
        self.add_gate(&[control], target, gates::p(lambda))
    }
    pub fn crz(&mut self, control: usize, target: usize, theta: f64) -> Result<()> {
        self.add_gate(&[control], target, gates::rz(theta))
    }
    /// Toffoli: flip `target` when both controls are |1>.
    pub fn ccx(&mut self, c1: usize, c2: usize, target: usize) -> Result<()> {
        self.add_gate(&[c1, c2], target, gates::X)
    }
    /// Swap two qubits, as three CNOTs (the kernel only knows controlled 2x2 gates).
    pub fn swap(&mut self, a: usize, b: usize) -> Result<()> {
        // Validate first so a bad swap records nothing, not one or two stray CNOTs.
        control_mask(self.num_qubits, &[a], b)?;
        self.cx(a, b)?;
        self.cx(b, a)?;
        self.cx(a, b)
    }

    /// Apply every instruction to an existing backend, with single-qubit gate fusion.
    ///
    /// Each gate is one full pass over the `2^n` amplitudes, and at large `n` that
    /// pass is limited by memory bandwidth, not arithmetic. So consecutive
    /// single-qubit gates on the same qubit are multiplied into one 2x2 matrix first
    /// (`H` then `RZ` becomes one gate), saving whole passes.
    ///
    /// This is exact because gates on *different* qubits commute. A qubit's
    /// pending matrix only has to be applied before the next multi-qubit gate that
    /// touches that qubit, or at the end.
    pub fn apply_to(&self, backend: &mut dyn QuantumBackend) -> Result<()> {
        if backend.num_qubits() != self.num_qubits {
            return Err(QsError::SizeMismatch { circuit: self.num_qubits, backend: backend.num_qubits() });
        }

        // pending[q] = product of single-qubit gates on q not yet sent to the backend.
        let mut pending: Vec<Option<Gate>> = vec![None; self.num_qubits];
        // Instructions were validated in `add_gate`, so the unchecked kernel is safe to call.
        let flush = |q: usize, backend: &mut dyn QuantumBackend, pending: &mut [Option<Gate>]| {
            if let Some(g) = pending[q].take() {
                backend.apply_gate_kernel(0, q, &g);
            }
        };

        for ins in &self.instructions {
            if ins.control_mask == 0 {
                let slot = &mut pending[ins.target];
                *slot = Some(match slot.take() {
                    Some(earlier) => gates::matmul(&ins.gate, &earlier),
                    None => ins.gate,
                });
            } else {
                // Flush every qubit this gate touches: the target and each control bit.
                flush(ins.target, backend, &mut pending);
                let mut mask = ins.control_mask;
                while mask != 0 {
                    flush(mask.trailing_zeros() as usize, backend, &mut pending);
                    mask &= mask - 1; // clear the lowest set bit
                }
                backend.apply_gate_kernel(ins.control_mask, ins.target, &ins.gate);
            }
        }
        for q in 0..self.num_qubits {
            flush(q, backend, &mut pending);
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
    fn fused_circuit_matches_gate_by_gate() {
        // Long runs of single-qubit gates (fused) interleaved with controlled gates
        // (which must flush the right qubits), including a two-control gate.
        let mut qc = QuantumCircuit::new(4).unwrap();
        let mut direct = QuantumState::new(4).unwrap();
        for layer in 0..3 {
            for q in 0..4 {
                let theta = 0.3 * (q + layer) as f64;
                qc.h(q).unwrap();
                qc.t(q).unwrap();
                qc.ry(q, theta).unwrap();
                direct.h(q).unwrap();
                direct.t(q).unwrap();
                direct.ry(q, theta).unwrap();
            }
            qc.cx(0, 1).unwrap();
            qc.cz(2, 3).unwrap();
            qc.add_gate(&[1, 3], 0, gates::X).unwrap();
            direct.cx(0, 1).unwrap();
            direct.cz(2, 3).unwrap();
            direct.apply_multi_controlled_gate(&[1, 3], 0, &gates::X).unwrap();
            qc.s(2).unwrap();
            direct.s(2).unwrap();
        }

        let fused = qc.statevector(BackendKind::Cpu).unwrap();
        let worst = fused.iter().zip(&direct.amplitudes).map(|(a, b)| (a - b).norm()).fold(0.0, f64::max);
        assert!(worst < 1e-12, "fusion changed the result by {worst}");
    }

    #[test]
    fn bad_gates_are_rejected_when_added() {
        let mut qc = QuantumCircuit::new(2).unwrap();
        assert_eq!(qc.h(2), Err(QsError::QubitOutOfRange { qubit: 2, num_qubits: 2 }));
        assert_eq!(qc.cx(0, 0), Err(QsError::DuplicateQubit(0)));
        assert_eq!(qc.swap(1, 1), Err(QsError::DuplicateQubit(1)));
        assert_eq!(qc.swap(0, 5), Err(QsError::QubitOutOfRange { qubit: 5, num_qubits: 2 }));
        assert_eq!(qc.unitary(&[], 0, [[gates::H[0][0]; 2]; 2]), Err(QsError::NotUnitary));
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
