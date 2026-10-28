"""Type stub for the compiled `qsengine` extension (gives IDEs autocomplete and types).

Bit order: qubit 0 is the rightmost character of every bitstring and bit 0 of
every basis index, the same convention as Qiskit.
"""

from typing import Literal, Optional

__version__: str

Backend = Literal["auto", "cpu", "gpu"]

class QuantumCircuit:
    """A quantum circuit on a fixed number of qubits, starting in |0...0>."""

    def __init__(self, num_qubits: int) -> None: ...
    @property
    def num_qubits(self) -> int: ...
    def __len__(self) -> int:
        """Number of gates recorded so far."""

    # Single-qubit gates
    def x(self, qubit: int) -> None: ...
    def y(self, qubit: int) -> None: ...
    def z(self, qubit: int) -> None: ...
    def h(self, qubit: int) -> None: ...
    def s(self, qubit: int) -> None: ...
    def t(self, qubit: int) -> None: ...
    def rx(self, qubit: int, theta: float) -> None: ...
    def ry(self, qubit: int, theta: float) -> None: ...
    def rz(self, qubit: int, theta: float) -> None: ...

    # Two-qubit gates
    def cx(self, control: int, target: int) -> None: ...
    def cz(self, control: int, target: int) -> None: ...

    # Execution
    def run(
        self, shots: int = 1024, seed: Optional[int] = None, backend: Backend = "auto"
    ) -> dict[str, int]:
        """Measure all qubits `shots` times; returns {bitstring: count}."""
    def statevector(self, backend: Backend = "auto") -> list[complex]:
        """Exact final amplitudes, length 2**num_qubits."""
    def probabilities(self, backend: Backend = "auto") -> list[float]:
        """Exact final basis-state probabilities, length 2**num_qubits."""
