import Link from "next/link";
import { CodeBlock, Output } from "../../../components/Code";
import { M, MBlock } from "../../../components/Math";

export const metadata = { title: "Qubits and bra-ket notation" };

export default function Qubits() {
  return (
    <>
      <h1>Qubits and bra-ket notation</h1>
      <p>
        A classical bit is either 0 or 1. A qubit has two basic states too, but it can also be in a blend of both at
        once. This page shows how physicists write those states down, which is the notation the rest of these pages
        use.
      </p>

      <h2 id="kets">Kets</h2>
      <p>
        A state is written inside a <em>ket</em>, the symbol <M>{String.raw`\ket{\;}`}</M>. The two basic states of a
        qubit are
      </p>
      <MBlock>{String.raw`\ket{0} = \begin{pmatrix} 1 \\ 0 \end{pmatrix}, \qquad \ket{1} = \begin{pmatrix} 0 \\ 1 \end{pmatrix}`}</MBlock>
      <p>
        Under the notation, a state is just a column of numbers. <M>{String.raw`\ket{0}`}</M> and{" "}
        <M>{String.raw`\ket{1}`}</M> play the role of 0 and 1, and every qsengine circuit starts with all its qubits in{" "}
        <M>{String.raw`\ket{0}`}</M>.
      </p>

      <h2 id="superposition">Superposition</h2>
      <p>Any other state of one qubit is a combination of the two basic states:</p>
      <MBlock>{String.raw`\ket{\psi} = \alpha\ket{0} + \beta\ket{1} = \begin{pmatrix} \alpha \\ \beta \end{pmatrix}`}</MBlock>
      <p>
        <M>{String.raw`\alpha`}</M> and <M>{String.raw`\beta`}</M> are <em>amplitudes</em>. They are complex numbers,
        and they must satisfy
      </p>
      <MBlock>{String.raw`|\alpha|^2 + |\beta|^2 = 1`}</MBlock>
      <p>
        because, as the <Link href="/docs/learn/measurement/">measurement page</Link> explains, the squares are
        probabilities and must add up to 1. A state with both amplitudes non-zero is in <em>superposition</em>.
      </p>

      <h2 id="hadamard">Making a superposition</h2>
      <p>The Hadamard gate turns <M>{String.raw`\ket{0}`}</M> into an equal superposition:</p>
      <MBlock>{String.raw`H\ket{0} = \frac{1}{\sqrt{2}}\ket{0} + \frac{1}{\sqrt{2}}\ket{1}`}</MBlock>
      <p>qsengine shows the two amplitudes directly. <M>{String.raw`1/\sqrt{2} \approx 0.7071`}</M>:</p>
      <CodeBlock code={"import qsengine\n\nqc = qsengine.QuantumCircuit(1)\nqc.h(0)\n\nprint(qc.statevector())\nprint(qc.run(shots=1000, seed=3))"} />
      <Output
        printed={"[(0.7071067811865476+0j), (0.7071067811865476+0j)]\n{'0': 493, '1': 507}"}
        rows={[["0", 493, 1000], ["1", 507, 1000]]}
        caption="Equal amplitudes, so 0 and 1 each come up about half the time."
      />

      <h2 id="gates">Gates are matrices</h2>
      <p>
        A gate on one qubit is a 2 by 2 matrix, and applying it multiplies the state by that matrix. The X gate swaps
        the two amplitudes, which is why it acts as a NOT:
      </p>
      <MBlock>{String.raw`X = \begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix}, \qquad X\ket{0} = \begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix}\begin{pmatrix} 1 \\ 0 \end{pmatrix} = \ket{1}`}</MBlock>
      <p>
        Every gate in qsengine is a matrix like this. The <Link href="/docs/gates/">gates page</Link> lists them all.
      </p>

      <h2 id="bras">Bras</h2>
      <p>
        The mirror image of a ket is a <em>bra</em>, <M>{String.raw`\bra{\psi}`}</M>: the same numbers laid out as a
        row and complex-conjugated. Putting a bra against a ket gives a single number, the overlap of two states:
      </p>
      <MBlock>{String.raw`\braket{0|\psi} = \alpha, \qquad \braket{1|\psi} = \beta`}</MBlock>
      <p>That is how you pick an amplitude out of a state, and it is the start of the measurement rule.</p>
    </>
  );
}
