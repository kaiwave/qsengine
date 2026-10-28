import { CodeBlock } from "../../components/Code";
import { GATES } from "../../content";

export const metadata = { title: "Gates" };

export default function Gates() {
  return (
    <>
      <h1>Gates</h1>
      <p>
        qsengine has eleven gates. Each is a method on <code>QuantumCircuit</code> that records the gate; the circuit
        runs only when you ask for a result. <code>q</code>, <code>control</code> and <code>target</code> are qubit
        numbers starting at 0, and angles are in radians.
      </p>
      <div className="table-wrap">
        <table>
          <thead>
            <tr><th scope="col">Gate</th><th scope="col">Method</th><th scope="col">What it does</th></tr>
          </thead>
          <tbody>
            {GATES.map(([name, call, what]) => (
              <tr key={name}>
                <th scope="row">{name}</th>
                <td><code>{call}</code></td>
                <td>{what}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <h2 id="rules">Rules every gate follows</h2>
      <ul>
        <li>A qubit number must be at least 0 and less than the circuit&rsquo;s <code>num_qubits</code>.</li>
        <li>A two-qubit gate needs two different qubits: <code>qc.cx(1, 1)</code> raises <code>ValueError</code>.</li>
        <li>A gate that raises is not recorded, so the circuit stays as it was.</li>
      </ul>

      <h2 id="combining">Combining gates</h2>
      <p>Gates apply in the order you add them. These lines make a three-qubit GHZ state:</p>
      <CodeBlock code={"qc = qsengine.QuantumCircuit(3)\nqc.h(0)\nqc.cx(0, 1)\nqc.cx(1, 2)"} />
      <p>
        Runs of single-qubit gates on the same qubit are merged into one before the simulation runs, so a long chain
        of rotations costs one pass over the state, not one per gate.
      </p>
    </>
  );
}
