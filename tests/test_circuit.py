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
SDG = S.conj()
TDG = T.conj()
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


def P(lam):
    return np.diag([1, np.exp(1j * lam)])


def U(theta, phi, lam):
    c, s = math.cos(theta / 2), math.sin(theta / 2)
    return np.array([[c, -np.exp(1j * lam) * s], [np.exp(1j * phi) * s, np.exp(1j * (phi + lam)) * c]])


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


def toffoli(n, c1, c2, t):
    """Identity everywhere except |11> on the controls, where X hits the target."""
    return np.eye(2**n) - full(n, {c1: P1, c2: P1}) + full(n, {c1: P1, c2: P1, t: X})


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
    singles = {"x": X, "y": Y, "z": Z, "h": H, "s": S, "t": T, "sdg": SDG, "tdg": TDG}
    rotations = {"rx": RX, "ry": RY, "rz": RZ, "p": P}
    for name, *args in gates:
        getattr(qc, name)(*args)
        if name in singles:
            ref.one(singles[name], *args)
        elif name in rotations:
            ref.one(rotations[name](args[1]), args[0])
        elif name == "u":
            ref.one(U(*args[1:]), args[0])
        elif name in ("cp", "crz"):
            ref.two({"cp": P, "crz": RZ}[name](args[2]), args[0], args[1])
        elif name == "ccx":
            ref.state = toffoli(n, *args) @ ref.state
        elif name == "swap":
            a, b = args
            for c, t in ((a, b), (b, a), (a, b)):
                ref.two(X, c, t)
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


@pytest.mark.parametrize("backend,tol", BACKENDS)
def test_new_gates_against_numpy(backend, tol):
    gates = [
        ("h", 0), ("h", 1), ("h", 2), ("sdg", 0), ("tdg", 1), ("p", 2, 0.9), ("u", 3, 0.4, -1.1, 2.3),
        ("cp", 0, 2, 1.7), ("crz", 3, 1, -0.6), ("ccx", 0, 1, 3), ("swap", 0, 3), ("h", 1),
    ]
    qc, ref = build_pair(4, gates)
    np.testing.assert_allclose(qc.statevector(backend=backend), ref.state, atol=tol)


def test_swap_moves_a_bit():
    qc = qsengine.QuantumCircuit(3)
    qc.x(0)
    qc.swap(0, 2)
    assert qc.run(shots=10, seed=0) == {"100": 10}


def test_unitary_matches_named_gate():
    a, b = qsengine.QuantumCircuit(3), qsengine.QuantumCircuit(3)
    for qc in (a, b):
        qc.h(0)
        qc.h(1)
    a.ccx(0, 1, 2)
    b.unitary(X, 2, controls=[0, 1])  # a NumPy array works as the matrix
    a.u(0, 0.3, 0.2, 0.1)
    b.unitary(U(0.3, 0.2, 0.1).tolist(), 0)  # so do nested lists
    np.testing.assert_allclose(a.statevector(backend="cpu"), b.statevector(backend="cpu"), atol=1e-12)


def test_unitary_rejects_bad_matrices():
    qc = qsengine.QuantumCircuit(2)
    with pytest.raises(ValueError, match="unitary"):
        qc.unitary([[1, 1], [0, 1]], 0)
    with pytest.raises(ValueError, match="2x2"):
        qc.unitary(np.eye(4), 0)
    with pytest.raises(ValueError, match="more than once"):
        qc.swap(1, 1)
    assert len(qc) == 0


def test_results_are_numpy_arrays():
    qc = qsengine.QuantumCircuit(2)
    qc.h(0)
    sv, probs = qc.statevector(backend="cpu"), qc.probabilities(backend="cpu")
    assert isinstance(sv, np.ndarray) and sv.dtype == np.complex128 and sv.shape == (4,)
    assert isinstance(probs, np.ndarray) and probs.dtype == np.float64 and probs.shape == (4,)


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
