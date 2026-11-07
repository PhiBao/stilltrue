"use client";

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
  const cited = new Set((claim.cited_evidence_ids ?? []).map((s) => s.toLowerCase()));
  return (
    <article
      id={`evidence-${claim.id}`}
      data-selected={selected}
      className={`evidence-card ${selected ? "is-selected" : ""}`}
    >
      <button
        type="button"
        className="evidence-select"
        aria-pressed={selected}
        aria-label={`Select claim ${claim.idx + 1} to locate it in the document`}
        onClick={() => onSelect(claim.id)}
      >
        <div className="evidence-top">
          <span className="evidence-index">
            {String(claim.idx + 1).padStart(2, "0")}
          </span>
          <span className={`chip chip--${claim.verdict}`}>
            {pending ? "Checking" : verdictLabel(claim.verdict)}
          </span>
          <span className="volatility">
            {claim.volatility}
            {claim.volatility_reason ? ` · ${claim.volatility_reason}` : ""}
          </span>
        </div>
        <p className="claim-quote">“{claim.quote}”</p>
      </button>

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
          {claim.evidence.map((e, i) => {
            const decisive = cited.has(e.id.toLowerCase());
            return (
              <a
                key={e.id}
                className={`source ${decisive ? "source--decisive" : ""}`}
                href={safeHref(e.url)}
                target="_blank"
                rel="noreferrer noopener"
              >
                <div className="source-head">
                  <span className="source-title">
                    [{i + 1}] {e.title} ↗
                    {decisive && <span className="decisive-tag"> decisive</span>}
                  </span>
                  <span className="source-meta">
                    {domain(e.url)} · pub {shortDate(e.published_date)} · got{" "}
                    {shortDate(e.retrieved_at)}
                  </span>
                </div>
                <p className="source-quote">{truncate(e.quote, 260)}</p>
              </a>
            );
          })}
        </div>
      )}

      {claim.verdict === "unverifiable" && claim.evidence.length === 0 && (
        <p className="no-source">No direct source found · not guessed</p>
      )}
    </article>
  );
}
