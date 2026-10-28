import { CodeBlock, Output } from "../../../components/Code";
import { M, MBlock } from "../../../components/Math";

export const metadata = { title: "The Bloch sphere" };

const POINTS = [
  [String.raw`\ket{0}`, String.raw`\theta = 0`, "North pole"],
  [String.raw`\ket{1}`, String.raw`\theta = \pi`, "South pole"],
  [String.raw`\ket{+} = \tfrac{1}{\sqrt2}(\ket0 + \ket1)`, String.raw`\theta = \tfrac{\pi}{2},\ \varphi = 0`, "On the equator, facing +X"],
  [String.raw`\ket{-} = \tfrac{1}{\sqrt2}(\ket0 - \ket1)`, String.raw`\theta = \tfrac{\pi}{2},\ \varphi = \pi`, "On the equator, facing −X"],
  [String.raw`\tfrac{1}{\sqrt2}(\ket0 + i\ket1)`, String.raw`\theta = \tfrac{\pi}{2},\ \varphi = \tfrac{\pi}{2}`, "On the equator, facing +Y"],
];

const ROTATIONS = [
  ["X", String.raw`\pi \text{ about } X`, "qc.x(q)"],
  ["Y", String.raw`\pi \text{ about } Y`, "qc.y(q)"],
  ["Z", String.raw`\pi \text{ about } Z`, "qc.z(q)"],
  ["S", String.raw`\tfrac{\pi}{2} \text{ about } Z`, "qc.s(q)"],
  ["T", String.raw`\tfrac{\pi}{4} \text{ about } Z`, "qc.t(q)"],
  ["H", String.raw`\pi \text{ about } \tfrac{X+Z}{\sqrt2}`, "qc.h(q)"],
  [String.raw`R_X(\theta),\ R_Y(\theta),\ R_Z(\theta)`, String.raw`\theta \text{ about } X,\ Y \text{ or } Z`, "qc.rx(q, theta)"],
];

export default function Bloch() {
  return (
    <>
      <h1>The Bloch sphere</h1>
      <p>
        Every state of a single qubit can be drawn as a point on the surface of a sphere. It turns gates from matrices
        into something you can picture: each one is a rotation.
      </p>

      <h2 id="angles">Two angles describe any qubit</h2>
      <p>
        An overall phase on the whole state cannot be measured, so it can be dropped. What is left of{" "}
        <M>{String.raw`\alpha\ket{0} + \beta\ket{1}`}</M> fits in two angles:
      </p>
      <MBlock>{String.raw`\ket{\psi} = \cos\tfrac{\theta}{2}\ket{0} + e^{i\varphi}\sin\tfrac{\theta}{2}\ket{1}, \qquad 0 \le \theta \le \pi,\ \ 0 \le \varphi < 2\pi`}</MBlock>
      <p>
        <M>{String.raw`\theta`}</M> is the latitude, measured down from the north pole, and{" "}
        <M>{String.raw`\varphi`}</M> is the longitude around the equator. <M>{String.raw`\theta`}</M> alone sets the
        measurement odds; <M>{String.raw`\varphi`}</M> is the relative phase.
      </p>

      <h2 id="landmarks">Landmarks</h2>
      <div className="table-wrap">
        <table>
          <thead><tr><th scope="col">State</th><th scope="col">Angles</th><th scope="col">Where it sits</th></tr></thead>
          <tbody>
            {POINTS.map(([state, angles, where]) => (
              <tr key={where}><th scope="row"><M>{state}</M></th><td><M>{angles}</M></td><td>{where}</td></tr>
            ))}
          </tbody>
        </table>
      </div>
      <p>The poles are the basis states. Every point on the equator is an equal superposition, differing only in phase.</p>

      <h2 id="rotations">Gates are rotations</h2>
      <div className="table-wrap">
        <table>
          <thead><tr><th scope="col">Gate</th><th scope="col">Rotation</th><th scope="col">Method</th></tr></thead>
          <tbody>
            {ROTATIONS.map(([gate, rot, call]) => (
              <tr key={call}><th scope="row"><M>{gate}</M></th><td><M>{rot}</M></td><td><code>{call}</code></td></tr>
            ))}
          </tbody>
        </table>
      </div>
      <p>
        A quarter turn about X takes the north pole to the equator, at the point facing −Y:
      </p>
      <MBlock>{String.raw`R_X(\tfrac{\pi}{2})\ket{0} = \tfrac{1}{\sqrt2}\left(\ket{0} - i\ket{1}\right)`}</MBlock>
      <CodeBlock code={"import math\nimport qsengine\n\nqc = qsengine.QuantumCircuit(1)\nqc.rx(0, math.pi / 2)\n\nprint(qc.statevector())\nprint(qc.probabilities())"} />
      <Output printed={"[(0.7071067811865476+0j), -0.7071067811865476j]\n[0.5000000000000001, 0.5000000000000001]"} caption="The second amplitude is −i/√2, exactly as the formula says, and the odds are 50/50." />

      <h2 id="limits">Where the picture stops</h2>
      <p>
        The sphere describes one qubit on its own. Two or more qubits that are entangled have no point on any sphere,
        which is exactly what makes them interesting. That is the subject of the next pages.
      </p>
    </>
  );
}
