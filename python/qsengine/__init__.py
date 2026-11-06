"""qsengine: a fast state-vector quantum simulator with a Rust core.

The simulation lives in the compiled `qsengine._core` module. This package
subclasses its `QuantumCircuit` in plain Python for everything that is not
simulation: drawing, `if_test` blocks and argument conveniences. Gate methods
are inherited unchanged, so recording a gate still costs one call into Rust.
"""

from contextlib import contextmanager
from typing import Iterator, Sequence, Union, overload

import numpy as np
import numpy.typing as npt

from . import _core
from ._core import __version__
from ._drawer import draw

__all__ = ["QuantumCircuit", "__version__"]


class QuantumCircuit(_core.QuantumCircuit):
    __doc__ = _core.QuantumCircuit.__doc__
    __slots__ = ()  # no per-instance __dict__, like the Rust class

    def __str__(self) -> str:
        """Text drawing of the circuit, one wire per qubit: `print(qc)`."""
        return draw(self)

    def measure(self, qubit: Union[int, Sequence[int]], clbit: Union[int, Sequence[int]]) -> None:
        """Measure `qubit` into classical bit `clbit`. Both may be equal-length lists."""
        if isinstance(qubit, int) and isinstance(clbit, int):
            return super().measure(qubit, clbit)
        qubits, clbits = list(qubit), list(clbit)  # type: ignore[arg-type]
        if len(qubits) != len(clbits):
            raise ValueError(f"measure got {len(qubits)} qubits but {len(clbits)} classical bits")
        for q, c in zip(qubits, clbits):
            super().measure(q, c)

    @contextmanager
    def if_test(self, clbits: Union[int, Sequence[int]], value: int) -> Iterator["QuantumCircuit"]:
        """Gates added inside the block run only when the classical bits equal `value`.

        `clbits` is one classical bit or a list; bit i of `value` is the value
        required of `clbits[i]`. For example `with qc.if_test(1, 1): qc.x(2)`
        flips qubit 2 in the shots where classical bit 1 was measured as 1.
        """
        self._set_condition([clbits] if isinstance(clbits, int) else list(clbits), value)
        try:
            yield self
        finally:
            self._clear_condition()

    @overload
    def expectation(self, paulis: str, backend: str = ...) -> float: ...
    @overload
    def expectation(self, paulis: Sequence[str], backend: str = ...) -> npt.NDArray[np.float64]: ...
    def expectation(self, paulis, backend="auto"):
        """Expectation value of a Pauli string such as "ZZI" (qubit 0 rightmost).

        Pass a list of strings to get a NumPy array from a single simulation;
        a weighted sum `coeffs @ qc.expectation(terms)` gives a Hamiltonian's energy.
        """
        if isinstance(paulis, str):
            return float(super().expectation([paulis], backend)[0])
        return super().expectation(list(paulis), backend)
