# qsengine: High-Performance Qubit Simulator
An end-to-end educational quantum computing platform consisting of:
1. **`qsengine`**: A blazingly fast state-vector simulator engine written in Rust, wrapped in an ergonomic Python API using **PyO3** and **maturin**.
2. **[qsengine.kaiwave.dev](https://qsengine.kaiwave.dev)**: A modern site covering foundational quantum mechanics, practical circuit building, and usage of qsengine.

---

### Project Scope Breakdown

| Component | Difficulty | Tech Stack | Notes |
| --- | --- | --- | --- |
| **Rust Core** | Medium | `nalgebra` or `ndarray` + Rayon | State vector simulation (tensor products, matrix multiplication) |
| **Python Bindings** | Low–Medium | `pyo3` + `maturin` | Expose circuit builder and gate applications |
| **Static Site** | Low | VitePress, Astro, or Docusaurus | Explains qubits, superposition, Bloch sphere, basic gates |

---

### Recommended 8-Week Timeline

**Weeks 1–2: State Vector Engine in Rust**

* Represent an $n$-qubit state as a complex vector of length $2^n$ (using `num-complex`).
* Implement single-qubit gates ($X, Y, Z, H$) via direct state indexing or Kronecker products.
* Add controlled gates ($CNOT$, $CZ$) and a basic measurement function using weighted random sampling.

**Weeks 3–4: Python Layer via PyO3**

* Initialize your workspace using `maturin new --binding pyo3`.
* Wrap a `QuantumCircuit` struct in Rust and expose methods like `.h(target)`, `.cx(control, target)`, and `.run()` to Python.
* Write unit tests in `pytest` to compare your simulation outputs against small NumPy calculations.

**Weeks 5–6: Performance & Benchmarking**

* Add multithreading to gate operations with **Rayon**.
* Benchmark execution times up to ~20 qubits against pure Python and basic Qiskit Aer to generate compelling performance plots.

**Weeks 7–8: Static Documentation Site**

* Use a doc generator with native KaTeX/MathJax support (like **VitePress** or **Starlight**).
* Cover foundational concepts: bra-ket notation, superposition, entanglement, and walk through an example algorithm (e.g., Deutsch-Jozsa or Quantum Teleportation) using your custom Python library.

### Potential Traps to Avoid

* **Exponential Memory Wall ($2^n$):** State vector memory scales as $2^n \times 16\text{ bytes}$. An 18-qubit simulation takes ~4 MB of RAM, 24 qubits takes ~256 MB, and 30 qubits demands ~16 GB. Cap your simulator's scope around 15–20 qubits before considering sparse matrices or stabilizer states.
* **FFI Boundary Overhead:** Avoid passing state vectors across the Rust-Python boundary on every single gate application. Keep the state entirely in Rust memory and only return counts or measurement bitstrings back to Python when `.measure()` or `.run()` finishes.
* 
---

## Architecture Overview

```
                      +-----------------------------+
                      |       Python Client         |
                      | (API / Circuit Construction)|
                      +--------------+--------------+
                                     |
                                     | PyO3 Bindings
                                     v
                      +-----------------------------+
                      |         Rust Core           |
                      |  - State Vector ($2^n$)      |
                      |  - Matrix Math & Rayon      |
                      |  - Non-destructive sampling |
                      +-----------------------------+
```

### Why Rust + Python?
Simulating an $n$-qubit quantum system requires maintaining a state vector of length $2^n$ with double-precision complex amplitudes ($\approx 16 \times 2^n$ bytes). Applying quantum gates translates to dense matrix multiplications and index shuffling.

* **Python** provides the approachable, interactive syntax familiar to researchers and students.
* **Rust** provides zero-cost abstractions, predictable memory usage, SIMD auto-vectorization, and data-race-free multithreading with `rayon`.
* **Zero FFI Boundary Penalty**: Circuits are constructed and fully executed in Rust memory. Complex state vectors are never serialized across the boundary on every gate—only final measurement results and expectation values are returned to Python.

---

## Repository Structure

```text
.
├── Cargo.toml               # Rust library manifest (`gpu` feature on by default)
├── pyproject.toml           # Maturin build configuration (weeks 3-4)
├── docs/RUST_GUIDE.md       # Every Rust feature used in the engine, explained
├── src/                     # Rust backend
│   ├── lib.rs               # Crate root, errors, best_backend() (PyO3 registration later)
│   ├── backend.rs           # QuantumBackend trait: shared API, measurement, conformance tests
│   ├── engine.rs            # QuantumState: multithreaded CPU state vector (f64)
│   ├── gpu.rs               # GpuQuantumState: portable GPU state vector via wgpu (f32)
│   ├── shaders/apply_gate.wgsl  # GPU kernel for every (controlled) gate
│   ├── gates.rs             # Pauli, Hadamard, S, T and rotation matrices
│   ├── main.rs              # Demo + CPU/GPU benchmark
│   └── circuit.rs           # Circuit IR & execution pipeline (weeks 3-4)
├── python/                  # High-level Python package wrapper
│   └── qcore/
│       ├── __init__.py
│       └── visualization.py # State visualizers / bloch sphere helpers
├── tests/                   # Python & Rust test suites
└── README.md
```

---

## Features

- [x] **Full State-Vector Simulation**: Simulate systems up to ~20 qubits with minimal memory footprint.
- [ ] **Core Gate Set**:
  - **Single Qubit**: $X, Y, Z, H, S, T, R_x(\theta), R_y(\theta), R_z(\theta)$
  - **Multi Qubit**: $CX$ (CNOT), $CZ$, $SWAP$
- [x] **Measurement Engine**: Monte-Carlo shot sampling and exact state probability vector calculation.
- [x] **Multithreaded Gate Dispatch**: Parallelized matrix application across amplitude blocks via `rayon`.
- [x] **Portable GPU Backend**: wgpu compute shaders (Vulkan / DX12 / Metal, any vendor). Falls back to CPU automatically when no GPU is present; disable entirely with `--no-default-features`.
- [ ] **Accompanying Educational Site**: Clean, readable static guides explaining quantum mechanics from linear algebra up to full algorithms.
