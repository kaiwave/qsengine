import Link from "next/link";
import { CodeBlock, Output } from "../../components/Code";

export const metadata = { title: "Running circuits" };

export default function Running() {
  return (
    <>
      <h1>Running circuits</h1>
      <p>
        A circuit can give you three kinds of result. Each call simulates the whole circuit from |0…0⟩ again, so you
        can call them as often as you like and the circuit never changes.
      </p>
      <div className="table-wrap">
        <table>
          <thead><tr><th scope="col">Method</th><th scope="col">Returns</th><th scope="col">Use it for</th></tr></thead>
          <tbody>
            <tr><th scope="row"><code>run(shots, seed, backend)</code></th><td><code>dict[str, int]</code></td><td>Measurement counts, as a real device gives them</td></tr>
            <tr><th scope="row"><code>probabilities(backend)</code></th><td><code>list[float]</code></td><td>The exact probability of each basis state</td></tr>
            <tr><th scope="row"><code>statevector(backend)</code></th><td><code>list[complex]</code></td><td>The exact amplitudes</td></tr>
          </tbody>
        </table>
      </div>

      <h2 id="shots">Shots and counts</h2>
      <p>
        <code>run()</code> measures every qubit <code>shots</code> times (1024 if you leave it out) and counts the
        results. Only the counts come back to Python; the state stays in Rust.
      </p>
      <CodeBlock code={"counts = qc.run(shots=1000, seed=1)\nprint(sum(counts.values()))"} />
      <Output printed="1000" />

      <h2 id="seeds">Seeds</h2>
      <p>
        Pass <code>seed</code> to get the same counts on every run, which is what you want in tests. Without it each
        run draws fresh randomness. The same seed gives the same counts on the same backend.
      </p>

      <h2 id="bit-order">Bit order</h2>
      <p>
        Qubit 0 is the rightmost character of every bitstring, the same convention as Qiskit. In the lists returned by{" "}
        <code>probabilities()</code> and <code>statevector()</code>, index <em>i</em> is the basis state whose binary
        form is <em>i</em>, again with qubit 0 as the lowest bit.
      </p>
      <CodeBlock code={"qc = qsengine.QuantumCircuit(3)\nqc.x(0)\nprint(qc.run(shots=10, seed=0))"} />
      <Output printed="{'001': 10}" />

      <h2 id="exact">Exact results</h2>
      <p>
        <code>probabilities()</code> and <code>statevector()</code> return all 2ⁿ values, so they grow quickly: 20
        qubits is about a million numbers. Use <code>run()</code> when you only need counts. See the{" "}
        <Link href="/docs/examples/#ghz">exact amplitudes example</Link>.
      </p>
    </>
  );
}
