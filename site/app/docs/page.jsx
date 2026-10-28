import Link from "next/link";
import { DOCS, VERSION } from "../content";

export const metadata = { title: "Documentation" };

export default function DocsIndex() {
  return (
    <>
      <h1>qsengine {VERSION} documentation</h1>
      <p>
        qsengine simulates quantum circuits from Python. Start with installation and your first circuit, read the
        user guide for how circuits run, and use the API reference when you need the exact signature of a method.
      </p>
      <div className="index">
        {DOCS.map((g) => (
          <section key={g.group}>
            <h2>{g.group}</h2>
            <ul>
              {g.items.map((it) => (
                <li key={it.href}>
                  <Link href={it.href}>{it.title}</Link>
                  <p>{it.blurb}</p>
                </li>
              ))}
            </ul>
          </section>
        ))}
      </div>
    </>
  );
}
