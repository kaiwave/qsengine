# Contributing to qsengine

Thanks for helping. Bug reports, examples, docs fixes and new gates are all welcome. For anything larger than a small fix, open an issue first so we can agree on the shape before you write the code.

## Set up

You need a recent Rust toolchain (tested with 1.88), Python 3.9 or newer, and Node 22 for the website.

```bash
git clone https://github.com/kaiwave/qsengine
cd qsengine
python -m venv .venv
source .venv/bin/activate       # Windows: .venv\Scripts\activate
pip install maturin pytest numpy
maturin develop --release       # build the Rust core and install the package in the venv
```

Run every check before you open a pull request:

```bash
cargo test --release            # Rust tests (GPU tests skip without a GPU)
cargo clippy --release --all-targets --features python
pytest                          # Python tests, checked against a NumPy reference
```

Re-run `maturin develop --release` after every Rust change; Python-only changes in `python/qsengine/` apply straight away.

## Where things live

| Path | What it is |
| --- | --- |
| `src/gates.rs` | 2x2 gate matrices |
| `src/circuit.rs` | `QuantumCircuit`: recording, validation, gate fusion, `run()` |
| `src/backend.rs` | The `QuantumBackend` trait, measurement, and the backend test suite |
| `src/engine.rs`, `src/gpu.rs`, `src/shaders/` | CPU and GPU backends |
| `src/pauli.rs` | Pauli-string expectation values |
| `src/python.rs` | PyO3 bindings, built as `qsengine._core` |
| `python/qsengine/` | The Python package: subclasses `_core.QuantumCircuit` and adds the drawer, `if_test` and conveniences |
| `tests/test_circuit.py` | Python tests against a NumPy reference simulator |
| `site/` | The documentation website (Next.js, static export) |

**Simulation stays in Rust.** Anything that is not simulation and is easier in Python (drawing, parsing, plotting helpers) can be a plain file in `python/qsengine/`, as long as it does not slow down recording gates or running circuits.

## Adding a gate

Use an existing gate of the same shape as your template: `p` for one qubit with an angle, `cp` for a controlled one. A gate touches these places:

1. **`src/gates.rs`**: the 2x2 matrix, as a `const` or a function of its angles. Add it to the `all_gates_are_unitary` test.
2. **`src/circuit.rs`**: a method that calls `self.push(Label::new("name", &[angles]), &[controls], target, matrix)`. The label name must match the Python method name.
3. **`src/python.rs`**: the `#[pymethods]` wrapper, with a one-line docstring.
4. **`python/qsengine/_core.pyi`**: the type signature.
5. **`python/qsengine/_drawer.py`**: the symbol `print(qc)` draws for it, in `SYMBOLS`.
6. **`tests/test_circuit.py`**: add it to `build_pair` and to a test that compares against the NumPy reference, on both backends.
7. **Docs**: `site/app/docs/gates/page.mdx`, the API reference in `site/app/docs/api/page.jsx`, and the gate table in `README.md`.

A gate made only of existing 2x2 matrices and controls needs no backend or shader changes: every backend runs every gate through one kernel.

## Pull requests

- Keep one change per pull request, with tests that fail without it.
- Write code like the code around it: the same naming, comment density and error style. Errors are `QsError` variants in Rust and become `ValueError` in Python.
- Every output printed on the website comes from really running the code. If your change alters an output, update the page.
- Note user-visible changes in `site/app/docs/release-notes/page.mdx`.

By contributing you agree that your work is released under the MIT licence.
