"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { DOCS, REPO, VERSION } from "../content";

// "/docs/gates" and "/docs/gates/" are the same page.
const norm = (p) => (p || "/").replace(/\/+$/, "") || "/";

export function SiteHeader() {
  const path = norm(usePathname());
  const links = [
    ["/docs/", "Documentation"],
    ["/docs/learn/qubits/", "Quantum physics"],
    ["/docs/examples/", "Examples"],
    ["/docs/release-notes/", "Release notes"],
  ];
  return (
    <header className="bar">
      <Link className="bar__mark" href="/">qsengine <span className="bar__version">{VERSION}</span></Link>
      <nav aria-label="Main">
        <ul className="bar__links">
          {links.map(([href, label]) => (
            <li key={href}>
              <Link href={href} aria-current={path === norm(href) ? "page" : undefined}>{label}</Link>
            </li>
          ))}
          <li><a href={REPO}>GitHub</a></li>
        </ul>
      </nav>
    </header>
  );
}

export function SideNav() {
  const path = norm(usePathname());
  return (
    <nav className="side" aria-label="Documentation">
      <Link className="side__home" href="/docs/" aria-current={path === "/docs" ? "page" : undefined}>Documentation contents</Link>
      {DOCS.map((g) => (
        <div className="side__group" key={g.group}>
          <h2 className="side__head">{g.group}</h2>
          <ul>
            {g.items.map((it) => (
              <li key={it.href}>
                <Link href={it.href} aria-current={path === norm(it.href) ? "page" : undefined}>{it.title}</Link>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </nav>
  );
}

// "Previous / Next" links at the foot of every docs page, in reading order.
export function PrevNext() {
  const path = norm(usePathname());
  const flat = DOCS.flatMap((g) => g.items);
  const i = flat.findIndex((it) => norm(it.href) === path);
  if (i === -1) return null;
  const prev = flat[i - 1];
  const next = flat[i + 1];
  return (
    <nav className="prevnext" aria-label="Previous and next page">
      {prev ? <Link href={prev.href}><span>Previous</span>{prev.title}</Link> : <span />}
      {next ? <Link href={next.href} className="prevnext__next"><span>Next</span>{next.title}</Link> : <span />}
    </nav>
  );
}
