import { readFileSync } from "node:fs";
import path from "node:path";
import GithubSlugger from "github-slugger";
import { SideNav, PrevNext } from "../components/Nav";
import { DOCS } from "../content";

// The `##` sections of a docs page, read from its source at build time.
// MDX ids come from rehype-slug, so run every heading through the same slugger in order;
// JSX pages give their <h2>s an explicit id.
function sectionsOf(href) {
  const dir = path.join(process.cwd(), "app", href);
  try {
    const src = readFileSync(path.join(dir, "page.mdx"), "utf8");
    const slugger = new GithubSlugger();
    const out = [];
    let fenced = false;
    for (const line of src.split("\n")) {
      if (line.startsWith("```")) fenced = !fenced;
      const m = !fenced && line.match(/^(#{1,6}) (.+?)\s*$/);
      if (!m) continue;
      const id = slugger.slug(m[2]);
      if (m[1] === "##") out.push({ id, text: m[2] });
    }
    return out;
  } catch {
    const src = readFileSync(path.join(dir, "page.jsx"), "utf8");
    return [...src.matchAll(/<h2 id="([^"]+)">([^<]+)<\/h2>/g)].map((m) => ({ id: m[1], text: m[2] }));
  }
}

const SECTIONS = Object.fromEntries(DOCS.flatMap((g) => g.items).map((it) => [it.href, sectionsOf(it.href)]));

// Every page under /docs: the contents on the left, the page on the right.
export default function DocsLayout({ children }) {
  return (
    <div className="docs">
      <SideNav sections={SECTIONS} />
      <main className="docs__main">
        <article className="prose">{children}</article>
        <PrevNext />
      </main>
    </div>
  );
}
