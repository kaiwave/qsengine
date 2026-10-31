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
//! | [`gates`]   | 2x2 gate matrices (Paulis, H, S, T, phase, U, rotations)       |
//! | [`backend`] | The [`QuantumBackend`] trait: the one API every simulator obeys |
//! | [`circuit`] | [`QuantumCircuit`]: record gates now, run them later           |
//! | [`engine`]  | [`QuantumState`]: multithreaded CPU backend (f64 precision)    |
//! | [`gpu`]     | [`GpuQuantumState`]: portable GPU backend via wgpu (f32)       |
//! | `python`    | PyO3 bindings (only with the `python` feature, built by maturin) |
//!
//! ## Choosing a backend
//!
//! Call [`best_backend`] (or use [`BackendKind::Auto`]) to get the GPU backend
//! for large registers when one is usable, and the CPU backend otherwise.
//! Machines without a GPU (or builds without the `gpu` feature) silently get
//! the CPU backend, so the same code runs everywhere.
//!
//! ## Bit-ordering convention
//!
//! Qubit `q` is bit `q` of the basis index (little-endian, same as Qiskit).
//! For 3 qubits, index `0b110 = 6` means q2 = 1, q1 = 1, q0 = 0, and is
//! printed as the bitstring `"110"` (qubit 0 on the right).
//!
//! ## Python
//!
//! The `python` module wraps [`QuantumCircuit`] as `qsengine.QuantumCircuit`.
//! Errors map from [`QsError`] to `ValueError` / `RuntimeError`. Build it with
//! `maturin develop --release` (see the README).

pub mod backend;
pub mod circuit;
pub mod engine;
pub mod gates;
#[cfg(feature = "gpu")]
pub mod gpu;
#[cfg(feature = "python")]
mod python;

pub use backend::QuantumBackend;
pub use circuit::QuantumCircuit;
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
    /// A circuit was applied to a backend with a different number of qubits.
    SizeMismatch { circuit: usize, backend: usize },
    /// A user-supplied gate matrix was not a 2x2 unitary.
    NotUnitary,
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
            QsError::SizeMismatch { circuit, backend } => {
                write!(f, "circuit has {circuit} qubits but backend has {backend}")
            }
            QsError::NotUnitary => write!(f, "gate matrix must be a 2x2 unitary"),
            QsError::Gpu(msg) => write!(f, "GPU error: {msg}"),
        }
    }
}

impl std::error::Error for QsError {}

/// Shorthand used throughout the crate.
pub type Result<T> = std::result::Result<T, QsError>;

/// Below this many qubits, [`BackendKind::Auto`] uses the CPU: the state is too
/// small to amortise GPU launch and read-back latency (~1 ms per run).
///
/// Measured with `benchmarks/benchmark.py` (Ryzen 5 5500 + Radeon RX 6600): the
/// GPU first wins at 14 qubits. At a tie the CPU is preferred for its f64 precision.
///
/// shortcut: one machine's crossover; a per-machine calibration could replace it if users report slow small circuits.
pub const GPU_MIN_QUBITS: usize = 14;

/// Which simulator to run on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendKind {
    /// GPU for registers of [`GPU_MIN_QUBITS`] or more when one is usable, else CPU.
    #[default]
    Auto,
    /// Multithreaded CPU, f64 precision. Always available.
    Cpu,
    /// GPU via wgpu, f32 precision. Errors if no GPU (or built without `gpu`).
    Gpu,
}

impl BackendKind {
    /// Create a |0...0> state of this kind.
    pub fn create(self, num_qubits: usize) -> Result<Box<dyn QuantumBackend + Send>> {
        match self {
            BackendKind::Cpu => Ok(Box::new(QuantumState::new(num_qubits)?)),
            #[cfg(feature = "gpu")]
            BackendKind::Gpu => Ok(Box::new(GpuQuantumState::new(num_qubits)?)),
            #[cfg(not(feature = "gpu"))]
            BackendKind::Gpu => Err(QsError::Gpu("qsengine was built without the `gpu` feature".into())),
            BackendKind::Auto => best_backend(num_qubits),
        }
    }
}

/// Return the fastest available backend for `num_qubits` qubits.
///
/// Uses the GPU for registers of at least [`GPU_MIN_QUBITS`] qubits when compiled
/// with the `gpu` feature and a GPU adapter exists; otherwise the multithreaded CPU backend.
pub fn best_backend(num_qubits: usize) -> Result<Box<dyn QuantumBackend + Send>> {
    #[cfg(feature = "gpu")]
    if num_qubits >= GPU_MIN_QUBITS {
        if let Ok(gpu) = GpuQuantumState::new(num_qubits) {
            return Ok(Box::new(gpu));
        }
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
