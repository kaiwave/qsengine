// Static export for GitHub Pages, served from the custom domain in public/CNAME,
// so pages live at the root. Set BASE_PATH=/qsengine only if the site is ever
// served from kaiwave.github.io/qsengine/ instead.
const basePath = process.env.BASE_PATH || "";

// The version shown on the site is read from the crate at build time, so bumping
// `version` in Cargo.toml (the same number pip installs) updates every page.
import { readFileSync } from "node:fs";
const cargo = readFileSync(new URL("../Cargo.toml", import.meta.url), "utf8");
const version = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!version) throw new Error("could not read version from ../Cargo.toml");

export default {
  output: "export",
  basePath,
  trailingSlash: true,
  images: { unoptimized: true },
  env: { QSENGINE_VERSION: version },
};
