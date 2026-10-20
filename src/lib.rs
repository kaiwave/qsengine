//! # qsengine
//!
//! A state-vector quantum simulator.
//!
//! An `n`-qubit register is stored as `2^n` complex amplitudes. Gates are
//! applied by updating those amplitudes in place, and measurement samples a
//! basis state with probability `|amplitude|^2`.
//!
//! ## Layout
//!
//! | Module      | Purpose                                                        |
//! |-------------|----------------------------------------------------------------|
//! | [`gates`]   | 2x2 gate matrices (X, Y, Z, H, S, T, rotations)                |
//! | [`backend`] | The [`QuantumBackend`] trait: the one API every simulator obeys |
//! | [`engine`]  | [`QuantumState`]: multithreaded CPU backend (f64 precision)    |
//! | [`gpu`]     | [`GpuQuantumState`]: portable GPU backend via wgpu (f32)       |
//!
//! ## Choosing a backend
//!
//! Call [`best_backend`] to get the GPU backend when one is usable and the CPU
//! backend otherwise. Machines without a GPU (or builds without the `gpu`
//! feature) silently get the CPU backend, so the same code runs everywhere.
//!
//! ## Bit-ordering convention
//!
//! Qubit `q` is bit `q` of the basis index (little-endian, same as Qiskit).
//! For 3 qubits, index `0b110 = 6` means q2 = 1, q1 = 1, q0 = 0, and is
//! printed as the bitstring `"110"` (qubit 0 on the right).
//!
//! ## Python readiness
//!
//! Everything returns plain data (`Vec`, `HashMap`, `usize`, `f64`) and errors
//! are a single [`QsError`] enum, so a future PyO3 layer can wrap
//! `Box<dyn QuantumBackend + Send>` in a `#[pyclass]` and map `QsError` to
//! `ValueError` / `RuntimeError` without touching the engine.

pub mod backend;
pub mod engine;
pub mod gates;
#[cfg(feature = "gpu")]
pub mod gpu;

pub use backend::QuantumBackend;
pub use engine::QuantumState;
pub use gates::Gate;
#[cfg(feature = "gpu")]
pub use gpu::GpuQuantumState;

use std::fmt;

/// Hard upper limit on register size. 30 qubits of `Complex<f64>` is 16 GiB,
/// already beyond most machines; it guards against accidental huge allocations.
pub const MAX_QUBITS: usize = 30;

/// Every error the engine can return.
#[derive(Debug, Clone, PartialEq)]
pub enum QsError {
    /// Requested more qubits than [`MAX_QUBITS`] (or than the GPU can hold).
    TooManyQubits { requested: usize, max: usize },
    /// A qubit index was `>= num_qubits`.
    QubitOutOfRange { qubit: usize, num_qubits: usize },
    /// The same qubit was used twice in one gate (e.g. CNOT with control == target).
    DuplicateQubit(usize),
    /// No usable GPU, or the GPU driver reported an error.
    Gpu(String),
}

impl fmt::Display for QsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QsError::TooManyQubits { requested, max } => {
                write!(f, "requested {requested} qubits, maximum is {max}")
            }
            QsError::QubitOutOfRange { qubit, num_qubits } => {
                write!(f, "qubit {qubit} out of range for a {num_qubits}-qubit register")
            }
            QsError::DuplicateQubit(q) => write!(f, "qubit {q} used more than once in one gate"),
            QsError::Gpu(msg) => write!(f, "GPU error: {msg}"),
        }
    }
}

impl std::error::Error for QsError {}

/// Shorthand used throughout the crate.
pub type Result<T> = std::result::Result<T, QsError>;

/// Return the fastest available backend for `num_qubits` qubits.
///
/// Tries the GPU first (when compiled with the `gpu` feature and a GPU adapter
/// exists), otherwise falls back to the multithreaded CPU backend.
///
/// shortcut: always prefers the GPU; for small registers (< ~16 qubits) the CPU is
/// usually faster due to GPU launch latency. Add a size threshold once benchmarks exist (weeks 5-6).
pub fn best_backend(num_qubits: usize) -> Result<Box<dyn QuantumBackend + Send>> {
    #[cfg(feature = "gpu")]
    if let Ok(gpu) = GpuQuantumState::new(num_qubits) {
        return Ok(Box::new(gpu));
    }
    Ok(Box::new(QuantumState::new(num_qubits)?))
}

/// Format a basis-state index as a bitstring, qubit 0 rightmost.
///
/// ```
/// assert_eq!(qsengine::format_bitstring(6, 3), "110");
/// ```
pub fn format_bitstring(index: usize, num_qubits: usize) -> String {
    format!("{index:0num_qubits$b}")
}
