# qsengine

A state-vector quantum simulator for Python. You build circuits in Python; a Rust core runs them on every CPU core or on any GPU, and hands back the results.

```bash
pip install qsengine
```

```python
import qsengine

qc = qsengine.QuantumCircuit(2)
qc.h(0)
qc.cx(0, 1)

print(qc.run(shots=1000, seed=1))   # {'00': 513, '11': 487}
```

**Documentation and guides:** [qsengine.kaiwave.dev](https://qsengine.kaiwave.dev)

---

## Why qsengine

- **Fast where it counts.** Small circuits run on the CPU with almost no per-call overhead; from 14 qubits up, the GPU takes over. On a 22-qubit benchmark qsengine's GPU backend is about 5 times faster than Qiskit Aer.
- **Any GPU, nothing to install.** The GPU backend uses [wgpu](https://wgpu.rs), so it runs on NVIDIA, AMD, Intel and Apple GPUs through Vulkan, DirectX 12 or Metal. No CUDA, no vendor toolkit.
- **Works everywhere.** Without a usable GPU the same code runs on the CPU, in full 64-bit precision.
- **Small, plain API.** One class, eleven gates, three ways to get a result. Mistakes raise a normal Python `ValueError` on the line that made them.
- **Built to learn with.** The website walks from bra-ket notation to a complete Deutsch-Jozsa run, and every example on it is real qsengine output.

## Install

```bash
pip install qsengine
```

One wheel per platform covers CPython 3.9 and newer on Windows, macOS and Linux. A GPU is optional.

## Using it

A circuit has a fixed number of qubits, all starting in |0⟩. Gate methods record gates; nothing runs until you ask for a result.

```python
import qsengine

qc = qsengine.QuantumCircuit(3)
qc.h(0)
qc.cx(0, 1)
qc.cx(1, 2)

qc.run(shots=1000)          # measurement counts: {'000': ..., '111': ...}
qc.probabilities()          # exact probabilities of all 8 basis states
qc.statevector()            # exact complex amplitudes
```

| Gates | Methods |
| --- | --- |
| Single qubit | `x`, `y`, `z`, `h`, `s`, `t` |
| Rotations (radians) | `rx(q, theta)`, `ry(q, theta)`, `rz(q, theta)` |
| Two qubit | `cx(control, target)`, `cz(control, target)` |

Bitstrings put **qubit 0 on the right**, the same convention as Qiskit. Pass `seed` to `run()` for repeatable counts.

### Choosing a backend

Every method that runs a circuit takes `backend="auto"` (the default), `"cpu"` or `"gpu"`.

| | CPU | GPU | auto |
| --- | --- | --- | --- |
| Precision | 64-bit | 32-bit (within 1e-5 of the CPU) | 64-bit below 14 qubits, 32-bit from 14 |
| Runs on | every CPU core | any NVIDIA, AMD, Intel or Apple GPU | whichever is faster |
| Without a GPU | works | raises `RuntimeError` | falls back to the CPU |

The full API, every error message and a guide to each topic are in the [documentation](https://qsengine.kaiwave.dev/docs/).

## Performance

Each run simulates 5 layers of (H on every qubit, RZ on every qubit, CNOT chain) and samples 1000 shots. Best of several runs on a Ryzen 5 5500 with a Radeon RX 6600, Windows 10, Qiskit Aer 0.18.2.

![qsengine against pure Python and Qiskit Aer](https://raw.githubusercontent.com/kaiwave/qsengine/main/benchmarks/benchmark.png)

| Qubits | Pure Python | qsengine CPU | qsengine GPU | Qiskit Aer |
| ---: | ---: | ---: | ---: | ---: |
| 6 | 2.2 ms | **0.11 ms** | 1.3 ms | 3.7 ms |
| 10 | 30 ms | **0.30 ms** | 1.2 ms | 5.8 ms |
| 14 | 740 ms | 2.4 ms | **1.3 ms** | 11.7 ms |
| 18 | 16.2 s | 25 ms | **4.9 ms** | 19 ms |
| 22 | not run | 1.03 s | **70 ms** | 367 ms |

Below 14 qubits the CPU backend is 7 to 60 times faster than Aer; from 14 qubits the GPU backend is 3 to 9 times faster. Above about 17 qubits the CPU backend is slower than Aer, which fuses multi-qubit gates. These are one machine's numbers: reproduce them with [`benchmarks/benchmark.py`](benchmarks/benchmark.py).

## Limitations

Version 1.0 measures every qubit at the end of the circuit. There is no SWAP gate and no mid-circuit measurement yet, so algorithms such as teleportation that branch on a measurement are not possible. A register holds at most 30 qubits (16 GiB of memory).

## Building from source

You need a recent Rust toolchain (tested with 1.88) and Python 3.9 or newer.

```bash
git clone https://github.com/kaiwave/qsengine
cd qsengine
python -m venv .venv
source .venv/bin/activate       # Windows: .venv\Scripts\activate
pip install maturin pytest numpy
maturin develop --release       # build and install into the venv
```

Tests:

```bash
cargo test --release            # Rust tests (GPU tests skip without a GPU)
pytest                          # Python tests, checked against a NumPy reference
```

To build without the GPU backend, use `cargo build --no-default-features` for Rust.

The website lives in `site/` (Next.js, static export, maths rendered with KaTeX at build time):

```bash
cd site
npm install
npm run dev                     # local preview
npm run build                   # static site in site/out/
```

## Project layout

```text
src/                Rust core and Python bindings
  engine.rs           CPU backend (64-bit, multithreaded with Rayon)
  gpu.rs              GPU backend (wgpu compute shader in shaders/)
  circuit.rs          QuantumCircuit: records gates, fuses them, runs them
  python.rs           PyO3 bindings for the qsengine Python module
qsengine.pyi        Type hints for editors
tests/              Python tests against NumPy
benchmarks/         Benchmark script, results and plot
site/               Documentation and learning website
docs/RUST_GUIDE.md  A guide to every Rust feature the engine uses
```

## Licence

MIT, see [LICENSE](LICENSE). Made by [kaiwave](https://kaiwave.dev).
