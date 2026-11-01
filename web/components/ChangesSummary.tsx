import type { Claim } from "@/lib/types";

export function ChangesSummary({ claims }: { claims: Claim[] }) {
  const changed = claims.filter(
    (c) => c.verdict === "changed" && c.corrected_text,
  );
  if (changed.length === 0) return null;
  return (
    <section className="changes-summary" aria-label="What changed">
      <div className="margin-head">
        <h2>What changed</h2>
        <span>{changed.length} repairs</span>
      </div>
      {changed.map((c) => (
        <div key={c.id} className="change-row">
          <span className="evidence-index">
            {String(c.idx + 1).padStart(2, "0")}
          </span>
          <div>
            <p className="change-was">
              <b>Was</b> {c.quote}
            </p>
            <p className="change-now">
              <b>Now</b> {c.corrected_text}
            </p>
          </div>
        </div>
      ))}
    </section>
  );
}
