import { CodeBlock } from "../../components/Code";
import { BENCH, REPO } from "../../content";

export const metadata = { title: "Benchmarks" };

export default function Benchmarks() {
  return (
    <>
      <h1>Benchmarks</h1>
      <p>
        Every simulator ran the same circuit: five layers of a Hadamard on every qubit, an RZ rotation on every qubit
        and a chain of CNOTs, then 1000 measurement shots. Times are the best of several runs.
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
        <li>Below 14 qubits the CPU backend is the fastest: 7 to 60 times faster than Qiskit Aer, because a call has almost no overhead.</li>
        <li>From 14 qubits the GPU backend is the fastest: 3 to 9 times faster than Aer.</li>
        <li>The CPU backend falls behind Aer above about 17 qubits, where Aer merges multi-qubit gates and works in cache-sized blocks.</li>
        <li>Against a plain Python loop, the CPU backend is about 100 times faster at 10 qubits and 640 times faster at 18.</li>
      </ul>

      <h2 id="setup">Setup</h2>
      <p>
        AMD Ryzen 5 5500 (6 cores), AMD Radeon RX 6600, Windows 10, Python 3.14, Qiskit Aer 0.18.2 with its default
        statevector method. These are one machine&rsquo;s numbers; run the script to get yours.
      </p>
      <CodeBlock code={"pip install qiskit qiskit-aer matplotlib\npython benchmarks/benchmark.py --max-qubits 22"} />
      <p>
        The script and the raw <a href={`${REPO}/blob/main/benchmarks/results.csv`}>results.csv</a> are in the{" "}
        <a href={`${REPO}/tree/main/benchmarks`}>benchmarks folder</a> of the repository.
      </p>
    </>
  );
}
