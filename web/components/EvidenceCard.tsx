import type { Claim } from "@/lib/types";
import { domain, safeHref, shortDate, truncate, verdictLabel } from "@/lib/format";

export function EvidenceCard({
  claim,
  selected,
  onSelect,
}: {
  claim: Claim;
  selected: boolean;
  onSelect: (id: string) => void;
}) {
  const pending = claim.verdict === "pending";
  return (
    <article
      id={`evidence-${claim.id}`}
      className={`evidence-card ${selected ? "is-selected" : ""}`}
      onClick={() => onSelect(claim.id)}
    >
      <div className="evidence-top">
        <span className="evidence-index">
          {String(claim.idx + 1).padStart(2, "0")}
        </span>
        <span className={`chip chip--${claim.verdict}`}>
          {pending ? "Checking" : verdictLabel(claim.verdict)}
        </span>
        <span className="volatility">{claim.volatility} volatility</span>
      </div>

      <p className="claim-quote">“{claim.quote}”</p>

      {pending && (
        <>
          <div className="skeleton" style={{ width: "82%" }} />
          <div className="skeleton" style={{ width: "64%" }} />
          <p className="claim-explain">
            Searching live sources: <span className="mono">{claim.query}</span>
          </p>
        </>
      )}

      {claim.verdict === "changed" && claim.corrected_text && (
        <div className="now-box">
          <b>Now</b>
          {claim.corrected_text}
        </div>
      )}

      {claim.explanation && (
        <p className="claim-explain">{claim.explanation}</p>
      )}

      {claim.evidence.length > 0 && (
        <div className="sources">
          {claim.evidence.slice(0, 3).map((e) => (
            <a
              key={e.id}
              className="source"
              href={safeHref(e.url)}
              target="_blank"
              rel="noreferrer noopener"
              onClick={(event) => event.stopPropagation()}
            >
              <div className="source-head">
                <span className="source-title">{e.title} ↗</span>
                <span className="source-meta">
                  {domain(e.url)} ·{" "}
                  {shortDate(e.published_date ?? e.retrieved_at)}
                </span>
              </div>
              <p className="source-quote">{truncate(e.quote, 210)}</p>
            </a>
          ))}
        </div>
      )}

      {claim.verdict === "unverifiable" && claim.evidence.length === 0 && (
        <p className="no-source">No direct source found · not guessed</p>
      )}
    </article>
  );
}
