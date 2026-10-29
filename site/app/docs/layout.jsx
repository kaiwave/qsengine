import { SideNav, PrevNext } from "../components/Nav";

// Every page under /docs: the contents on the left, the page on the right.
export default function DocsLayout({ children }) {
  return (
    <div className="docs">
      <SideNav />
      <main className="docs__main">
        <article className="prose">{children}</article>
        <PrevNext />
      </main>
    </div>
  );
}
