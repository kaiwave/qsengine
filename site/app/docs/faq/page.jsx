import { FAQ } from "../../content";

export const metadata = { title: "Questions" };

export default function Faq() {
  return (
    <>
      <h1>Questions</h1>
      <p>Short answers to the questions people ask first. The user guide has the detail.</p>
      <dl className="faq">
        {FAQ.map(([q, a]) => (
          <div key={q}>
            <dt>{q}</dt>
            <dd>{a}</dd>
          </div>
        ))}
      </dl>
    </>
  );
}
