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

use crate::backend::{QuantumBackend, check_finite, control_mask};
use crate::gates::{self, Gate};
use crate::{BackendKind, MAX_QUBITS, QsError, Result};
use num_complex::Complex64;
use rand::RngCore;
use std::collections::HashMap;
use std::fmt;

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
    drawing: Vec<DrawOp>,
}

/// One gate as the user wrote it, kept only for drawing: `swap` is one entry
/// here but three CNOTs in `instructions`.
#[derive(Debug, Clone, PartialEq)]
struct DrawOp {
    label: String,
    controls: Vec<usize>,
    targets: Vec<usize>,
}

/// Drawing label for a gate with angle parameters, e.g. `RX(1.57)`.
fn label(name: &str, angles: &[f64]) -> String {
    let args: Vec<String> = angles.iter().map(|a| format!("{a:.2}")).collect();
    format!("{name}({})", args.join(","))
}

impl QuantumCircuit {
    /// An empty circuit on `num_qubits` qubits.
    pub fn new(num_qubits: usize) -> Result<Self> {
        if num_qubits > MAX_QUBITS {
            return Err(QsError::TooManyQubits { requested: num_qubits, max: MAX_QUBITS });
        }
        Ok(Self { num_qubits, instructions: Vec::new(), drawing: Vec::new() })
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
        self.push("U".into(), controls, target, gate)
    }

    /// Validate and record one gate, with the label [`Display`](fmt::Display) draws for it.
    fn push(&mut self, label: String, controls: &[usize], target: usize, gate: Gate) -> Result<()> {
        let control_mask = control_mask(self.num_qubits, controls, target)?;
        check_finite(&gate)?;
        self.instructions.push(Instruction { control_mask, target, gate });
        self.drawing.push(DrawOp { label, controls: controls.to_vec(), targets: vec![target] });
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
    pub fn x(&mut self, q: usize) -> Result<()> { self.push("X".into(), &[], q, gates::X) }
    pub fn y(&mut self, q: usize) -> Result<()> { self.push("Y".into(), &[], q, gates::Y) }
    pub fn z(&mut self, q: usize) -> Result<()> { self.push("Z".into(), &[], q, gates::Z) }
    pub fn h(&mut self, q: usize) -> Result<()> { self.push("H".into(), &[], q, gates::H) }
    pub fn s(&mut self, q: usize) -> Result<()> { self.push("S".into(), &[], q, gates::S) }
    pub fn t(&mut self, q: usize) -> Result<()> { self.push("T".into(), &[], q, gates::T) }
    pub fn rx(&mut self, q: usize, theta: f64) -> Result<()> { self.push(label("RX", &[theta]), &[], q, gates::rx(theta)) }
    pub fn ry(&mut self, q: usize, theta: f64) -> Result<()> { self.push(label("RY", &[theta]), &[], q, gates::ry(theta)) }
    pub fn rz(&mut self, q: usize, theta: f64) -> Result<()> { self.push(label("RZ", &[theta]), &[], q, gates::rz(theta)) }
    pub fn cx(&mut self, control: usize, target: usize) -> Result<()> {
        self.push("X".into(), &[control], target, gates::X)
    }
    pub fn cz(&mut self, control: usize, target: usize) -> Result<()> {
        self.push("Z".into(), &[control], target, gates::Z)
    }
    pub fn sdg(&mut self, q: usize) -> Result<()> { self.push("S†".into(), &[], q, gates::SDG) }
    pub fn tdg(&mut self, q: usize) -> Result<()> { self.push("T†".into(), &[], q, gates::TDG) }
    pub fn p(&mut self, q: usize, lambda: f64) -> Result<()> { self.push(label("P", &[lambda]), &[], q, gates::p(lambda)) }
    pub fn u(&mut self, q: usize, theta: f64, phi: f64, lambda: f64) -> Result<()> {
        self.push(label("U", &[theta, phi, lambda]), &[], q, gates::u(theta, phi, lambda))
    }
    pub fn cp(&mut self, control: usize, target: usize, lambda: f64) -> Result<()> {
        self.push(label("P", &[lambda]), &[control], target, gates::p(lambda))
    }
    pub fn crz(&mut self, control: usize, target: usize, theta: f64) -> Result<()> {
        self.push(label("RZ", &[theta]), &[control], target, gates::rz(theta))
    }
    /// Toffoli: flip `target` when both controls are |1>.
    pub fn ccx(&mut self, c1: usize, c2: usize, target: usize) -> Result<()> {
        self.push("X".into(), &[c1, c2], target, gates::X)
    }
    /// Swap two qubits, as three CNOTs (the kernel only knows controlled 2x2 gates).
    pub fn swap(&mut self, a: usize, b: usize) -> Result<()> {
        // Validate first so a bad swap records nothing, not one or two stray CNOTs.
        control_mask(self.num_qubits, &[a], b)?;
        for (c, t) in [(a, b), (b, a), (a, b)] {
            self.instructions.push(Instruction { control_mask: 1 << c, target: t, gate: gates::X });
        }
        self.drawing.push(DrawOp { label: "×".into(), controls: vec![], targets: vec![a, b] });
        Ok(())
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
        Ok(self.simulate(kind)?.into_amplitudes())
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

/// Text drawing, one wire per qubit (q0 on top), one column per gate:
///
/// ```text
/// q0: ──H──●─
///          │
/// q1: ─────X─
/// ```
///
/// `●` marks a control, `×` the two ends of a swap.
///
/// shortcut: columns are never packed or folded, so a long circuit draws as one very wide line; add folding if learners hit it.
impl fmt::Display for QuantumCircuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.num_qubits;
        let name_w = format!("q{}", n.saturating_sub(1)).len();
        // Even rows are qubit wires, odd rows the gaps between them.
        let mut rows: Vec<String> = (0..(2 * n).saturating_sub(1))
            .map(|r| if r % 2 == 0 { format!("{:>name_w$}: ─", format!("q{}", r / 2)) } else { " ".repeat(name_w + 3) })
            .collect();

        for op in &self.drawing {
            let w = op.label.chars().count() + 2;
            let mid = w / 2;
            // Rows from the gate's top qubit to its bottom one carry the vertical line.
            let qubits = op.controls.iter().chain(&op.targets);
            let (lo, hi) = (2 * qubits.clone().min().unwrap(), 2 * qubits.max().unwrap());
            let mark = |c: char, fill: &str| format!("{}{c}{}", fill.repeat(mid), fill.repeat(w - mid - 1));
            for (r, row) in rows.iter_mut().enumerate() {
                let (q, wire, inside) = (r / 2, r % 2 == 0, lo <= r && r <= hi);
                let cell = match (wire, inside) {
                    (true, _) if op.targets.contains(&q) => format!("─{}─", op.label),
                    (true, _) if op.controls.contains(&q) => mark('●', "─"),
                    (true, true) => mark('┼', "─"),
                    (true, false) => "─".repeat(w),
                    (false, true) => mark('│', " "),
                    (false, false) => " ".repeat(w),
                };
                row.push_str(&cell);
            }
        }
        let lines: Vec<&str> = rows.iter().map(|r| r.trim_end()).collect();
        f.write_str(&lines.join("\n"))
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
    fn non_finite_angles_are_rejected() {
        let mut qc = QuantumCircuit::new(2).unwrap();
        assert_eq!(qc.rx(0, f64::NAN), Err(QsError::NonFinite));
        assert_eq!(qc.cp(0, 1, f64::INFINITY), Err(QsError::NonFinite));
        assert!(qc.instructions().is_empty());
    }

    #[test]
    fn draws_gates_controls_and_swaps() {
        let mut qc = QuantumCircuit::new(3).unwrap();
        qc.h(0).unwrap();
        qc.cx(0, 2).unwrap();
        qc.rz(1, 0.5).unwrap();
        qc.swap(0, 1).unwrap();
        let expect = "\
q0: ──H──●────────────×─
         │            │
q1: ─────┼──RZ(0.50)──×─
         │
q2: ─────X──────────────";
        assert_eq!(qc.to_string(), expect);
        assert_eq!(qc.instructions().len(), 6); // the swap is still three CNOTs
        assert_eq!(QuantumCircuit::new(1).unwrap().to_string(), "q0: ─");
    }

    #[test]
    fn size_mismatch_is_an_error() {
        let qc = QuantumCircuit::new(2).unwrap();
        let mut s = QuantumState::new(3).unwrap();
        assert_eq!(qc.apply_to(&mut s), Err(QsError::SizeMismatch { circuit: 2, backend: 3 }));
    }
}
