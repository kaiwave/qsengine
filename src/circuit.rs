//! [`QuantumCircuit`]: a list of operations, built first and executed later.
//!
//! Building and running are separate steps. Gate calls like `h(0)` only record an
//! [`Instruction`] (validated straight away, so mistakes surface where they were
//! made). [`run`](QuantumCircuit::run) then picks a backend, applies every
//! instruction and samples the result.
//!
//! This split is what keeps the Python layer fast: Python only appends small
//! instructions, and the whole simulation happens in Rust in one call, with the
//! state vector never crossing into Python.
//!
//! ## Two ways to run
//!
//! A circuit whose measurements all come at the end is simulated **once**, then
//! sampled `shots` times. A circuit that measures or resets a qubit and then keeps
//! going (mid-circuit measurement) has a random state, so each shot is simulated
//! on its own; see [`run`](QuantumCircuit::run).

use crate::backend::{QuantumBackend, check_finite, control_mask};
use crate::gates::{self, Gate};
use crate::pauli::Pauli;
use crate::{BackendKind, GPU_MIN_QUBITS_DYNAMIC, MAX_CLBITS, MAX_QUBITS, QsError, Result};
use num_complex::Complex64;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use std::collections::HashMap;

/// A gate's name and angles as the user wrote it, e.g. `rx` and `[0.5]`.
/// Kept for drawing (in Python) and later for export; never formatted while recording.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Label {
    /// The method that recorded it: `"h"`, `"cx"`, `"rz"`, `"unitary"`, ...
    pub name: &'static str,
    params: [f64; 3],
    num_params: u8,
}

impl Label {
    fn new(name: &'static str, angles: &[f64]) -> Self {
        let mut params = [0.0; 3];
        params[..angles.len()].copy_from_slice(angles);
        Label { name, params, num_params: angles.len() as u8 }
    }

    /// The gate's angles, in the order its method takes them.
    pub fn params(&self) -> &[f64] {
        &self.params[..self.num_params as usize]
    }
}

/// A classical condition: the gate runs only when `clbits & mask == value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Condition {
    pub mask: usize,
    pub value: usize,
}

impl Condition {
    fn holds(&self, clbits: usize) -> bool {
        clbits & self.mask == self.value
    }
}

/// One recorded operation.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Apply `gate` to `target` where all `control_mask` bits are 1: the exact
    /// arguments of [`QuantumBackend::apply_gate_kernel`]. With a `condition`,
    /// only in shots whose classical bits match it.
    Gate { control_mask: usize, target: usize, gate: Gate, label: Label, condition: Option<Condition> },
    /// Swap two qubits; run as three CNOTs, kept as one operation for drawing.
    Swap { a: usize, b: usize, condition: Option<Condition> },
    /// Measure `qubit` and store the outcome in classical bit `clbit`.
    Measure { qubit: usize, clbit: usize },
    /// Put `qubit` back to |0>.
    Reset { qubit: usize },
}

/// An ordered list of operations on a fixed number of qubits, all starting in
/// |0...0>, and of classical bits, all starting at 0.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumCircuit {
    num_qubits: usize,
    num_clbits: usize,
    instructions: Vec<Instruction>,
    /// Attached to every gate recorded while set (Python's `with qc.if_test(...)`).
    condition: Option<Condition>,
}

impl QuantumCircuit {
    /// An empty circuit on `num_qubits` qubits and no classical bits.
    pub fn new(num_qubits: usize) -> Result<Self> {
        Self::with_clbits(num_qubits, 0)
    }

    /// An empty circuit on `num_qubits` qubits and `num_clbits` classical bits.
    pub fn with_clbits(num_qubits: usize, num_clbits: usize) -> Result<Self> {
        if num_qubits > MAX_QUBITS {
            return Err(QsError::TooManyQubits { requested: num_qubits, max: MAX_QUBITS });
        }
        if num_clbits > MAX_CLBITS {
            return Err(QsError::InvalidArgument(format!(
                "requested {num_clbits} classical bits, maximum is {MAX_CLBITS}"
            )));
        }
        Ok(Self { num_qubits, num_clbits, instructions: Vec::new(), condition: None })
    }

    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    pub fn num_clbits(&self) -> usize {
        self.num_clbits
    }

    /// The recorded operations, in order.
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    /// True if the circuit measures into classical bits. Then [`run`](Self::run)
    /// counts classical-bit values; otherwise it measures every qubit at the end.
    pub fn has_measurements(&self) -> bool {
        self.instructions.iter().any(|i| matches!(i, Instruction::Measure { .. }))
    }

    /// Record `gate` on `target`, controlled on every qubit in `controls`
    /// (empty for a plain single-qubit gate).
    pub fn add_gate(&mut self, controls: &[usize], target: usize, gate: Gate) -> Result<()> {
        self.push(Label::new("unitary", &[]), controls, target, gate)
    }

    /// Validate and record one gate. No allocation: this is the hot path of building a circuit.
    fn push(&mut self, label: Label, controls: &[usize], target: usize, gate: Gate) -> Result<()> {
        let control_mask = control_mask(self.num_qubits, controls, target)?;
        check_finite(&gate)?;
        self.instructions.push(Instruction::Gate { control_mask, target, gate, label, condition: self.condition });
        Ok(())
    }

    /// Record a user-supplied 2x2 `gate`, after checking it is unitary to within 1e-9.
    /// The matrix recorded is the nearest exact unitary, so the tolerated rounding
    /// error cannot change the state's norm and add up over a long circuit.
    /// [`add_gate`](Self::add_gate) trusts its matrix; this is the entry point for untrusted input.
    pub fn unitary(&mut self, controls: &[usize], target: usize, gate: Gate) -> Result<()> {
        if !gates::is_unitary(&gate, 1e-9) {
            return Err(QsError::NotUnitary);
        }
        self.add_gate(controls, target, gates::nearest_unitary(&gate))
    }

    // Named gates. The first eleven match `QuantumBackend`; the rest are circuit-only.
    pub fn x(&mut self, q: usize) -> Result<()> { self.push(Label::new("x", &[]), &[], q, gates::X) }
    pub fn y(&mut self, q: usize) -> Result<()> { self.push(Label::new("y", &[]), &[], q, gates::Y) }
    pub fn z(&mut self, q: usize) -> Result<()> { self.push(Label::new("z", &[]), &[], q, gates::Z) }
    pub fn h(&mut self, q: usize) -> Result<()> { self.push(Label::new("h", &[]), &[], q, gates::H) }
    pub fn s(&mut self, q: usize) -> Result<()> { self.push(Label::new("s", &[]), &[], q, gates::S) }
    pub fn t(&mut self, q: usize) -> Result<()> { self.push(Label::new("t", &[]), &[], q, gates::T) }
    pub fn rx(&mut self, q: usize, theta: f64) -> Result<()> { self.push(Label::new("rx", &[theta]), &[], q, gates::rx(theta)) }
    pub fn ry(&mut self, q: usize, theta: f64) -> Result<()> { self.push(Label::new("ry", &[theta]), &[], q, gates::ry(theta)) }
    pub fn rz(&mut self, q: usize, theta: f64) -> Result<()> { self.push(Label::new("rz", &[theta]), &[], q, gates::rz(theta)) }
    pub fn cx(&mut self, control: usize, target: usize) -> Result<()> {
        self.push(Label::new("cx", &[]), &[control], target, gates::X)
    }
    pub fn cz(&mut self, control: usize, target: usize) -> Result<()> {
        self.push(Label::new("cz", &[]), &[control], target, gates::Z)
    }
    pub fn sdg(&mut self, q: usize) -> Result<()> { self.push(Label::new("sdg", &[]), &[], q, gates::SDG) }
    pub fn tdg(&mut self, q: usize) -> Result<()> { self.push(Label::new("tdg", &[]), &[], q, gates::TDG) }
    pub fn p(&mut self, q: usize, lambda: f64) -> Result<()> { self.push(Label::new("p", &[lambda]), &[], q, gates::p(lambda)) }
    pub fn u(&mut self, q: usize, theta: f64, phi: f64, lambda: f64) -> Result<()> {
        self.push(Label::new("u", &[theta, phi, lambda]), &[], q, gates::u(theta, phi, lambda))
    }
    pub fn cp(&mut self, control: usize, target: usize, lambda: f64) -> Result<()> {
        self.push(Label::new("cp", &[lambda]), &[control], target, gates::p(lambda))
    }
    pub fn crz(&mut self, control: usize, target: usize, theta: f64) -> Result<()> {
        self.push(Label::new("crz", &[theta]), &[control], target, gates::rz(theta))
    }
    /// Toffoli: flip `target` when both controls are |1>.
    pub fn ccx(&mut self, c1: usize, c2: usize, target: usize) -> Result<()> {
        self.push(Label::new("ccx", &[]), &[c1, c2], target, gates::X)
    }
    /// Swap two qubits (run as three CNOTs: the kernel only knows controlled 2x2 gates).
    pub fn swap(&mut self, a: usize, b: usize) -> Result<()> {
        control_mask(self.num_qubits, &[a], b)?;
        self.instructions.push(Instruction::Swap { a, b, condition: self.condition });
        Ok(())
    }

    /// Measure `qubit` into classical bit `clbit`. Later gates may depend on it
    /// through a condition; a later measurement into the same bit overwrites it.
    pub fn measure(&mut self, qubit: usize, clbit: usize) -> Result<()> {
        self.check_unconditioned("measure")?;
        control_mask(self.num_qubits, &[], qubit)?;
        self.check_clbit(clbit)?;
        self.instructions.push(Instruction::Measure { qubit, clbit });
        Ok(())
    }

    /// Reset `qubit` to |0>, whatever state it is in.
    pub fn reset(&mut self, qubit: usize) -> Result<()> {
        self.check_unconditioned("reset")?;
        control_mask(self.num_qubits, &[], qubit)?;
        self.instructions.push(Instruction::Reset { qubit });
        Ok(())
    }

    /// Make every gate recorded from now on conditional: it runs only in shots
    /// where `clbits[i]` equals bit `i` of `value`, for every `i`. Undo with
    /// [`clear_condition`](Self::clear_condition). Conditions do not nest.
    pub fn set_condition(&mut self, clbits: &[usize], value: usize) -> Result<()> {
        if self.condition.is_some() {
            return Err(QsError::InvalidArgument("if_test blocks cannot be nested".into()));
        }
        if clbits.is_empty() {
            return Err(QsError::InvalidArgument("a condition needs at least one classical bit".into()));
        }
        if clbits.len() < usize::BITS as usize && value >> clbits.len() != 0 {
            return Err(QsError::InvalidArgument(format!(
                "condition value {value} does not fit in {} classical bits",
                clbits.len()
            )));
        }
        let mut cond = Condition { mask: 0, value: 0 };
        for (i, &c) in clbits.iter().enumerate() {
            self.check_clbit(c)?;
            if cond.mask >> c & 1 == 1 {
                return Err(QsError::InvalidArgument(format!("classical bit {c} used more than once in one condition")));
            }
            cond.mask |= 1 << c;
            cond.value |= (value >> i & 1) << c;
        }
        self.condition = Some(cond);
        Ok(())
    }

    /// End the condition started by [`set_condition`](Self::set_condition).
    pub fn clear_condition(&mut self) {
        self.condition = None;
    }

    fn check_clbit(&self, clbit: usize) -> Result<()> {
        if clbit < self.num_clbits { Ok(()) } else { Err(QsError::ClbitOutOfRange { clbit, num_clbits: self.num_clbits }) }
    }

    fn check_unconditioned(&self, what: &str) -> Result<()> {
        match self.condition {
            None => Ok(()),
            Some(_) => Err(QsError::InvalidArgument(format!("{what} cannot be inside if_test; only gates can"))),
        }
    }

    /// Index where the final run of measurements starts (the length if there is none).
    fn trailing_measures_start(&self) -> usize {
        self.instructions.iter().rposition(|i| !matches!(i, Instruction::Measure { .. })).map_or(0, |p| p + 1)
    }

    /// True if a measurement or reset comes before more work, so the state is random
    /// part-way and each shot must be simulated on its own.
    pub fn is_dynamic(&self) -> bool {
        let end = self.trailing_measures_start();
        self.instructions[..end].iter().any(|i| matches!(i, Instruction::Measure { .. } | Instruction::Reset { .. }))
    }

    /// Apply the circuit to an existing backend. Measurements at the very end are
    /// skipped, so this gives the state just before them.
    ///
    /// Fails with [`QsError::MidCircuitMeasurement`] if the circuit measures or
    /// resets part-way: its state is then different in every shot.
    pub fn apply_to(&self, backend: &mut dyn QuantumBackend) -> Result<()> {
        if backend.num_qubits() != self.num_qubits {
            return Err(QsError::SizeMismatch { circuit: self.num_qubits, backend: backend.num_qubits() });
        }
        if self.is_dynamic() {
            return Err(QsError::MidCircuitMeasurement);
        }
        // No measurement runs, so the random generator is never used.
        let end = self.trailing_measures_start();
        self.execute(&self.instructions[..end], backend, &mut 0, &mut StdRng::seed_from_u64(0))
    }

    /// Run `instructions` on `backend`, updating `clbits`, with single-qubit gate fusion.
    ///
    /// Each gate is one full pass over the `2^n` amplitudes, and at large `n` that
    /// pass is limited by memory bandwidth, not arithmetic. So consecutive
    /// single-qubit gates on the same qubit are multiplied into one 2x2 matrix first
    /// (`H` then `RZ` becomes one gate), saving whole passes.
    ///
    /// This is exact because operations on *different* qubits commute. A qubit's
    /// pending matrix only has to be applied before the next multi-qubit gate,
    /// measurement or reset that touches that qubit, or at the end. Conditions are
    /// checked in order, so `clbits` always holds every earlier measurement.
    fn execute(
        &self,
        instructions: &[Instruction],
        backend: &mut dyn QuantumBackend,
        clbits: &mut usize,
        rng: &mut dyn RngCore,
    ) -> Result<()> {
        // pending[q] = product of single-qubit gates on q not yet sent to the backend.
        let mut pending: Vec<Option<Gate>> = vec![None; self.num_qubits];
        // Instructions were validated when recorded, so the unchecked kernel is safe to call.
        let flush = |q: usize, backend: &mut dyn QuantumBackend, pending: &mut [Option<Gate>]| {
            if let Some(g) = pending[q].take() {
                backend.apply_gate_kernel(0, q, &g);
            }
        };
        let skipped = |condition: &Option<Condition>, clbits: usize| condition.is_some_and(|c| !c.holds(clbits));

        for ins in instructions {
            match ins {
                Instruction::Gate { condition, .. } | Instruction::Swap { condition, .. } if skipped(condition, *clbits) => {}
                &Instruction::Gate { control_mask: 0, target, ref gate, .. } => {
                    let slot = &mut pending[target];
                    *slot = Some(match slot.take() {
                        Some(earlier) => gates::matmul(gate, &earlier),
                        None => *gate,
                    });
                }
                &Instruction::Gate { control_mask, target, ref gate, .. } => {
                    // Flush every qubit this gate touches: the target and each control bit.
                    flush(target, backend, &mut pending);
                    let mut mask = control_mask;
                    while mask != 0 {
                        flush(mask.trailing_zeros() as usize, backend, &mut pending);
                        mask &= mask - 1; // clear the lowest set bit
                    }
                    backend.apply_gate_kernel(control_mask, target, gate);
                }
                &Instruction::Swap { a, b, .. } => {
                    flush(a, backend, &mut pending);
                    flush(b, backend, &mut pending);
                    for (c, t) in [(a, b), (b, a), (a, b)] {
                        backend.apply_gate_kernel(1 << c, t, &gates::X);
                    }
                }
                &Instruction::Measure { qubit, clbit } => {
                    flush(qubit, backend, &mut pending);
                    let one = backend.measure_qubit(qubit, rng)?;
                    *clbits = (*clbits & !(1 << clbit)) | (usize::from(one) << clbit);
                }
                &Instruction::Reset { qubit } => {
                    flush(qubit, backend, &mut pending);
                    backend.reset_qubit(qubit, rng)?;
                }
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

    /// Final amplitudes after running the circuit (before any final measurements).
    pub fn statevector(&self, kind: BackendKind) -> Result<Vec<Complex64>> {
        Ok(self.simulate(kind)?.into_amplitudes())
    }

    /// Final basis-state probabilities after running the circuit (before any final measurements).
    pub fn probabilities(&self, kind: BackendKind) -> Result<Vec<f64>> {
        Ok(self.simulate(kind)?.probabilities())
    }

    /// Expectation value of each Pauli string (e.g. `"ZZI"`, qubit 0 rightmost)
    /// in the final state, from one simulation. See [`crate::pauli`].
    pub fn expectation(&self, paulis: &[&str], kind: BackendKind) -> Result<Vec<f64>> {
        // Parse first, so a typo fails before a long simulation.
        let parsed = paulis.iter().map(|s| Pauli::parse(s, self.num_qubits)).collect::<Result<Vec<_>>>()?;
        let amps = self.statevector(kind)?;
        Ok(parsed.iter().map(|p| p.expectation(&amps)).collect())
    }

    /// Run the circuit `shots` times and count the results.
    ///
    /// Keys are classical-bit values if the circuit measures (see
    /// [`has_measurements`](Self::has_measurements)), otherwise basis indices from
    /// measuring every qubit at the end. Format them with [`crate::format_bitstring`].
    pub fn run(&self, shots: usize, kind: BackendKind, rng: &mut dyn RngCore) -> Result<HashMap<usize, usize>> {
        if !self.is_dynamic() {
            // Fast path: simulate once, sample the final state, then read each
            // sample through the final measurements into classical bits.
            let counts = self.simulate(kind)?.sample(shots, rng);
            let end = self.trailing_measures_start();
            if end == self.instructions.len() {
                return Ok(counts);
            }
            let mut out = HashMap::new();
            for (index, n) in counts {
                let mut clbits = 0;
                for ins in &self.instructions[end..] {
                    if let &Instruction::Measure { qubit, clbit } = ins {
                        clbits = (clbits & !(1 << clbit)) | ((index >> qubit & 1) << clbit);
                    }
                }
                *out.entry(clbits).or_insert(0) += n;
            }
            return Ok(out);
        }

        // Mid-circuit measurement: every shot is its own simulation.
        // shortcut: the gates before the first measurement are re-run every shot; caching that prefix state would help deep circuits with many shots.
        let kind = match kind {
            BackendKind::Auto if self.num_qubits < GPU_MIN_QUBITS_DYNAMIC => BackendKind::Cpu,
            other => other,
        };
        let mut backend = kind.create(self.num_qubits)?;
        let measures = self.has_measurements();
        let mut out = HashMap::new();
        for shot in 0..shots {
            if shot > 0 {
                backend.collapse_to(0);
            }
            let mut clbits = 0;
            self.execute(&self.instructions, backend.as_mut(), &mut clbits, rng)?;
            let key = if measures { clbits } else { backend.measure(rng) };
            *out.entry(key).or_insert(0) += 1;
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QuantumState;

    #[test]
    fn fused_circuit_matches_gate_by_gate() {
        // Long runs of single-qubit gates (fused) interleaved with controlled gates
        // (which must flush the right qubits), including a two-control gate and a swap.
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
            qc.swap(0, 2).unwrap();
            direct.cx(0, 1).unwrap();
            direct.cz(2, 3).unwrap();
            direct.apply_multi_controlled_gate(&[1, 3], 0, &gates::X).unwrap();
            direct.cx(0, 2).unwrap();
            direct.cx(2, 0).unwrap();
            direct.cx(0, 2).unwrap();
            qc.s(2).unwrap();
            direct.s(2).unwrap();
        }

        let fused = qc.statevector(BackendKind::Cpu).unwrap();
        let worst = fused.iter().zip(&direct.amplitudes).map(|(a, b)| (a - b).norm()).fold(0.0, f64::max);
        assert!(worst < 1e-12, "fusion changed the result by {worst}");
    }

    #[test]
    fn bad_gates_are_rejected_when_added() {
        let mut qc = QuantumCircuit::with_clbits(2, 1).unwrap();
        assert_eq!(qc.h(2), Err(QsError::QubitOutOfRange { qubit: 2, num_qubits: 2 }));
        assert_eq!(qc.cx(0, 0), Err(QsError::DuplicateQubit(0)));
        assert_eq!(qc.swap(1, 1), Err(QsError::DuplicateQubit(1)));
        assert_eq!(qc.swap(0, 5), Err(QsError::QubitOutOfRange { qubit: 5, num_qubits: 2 }));
        assert_eq!(qc.unitary(&[], 0, [[gates::H[0][0]; 2]; 2]), Err(QsError::NotUnitary));
        assert_eq!(qc.rx(0, f64::NAN), Err(QsError::NonFinite));
        assert_eq!(qc.cp(0, 1, f64::INFINITY), Err(QsError::NonFinite));
        assert_eq!(qc.measure(0, 1), Err(QsError::ClbitOutOfRange { clbit: 1, num_clbits: 1 }));
        assert!(qc.set_condition(&[0], 2).is_err()); // 2 does not fit in one bit
        qc.set_condition(&[0], 1).unwrap();
        assert!(qc.set_condition(&[0], 1).is_err()); // nested
        assert!(qc.measure(0, 0).is_err()); // only gates can be conditional
        assert!(qc.instructions().is_empty());
    }

    #[test]
    fn nearly_unitary_matrices_do_not_drift_the_norm() {
        // H scaled by 1 + 1e-10 passes the 1e-9 check. Used as given, 2000 of them
        // would grow the norm squared by about 4e-7; projected, only rounding remains.
        // The CNOTs stop the gates fusing into one matrix.
        let near_h = gates::H.map(|row| row.map(|z| z * (1.0 + 1e-10)));
        let mut qc = QuantumCircuit::new(2).unwrap();
        for _ in 0..2000 {
            qc.unitary(&[], 0, near_h).unwrap();
            qc.cx(0, 1).unwrap();
        }
        let norm_sq: f64 = qc.statevector(BackendKind::Cpu).unwrap().iter().map(|a| a.norm_sqr()).sum();
        assert!((norm_sq - 1.0).abs() < 1e-10, "norm squared drifted to {norm_sq}");
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
    fn final_measurements_map_qubits_to_clbits_on_the_fast_path() {
        // x(0), then qubit 0 -> clbit 2 and qubit 1 -> clbit 0: always clbits 0b100.
        let mut qc = QuantumCircuit::with_clbits(2, 3).unwrap();
        qc.x(0).unwrap();
        qc.measure(0, 2).unwrap();
        qc.measure(1, 0).unwrap();
        assert!(!qc.is_dynamic());
        let counts = qc.run(50, BackendKind::Cpu, &mut StdRng::seed_from_u64(1)).unwrap();
        assert_eq!(counts, HashMap::from([(0b100, 50)]));
        // The exact state ignores the final measurements.
        assert!((qc.probabilities(BackendKind::Cpu).unwrap()[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn teleportation_moves_a_one() {
        // Teleport |1> from qubit 0 to qubit 2: c2 must read 1 in every shot,
        // whatever the two random measurements c0 and c1 gave.
        let mut qc = QuantumCircuit::with_clbits(3, 3).unwrap();
        qc.x(0).unwrap();
        qc.h(1).unwrap();
        qc.cx(1, 2).unwrap();
        qc.cx(0, 1).unwrap();
        qc.h(0).unwrap();
        qc.measure(0, 0).unwrap();
        qc.measure(1, 1).unwrap();
        qc.set_condition(&[1], 1).unwrap();
        qc.x(2).unwrap();
        qc.clear_condition();
        qc.set_condition(&[0], 1).unwrap();
        qc.z(2).unwrap();
        qc.clear_condition();
        qc.measure(2, 2).unwrap();
        assert!(qc.is_dynamic());
        assert_eq!(qc.statevector(BackendKind::Cpu), Err(QsError::MidCircuitMeasurement));

        let counts = qc.run(400, BackendKind::Cpu, &mut StdRng::seed_from_u64(5)).unwrap();
        assert_eq!(counts.values().sum::<usize>(), 400);
        assert!(counts.keys().all(|k| k & 0b100 != 0), "{counts:?}");
        assert_eq!(counts.len(), 4, "c0 and c1 should take all four values: {counts:?}");
    }

    #[test]
    fn reset_without_measurements_measures_every_qubit_at_the_end() {
        let mut qc = QuantumCircuit::new(2).unwrap();
        qc.h(0).unwrap();
        qc.x(1).unwrap();
        qc.reset(0).unwrap();
        qc.h(0).unwrap();
        qc.h(0).unwrap();
        let counts = qc.run(100, BackendKind::Cpu, &mut StdRng::seed_from_u64(2)).unwrap();
        assert_eq!(counts, HashMap::from([(0b10, 100)]));
    }

    #[test]
    fn expectation_of_bell_state() {
        let mut qc = QuantumCircuit::new(2).unwrap();
        qc.h(0).unwrap();
        qc.cx(0, 1).unwrap();
        let e = qc.expectation(&["ZZ", "XX", "ZI"], BackendKind::Cpu).unwrap();
        assert!((e[0] - 1.0).abs() < 1e-12 && (e[1] - 1.0).abs() < 1e-12 && e[2].abs() < 1e-12);
    }

    #[test]
    fn size_mismatch_is_an_error() {
        let qc = QuantumCircuit::new(2).unwrap();
        let mut s = QuantumState::new(3).unwrap();
        assert_eq!(qc.apply_to(&mut s), Err(QsError::SizeMismatch { circuit: 2, backend: 3 }));
    }
}
