import CopyButton from "./CopyButton";

// A copyable code block, as the docs show every snippet.
export function CodeBlock({ code, label = "Copy code" }) {
  return (
    <div className="codeblock">
      <pre><code>{code}</code></pre>
      <CopyButton text={code} label={label} />
    </div>
  );
}

// What the snippet above it printed, plus an optional bar for each outcome.
export function Output({ printed, rows = [], caption }) {
  return (
    <div className="output" data-ui>
      <p className="output__label">Output</p>
      <pre><code>{printed}</code></pre>
      {rows.length > 0 && (
        <dl className="bars">
          {rows.map(([label, value, max]) => (
            <div className="bars__row" key={label}>
              <dt><code>{label}</code></dt>
              <dd>
                <span className="bars__bar" style={{ "--w": value / max }} />
                <span className="bars__value">{value}</span>
              </dd>
            </div>
          ))}
        </dl>
      )}
      {caption && <p className="output__caption">{caption}</p>}
    </div>
  );
}

// One full example: a sentence, the code, its output.
export function Example({ ex, headingLevel = 2 }) {
  const H = `h${headingLevel}`;
  return (
    <section className="example" id={ex.id}>
      <H>{ex.name}</H>
      <p>{ex.intro}</p>
      <CodeBlock code={ex.code} label={`Copy the ${ex.name} example`} />
      <Output printed={ex.printed} rows={ex.rows} caption={ex.caption} />
    </section>
  );
}
