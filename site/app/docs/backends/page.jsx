import { CodeBlock } from "../../components/Code";
import { COMPARE } from "../../content";

export const metadata = { title: "CPU and GPU backends" };

export default function Backends() {
  return (
    <>
      <h1>CPU and GPU backends</h1>
      <p>
        Every method that runs a circuit takes a <code>backend</code> argument. You rarely need it: the default,{" "}
        <code>&quot;auto&quot;</code>, picks the faster backend for the circuit&rsquo;s size and falls back to the CPU
        when there is no GPU.
      </p>
      <CodeBlock code={'qc.run(shots=1000)                 # auto\nqc.run(shots=1000, backend="cpu")  # always the CPU\nqc.run(shots=1000, backend="gpu")  # the GPU, or RuntimeError'} />

      <h2 id="compared">The three choices compared</h2>
      <div className="table-wrap">
        <table>
          <thead>
            <tr><th scope="col"><span className="sr">Property</span></th><th scope="col">CPU</th><th scope="col">GPU</th><th scope="col">auto</th></tr>
          </thead>
          <tbody>
            {COMPARE.map(([name, cpu, gpu, auto], i) => (
              <tr key={name}>
                <th scope="row">{name}</th>
                {[cpu, gpu, auto].map((v) => (
                  <td key={v}>{i === 0 ? <code>{v}</code> : v}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <h2 id="switch">Why auto switches at 14 qubits</h2>
      <p>
        Launching work on a GPU and reading the result back costs about a millisecond, whatever the circuit. Below 14
        qubits the CPU finishes before that. From 14 qubits up the GPU is faster, and the gap grows with every qubit:
        at 22 qubits it is about 15 times faster than the CPU backend. The crossover was measured on one desktop (Ryzen
        5 5500 and Radeon RX 6600) and may sit a qubit higher or lower on yours.
      </p>

      <h2 id="precision">Precision</h2>
      <p>
        The CPU computes in 64-bit floats. GPUs generally lack fast 64-bit arithmetic, so the GPU backend uses 32-bit
        floats and agrees with the CPU to about 1e-5. Counts from <code>run()</code> are unaffected in practice; pass{" "}
        <code>backend=&quot;cpu&quot;</code> when you compare amplitudes to many decimal places.
      </p>

      <h2 id="gpus">Supported GPUs</h2>
      <p>
        qsengine reaches the GPU through wgpu, which speaks Vulkan, DirectX 12 and Metal. Recent NVIDIA, AMD, Intel and
        Apple GPUs all work with their normal drivers, and there is no CUDA or vendor toolkit to install.
      </p>
    </>
  );
}
