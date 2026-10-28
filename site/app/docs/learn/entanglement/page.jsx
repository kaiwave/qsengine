import Link from "next/link";
import { CodeBlock, Output } from "../../../components/Code";
import { M, MBlock } from "../../../components/Math";
import { BELL } from "../../../content";

export const metadata = { title: "Entanglement" };

export default function Entanglement() {
  return (
    <>
      <h1>Entanglement</h1>
      <p>
        Two qubits are entangled when their joint state cannot be split into a state for each one. Measuring one then
        tells you about the other, however far apart they are.
      </p>

      <h2 id="bell">Building a Bell state</h2>
      <p>Start with both qubits in <M>{String.raw`\ket{0}`}</M> and apply a Hadamard to qubit 0:</p>
      <MBlock>{String.raw`\ket{00} \xrightarrow{\;H_0\;} \tfrac{1}{\sqrt2}\left(\ket{00} + \ket{01}\right)`}</MBlock>
      <p>
        Qubit 0 is the right-hand digit, so it is now in superposition and qubit 1 is still 0. This is still a product
        state. Now a CNOT with qubit 0 as control flips qubit 1 only in the term where qubit 0 is 1:
      </p>
      <MBlock>{String.raw`\tfrac{1}{\sqrt2}\left(\ket{00} + \ket{01}\right) \xrightarrow{\;\mathrm{CNOT}_{0 \to 1}\;} \tfrac{1}{\sqrt2}\left(\ket{00} + \ket{11}\right)`}</MBlock>
      <p>That last state is a <em>Bell state</em>. In qsengine it is two lines:</p>
      <CodeBlock code={BELL.code} label="Copy the Bell state example" />
      <Output printed={BELL.printed} rows={BELL.rows} caption={BELL.caption} />

      <h2 id="why">Why it cannot be split</h2>
      <p>If the Bell state were a product of two single-qubit states, then by the tensor product rule</p>
      <MBlock>{String.raw`(\alpha\ket0 + \beta\ket1)\otimes(\gamma\ket0 + \delta\ket1) = \alpha\gamma\ket{00} + \alpha\delta\ket{01} + \beta\gamma\ket{10} + \beta\delta\ket{11}`}</MBlock>
      <p>
        and we would need <M>{String.raw`\alpha\gamma = \beta\delta = \tfrac{1}{\sqrt2}`}</M> while{" "}
        <M>{String.raw`\alpha\delta = \beta\gamma = 0`}</M>. The second pair forces one of the four numbers to be zero,
        which breaks the first pair. No choice works, so the state is entangled.
      </p>

      <h2 id="correlation">Correlation, not communication</h2>
      <p>
        Each qubit on its own looks like a fair coin: 0 or 1, half the time each. Only the pair is special, because the
        two results always agree. Compare a Hadamard on both qubits, which gives four outcomes with no link between
        the qubits:
      </p>
      <CodeBlock code={"qc = qsengine.QuantumCircuit(2)\nqc.h(0)\nqc.cx(0, 1)\nprint(qc.probabilities())   # entangled\n\nqc = qsengine.QuantumCircuit(2)\nqc.h(0)\nqc.h(1)\nprint(qc.probabilities())   # independent"} />
      <Output printed={"[0.5000000000000001, 0.0, 0.0, 0.5000000000000001]\n[0.2500000000000001, 0.2500000000000001, 0.2500000000000001, 0.2500000000000001]"} />
      <p>
        Because each side alone is random, entanglement cannot send a message faster than light. It can still do
        useful work; teleportation and many algorithms are built on it.
      </p>

      <h2 id="ghz">More than two qubits</h2>
      <p>Chaining CNOTs spreads the entanglement to any number of qubits. Three gives the GHZ state:</p>
      <MBlock>{String.raw`\tfrac{1}{\sqrt2}\left(\ket{000} + \ket{111}\right)`}</MBlock>
      <p>
        The <Link href="/docs/examples/#ghz">exact amplitudes example</Link> builds it and prints all eight amplitudes.
      </p>
    </>
  );
}
