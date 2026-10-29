import { CodeBlock } from "../../components/Code";
import { VERSION } from "../../content";

export const metadata = { title: "API reference" };

// [signature, anchor, description, returns or raises]
const METHODS = [
  ["QuantumCircuit(num_qubits)", "init", "Create an empty circuit on num_qubits qubits, all starting in |0⟩. At most 30.", "Raises ValueError if num_qubits is above 30."],
  ["qc.num_qubits", "num-qubits", "The number of qubits. Read-only.", "Returns int."],
  ["len(qc)", "len", "The number of gates recorded so far.", "Returns int."],
  ["qc.h(q), qc.x(q), qc.y(q), qc.z(q), qc.s(q), qc.t(q)", "single", "Record a single-qubit gate on qubit q.", "Raises ValueError if q is out of range."],
  ["qc.rx(q, theta), qc.ry(q, theta), qc.rz(q, theta)", "rotations", "Record a rotation of theta radians about the X, Y or Z axis.", "Raises ValueError if q is out of range."],
  ["qc.cx(control, target), qc.cz(control, target)", "two-qubit", "Record a CNOT or controlled-Z gate.", "Raises ValueError if a qubit is out of range or control equals target."],
  ['qc.run(shots=1024, seed=None, backend="auto")', "run", "Simulate the circuit and measure every qubit shots times. Pass seed for repeatable counts.", "Returns dict[str, int] mapping bitstrings (qubit 0 on the right) to counts."],
  ['qc.probabilities(backend="auto")', "probabilities", "Simulate the circuit and return the exact probability of each of the 2ⁿ basis states.", "Returns list[float]."],
  ['qc.statevector(backend="auto")', "statevector", "Simulate the circuit and return the exact amplitudes.", "Returns list[complex]."],
  ["qsengine.__version__", "version", "The installed version.", `Returns str, for example "${VERSION}".`],
];

const ERRORS = [
  ["QuantumCircuit(31)", "ValueError: requested 31 qubits, maximum is 30"],
  ["qc.h(2) on 2 qubits", "ValueError: qubit 2 out of range for a 2-qubit register"],
  ["qc.cz(0, 0)", "ValueError: qubit 0 used more than once in one gate"],
  ['qc.run(backend="tpu")', 'ValueError: unknown backend "tpu"; expected "auto", "cpu" or "gpu"'],
  ["qc.x(-1)", "OverflowError: can't convert negative int to unsigned"],
  ['backend="gpu" with no usable GPU', "RuntimeError, with the reason from the driver"],
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
