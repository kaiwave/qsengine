import "./globals.css";
import Link from "next/link";
import { SiteHeader } from "./components/Nav";
import { REPO, PYPI, VERSION } from "./content";

export const metadata = {
  title: { default: "qsengine: quantum circuits in Python, run in Rust", template: "%s | qsengine" },
  description: "qsengine is a state-vector quantum simulator for Python with a Rust core that runs on any CPU or GPU.",
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
          <p>qsengine {VERSION}, MIT licence.</p>
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
