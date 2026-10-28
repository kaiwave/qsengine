import Link from "next/link";
import { CodeBlock, Output } from "../../../components/Code";
import { M, MBlock } from "../../../components/Math";

export const metadata = { title: "Measurement" };

export default function Measurement() {
  return (
    <>
      <h1>Measurement</h1>
      <p>
        You never see amplitudes directly. Measuring a qubit always gives a plain 0 or 1, and the amplitudes only
        decide how likely each one is.
      </p>

      <h2 id="born-rule">The Born rule</h2>
      <p>For a qubit in the state <M>{String.raw`\alpha\ket{0} + \beta\ket{1}`}</M>, measurement gives</p>
      <MBlock>{String.raw`P(0) = |\alpha|^2 = |\braket{0|\psi}|^2, \qquad P(1) = |\beta|^2 = |\braket{1|\psi}|^2`}</MBlock>
      <p>
        After the measurement the qubit is left in whichever state was seen. The superposition is gone, so measuring
        again gives the same answer.
      </p>

      <h2 id="shots">Shots</h2>
      <p>
        One measurement tells you almost nothing about the probabilities. Real quantum computers, and qsengine, run the
        circuit many times and count the results. Each run is a <em>shot</em>. A rotation by{" "}
        <M>{String.raw`\pi/3`}</M> about the Y axis gives
      </p>
      <MBlock>{String.raw`R_Y(\tfrac{\pi}{3})\ket{0} = \cos\tfrac{\pi}{6}\ket{0} + \sin\tfrac{\pi}{6}\ket{1}, \qquad P(0) = \cos^2\tfrac{\pi}{6} = 0.75`}</MBlock>
      <CodeBlock code={"import math\nimport qsengine\n\nqc = qsengine.QuantumCircuit(1)\nqc.ry(0, math.pi / 3)\n\nprint(qc.probabilities())\nprint(qc.run(shots=1000, seed=7))"} />
      <Output
        printed={"[0.7500000000000001, 0.24999999999999994]\n{'1': 257, '0': 743}"}
        rows={[["0", 743, 1000], ["1", 257, 1000]]}
        caption="probabilities() gives the exact 0.75; 1000 shots give 743, close to it but not exact."
      />
      <p>
        The more shots, the closer the counts get to the probabilities. <code>probabilities()</code> skips sampling
        altogether, which no real device can do. See <Link href="/docs/running/">Running circuits</Link>.
      </p>

      <h2 id="phase">Phase: what measurement cannot see</h2>
      <p>
        Amplitudes can be negative or complex, and squaring hides the difference. These two states give exactly the
        same measurement statistics:
      </p>
      <MBlock>{String.raw`\ket{+} = \tfrac{1}{\sqrt{2}}\left(\ket{0} + \ket{1}\right), \qquad \ket{-} = \tfrac{1}{\sqrt{2}}\left(\ket{0} - \ket{1}\right)`}</MBlock>
      <p>
        The sign is called the <em>relative phase</em>, and it is not just bookkeeping: a later gate can turn it into a
        difference you can measure. Hadamard twice returns to <M>{String.raw`\ket{0}`}</M>, but a Z gate in the middle
        flips the phase and the result becomes <M>{String.raw`\ket{1}`}</M>:
      </p>
      <CodeBlock code={"qc = qsengine.QuantumCircuit(1)\nqc.h(0)\nqc.h(0)\nprint(qc.probabilities())\n\nqc = qsengine.QuantumCircuit(1)\nqc.h(0)\nqc.z(0)\nqc.h(0)\nprint(qc.probabilities())"} />
      <Output printed={"[1.0000000000000004, 0.0]\n[0.0, 1.0000000000000004]"} caption="The Z gate changed nothing you could measure on its own, yet it flipped the final answer." />
      <p>
        This is the effect quantum algorithms rely on: steer the phases so wrong answers cancel and right answers add
        up. The <Link href="/docs/learn/deutsch-jozsa/">Deutsch-Jozsa algorithm</Link> is a complete example.
      </p>
      <p>
        The tiny 4 in the last digit is ordinary floating-point rounding; it is far below anything sampling could show.
      </p>
    </>
  );
}
