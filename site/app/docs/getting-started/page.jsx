import Link from "next/link";
import { CodeBlock, Output } from "../../components/Code";
import { BELL, VERSION } from "../../content";

export const metadata = { title: "Installation and first circuit" };

export default function GettingStarted() {
  return (
    <>
      <h1>Installation and first circuit</h1>
      <p>This page installs qsengine, builds a two-qubit circuit and reads its measurement counts. It takes a few minutes.</p>

      <h2 id="install">Install</h2>
      <p>qsengine needs CPython 3.9 or newer. One wheel per platform covers every version, so there is nothing to compile.</p>
      <CodeBlock code="pip install qsengine" label="Copy the install command" />
      <p>Check that it imports and shows its version:</p>
      <CodeBlock code={"import qsengine\nprint(qsengine.__version__)"} />
      <Output printed={VERSION} />
      <p>
        A GPU is optional. With one, large circuits run on it; without one, everything runs on the CPU. See{" "}
        <Link href="/docs/backends/">CPU and GPU backends</Link>.
      </p>

      <h2 id="first-circuit">Your first circuit</h2>
      <p>
        A circuit has a fixed number of qubits that all start in |0⟩. You add gates by calling methods on it. Nothing
        runs until you ask for a result.
      </p>
      <CodeBlock code={BELL.code} label="Copy the first circuit" />
      <Output printed={BELL.printed} rows={BELL.rows} caption={BELL.caption} />
      <p>Line by line:</p>
      <ul>
        <li><code>QuantumCircuit(2)</code> makes a circuit on qubits 0 and 1.</li>
        <li><code>qc.h(0)</code> puts qubit 0 into an equal superposition of 0 and 1.</li>
        <li><code>qc.cx(0, 1)</code> flips qubit 1 whenever qubit 0 is 1, which ties the two together.</li>
        <li><code>qc.run(shots=1000, seed=1)</code> simulates the circuit once, measures it 1000 times and returns how often each result appeared.</li>
      </ul>
      <p>
        The keys are bitstrings with qubit 0 on the right. <code>seed</code> makes the counts repeatable; leave it out
        for fresh randomness.
      </p>

      <h2 id="next">Next steps</h2>
      <ul>
        <li><Link href="/docs/examples/">Examples</Link> show exact amplitudes, rotations and error handling.</li>
        <li><Link href="/docs/gates/">Gates</Link> lists every gate you can add.</li>
        <li><Link href="/docs/running/">Running circuits</Link> explains shots, seeds and exact states.</li>
      </ul>
    </>
  );
}
