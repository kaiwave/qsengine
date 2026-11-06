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
//! | [`pauli`]   | Expectation values of Pauli strings such as `"ZZI"`            |
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
//! The `python` module wraps [`QuantumCircuit`] as `qsengine._core.QuantumCircuit`,
//! which `python/qsengine/__init__.py` extends in plain Python (drawing, `if_test`).
//! Errors map from [`QsError`] to `ValueError` / `RuntimeError` / `MemoryError`. Build it with
//! `maturin develop --release` (see the README).

pub mod backend;
pub mod circuit;
pub mod engine;
pub mod gates;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod pauli;
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

/// Most classical bits a circuit can have: one shot's bits are packed into a `usize`.
pub const MAX_CLBITS: usize = usize::BITS as usize;

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
    /// A gate angle (or matrix entry) was NaN or infinite.
    NonFinite,
    /// The state vector did not fit in memory.
    OutOfMemory { bytes: usize },
    /// A classical bit index was `>= num_clbits`.
    ClbitOutOfRange { clbit: usize, num_clbits: usize },
    /// An exact result was asked of a circuit that measures or resets part-way,
    /// whose final state is random.
    MidCircuitMeasurement,
    /// Any other invalid argument, with a message that says what was wrong.
    InvalidArgument(String),
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
            QsError::NonFinite => write!(f, "gate angles must be finite numbers, not NaN or infinity"),
            QsError::OutOfMemory { bytes } => {
                write!(f, "not enough memory for the state vector ({:.1} GiB)", *bytes as f64 / (1u64 << 30) as f64)
            }
            QsError::ClbitOutOfRange { clbit, num_clbits } => {
                write!(f, "classical bit {clbit} out of range for a circuit with {num_clbits} classical bits")
            }
            QsError::MidCircuitMeasurement => write!(
                f,
                "the circuit measures or resets a qubit before a later gate, so its final state is random; use run()"
            ),
            QsError::InvalidArgument(msg) => f.write_str(msg),
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

/// The same crossover for circuits with mid-circuit measurement, which
/// [`BackendKind::Auto`] runs on the CPU below this many qubits.
///
/// Every mid-circuit measurement waits for one number to come back from the GPU
/// (about 1 ms), and every shot is its own simulation, so the GPU only pays off
/// for large states. Measured with a 3-layer circuit that measures every qubit
/// part-way: CPU 21 ms/shot vs GPU 48 at 18 qubits, CPU 82 vs GPU 51 at 20.
///
/// shortcut: one machine's crossover, like [`GPU_MIN_QUBITS`].
pub const GPU_MIN_QUBITS_DYNAMIC: usize = 20;

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
