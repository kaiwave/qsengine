//! Single-qubit gate matrices.
//!
//! Every gate here is a 2x2 unitary matrix stored row-major:
//!
//! ```text
//! gate = [[g00, g01],
//!         [g10, g11]]
//! ```
//!
//! Applying it to a qubit in state `a|0> + b|1>` produces
//! `(g00*a + g01*b)|0> + (g10*a + g11*b)|1>`.
//!
//! Multi-qubit gates such as CNOT and CZ are *controlled* versions of these
//! matrices (CNOT = controlled-X, CZ = controlled-Z), so backends only need one
//! kernel. See [`crate::QuantumBackend::apply_controlled_gate`].

use num_complex::Complex64;
use std::f64::consts::FRAC_1_SQRT_2;

/// A 2x2 complex matrix, row-major.
pub type Gate = [[Complex64; 2]; 2];

// Short const constructors so the matrices below read like the textbook.
const fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}
const O: Complex64 = c(0.0, 0.0);
const I: Complex64 = c(1.0, 0.0);

/// Identity: leaves the qubit unchanged.
pub const ID: Gate = [[I, O], [O, I]];

/// Pauli-X (NOT / bit flip): |0> <-> |1>.
pub const X: Gate = [[O, I], [I, O]];

/// Pauli-Y: bit flip and phase flip, |0> -> i|1>, |1> -> -i|0>.
pub const Y: Gate = [[O, c(0.0, -1.0)], [c(0.0, 1.0), O]];

/// Pauli-Z (phase flip): |1> -> -|1>.
pub const Z: Gate = [[I, O], [O, c(-1.0, 0.0)]];

/// Hadamard: |0> -> (|0>+|1>)/sqrt2, |1> -> (|0>-|1>)/sqrt2. Creates superposition.
pub const H: Gate = [
    [c(FRAC_1_SQRT_2, 0.0), c(FRAC_1_SQRT_2, 0.0)],
    [c(FRAC_1_SQRT_2, 0.0), c(-FRAC_1_SQRT_2, 0.0)],
];

/// S (sqrt Z): |1> -> i|1>.
pub const S: Gate = [[I, O], [O, c(0.0, 1.0)]];

/// T (fourth root of Z): |1> -> e^(i*pi/4)|1>.
pub const T: Gate = [[I, O], [O, c(FRAC_1_SQRT_2, FRAC_1_SQRT_2)]];

/// Rotation about the X axis of the Bloch sphere by `theta` radians.
pub fn rx(theta: f64) -> Gate {
    let (s, co) = (theta / 2.0).sin_cos();
    [[c(co, 0.0), c(0.0, -s)], [c(0.0, -s), c(co, 0.0)]]
}

/// Rotation about the Y axis of the Bloch sphere by `theta` radians.
pub fn ry(theta: f64) -> Gate {
    let (s, co) = (theta / 2.0).sin_cos();
    [[c(co, 0.0), c(-s, 0.0)], [c(s, 0.0), c(co, 0.0)]]
}

/// Rotation about the Z axis of the Bloch sphere by `theta` radians.
pub fn rz(theta: f64) -> Gate {
    let (s, co) = (theta / 2.0).sin_cos();
    [[c(co, -s), O], [O, c(co, s)]]
}

/// Matrix product `a · b`: the single gate equivalent to applying `b` first, then `a`.
/// Used by [`crate::QuantumCircuit`] to fuse runs of gates into one pass over the state.
pub fn matmul(a: &Gate, b: &Gate) -> Gate {
    let mut out = [[O; 2]; 2];
    for (r, row) in out.iter_mut().enumerate() {
        for (col, cell) in row.iter_mut().enumerate() {
            *cell = a[r][0] * b[0][col] + a[r][1] * b[1][col];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// U * U^dagger should be the identity for every gate.
    fn assert_unitary(g: &Gate) {
        for r in 0..2 {
            for col in 0..2 {
                let sum = g[r][0] * g[col][0].conj() + g[r][1] * g[col][1].conj();
                let expect = if r == col { 1.0 } else { 0.0 };
                assert!((sum - c(expect, 0.0)).norm() < 1e-12, "not unitary: {g:?}");
            }
        }
    }

    #[test]
    fn all_gates_are_unitary() {
        for g in [ID, X, Y, Z, H, S, T, rx(0.3), ry(1.1), rz(-2.0)] {
            assert_unitary(&g);
        }
    }
}
