import Link from "next/link";
import { CodeBlock, Output } from "../../../components/Code";
import { M, MBlock } from "../../../components/Math";

export const metadata = { title: "Many qubits" };

export default function ManyQubits() {
  return (
    <>
      <h1>Many qubits</h1>
      <p>
        Two qubits do not have 2 amplitudes each; together they have 4. Every extra qubit doubles the count, and that
        doubling is both the power of quantum computing and the reason simulators run out of memory.
      </p>

      <h2 id="basis">The basis states</h2>
      <p>Two qubits have four basis states, one for each pair of classical values:</p>
      <MBlock>{String.raw`\ket{00},\ \ket{01},\ \ket{10},\ \ket{11}`}</MBlock>
      <p>and a general state is a combination of all four:</p>
      <MBlock>{String.raw`\ket{\psi} = a_{00}\ket{00} + a_{01}\ket{01} + a_{10}\ket{10} + a_{11}\ket{11}, \qquad \sum |a|^2 = 1`}</MBlock>
      <p>
        With <M>n</M> qubits there are <M>{String.raw`2^n`}</M> basis states and <M>{String.raw`2^n`}</M> amplitudes.
        qsengine stores each as two 64-bit numbers, 16 bytes, so 20 qubits take 16 MiB and 30 take 16 GiB.
      </p>

      <h2 id="tensor">Combining qubits: the tensor product</h2>
      <p>
        When qubits are independent, the joint state is their <em>tensor product</em>, written{" "}
        <M>{String.raw`\otimes`}</M>. Each amplitude of the result is a product of one amplitude from each qubit:
      </p>
      <MBlock>{String.raw`(\alpha\ket{0} + \beta\ket{1}) \otimes (\gamma\ket{0} + \delta\ket{1}) = \alpha\gamma\ket{00} + \alpha\delta\ket{01} + \beta\gamma\ket{10} + \beta\delta\ket{11}`}</MBlock>
      <p>A Hadamard on each of two qubits gives every basis state the same amplitude, <M>{String.raw`\tfrac12`}</M>:</p>
      <CodeBlock code={"import qsengine\n\nqc = qsengine.QuantumCircuit(2)\nqc.h(0)\nqc.h(1)\n\nprint(qc.probabilities())\nprint(qc.run(shots=1000, seed=5))"} />
      <Output
        printed={"[0.2500000000000001, 0.2500000000000001, 0.2500000000000001, 0.2500000000000001]\n{'10': 262, '00': 235, '11': 245, '01': 258}" /* taste-skill-ignore: real output */}
        rows={[["00", 235, 1000], ["01", 258, 1000], ["10", 262, 1000], ["11", 245, 1000]] /* taste-skill-ignore: basis states, real data */}
        caption="Four outcomes, each about a quarter of the time: two independent coin flips."
      />

      <h2 id="order">Which bit is which</h2>
      <p>
        In <M>{String.raw`\ket{01}`}</M>, qsengine reads the rightmost digit as qubit 0, like Qiskit. So flipping qubit 0
        of two gives <M>{String.raw`\ket{01}`}</M>, which is basis index 1:
      </p>
      <CodeBlock code={"qc = qsengine.QuantumCircuit(2)\nqc.x(0)\n\nprint(qc.statevector())\nprint(qc.run(shots=100, seed=0))"} />
      <Output printed={"[0j, (1+0j), 0j, 0j]\n{'01': 100}" /* taste-skill-ignore: real output */} caption="Amplitude 1 at index 1, and every shot reads 01." />
      <p>
        <Link href="/docs/running/#bit-order">Running circuits</Link> has the same rule for the lists qsengine returns.
      </p>

      <h2 id="gates">Gates on one qubit of many</h2>
      <p>
        A single-qubit gate on qubit <M>k</M> acts as the tensor product of that gate with the identity on every other
        qubit. qsengine never builds that <M>{String.raw`2^n \times 2^n`}</M> matrix: it updates the amplitudes in
        pairs, which is why it stays fast at 20 qubits and beyond.
      </p>
      <p>
        Not every two-qubit state can be written as a tensor product. The ones that cannot are{" "}
        <Link href="/docs/learn/entanglement/">entangled</Link>.
      </p>
    </>
  );
}
