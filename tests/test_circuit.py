"""Check qsengine against a tiny, obviously-correct NumPy simulator.

The NumPy reference builds each gate as a full 2**n x 2**n matrix with
Kronecker products. That's slow, but it's a completely different method from
qsengine's in-place kernels, so agreement is strong evidence both are right.

Run with:  pytest
"""

import math

import numpy as np
import pytest

import qsengine

# --- NumPy reference ---------------------------------------------------------

I2 = np.eye(2, dtype=complex)
X = np.array([[0, 1], [1, 0]], dtype=complex)
Y = np.array([[0, -1j], [1j, 0]], dtype=complex)
Z = np.diag([1, -1]).astype(complex)
H = np.array([[1, 1], [1, -1]], dtype=complex) / math.sqrt(2)
S = np.diag([1, 1j])
T = np.diag([1, np.exp(1j * math.pi / 4)])
P0 = np.diag([1, 0]).astype(complex)  # |0><0|
P1 = np.diag([0, 1]).astype(complex)  # |1><1|


def RX(t):
    c, s = math.cos(t / 2), math.sin(t / 2)
    return np.array([[c, -1j * s], [-1j * s, c]])


def RY(t):
    c, s = math.cos(t / 2), math.sin(t / 2)
    return np.array([[c, -s], [s, c]], dtype=complex)


def RZ(t):
    return np.diag([np.exp(-1j * t / 2), np.exp(1j * t / 2)])


def full(n, ops):
    """Kronecker product placing ops[q] on qubit q and identity elsewhere.

    Qubit 0 is the least-significant bit, so it is the *rightmost* factor.
    """
    out = np.array([[1]], dtype=complex)
    for q in reversed(range(n)):
        out = np.kron(out, ops.get(q, I2))
    return out


def controlled(n, c, t, U):
    """|0><0| on the control does nothing; |1><1| applies U to the target."""
    return full(n, {c: P0}) + full(n, {c: P1, t: U})


class Reference:
    """Mirror of QuantumCircuit that records the same gates as NumPy matrices."""

    def __init__(self, n):
        self.n = n
        self.state = np.zeros(2**n, dtype=complex)
        self.state[0] = 1

    def one(self, U, q):
        self.state = full(self.n, {q: U}) @ self.state

    def two(self, U, c, t):
        self.state = controlled(self.n, c, t, U) @ self.state


def build_pair(n, gates):
    """Apply the same gate list to a qsengine circuit and to the reference."""
    qc, ref = qsengine.QuantumCircuit(n), Reference(n)
    singles = {"x": X, "y": Y, "z": Z, "h": H, "s": S, "t": T}
    rotations = {"rx": RX, "ry": RY, "rz": RZ}
    for name, *args in gates:
        getattr(qc, name)(*args)
        if name in singles:
            ref.one(singles[name], *args)
        elif name in rotations:
            ref.one(rotations[name](args[1]), args[0])
        else:
            ref.two({"cx": X, "cz": Z}[name], *args)
    return qc, ref


def gpu_available():
    try:
        qsengine.QuantumCircuit(1).statevector(backend="gpu")
        return True
    except RuntimeError:
        return False


BACKENDS = [
    pytest.param("cpu", 1e-12, id="cpu"),
    pytest.param(
        "gpu",
        1e-5,  # the GPU computes in f32
        id="gpu",
        marks=pytest.mark.skipif(not gpu_available(), reason="no usable GPU"),
    ),
]

# --- Tests -------------------------------------------------------------------


@pytest.mark.parametrize("backend,tol", BACKENDS)
def test_bell_state(backend, tol):
    qc, ref = build_pair(2, [("h", 0), ("cx", 0, 1)])
    np.testing.assert_allclose(qc.statevector(backend=backend), ref.state, atol=tol)


@pytest.mark.parametrize("backend,tol", BACKENDS)
def test_every_gate_against_numpy(backend, tol):
    gates = [
        ("h", 0), ("h", 1), ("x", 2), ("y", 3), ("z", 1), ("s", 0), ("t", 2),
        ("rx", 1, 0.3), ("ry", 2, 1.2), ("rz", 3, -0.7),
        ("cx", 0, 3), ("cz", 2, 1), ("cx", 3, 1), ("h", 2), ("t", 3),
    ]
    qc, ref = build_pair(4, gates)
    np.testing.assert_allclose(qc.statevector(backend=backend), ref.state, atol=tol)
    np.testing.assert_allclose(qc.probabilities(backend=backend), np.abs(ref.state) ** 2, atol=tol)


@pytest.mark.parametrize("backend,tol", BACKENDS)
def test_random_circuits_against_numpy(backend, tol):
    rng = np.random.default_rng(0)
    for _ in range(20):
        n = int(rng.integers(1, 6))
        gates = []
        for _ in range(30):
            q = int(rng.integers(n))
            kind = rng.choice(["h", "x", "y", "z", "s", "t", "rx", "ry", "rz", "cx", "cz"])
            if kind in ("cx", "cz"):
                if n < 2:
                    continue
                c, t = rng.choice(n, size=2, replace=False)
                gates.append((kind, int(c), int(t)))
            elif kind.startswith("r"):
                gates.append((kind, q, float(rng.uniform(-math.pi, math.pi))))
            else:
                gates.append((kind, q))
        qc, ref = build_pair(n, gates)
        np.testing.assert_allclose(qc.statevector(backend=backend), ref.state, atol=tol)


def test_run_returns_bitstring_counts():
    qc, _ = build_pair(2, [("h", 0), ("cx", 0, 1)])
    counts = qc.run(shots=4000, seed=1)
    assert set(counts) <= {"00", "11"}
    assert sum(counts.values()) == 4000
    assert abs(counts["00"] / 4000 - 0.5) < 0.05


def test_bit_order_qubit_zero_is_rightmost():
    qc = qsengine.QuantumCircuit(3)
    qc.x(0)
    assert qc.run(shots=10, seed=0) == {"001": 10}


def test_seed_makes_runs_reproducible():
    qc, _ = build_pair(3, [("h", 0), ("h", 1), ("h", 2)])
    assert qc.run(shots=500, seed=42) == qc.run(shots=500, seed=42)


def test_running_does_not_change_the_circuit():
    qc, ref = build_pair(2, [("h", 0)])
    qc.run(shots=100)
    qc.run(shots=100)
    np.testing.assert_allclose(qc.statevector(backend="cpu"), ref.state, atol=1e-12)


def test_invalid_input_raises_value_error():
    qc = qsengine.QuantumCircuit(2)
    with pytest.raises(ValueError, match="out of range"):
        qc.h(2)
    with pytest.raises(ValueError, match="more than once"):
        qc.cx(1, 1)
    with pytest.raises(ValueError, match="unknown backend"):
        qc.run(backend="tpu")
    with pytest.raises(ValueError, match="maximum"):
        qsengine.QuantumCircuit(64)
    assert len(qc) == 0  # failed gates were not recorded


def test_repr_and_len():
    qc = qsengine.QuantumCircuit(2)
    qc.h(0)
    qc.cx(0, 1)
    assert len(qc) == 2
    assert qc.num_qubits == 2
    assert repr(qc) == "QuantumCircuit(num_qubits=2, gates=2)"
