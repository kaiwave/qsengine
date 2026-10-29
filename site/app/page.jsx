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
          {DOCS.map((g) => (
            <div key={g.group}>
              <h3>{g.group}</h3>
              <ul>
                {g.items.map((it) => (
                  <li key={it.href}>
                    <Link href={it.href}>{it.title}</Link>
                    <span>{it.blurb}</span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </section>

      <section className="release" aria-labelledby="release-title">
        <h2 id="release-title">What is in {VERSION}</h2>
        <ul className="release__list">
          <li><strong>Two backends, one API.</strong> A multithreaded CPU backend in 64-bit precision and a GPU backend for any vendor, chosen automatically.</li>
          <li><strong>Eleven gates.</strong> X, Y, Z, H, S, T, three rotations, CNOT and controlled Z.</li>
          <li><strong>Measured speed.</strong> 70 ms for 22 qubits on the GPU, against 367 ms for Qiskit Aer on the same circuit.</li>
          <li><strong>One wheel per platform</strong> for every CPython from 3.9.</li>
        </ul>
        <Link href="/docs/release-notes/">Full release notes</Link>
      </section>
    </main>
  );
}
