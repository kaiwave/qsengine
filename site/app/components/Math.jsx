import katex from "katex";

// Rendered to HTML at build time, so pages ship finished maths and no maths JavaScript.
// throwOnError makes a typo in a formula fail the build instead of showing red text.
function render(tex, displayMode) {
  return { __html: katex.renderToString(tex, { displayMode, throwOnError: true, output: "html" }) };
}

// Inline maths: <M>{String.raw`\ket{0}`}</M>
export function M({ children }) {
  return <span dangerouslySetInnerHTML={render(children, false)} />;
}

// A displayed equation on its own line.
export function MBlock({ children }) {
  return <div className="math-block" dangerouslySetInnerHTML={render(children, true)} />;
}
