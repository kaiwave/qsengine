"""Benchmark qsengine against a pure-Python simulator and Qiskit Aer.

Every simulator runs the same circuit and does the same job: simulate it, then
sample `--shots` measurement outcomes. That's the real end-to-end workload,
where only the counts come back to Python.

The circuit has `--layers` repetitions of:
    H on every qubit -> RZ(0.3) on every qubit -> CNOT chain 0->1->...->n-1

Usage (from the repo root, inside the venv after `maturin develop --release`):
    python benchmarks/benchmark.py                  # 2..20 qubits
    python benchmarks/benchmark.py --max-qubits 24

Writes benchmarks/results.csv and benchmarks/benchmark.png.
Optional dependencies: qiskit + qiskit-aer (skipped if missing), matplotlib (for the plot).
"""

import argparse
import cmath
import csv
import math
import random
import time
from pathlib import Path

import qsengine

HERE = Path(__file__).parent
THETA = 0.3


def circuit_gates(n, layers):
    """The benchmark circuit as a plain list of (name, *args) tuples."""
    gates = []
    for _ in range(layers):
        gates += [("h", q) for q in range(n)]
        gates += [("rz", q, THETA) for q in range(n)]
        gates += [("cx", q, q + 1) for q in range(n - 1)]
    return gates


# --- Simulators: each `prepare_*` does the untimed setup and returns a zero-arg
# --- function that simulates + samples (the part that gets timed).


def prepare_pure_python(n, gates, shots):
    """Textbook state-vector loop in plain Python: the 'no Rust' baseline."""
    r = 1 / math.sqrt(2)
    rz = [[cmath.exp(-0.5j * THETA), 0], [0, cmath.exp(0.5j * THETA)]]
    matrices = {"h": [[r, r], [r, -r]], "rz": rz, "cx": [[0, 1], [1, 0]]}

    def apply(state, U, target, control):
        (a, b), (c, d) = U
        stride = 1 << target
        for i in range(len(state)):
            if i & stride or (control is not None and not (i >> control) & 1):
                continue
            j = i | stride
            x, y = state[i], state[j]
            state[i], state[j] = a * x + b * y, c * x + d * y

    def run():
        state = [0j] * (1 << n)
        state[0] = 1 + 0j
        for name, *args in gates:
            if name == "cx":
                apply(state, matrices["cx"], args[1], args[0])
            else:
                apply(state, matrices[name], args[0], None)
        probs = [abs(a) ** 2 for a in state]
        return random.choices(range(len(state)), weights=probs, k=shots)

    return run


def prepare_qsengine(backend):
    def prepare(n, gates, shots):
        qc = qsengine.QuantumCircuit(n)
        for name, *args in gates:
            getattr(qc, name)(*args)
        return lambda: qc.run(shots=shots, backend=backend)

    return prepare


def prepare_aer(n, gates, shots):
    from qiskit import QuantumCircuit, transpile
    from qiskit_aer import AerSimulator

    sim = AerSimulator(method="statevector")
    qc = QuantumCircuit(n)
    for name, *args in gates:
        if name == "rz":
            qc.rz(args[1], args[0])  # Qiskit's order is rz(angle, qubit)
        else:
            getattr(qc, name)(*args)
    qc.measure_all()
    qc = transpile(qc, sim)  # once, outside the timed region
    return lambda: sim.run(qc, shots=shots).result().get_counts()


# --- Harness -----------------------------------------------------------------


def best_time(fn, budget=0.5, max_reps=7):
    """Fastest of several runs: repeat until `budget` seconds or `max_reps` runs."""
    times = []
    while len(times) < max_reps and sum(times) < budget:
        start = time.perf_counter()
        fn()
        times.append(time.perf_counter() - start)
    return min(times)


def available_simulators():
    sims = {
        "pure Python": prepare_pure_python,
        "qsengine (CPU)": prepare_qsengine("cpu"),
    }
    try:
        qsengine.QuantumCircuit(1).run(shots=1, backend="gpu")  # also warms up the GPU
        sims["qsengine (GPU)"] = prepare_qsengine("gpu")
    except RuntimeError as e:
        print(f"skipping qsengine (GPU): {e}")
    try:
        import qiskit_aer  # noqa: F401

        sims["Qiskit Aer"] = prepare_aer
    except ImportError:
        print("skipping Qiskit Aer: pip install qiskit qiskit-aer")
    return sims


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--min-qubits", type=int, default=2)
    p.add_argument("--max-qubits", type=int, default=20)
    p.add_argument("--layers", type=int, default=5)
    p.add_argument("--shots", type=int, default=1000)
    p.add_argument("--time-limit", type=float, default=10.0,
                   help="stop a simulator once one run takes longer than this (seconds)")
    args = p.parse_args()

    sims = available_simulators()
    rows, stopped = [], set()
    print(f"{'qubits':>6}  " + "  ".join(f"{name:>15}" for name in sims))
    for n in range(args.min_qubits, args.max_qubits + 1):
        gates = circuit_gates(n, args.layers)
        line = []
        for name, prepare in sims.items():
            if name in stopped:
                line.append(f"{'-':>15}")
                continue
            seconds = best_time(prepare(n, gates, args.shots))
            rows.append({"qubits": n, "simulator": name, "seconds": seconds})
            line.append(f"{seconds * 1000:>12.3f} ms")
            if seconds > args.time_limit:
                stopped.add(name)  # each extra qubit doubles the work, so stop here
        print(f"{n:>6}  " + "  ".join(line), flush=True)

    with open(HERE / "results.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["qubits", "simulator", "seconds"])
        w.writeheader()
        w.writerows(rows)
    plot(rows, args)


def plot(rows, args):
    try:
        import matplotlib

        matplotlib.use("Agg")  # file output only, no window
        import matplotlib.pyplot as plt
    except ImportError:
        print("skipping plot: pip install matplotlib")
        return

    by_sim = {}
    for r in rows:
        by_sim.setdefault(r["simulator"], []).append((r["qubits"], r["seconds"]))

    # One fixed colour per simulator so both panels match.
    colors = dict(zip(by_sim, plt.rcParams["axes.prop_cycle"].by_key()["color"]))
    fig, (ax_t, ax_s) = plt.subplots(1, 2, figsize=(12, 4.8))
    for name, pts in by_sim.items():
        xs, ys = zip(*pts)
        ax_t.plot(xs, [y * 1000 for y in ys], marker="o", ms=3, label=name, color=colors[name])
    ax_t.set(yscale="log", xlabel="qubits", ylabel="time per run (ms, log scale)",
             title=f"Simulate + {args.shots} shots ({args.layers} layers of H, RZ, CNOT chain)")
    ax_t.grid(True, which="both", alpha=0.3)
    ax_t.legend()

    # Speedup over Aer (or over pure Python if Aer is missing).
    ref_name = "Qiskit Aer" if "Qiskit Aer" in by_sim else "pure Python"
    ref = dict(by_sim[ref_name])
    for name, pts in by_sim.items():
        if name.startswith("qsengine"):
            xs = [q for q, _ in pts if q in ref]
            ax_s.plot(xs, [ref[q] / s for q, s in pts if q in ref], marker="o", ms=3, label=name, color=colors[name])
    ax_s.axhline(1, color="gray", lw=1, ls="--")
    ax_s.set(yscale="log", xlabel="qubits", ylabel=f"speedup vs {ref_name} (x, log scale)",
             title=f"qsengine speedup over {ref_name} (>1 = faster)")
    ax_s.grid(True, which="both", alpha=0.3)
    ax_s.legend()

    for ax in (ax_t, ax_s):
        ax.xaxis.get_major_locator().set_params(integer=True)
    fig.tight_layout()
    fig.savefig(HERE / "benchmark.png", dpi=130)
    print(f"wrote {HERE / 'results.csv'} and {HERE / 'benchmark.png'}")


if __name__ == "__main__":
    main()
