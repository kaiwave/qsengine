import { Example } from "../../components/Code";
import { EXAMPLES, VERSION } from "../../content";

export const metadata = { title: "Examples" };

export default function Examples() {
  return (
    <>
      <h1>Examples</h1>
      <p>
        Each example is a complete program. Copy it, run it, and you will see the output shown under it: every output
        on this page was produced by running the code with qsengine {VERSION}.
      </p>
      <ul>
        {EXAMPLES.map((ex) => (
          <li key={ex.id}><a href={`#${ex.id}`}>{ex.name}</a></li>
        ))}
      </ul>
      {EXAMPLES.map((ex) => <Example key={ex.id} ex={ex} />)}
    </>
  );
}
