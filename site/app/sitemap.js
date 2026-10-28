import { DOCS, SITE } from "./content";

export const dynamic = "force-static";

// Every page, for search engines: the home page, the docs index and each topic.
export default function sitemap() {
  const paths = ["/", "/docs/", ...DOCS.flatMap((g) => g.items.map((it) => it.href))];
  return paths.map((p) => ({ url: `${SITE}${p}` }));
}
