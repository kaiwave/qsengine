// Static export for GitHub Pages, served from the custom domain in public/CNAME,
// so pages live at the root. Set BASE_PATH=/qsengine only if the site is ever
// served from kaiwave.github.io/qsengine/ instead.
const basePath = process.env.BASE_PATH || "";

// The version shown on the site is read from the crate at build time, so bumping
// `version` in Cargo.toml (the same number pip installs) updates every page.
import { readFileSync } from "node:fs";
import createMDX from "@next/mdx";
const cargo = readFileSync(new URL("../Cargo.toml", import.meta.url), "utf8");
const version = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!version) throw new Error("could not read version from ../Cargo.toml");

// Docs pages are MDX: Markdown with $maths$ (KaTeX, rendered at build time) and
// GitHub-style tables. Plugins are named as strings because Turbopack needs
// serialisable options. throwOnError makes a broken formula fail the build.
const withMDX = createMDX({
  options: {
    remarkPlugins: ["remark-gfm", "remark-math"],
    rehypePlugins: ["rehype-slug", ["rehype-katex", { throwOnError: true }]],
  },
});

export default withMDX({
  output: "export",
  pageExtensions: ["js", "jsx", "mdx"],
  basePath,
  trailingSlash: true,
  images: { unoptimized: true },
  env: { QSENGINE_VERSION: version },
});
