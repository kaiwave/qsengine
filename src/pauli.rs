//! Expectation values of Pauli strings, such as `"ZZI"`: the building block of VQE and QAOA.
//!
//! A Pauli string has one letter per qubit, read like a bitstring: the rightmost
//! letter acts on qubit 0. Its expectation value is `<psi|P|psi>`, a real number
//! between -1 and 1.
//!
//! Nothing is built as a matrix. On a basis state `|j>`, X flips a bit, Z adds a
//! sign when the bit is 1, and Y = iXZ does both. So the whole string maps `|j>`
//! to `i^ys * (-1)^popcount(j & phase) * |j ^ flip>`, and the expectation is one
//! pass over the amplitudes.

use crate::{QsError, Result};
use num_complex::Complex64;
use rayon::prelude::*;

/// Amplitudes per Rayon task; chunks are summed in order so results are reproducible.
const CHUNK: usize = 1 << 14;

/// A parsed Pauli string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pauli {
    /// Qubits with X or Y: their bit is flipped.
    flip: usize,
    /// Qubits with Z or Y: a 1 there flips the sign.
    phase: usize,
    /// Number of Ys, for the overall factor `i^ys`.
    ys: u32,
}

impl Pauli {
    /// Parse a string of `I`, `X`, `Y` and `Z` with one letter per qubit, qubit 0 rightmost.
    pub fn parse(s: &str, num_qubits: usize) -> Result<Self> {
        let letters = s.chars().count();
        if letters != num_qubits {
            return Err(QsError::InvalidArgument(format!(
                "Pauli string {s:?} has {letters} letters but the circuit has {num_qubits} qubits"
            )));
        }
        let mut p = Pauli { flip: 0, phase: 0, ys: 0 };
        for (q, ch) in s.chars().rev().enumerate() {
            match ch {
                'I' => {}
                'X' => p.flip |= 1 << q,
                'Z' => p.phase |= 1 << q,
                'Y' => {
                    p.flip |= 1 << q;
                    p.phase |= 1 << q;
                    p.ys += 1;
                }
                _ => {
                    return Err(QsError::InvalidArgument(format!(
                        "Pauli string {s:?} may only contain I, X, Y and Z"
                    )));
                }
            }
        }
        Ok(p)
    }

    /// `<psi|P|psi>` for the normalised state `amps`.
    pub fn expectation(&self, amps: &[Complex64]) -> f64 {
        // (P psi)_i = i^ys * (-1)^popcount((i ^ flip) & phase) * psi_(i ^ flip)
        let partial: Vec<Complex64> = amps
            .par_chunks(CHUNK)
            .enumerate()
            .map(|(c, chunk)| {
                let mut acc = Complex64::new(0.0, 0.0);
                for (j, a) in chunk.iter().enumerate() {
                    let k = (c * CHUNK + j) ^ self.flip;
                    let term = a.conj() * amps[k];
                    if (k & self.phase).count_ones() % 2 == 0 { acc += term } else { acc -= term }
                }
                acc
            })
            .collect();
        let sum: Complex64 = partial.iter().sum();
        // Multiply by i^ys and keep the real part (P is Hermitian, so the imaginary part is rounding).
        match self.ys % 4 {
            0 => sum.re,
            1 => -sum.im,
            2 => -sum.re,
            _ => sum.im,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_1_SQRT_2;

    fn c(re: f64, im: f64) -> Complex64 {
        Complex64::new(re, im)
    }

    #[test]
    fn single_qubit_paulis() {
        let r = FRAC_1_SQRT_2;
        let e = |s: &str, amps: &[Complex64]| Pauli::parse(s, 1).unwrap().expectation(amps);
        // |0>: <Z> = 1, <X> = <Y> = 0.
        assert_eq!(e("Z", &[c(1.0, 0.0), c(0.0, 0.0)]), 1.0);
        assert_eq!(e("X", &[c(1.0, 0.0), c(0.0, 0.0)]), 0.0);
        // |+>: <X> = 1. |+i> = (|0> + i|1>)/sqrt2: <Y> = 1.
        assert!((e("X", &[c(r, 0.0), c(r, 0.0)]) - 1.0).abs() < 1e-12);
        assert!((e("Y", &[c(r, 0.0), c(0.0, r)]) - 1.0).abs() < 1e-12);
        assert!((e("I", &[c(r, 0.0), c(0.0, r)]) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn bell_state_correlations() {
        // (|00> + |11>)/sqrt2: <ZZ> = <XX> = 1, <YY> = -1, <ZI> = 0.
        let r = FRAC_1_SQRT_2;
        let bell = [c(r, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(r, 0.0)];
        let e = |s: &str| Pauli::parse(s, 2).unwrap().expectation(&bell);
        assert!((e("ZZ") - 1.0).abs() < 1e-12);
        assert!((e("XX") - 1.0).abs() < 1e-12);
        assert!((e("YY") + 1.0).abs() < 1e-12);
        assert!(e("ZI").abs() < 1e-12);
    }

    #[test]
    fn rejects_bad_strings() {
        assert!(matches!(Pauli::parse("ZZ", 3), Err(QsError::InvalidArgument(_))));
        assert!(matches!(Pauli::parse("ZA", 2), Err(QsError::InvalidArgument(_))));
    }
}
