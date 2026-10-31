// Everything the site says about qsengine, in one place.
// Every printed output was produced by running the code with qsengine 1.0.1 (re-check them after a release that changes behaviour);
// benchmark figures come from benchmarks/results.csv.

// From Cargo.toml, set in next.config.mjs.
export const VERSION = process.env.QSENGINE_VERSION;
export const SITE = "https://qsengine.kaiwave.dev";
export const REPO = "https://github.com/kaiwave/qsengine";
export const PYPI = "https://pypi.org/project/qsengine/";

// The documentation, grouped the way the /docs index and the sidebar show it.
export const DOCS = [
  {
    group: "Getting started",
    items: [
      { href: "/docs/getting-started/", title: "Installation and first circuit", blurb: "Install the wheel, build a Bell state, read the counts." },
      { href: "/docs/examples/", title: "Examples", blurb: "Complete programs with their real output." },
    ],
  },
  {
    group: "Quantum physics",
    items: [
      { href: "/docs/learn/qubits/", title: "Qubits and bra-ket notation", blurb: "What a qubit is and how to write its state." },
      { href: "/docs/learn/measurement/", title: "Measurement", blurb: "Amplitudes, probabilities, shots and phase." },
      { href: "/docs/learn/bloch-sphere/", title: "The Bloch sphere", blurb: "One qubit as a point on a sphere; gates as rotations." },
      { href: "/docs/learn/many-qubits/", title: "Many qubits", blurb: "Tensor products, 2ⁿ amplitudes and bit order." },
      { href: "/docs/learn/entanglement/", title: "Entanglement", blurb: "Bell states, step by step, and why they cannot be split." },
      { href: "/docs/learn/deutsch-jozsa/", title: "The Deutsch-Jozsa algorithm", blurb: "A whole quantum algorithm, built and run with qsengine." },
    ],
  },
  {
    group: "User guide",
    items: [
      { href: "/docs/gates/", title: "Gates", blurb: "Every gate, the method that applies it, and your own unitaries." },
      { href: "/docs/running/", title: "Running circuits", blurb: "Shots and counts, exact states, seeds and bit order." },
      { href: "/docs/backends/", title: "CPU and GPU backends", blurb: "Precision, hardware and how auto chooses." },
    ],
  },
  {
    group: "Reference",
    items: [
      { href: "/docs/api/", title: "API reference", blurb: "QuantumCircuit, every method, every error." },
      { href: "/docs/benchmarks/", title: "Benchmarks", blurb: "qsengine against pure Python and Qiskit Aer, up to 22 qubits." },
    ],
  },
  {
    group: "Help",
    items: [
      { href: "/docs/faq/", title: "Questions", blurb: "Short answers to common questions." },
      { href: "/docs/release-notes/", title: "Release notes", blurb: `What is in version ${VERSION}.` },
    ],
  },
];

export const EXAMPLES = [
  {
    id: "bell",
    name: "Bell state",
    intro: "Two qubits put into an entangled state, then measured 1000 times.",
    code: `import qsengine

qc = qsengine.QuantumCircuit(2)
qc.h(0)
qc.cx(0, 1)

counts = qc.run(shots=1000, seed=1)
print(counts)`,
    printed: `{'00': 513, '11': 487}`,
    caption: "Only 00 and 11 appear: the two qubits always agree.",
    rows: [["00", 513, 1000], ["01", 0, 1000], ["10", 0, 1000], ["11", 487, 1000]], // taste-skill-ignore: basis states, real data
  },
  {
    id: "ghz",
    name: "Exact amplitudes",
    intro: "A three-qubit GHZ state read exactly, without sampling.",
    code: `import qsengine

qc = qsengine.QuantumCircuit(3)
qc.h(0)
qc.cx(0, 1)
qc.cx(1, 2)

print(qc.probabilities())
print(qc.statevector())`,
    printed: `[0.5 0.  0.  0.  0.  0.  0.  0.5]
[0.70710678+0.j 0.        +0.j 0.        +0.j 0.        +0.j
 0.        +0.j 0.        +0.j 0.        +0.j 0.70710678+0.j]`,
    caption: "The exact probability of each of the 8 basis states.",
    rows: ["000", "001", "010", "011", "100", "101", "110", "111"].map((b) => [b, b === "000" || b === "111" ? 0.5 : 0, 1]),
  },
  {
    id: "rotation",
    name: "Rotation",
    intro: "One qubit rotated by π/3 about the Y axis.",
    code: `import math
import qsengine

qc = qsengine.QuantumCircuit(1)
qc.ry(0, math.pi / 3)

print(qc.probabilities())
print(qc.run(shots=1000, seed=7))`,
    printed: `[0.75 0.25]
{'1': 257, '0': 743}`,
    caption: "cos²(π/6) = 0.75, and 743 of 1000 shots measured 0.",
    rows: [["0", 743, 1000], ["1", 257, 1000]],
  },
  {
    id: "errors",
    name: "Bad input",
    intro: "Mistakes raise a plain Python ValueError on the line that made them.",
    code: `import qsengine

qc = qsengine.QuantumCircuit(2)
qc.cx(1, 1)`,
    printed: `Traceback (most recent call last):
  File "example.py", line 4, in <module>
    qc.cx(1, 1)
ValueError: qubit 1 used more than once in one gate`,
    caption: "The gate is not recorded, so the circuit stays valid.",
    rows: [],
  },
];

export const BELL = EXAMPLES[0];

// Simulate + 1000 shots, best of several runs, ms. Ryzen 5 5500 + Radeon RX 6600, Windows 10, Qiskit Aer 0.18.2.
export const BENCH = [
  { qubits: 6, python: "2.2 ms", cpu: "0.11 ms", gpu: "1.3 ms", aer: "3.7 ms" },
  { qubits: 10, python: "30 ms", cpu: "0.30 ms", gpu: "1.2 ms", aer: "5.7 ms" },
  { qubits: 14, python: "739 ms", cpu: "2.4 ms", gpu: "1.3 ms", aer: "11.7 ms" },
  { qubits: 18, python: "16.2 s", cpu: "25 ms", gpu: "4.9 ms", aer: "19 ms" },
  { qubits: 22, python: "not run", cpu: "1.03 s", gpu: "70 ms", aer: "367 ms" },
];



export const FAQ = [
  ["Do I need a GPU?", "No. Without one, or without a working driver, the default backend runs everything on the CPU and your code does not change."],
  ["Which GPUs work?", "Any that supports Vulkan, DirectX 12 or Metal: NVIDIA, AMD, Intel and Apple. qsengine talks to them through wgpu, so there is no CUDA to install."],
  ["How many qubits can I simulate?", "Memory doubles with every qubit: 16 bytes for each of the 2ⁿ amplitudes. 20 qubits take 16 MiB, 26 take 1 GiB, and 30 qubits (16 GiB) is the hard limit."],
  ["Why do GPU results differ in the last digits?", "The GPU computes in 32-bit floats and the CPU in 64-bit. They agree to about 1e-5, far below what sampling can show."],
  ["Which way round are the bits?", "Qubit 0 is the rightmost character, as in Qiskit. On three qubits, qc.x(0) measures as \"001\"."],
  ["Is it faster than Qiskit Aer?", "In our benchmark the GPU backend is 3 to 9 times faster from 14 qubits. The CPU backend is faster up to about 16 qubits and slower above 17, where Aer fuses multi-qubit gates."],
  ["Will the same seed give the same counts?", "Yes. run(seed=42) returns identical counts every time on the same backend."],
  ["What is missing?", "Mid-circuit measurement is not in 1.0. Every qubit is measured at the end of the circuit."],
  ["Which Python versions work?", "CPython 3.9 and newer. One wheel per platform covers all of them."],
  ["Is it free?", "Yes. qsengine is open source under the MIT licence."],
];
