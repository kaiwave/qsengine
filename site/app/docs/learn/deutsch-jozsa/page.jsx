import Link from "next/link";
import { CodeBlock, Output } from "../../../components/Code";
import { M, MBlock } from "../../../components/Math";

export const metadata = { title: "The Deutsch-Jozsa algorithm" };

const PROGRAM = `import qsengine

N = 3  # input qubits; qubit N is the helper


def deutsch_jozsa(oracle):
    qc = qsengine.QuantumCircuit(N + 1)
    qc.x(N)                  # helper starts in |1>
    for q in range(N + 1):
        qc.h(q)              # superposition of every input
    oracle(qc)               # one call to f
    for q in range(N + 1):
        qc.h(q)              # interfere
    return qc.run(shots=1000, seed=11)


def constant_zero(qc):       # f(x) = 0 for every x
    pass


def constant_one(qc):        # f(x) = 1 for every x
    qc.x(N)


def parity(qc):              # f(x) = x0 xor x1 xor x2
    for q in range(N):
        qc.cx(q, N)


def middle_bit(qc):          # f(x) = x1
    qc.cx(1, N)


for oracle in [constant_zero, constant_one, parity, middle_bit]:
    print(oracle.__name__, deutsch_jozsa(oracle))`;

export default function DeutschJozsa() {
  return (
    <>
      <h1>The Deutsch-Jozsa algorithm</h1>
      <p>
        This is the first algorithm where a quantum computer provably needs fewer steps than any classical one. It is
        small enough to run in full on qsengine and uses every idea from the earlier pages: superposition, phase and
        interference.
      </p>

      <h2 id="problem">The problem</h2>
      <p>
        You are given a function <M>{String.raw`f`}</M> that takes <M>n</M> bits and returns one bit. You are promised
        it is one of two kinds:
      </p>
      <ul>
        <li><strong>Constant:</strong> it returns the same value for every input.</li>
        <li><strong>Balanced:</strong> it returns 0 for exactly half the inputs and 1 for the other half.</li>
      </ul>
      <p>
        Which kind is it? Classically you may have to try <M>{String.raw`2^{n-1} + 1`}</M> inputs before you can be
        sure. Deutsch-Jozsa answers with <strong>one</strong> call to <M>f</M>.
      </p>

      <h2 id="oracle">The function as a gate</h2>
      <p>
        On a quantum computer <M>f</M> is given as an <em>oracle</em>: a gate that flips a helper qubit when{" "}
        <M>{String.raw`f(x) = 1`}</M>,
      </p>
      <MBlock>{String.raw`U_f\ket{x}\ket{y} = \ket{x}\ket{y \oplus f(x)}`}</MBlock>
      <p>
        With qsengine&rsquo;s gates, a constant oracle is nothing at all (<M>{String.raw`f = 0`}</M>) or an X on the
        helper (<M>{String.raw`f = 1`}</M>), and a balanced oracle is a set of CNOTs from input qubits to the helper.
      </p>

      <h2 id="steps">The steps</h2>
      <p>
        <strong>Prepare.</strong> Put the helper in <M>{String.raw`\ket{1}`}</M>, then apply a Hadamard to every
        qubit. The inputs become an equal superposition of all <M>{String.raw`2^n`}</M> values and the helper becomes{" "}
        <M>{String.raw`\ket{-}`}</M>:
      </p>
      <MBlock>{String.raw`\frac{1}{\sqrt{2^n}} \sum_{x} \ket{x} \otimes \ket{-}`}</MBlock>
      <p>
        <strong>Query.</strong> Because the helper is <M>{String.raw`\ket{-}`}</M>, flipping it only multiplies the
        state by <M>-1</M>. The answer of <M>f</M> moves into the phase of each input:
      </p>
      <MBlock>{String.raw`\frac{1}{\sqrt{2^n}} \sum_{x} (-1)^{f(x)} \ket{x} \otimes \ket{-}`}</MBlock>
      <p>
        <strong>Interfere.</strong> Apply a Hadamard to every input again. The amplitude of the all-zero input becomes
      </p>
      <MBlock>{String.raw`\frac{1}{2^n} \sum_{x} (-1)^{f(x)} = \begin{cases} \pm 1 & f \text{ constant} \\ 0 & f \text{ balanced} \end{cases}`}</MBlock>
      <p>
        If <M>f</M> is constant every term has the same sign and they add up: the inputs always measure as all zeros.
        If it is balanced, half the terms cancel the other half and all zeros never appears.
      </p>
      <p>
        <strong>Read.</strong> Measure the inputs. All zeros means constant; anything else means balanced.
      </p>

      <h2 id="run">Running it</h2>
      <p>
        Here is the whole algorithm for three input bits, with four oracles. A final Hadamard on the helper returns it
        from <M>{String.raw`\ket{-}`}</M> to <M>{String.raw`\ket{1}`}</M>, so every shot gives the same answer.
      </p>
      <CodeBlock code={PROGRAM} label="Copy the Deutsch-Jozsa program" />
      <Output
        printed={"constant_zero {'1000': 1000}\nconstant_one {'1000': 1000}\nparity {'1111': 1000}\nmiddle_bit {'1010': 1000}"}
        caption="Every shot of every run gives one bitstring: no sampling noise, the answer is certain."
      />

      <h2 id="reading">Reading the result</h2>
      <p>
        The leftmost digit is the helper (qubit 3), which is always 1. The other three are the inputs, qubit 0 on the
        right:
      </p>
      <ul>
        <li>Both constant oracles give inputs <code>000</code>: constant.</li>
        <li>The parity oracle gives <code>111</code> and the middle-bit oracle gives <code>010</code>: not all zeros, so balanced.</li>
      </ul>
      <p>
        A balanced result also names the inputs the function depends on: parity uses all three bits and lights up all
        three, the middle-bit function lights up only qubit 1.
      </p>

      <h2 id="next">Where next</h2>
      <p>
        Each oracle was called once, against five calls a classical program would need in the worst case for three
        bits. The gap grows exponentially with <M>n</M>. Try the program with a larger <code>N</code>, or write your
        own balanced oracle; the <Link href="/docs/gates/">gates page</Link> lists everything you can use.
      </p>
    </>
  );
}
