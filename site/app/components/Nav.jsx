"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
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

// `sections` maps each page's href to its [{ id, text }] sections, which open under the page.
export function SideNav({ sections }) {
  const path = norm(usePathname());
  const [open, setOpen] = useState(() => new Set([path]));
  // The page you navigate to opens; pages you opened by hand stay open.
  useEffect(() => setOpen((s) => (s.has(path) ? s : new Set(s).add(path))), [path]);
  const toggle = (p) => setOpen((s) => {
    const next = new Set(s);
    next.has(p) ? next.delete(p) : next.add(p);
    return next;
  });
  return (
    <nav className="side" aria-label="Documentation">
      <Link className="side__home" href="/docs/" aria-current={path === "/docs" ? "page" : undefined}>Documentation contents</Link>
      {DOCS.map((g) => (
        <details className="side__group" key={g.group} open>
          <summary className="side__head"><h2>{g.group}</h2></summary>
          <ul>
            {g.items.map((it) => {
              const p = norm(it.href);
              const subs = sections[it.href] ?? [];
              const isOpen = subs.length > 0 && open.has(p);
              return (
                <li key={it.href} className={isOpen ? "side__open" : undefined}>
                  <div className="side__row">
                    <Link href={it.href} aria-current={path === p ? "page" : undefined}>{it.title}</Link>
                    {subs.length > 0 && (
                      <button type="button" className="side__toggle" aria-expanded={isOpen}
                        aria-label={`Sections of ${it.title}`} onClick={() => toggle(p)} />
                    )}
                  </div>
                  {isOpen && (
                    <ul className="side__sections">
                      {subs.map((s) => (
                        <li key={s.id}><Link href={`${it.href}#${s.id}`}>{s.text}</Link></li>
                      ))}
                    </ul>
                  )}
                </li>
              );
            })}
          </ul>
        </details>
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
