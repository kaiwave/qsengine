import Link from "next/link";

export const metadata = { title: "Page not found" };

export default function NotFound() {
  return (
    <main className="missing">
      <h1>Page not found</h1>
      <p>That address is not part of the qsengine site. It may have moved when the documentation was reorganised.</p>
      <p>
        <Link className="button" href="/docs/">Documentation contents</Link>
      </p>
    </main>
  );
}
