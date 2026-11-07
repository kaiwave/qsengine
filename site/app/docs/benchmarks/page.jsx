import { CodeBlock } from "../../components/Code";
import { BENCH, REPO } from "../../content";

export const metadata = { title: "Benchmarks" };

export default function Benchmarks() {
  return (
    <>
      <h1>Benchmarks</h1>
      <p>
        Every simulator ran the same circuit: five layers of a Hadamard on every qubit, an RZ rotation on every qubit
        and a chain of CNOTs, then 1000 measurement shots. Times are the best of at least three runs.
      </p>

      <h2 id="results">Results</h2>
      <div className="table-wrap">
        <table className="numbers">
          <thead>
            <tr>
              <th scope="col">Qubits</th>
              <th scope="col">Pure Python</th>
              <th scope="col">qsengine CPU</th>
              <th scope="col">qsengine GPU</th>
              <th scope="col">Qiskit Aer</th>
            </tr>
          </thead>
          <tbody>
            {BENCH.map((r) => (
              <tr key={r.qubits}>
                <th scope="row">{r.qubits}</th>
                <td>{r.python}</td>
                <td>{r.cpu}</td>
                <td>{r.gpu}</td>
                <td>{r.aer}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p>The pure Python loop was stopped after 18 qubits, when one run passed 10 seconds.</p>

      <h2 id="reading">What the numbers say</h2>
      <ul>
        <li>Below 14 qubits the CPU backend is the fastest: 8 to 50 times faster than Qiskit Aer, because a call has almost no overhead. Aer&rsquo;s time there is mostly its fixed per-call cost (about 2 ms to submit a job and build its result), so this measures call overhead more than simulation.</li>
        <li>From 14 qubits the GPU backend is the fastest: 3 to 8 times faster than Aer. The GPU backend computes in 32-bit floats and Aer ran in 64-bit; Aer&rsquo;s <code>precision=&quot;single&quot;</code> option would be the like-for-like comparison.</li>
        <li>The CPU backend stays ahead of Aer up to 19 qubits and falls behind from 20. At 20 qubits a 64-bit state is 16 MiB, the size of this CPU&rsquo;s L3 cache, and the CPU backend&rsquo;s time per extra qubit jumps from about 2&times; to 3&ndash;4&times;, so the likely cause is the state no longer fitting in cache.</li>
        <li>There is one benchmark circuit. qsengine merges the H and RZ on each qubit into one pass over the state, which suits this circuit; other circuits may compare differently.</li>
        <li>Against a plain Python loop, the CPU backend is about 100 times faster at 10 qubits and 840 times faster at 18.</li>
      </ul>

      <h2 id="setup">Setup</h2>
      <p>
        AMD Ryzen 5 5500 (6 cores), AMD Radeon RX 6600, Windows 10, Python 3.14, Qiskit Aer 0.18.2 with its default
        statevector method in 64-bit precision. These are one machine&rsquo;s numbers; run the script to get yours.
      </p>
      <CodeBlock code={"pip install qiskit qiskit-aer matplotlib\npython benchmarks/benchmark.py --max-qubits 22"} />
      <p>
        The script and the raw <a href={`${REPO}/blob/main/benchmarks/results.csv`}>results.csv</a> are in the{" "}
        <a href={`${REPO}/tree/main/benchmarks`}>benchmarks folder</a> of the repository.
      </p>
    </>
  );
}
