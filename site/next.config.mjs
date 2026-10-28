// Static export for GitHub Pages. The site lives at /<repo>/ on Pages, so the
// deploy workflow sets BASE_PATH=/qubit-simulator-engine; locally it is empty.
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
