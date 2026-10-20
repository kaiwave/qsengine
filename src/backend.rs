//! The [`QuantumBackend`] trait: the single interface every simulator implements.
//!
//! A backend only has to provide four primitive operations:
//!
//! 1. [`num_qubits`](QuantumBackend::num_qubits)
//! 2. [`apply_gate_kernel`](QuantumBackend::apply_gate_kernel): the one gate loop
//! 3. [`amplitudes`](QuantumBackend::amplitudes): read the state back
//! 4. [`collapse_to`](QuantumBackend::collapse_to): overwrite with a basis state
//!
//! Everything else (input validation, named gates, measurement and sampling)
//! is written once here as provided methods, so a new backend (a distributed one
//! or a different GPU API) gets the whole API by implementing those four.

use crate::gates::{self, Gate};
use crate::{QsError, Result};
use num_complex::Complex64;
use rand::{Rng, RngCore};
use std::collections::HashMap;

pub trait QuantumBackend {
    // ---------------------------------------------------------------------
    // Required primitives
    // ---------------------------------------------------------------------

    /// Number of qubits in the register.
    fn num_qubits(&self) -> usize;

    /// Apply the 2x2 `gate` to qubit `target`, but only on basis states where
    /// every bit set in `control_mask` is 1.
    ///
    /// `control_mask == 0` is an ordinary single-qubit gate. CNOT with control
    /// `c` is `control_mask = 1 << c` with `gate = X`.
    ///
    /// Callers must have validated the inputs already: `target < num_qubits`,
    /// the mask only uses valid qubits, and the mask does not contain `target`.
    /// Use the provided methods ([`apply_gate`](Self::apply_gate), etc.), which validate.
    fn apply_gate_kernel(&mut self, control_mask: usize, target: usize, gate: &Gate);

    /// Copy of all `2^n` amplitudes, in basis-index order.
    fn amplitudes(&self) -> Vec<Complex64>;

    /// Replace the state with the basis state `|index>` (amplitude 1 there, 0 elsewhere).
    fn collapse_to(&mut self, index: usize);

    // ---------------------------------------------------------------------
    // Provided: validation and generic gate application
    // ---------------------------------------------------------------------

    /// Size of the state vector, `2^n`.
    fn dimension(&self) -> usize {
        1 << self.num_qubits()
    }

    /// Apply `gate` to `target` only where all `controls` are |1>.
    ///
    /// This is the validated entry point all named gates route through. An empty
    /// `controls` slice gives a plain single-qubit gate; two controls give a
    /// Toffoli-style gate.
    fn apply_multi_controlled_gate(
        &mut self,
        controls: &[usize],
        target: usize,
        gate: &Gate,
    ) -> Result<()> {
        let n = self.num_qubits();
        let check = |q: usize| {
            if q < n { Ok(()) } else { Err(QsError::QubitOutOfRange { qubit: q, num_qubits: n }) }
        };
        check(target)?;
        let mut mask = 0usize;
        for &c in controls {
            check(c)?;
            let bit = 1 << c;
            if c == target || mask & bit != 0 {
                return Err(QsError::DuplicateQubit(c));
            }
            mask |= bit;
        }
        self.apply_gate_kernel(mask, target, gate);
        Ok(())
    }

    /// Apply a single-qubit `gate` to `target`.
    fn apply_gate(&mut self, target: usize, gate: &Gate) -> Result<()> {
        self.apply_multi_controlled_gate(&[], target, gate)
    }

    /// Apply `gate` to `target` only where `control` is |1>.
    fn apply_controlled_gate(&mut self, control: usize, target: usize, gate: &Gate) -> Result<()> {
        self.apply_multi_controlled_gate(&[control], target, gate)
    }

    // ---------------------------------------------------------------------
    // Provided: named gates (these mirror the planned Python API: `.h(0)`, `.cx(0, 1)`)
    // ---------------------------------------------------------------------

    /// Pauli-X on `q`.
    fn x(&mut self, q: usize) -> Result<()> { self.apply_gate(q, &gates::X) }
    /// Pauli-Y on `q`.
    fn y(&mut self, q: usize) -> Result<()> { self.apply_gate(q, &gates::Y) }
    /// Pauli-Z on `q`.
    fn z(&mut self, q: usize) -> Result<()> { self.apply_gate(q, &gates::Z) }
    /// Hadamard on `q`.
    fn h(&mut self, q: usize) -> Result<()> { self.apply_gate(q, &gates::H) }
    /// S gate on `q`.
    fn s(&mut self, q: usize) -> Result<()> { self.apply_gate(q, &gates::S) }
    /// T gate on `q`.
    fn t(&mut self, q: usize) -> Result<()> { self.apply_gate(q, &gates::T) }
    /// X-rotation by `theta` on `q`.
    fn rx(&mut self, q: usize, theta: f64) -> Result<()> { self.apply_gate(q, &gates::rx(theta)) }
    /// Y-rotation by `theta` on `q`.
    fn ry(&mut self, q: usize, theta: f64) -> Result<()> { self.apply_gate(q, &gates::ry(theta)) }
    /// Z-rotation by `theta` on `q`.
    fn rz(&mut self, q: usize, theta: f64) -> Result<()> { self.apply_gate(q, &gates::rz(theta)) }
    /// CNOT: flip `target` when `control` is |1>.
    fn cx(&mut self, control: usize, target: usize) -> Result<()> {
        self.apply_controlled_gate(control, target, &gates::X)
    }
    /// Controlled-Z: phase-flip |11>. Symmetric in its two qubits.
    fn cz(&mut self, control: usize, target: usize) -> Result<()> {
        self.apply_controlled_gate(control, target, &gates::Z)
    }

    // ---------------------------------------------------------------------
    // Provided: probabilities and measurement
    // ---------------------------------------------------------------------

    /// Probability of each basis state, `|amplitude|^2`, in basis-index order.
    fn probabilities(&self) -> Vec<f64> {
        self.amplitudes().iter().map(|a| a.norm_sqr()).collect()
    }

    /// Measure every qubit in the computational basis.
    ///
    /// Picks basis state `i` with probability `|amp_i|^2`, collapses the state
    /// to `|i>`, and returns `i`. Use [`crate::format_bitstring`] to print it.
    fn measure(&mut self, rng: &mut dyn RngCore) -> usize {
        let cumulative = cumulative_probabilities(&self.probabilities());
        let outcome = sample_index(&cumulative, rng);
        // Global phase is unobservable, so collapsing to amplitude exactly 1 is valid.
        self.collapse_to(outcome);
        outcome
    }

    /// Simulate `shots` measurements *without* disturbing the state.
    ///
    /// Returns `basis index -> number of times seen`. This is how real circuits
    /// are run: prepare once, sample many times. Costs `O(2^n + shots * n)`.
    fn sample(&self, shots: usize, rng: &mut dyn RngCore) -> HashMap<usize, usize> {
        let cumulative = cumulative_probabilities(&self.probabilities());
        let mut counts = HashMap::new();
        for _ in 0..shots {
            *counts.entry(sample_index(&cumulative, rng)).or_insert(0) += 1;
        }
        counts
    }
}

/// Running sum of `probs`: `out[i] = probs[0] + ... + probs[i]`.
fn cumulative_probabilities(probs: &[f64]) -> Vec<f64> {
    probs
        .iter()
        .scan(0.0, |acc, p| {
            *acc += p;
            Some(*acc)
        })
        .collect()
}

/// Weighted random pick from a cumulative distribution using binary search.
///
/// The random number is scaled by the final total, so tiny floating-point drift
/// away from a total of exactly 1.0 cannot bias or break sampling.
fn sample_index(cumulative: &[f64], rng: &mut dyn RngCore) -> usize {
    let total = *cumulative.last().expect("state vector is never empty");
    let r = rng.random::<f64>() * total;
    // First index whose cumulative value exceeds r. Zero-probability states share
    // their predecessor's cumulative value, so they can never be picked.
    cumulative.partition_point(|&c| c <= r).min(cumulative.len() - 1)
}

/// Backend-agnostic correctness tests. Every backend runs this same suite, so a
/// new backend is "done" when it passes. `tol` allows for f32 backends.
#[cfg(test)]
pub(crate) mod conformance {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    type Make = fn(usize) -> Box<dyn QuantumBackend>;

    pub fn run_all(make: Make, tol: f64) {
        starts_in_zero_state(make, tol);
        x_flips_bit(make, tol);
        bell_state(make, tol);
        cz_phase(make, tol);
        y_and_rotations(make, tol);
        measurement_collapses(make);
        sampling_matches_probabilities(make);
        validates_inputs(make);
    }

    fn close(a: Complex64, b: Complex64, tol: f64) -> bool {
        (a - b).norm() < tol
    }

    fn starts_in_zero_state(make: Make, tol: f64) {
        let s = make(3);
        assert_eq!(s.dimension(), 8);
        let p = s.probabilities();
        assert!((p[0] - 1.0).abs() < tol && p[1..].iter().all(|&x| x < tol));
    }

    fn x_flips_bit(make: Make, tol: f64) {
        let mut s = make(3);
        s.x(1).unwrap();
        // Qubit 1 set -> basis index 0b010 = 2.
        assert!((s.probabilities()[2] - 1.0).abs() < tol);
    }

    fn bell_state(make: Make, tol: f64) {
        // H(0) then CNOT(0->1) gives (|00> + |11>)/sqrt2.
        let mut s = make(2);
        s.h(0).unwrap();
        s.cx(0, 1).unwrap();
        let a = s.amplitudes();
        let r = std::f64::consts::FRAC_1_SQRT_2;
        assert!(close(a[0], Complex64::new(r, 0.0), tol));
        assert!(close(a[3], Complex64::new(r, 0.0), tol));
        assert!(a[1].norm() < tol && a[2].norm() < tol);
    }

    fn cz_phase(make: Make, tol: f64) {
        // |11> picks up a -1 phase; CZ is symmetric so either qubit can be control.
        let mut s = make(2);
        s.x(0).unwrap();
        s.x(1).unwrap();
        s.cz(1, 0).unwrap();
        assert!(close(s.amplitudes()[3], Complex64::new(-1.0, 0.0), tol));
    }

    fn y_and_rotations(make: Make, tol: f64) {
        // Y|0> = i|1>.
        let mut s = make(1);
        s.y(0).unwrap();
        assert!(close(s.amplitudes()[1], Complex64::new(0.0, 1.0), tol));
        // RX(pi) = -iX, so RX(pi)|0> = -i|1>.
        let mut s = make(1);
        s.rx(0, std::f64::consts::PI).unwrap();
        assert!(close(s.amplitudes()[1], Complex64::new(0.0, -1.0), tol));
    }

    fn measurement_collapses(make: Make) {
        let mut rng = StdRng::seed_from_u64(7);
        let mut s = make(2);
        s.h(0).unwrap();
        s.cx(0, 1).unwrap();
        let m = s.measure(&mut rng);
        assert!(m == 0 || m == 3, "Bell state can only give 00 or 11");
        // After collapse, measuring again must give the same answer.
        assert_eq!(s.measure(&mut rng), m);
    }

    fn sampling_matches_probabilities(make: Make) {
        let mut rng = StdRng::seed_from_u64(42);
        let mut s = make(2);
        s.h(0).unwrap();
        s.cx(0, 1).unwrap();
        let counts = s.sample(10_000, &mut rng);
        assert_eq!(counts.keys().filter(|&&k| k != 0 && k != 3).count(), 0);
        let zeros = counts[&0] as f64 / 10_000.0;
        assert!((zeros - 0.5).abs() < 0.03, "got {zeros}");
        // Sampling must not disturb the state.
        assert!((s.probabilities()[0] - 0.5).abs() < 1e-5);
    }

    fn validates_inputs(make: Make) {
        let mut s = make(2);
        assert_eq!(s.h(2), Err(QsError::QubitOutOfRange { qubit: 2, num_qubits: 2 }));
        assert_eq!(s.cx(1, 1), Err(QsError::DuplicateQubit(1)));
        assert_eq!(s.cx(5, 0), Err(QsError::QubitOutOfRange { qubit: 5, num_qubits: 2 }));
    }
}
