import Link from "next/link";
import { CodeBlock } from "./components/Code";
import { DOCS, VERSION } from "./content";

export default function Home() {
  return (
    <main>
      <section className="intro">
        <h1 className="intro__title">qsengine</h1>
        <p className="intro__lead">
          A state-vector quantum simulator for Python. You build circuits in Python; a Rust core runs them on every
          CPU core or on any GPU, and hands back the counts.
        </p>
        <div className="intro__install">
          <p>Install the latest release, {VERSION}:</p>
          <CodeBlock code="pip install qsengine" label="Copy the install command" />
        </div>
        <div className="intro__actions">
          <Link className="button" href="/docs/getting-started/">Your first circuit</Link>
          <Link href="/docs/">Read the documentation</Link>
        </div>
      </section>

      <section className="contents" aria-labelledby="contents-title">
        <h2 id="contents-title" className="contents__title">Documentation</h2>
        <div className="contents__groups">
          {DOCS.filter((g) => g.items.some((it) => it.key)).map((g) => (
            <div key={g.group}>
              <h3>{g.group}</h3>
              <ul>
                {g.items.filter((it) => it.key).map((it) => (
                  <li key={it.href}>
                    <Link href={it.href}>{it.title}</Link>
                    <span>{it.blurb}</span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
        <Link href="/docs/">All documentation</Link>
      </section>

      <section className="release" aria-labelledby="release-title">
        <h2 id="release-title">What is in qsengine</h2>
        <ul className="release__list">
          <li><strong>Two backends, one API.</strong> A multithreaded CPU backend in 64-bit precision and a GPU backend for any vendor, chosen automatically.</li>
          <li><strong>Nineteen gates.</strong> Paulis, phases, rotations, U, controlled gates, SWAP and Toffoli, plus any 2x2 unitary you supply.</li>
          <li><strong>Mid-circuit measurement.</strong> Measure and reset qubits part-way, and condition gates on the results: enough for teleportation and error correction.</li>
          <li><strong>Expectation values.</strong> <code>expectation(&quot;ZZI&quot;)</code> for Pauli strings, the core of VQE and QAOA.</li>
          <li><strong>Measured speed.</strong> 77 ms for 22 qubits on the GPU, against 431 ms for Qiskit Aer on the same circuit.</li>
          <li><strong>One wheel per platform</strong> for every CPython from 3.9.</li>
        </ul>
        <Link href="/docs/release-notes/">Full release notes</Link>
      </section>
    </main>
  );
}
