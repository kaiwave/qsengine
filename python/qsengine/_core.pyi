"""Type stub for the compiled `qsengine._core` extension (gives IDEs autocomplete and types).

Import `qsengine`, not `qsengine._core`: the package's `QuantumCircuit` subclasses
this one and adds drawing, `if_test` and conveniences.

Bit order: qubit 0 is the rightmost character of every bitstring and bit 0 of
every basis index, the same convention as Qiskit.
"""

from typing import Literal, Optional, Sequence, Union

import numpy as np
import numpy.typing as npt

__version__: str

Backend = Literal["auto", "cpu", "gpu"]

class QuantumCircuit:
    """A quantum circuit on a fixed number of qubits, starting in |0...0>."""

    def __init__(self, num_qubits: int, num_clbits: int = 0) -> None: ...
    @property
    def num_qubits(self) -> int: ...
    @property
    def num_clbits(self) -> int: ...
    def __len__(self) -> int:
        """Number of operations (gates, measurements, resets) recorded so far."""

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
    def sdg(self, qubit: int) -> None: ...
    def tdg(self, qubit: int) -> None: ...
    def p(self, qubit: int, lam: float) -> None: ...
    def u(self, qubit: int, theta: float, phi: float, lam: float) -> None: ...

    # Two-qubit gates
    def cx(self, control: int, target: int) -> None: ...
    def cz(self, control: int, target: int) -> None: ...
    def cp(self, control: int, target: int, lam: float) -> None: ...
    def crz(self, control: int, target: int, theta: float) -> None: ...
    def swap(self, qubit1: int, qubit2: int) -> None: ...

    # Three-qubit gates
    def ccx(self, control1: int, control2: int, target: int) -> None: ...

    # Any 2x2 unitary, optionally controlled
    def unitary(
        self,
        matrix: Union[Sequence[Sequence[complex]], npt.ArrayLike],
        target: int,
        controls: Sequence[int] = ...,
    ) -> None:
        """Apply a 2x2 unitary to `target`; raises ValueError if it is not unitary within 1e-9.

        A nearly unitary matrix is replaced by the nearest unitary matrix.
        """

    # Measurement and reset
    def measure(self, qubit: int, clbit: int) -> None:
        """Measure `qubit` into classical bit `clbit`, part-way through or at the end."""
    def reset(self, qubit: int) -> None:
        """Reset `qubit` to |0>."""

    # Execution
    def run(
        self, shots: int = 1024, seed: Optional[int] = None, backend: Backend = "auto"
    ) -> dict[str, int]:
        """Run `shots` times; returns {bitstring: count} over classical bits, or over qubits if nothing is measured."""
    def statevector(self, backend: Backend = "auto") -> npt.NDArray[np.complex128]:
        """Final amplitudes, length 2**num_qubits; 32-bit precision from 14 qubits under "auto" with a GPU."""
    def probabilities(self, backend: Backend = "auto") -> npt.NDArray[np.float64]:
        """Final basis-state probabilities, length 2**num_qubits; 32-bit precision from 14 qubits under "auto" with a GPU."""
    def expectation(self, paulis: Sequence[str], backend: Backend = "auto") -> npt.NDArray[np.float64]:
        """Expectation value of each Pauli string, from one simulation."""

    # Used by the Python package
    def _ops(self) -> list[tuple]: ...
    def _set_condition(self, clbits: Sequence[int], value: int) -> None: ...
    def _clear_condition(self) -> None: ...
