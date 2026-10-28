import { VERSION } from "../../content";

export const metadata = { title: "Release notes" };

export default function ReleaseNotes() {
  return (
    <>
      <h1>Release notes</h1>

      <h2 id={`v${VERSION}`}>Version {VERSION}</h2>
      <p>The first release.</p>

      <h3>Simulation</h3>
      <ul>
        <li>State-vector simulation of up to 30 qubits.</li>
        <li>Eleven gates: X, Y, Z, H, S, T, RX, RY, RZ, CNOT and controlled Z.</li>
        <li>Single-qubit gate fusion: consecutive gates on one qubit cost one pass over the state.</li>
        <li><code>run()</code> for measurement counts, <code>probabilities()</code> and <code>statevector()</code> for exact results.</li>
        <li>Repeatable counts with <code>seed</code>.</li>
      </ul>

      <h3>Backends</h3>
      <ul>
        <li>A CPU backend in 64-bit precision that uses every core.</li>
        <li>A GPU backend in 32-bit precision for NVIDIA, AMD, Intel and Apple GPUs, through Vulkan, DirectX 12 or Metal.</li>
        <li><code>backend=&quot;auto&quot;</code>, the default, uses the CPU below 14 qubits and the GPU from 14, and falls back to the CPU when no GPU is available.</li>
      </ul>

      <h3>Packaging</h3>
      <ul>
        <li>One wheel per platform for CPython 3.9 and newer.</li>
        <li>Type hints for editors and type checkers.</li>
        <li>Released under the MIT licence.</li>
      </ul>

      <h3>Known limits</h3>
      <ul>
        <li>No SWAP gate and no mid-circuit measurement yet.</li>
        <li>Above about 17 qubits the CPU backend is slower than Qiskit Aer; the GPU backend is not.</li>
      </ul>
    </>
  );
}
