//! CPU backend: [`QuantumState`], a multithreaded state vector in `f64` precision.
//!
//! This is the reference implementation and the fallback when no GPU is
//! available. Gate application is parallelised across CPU cores with Rayon.

use crate::backend::QuantumBackend;
use crate::gates::Gate;
use crate::{MAX_QUBITS, QsError, Result};
use num_complex::Complex64;
use rayon::prelude::*;

/// Minimum number of amplitude pairs handed to one Rayon task. Below this the
/// cost of scheduling work on another thread outweighs the arithmetic, so small
/// registers effectively run single-threaded.
const MIN_PARALLEL_LEN: usize = 1 << 14;

/// An `n`-qubit state stored as a complex vector of length `2^n`.
#[derive(Debug, Clone)]
pub struct QuantumState {
    pub num_qubits: usize,
    /// `amplitudes[i]` is the amplitude of basis state `|i>` (see the bit-ordering
    /// note in the crate docs). Public for teaching/inspection; keep it normalised.
    pub amplitudes: Vec<Complex64>,
}

impl QuantumState {
    /// Create the all-zero state |0...0> for `num_qubits` qubits.
    ///
    /// For n qubits:
    /// - state length = 2^n
    /// - index 0 corresponds to |00...0>
    /// - amplitude at index 0 = 1 + 0i
    /// - all others = 0 + 0i
    pub fn new(num_qubits: usize) -> Result<Self> {
        if num_qubits > MAX_QUBITS {
            return Err(QsError::TooManyQubits { requested: num_qubits, max: MAX_QUBITS });
        }
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); 1 << num_qubits];
        amplitudes[0] = Complex64::new(1.0, 0.0);
        Ok(Self { num_qubits, amplitudes })
    }

    /// Probability of measuring basis state `index`, i.e. `|amplitude|^2`.
    /// Panics if `index >= 2^n`, like normal slice indexing.
    pub fn probability_of_index(&self, index: usize) -> f64 {
        self.amplitudes[index].norm_sqr()
    }
}

impl QuantumBackend for QuantumState {
    fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// The core gate loop.
    ///
    /// The target qubit `t` splits the basis states into pairs `(i0, i1)` that differ
    /// only in bit `t` (`i1 = i0 + 2^t`). Each pair is multiplied by the 2x2 gate:
    ///
    /// ```text
    /// [a0']   [g00 g01] [a0]
    /// [a1'] = [g10 g11] [a1]
    /// ```
    ///
    /// The vector is laid out as consecutive blocks of `2 * 2^t` amplitudes; in each
    /// block the first half holds the `i0` entries and the second half the matching
    /// `i1` entries. Splitting each block in two gives Rust two disjoint `&mut`
    /// slices, so the pairs can be updated in parallel with no `unsafe`.
    fn apply_gate_kernel(&mut self, control_mask: usize, target: usize, gate: &Gate) {
        let stride = 1usize << target;
        let block = stride << 1;
        let [[g00, g01], [g10, g11]] = *gate;

        // Update one pair; skipped unless all control bits of i0 are set.
        // (i0 has the target bit clear and the mask never contains the target,
        // so testing i0 alone is enough.)
        let update = |i0: usize, a0: &mut Complex64, a1: &mut Complex64| {
            if i0 & control_mask == control_mask {
                let (x0, x1) = (*a0, *a1);
                *a0 = g00 * x0 + g01 * x1;
                *a1 = g10 * x0 + g11 * x1;
            }
        };

        if stride >= MIN_PARALLEL_LEN {
            // High target qubit: few, large blocks. Parallelise inside each block.
            for (b, chunk) in self.amplitudes.chunks_mut(block).enumerate() {
                let base = b * block;
                let (lo, hi) = chunk.split_at_mut(stride);
                lo.par_iter_mut()
                    .zip(hi.par_iter_mut())
                    .enumerate()
                    .with_min_len(MIN_PARALLEL_LEN)
                    .for_each(|(j, (a0, a1))| update(base + j, a0, a1));
            }
        } else {
            // Low target qubit: many small blocks. Parallelise across blocks.
            self.amplitudes
                .par_chunks_mut(block)
                .enumerate()
                .with_min_len((MIN_PARALLEL_LEN / stride).max(1))
                .for_each(|(b, chunk)| {
                    let base = b * block;
                    let (lo, hi) = chunk.split_at_mut(stride);
                    for (j, (a0, a1)) in lo.iter_mut().zip(hi).enumerate() {
                        update(base + j, a0, a1);
                    }
                });
        }
    }

    fn amplitudes(&self) -> Vec<Complex64> {
        self.amplitudes.clone()
    }

    fn probabilities(&self) -> Vec<f64> {
        // Override the default to skip the amplitude copy.
        self.amplitudes.par_iter().map(|a| a.norm_sqr()).collect()
    }

    fn collapse_to(&mut self, index: usize) {
        self.amplitudes.fill(Complex64::new(0.0, 0.0));
        self.amplitudes[index] = Complex64::new(1.0, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::conformance;

    #[test]
    fn passes_backend_conformance_suite() {
        conformance::run_all(|n| Box::new(QuantumState::new(n).unwrap()) as Box<dyn QuantumBackend>, 1e-12);
    }

    #[test]
    fn rejects_too_many_qubits() {
        assert!(matches!(
            QuantumState::new(MAX_QUBITS + 1),
            Err(QsError::TooManyQubits { .. })
        ));
    }

    #[test]
    fn parallel_paths_match_on_large_register() {
        // 16 qubits is above MIN_PARALLEL_LEN, so both kernel branches run.
        let n = 16;
        let mut s = QuantumState::new(n).unwrap();
        for q in 0..n {
            s.h(q).unwrap();
        }
        // H on every qubit = uniform superposition.
        let expect = 1.0 / (1 << n) as f64;
        assert!(s.probabilities().iter().all(|p| (p - expect).abs() < 1e-12));
        // ...and H again returns to |0...0>.
        for q in 0..n {
            s.h(q).unwrap();
        }
        assert!((s.probability_of_index(0) - 1.0).abs() < 1e-9);
    }
}
