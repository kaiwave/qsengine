import { CodeBlock } from "../../components/Code";
import { VERSION } from "../../content";

export const metadata = { title: "API reference" };

// [signature, anchor, description, returns or raises]
const METHODS = [
  ["QuantumCircuit(num_qubits, num_clbits=0)", "init", "Create an empty circuit on num_qubits qubits, all starting in |0⟩, and num_clbits classical bits, all starting at 0. At most 30 qubits and 64 classical bits.", "Raises ValueError if either is above its maximum."],
  ["qc.num_qubits, qc.num_clbits", "num-qubits", "The number of qubits and of classical bits. Read-only.", "Returns int."],
  ["len(qc)", "len", "The number of operations recorded so far: gates, measurements and resets. A swap counts as one.", "Returns int."],
  ["print(qc), str(qc)", "str", "A text drawing of the circuit, one wire per qubit with qubit 0 on top. ● marks a control, × the ends of a swap.", "Returns str."],
  ["qc.h(q), qc.x(q), qc.y(q), qc.z(q), qc.s(q), qc.sdg(q), qc.t(q), qc.tdg(q)", "single", "Record a single-qubit gate on qubit q.", "Raises ValueError if q is out of range."],
  ["qc.rx(q, theta), qc.ry(q, theta), qc.rz(q, theta), qc.p(q, lam)", "rotations", "Record a rotation of theta radians about the X, Y or Z axis, or a phase of lam radians.", "Raises ValueError if q is out of range or the angle is NaN or infinite."],
  ["qc.u(q, theta, phi, lam)", "u", "Record the general single-qubit gate U, in the same convention as Qiskit.", "Raises ValueError if q is out of range or an angle is NaN or infinite."],
  ["qc.cx(control, target), qc.cz(control, target), qc.cp(control, target, lam), qc.crz(control, target, theta)", "two-qubit", "Record a CNOT, controlled-Z, controlled-phase or controlled-RZ gate.", "Raises ValueError if a qubit is out of range, control equals target, or an angle is NaN or infinite."],
  ["qc.swap(qubit1, qubit2)", "swap", "Record a SWAP, as three CNOTs.", "Raises ValueError if a qubit is out of range or the two are equal."],
  ["qc.ccx(control1, control2, target)", "ccx", "Record a Toffoli gate.", "Raises ValueError if a qubit is out of range or used twice."],
  ["qc.measure(qubit, clbit)", "measure", "Measure qubit into classical bit clbit, part-way through the circuit or at the end. Both may be equal-length lists, as in qc.measure([0, 1], [0, 1]).", "Raises ValueError if a qubit or classical bit is out of range, or inside if_test."],
  ["qc.reset(qubit)", "reset", "Put qubit back to |0⟩, whatever state it is in, so it can be used again.", "Raises ValueError if qubit is out of range, or inside if_test."],
  ["with qc.if_test(clbits, value):", "if-test", "Gates added inside the block run only in shots where the classical bits equal value. clbits is one classical bit or a list; bit i of value is the value required of clbits[i].", "Raises ValueError if a classical bit is out of range, value does not fit, or blocks are nested."],
  ["qc.unitary(matrix, target, controls=[])", "unitary", "Record any 2x2 unitary matrix (nested lists or a NumPy array) on target, applied only where every qubit in controls is |1⟩.", "Raises ValueError if the matrix is not 2x2 or not unitary, or a qubit is out of range or used twice."],
  ['qc.run(shots=1024, seed=None, backend="auto")', "run", "Simulate the circuit shots times and count the results. If the circuit measures, the bitstrings are its classical bits; otherwise every qubit is measured at the end. A circuit that measures part-way is simulated once per shot. Pass seed for repeatable counts.", "Returns dict[str, int] mapping bitstrings (bit 0 on the right) to counts."],
  ['qc.probabilities(backend="auto")', "probabilities", "Simulate the circuit and return the exact probability of each of the 2ⁿ basis states. Measurements at the very end are ignored.", "Returns a NumPy float64 array. Raises ValueError if the circuit measures or resets before a later gate."],
  ['qc.statevector(backend="auto")', "statevector", "Simulate the circuit and return the exact amplitudes. Measurements at the very end are ignored.", "Returns a NumPy complex128 array. Raises ValueError if the circuit measures or resets before a later gate."],
  ['qc.expectation(paulis, backend="auto")', "expectation", "The expectation value of a Pauli string such as \"ZZI\" (one letter per qubit, qubit 0 on the right), or of each string in a list, from one simulation.", "Returns float for one string, a NumPy float64 array for a list. Raises ValueError for a string of the wrong length or with letters other than I, X, Y, Z."],
  ["qsengine.__version__", "version", "The installed version.", `Returns str, for example "${VERSION}".`],
];

const ERRORS = [
  ["QuantumCircuit(31)", "ValueError: requested 31 qubits, maximum is 30"],
  ["qc.h(2) on 2 qubits", "ValueError: qubit 2 out of range for a 2-qubit register"],
  ["qc.cz(0, 0)", "ValueError: qubit 0 used more than once in one gate"],
  ['qc.rx(0, float("nan"))', "ValueError: gate angles must be finite numbers, not NaN or infinity"],
  ["qc.unitary([[1, 1], [0, 1]], 0)", "ValueError: gate matrix must be a 2x2 unitary"],
  ["qc.measure(0, 1) with 1 classical bit", "ValueError: classical bit 1 out of range for a circuit with 1 classical bits"],
  ["qc.statevector() after a mid-circuit measurement", "ValueError: the circuit measures or resets a qubit before a later gate, so its final state is random; use run()"],
  ['qc.expectation("ZQ")', 'ValueError: Pauli string "ZQ" may only contain I, X, Y and Z'],
  ["qc.unitary(numpy.eye(4), 0)", "ValueError: matrix must be 2x2"],
  ['qc.run(backend="tpu")', 'ValueError: unknown backend "tpu"; expected "auto", "cpu" or "gpu"'],
  ["qc.x(-1)", "OverflowError: can't convert negative int to unsigned"],
  ['backend="gpu" with no usable GPU', "RuntimeError, with the reason from the driver"],
  ["A state too large for memory", "MemoryError: not enough memory for the state vector (16.0 GiB)"],
];

export default function Api() {
  return (
    <>
      <h1>API reference</h1>
      <p>
        The whole public API is one class, <code>qsengine.QuantumCircuit</code>, and a version string. Backends are
        chosen with the <code>backend</code> argument: <code>&quot;auto&quot;</code>, <code>&quot;cpu&quot;</code> or{" "}
        <code>&quot;gpu&quot;</code>.
      </p>
      <CodeBlock code={"import qsengine\n\nqc = qsengine.QuantumCircuit(2)"} />

      <h2 id="quantumcircuit">QuantumCircuit</h2>
      <dl className="api">
        {METHODS.map(([sig, id, what, ret]) => (
          <div key={id} id={id}>
            <dt><code>{sig}</code></dt>
            <dd>
              <p>{what}</p>
              <p className="api__returns">{ret}</p>
            </dd>
          </div>
        ))}
      </dl>

      <h2 id="errors">Errors</h2>
      <p>
        Invalid input raises immediately, on the line that caused it, and the gate is not recorded. These are the
        exact messages:
      </p>
      <div className="table-wrap">
        <table>
          <thead><tr><th scope="col">Call</th><th scope="col">Raises</th></tr></thead>
          <tbody>
            {ERRORS.map(([call, err]) => (
              <tr key={call}><th scope="row"><code>{call}</code></th><td><code>{err}</code></td></tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}
