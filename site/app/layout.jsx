import "katex/dist/katex.min.css";
import "./globals.css";
import Link from "next/link";
import { SiteHeader } from "./components/Nav";
import { REPO, PYPI, SITE, VERSION } from "./content";

export const metadata = {
  title: { default: "qsengine: quantum circuits in Python, run in Rust", template: "%s | qsengine" },
  description: "qsengine is a state-vector quantum simulator for Python with a Rust core that runs on any CPU or GPU.",
  metadataBase: new URL(SITE),
  openGraph: {
    type: "website",
    siteName: "qsengine",
    title: "qsengine: quantum circuits in Python, run in Rust",
    description: "A fast state-vector quantum simulator for Python, with documentation and guides from bra-ket notation to Deutsch-Jozsa.",
  },
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="" />
        <link
          rel="stylesheet"
          href="https://fonts.googleapis.com/css2?family=Red+Hat+Display:wght@900&family=Red+Hat+Text:wght@400;500;700&display=swap"
        />
      </head>
      <body>
        <SiteHeader />
        {children}
        <footer className="foot">
          <p>qsengine {VERSION} by <a href="https://kaiwave.dev">kaiwave</a>, MIT licence.</p>
          <ul>
            <li><Link href="/docs/">Documentation</Link></li>
            <li><a href={REPO}>Source on GitHub</a></li>
            <li><a href={`${REPO}/issues`}>Report an issue</a></li>
            <li><a href={PYPI}>PyPI</a></li>
          </ul>
        </footer>
      </body>
    </html>
  );
}
