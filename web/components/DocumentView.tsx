"use client";

import { useState } from "react";
import type { Check, Claim } from "@/lib/types";
import { verdictLabel } from "@/lib/format";

type Segment =
  | { kind: "text"; text: string }
  | { kind: "claim"; text: string; claim: Claim }
  | { kind: "corrected"; text: string };

function originalSegments(text: string, claims: Claim[]): Segment[] {
  const sorted = [...claims]
    .filter((c) => c.start >= 0 && c.end <= text.length && c.end > c.start)
    .sort((a, b) => a.start - b.start);
  const out: Segment[] = [];
  let cursor = 0;
  for (const claim of sorted) {
    if (claim.start < cursor) continue;
    if (claim.start > cursor) {
      out.push({ kind: "text", text: text.slice(cursor, claim.start) });
    }
    out.push({
      kind: "claim",
      text: text.slice(claim.start, claim.end),
      claim,
    });
    cursor = claim.end;
  }
  if (cursor < text.length) {
    out.push({ kind: "text", text: text.slice(cursor) });
  }
  return out;
}

function correctedSegments(text: string, claims: Claim[]): Segment[] {
  const marks = claims
    .filter((c) => c.verdict === "changed" && c.corrected_text)
    .map((c) => ({ text: c.corrected_text as string, start: text.indexOf(c.corrected_text as string) }))
    .filter((m) => m.start >= 0)
    .sort((a, b) => a.start - b.start);
  const out: Segment[] = [];
  let cursor = 0;
  for (const mark of marks) {
    if (mark.start < cursor) continue;
    if (mark.start > cursor) {
      out.push({ kind: "text", text: text.slice(cursor, mark.start) });
    }
    out.push({ kind: "corrected", text: mark.text });
    cursor = mark.start + mark.text.length;
  }
  if (cursor < text.length) {
    out.push({ kind: "text", text: text.slice(cursor) });
  }
  return out;
}

export function DocumentView({
  check,
  selected,
  onSelect,
  view,
  onViewChange,
}: {
  check: Check;
  selected: string | null;
  onSelect: (id: string) => void;
  view: "original" | "repaired";
  onViewChange: (view: "original" | "repaired") => void;
}) {
  const [copied, setCopied] = useState(false);
  const running = check.status !== "done" && check.status !== "failed";
  const repaired = view === "repaired" && check.corrected_text;

  async function copyCorrected() {
    if (!check.corrected_text) return;
    try {
      await navigator.clipboard.writeText(check.corrected_text);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    } catch {
      setCopied(false);
    }
  }
  const segments = repaired
    ? correctedSegments(check.corrected_text as string, check.claims)
    : originalSegments(check.input_text, check.claims);

  return (
    <section className="document-panel" aria-label="The answer under review">
      <div className="document-head">
        <span className="document-title">The answer · {check.source}</span>
        {check.corrected_text && (
          <div className="view-toggle" role="group" aria-label="Document view">
            <button
              type="button"
              aria-pressed={view === "original"}
              onClick={() => onViewChange("original")}
            >
              Original
            </button>
            <button
              type="button"
              aria-pressed={view === "repaired"}
              onClick={() => onViewChange("repaired")}
            >
              Repaired
            </button>
          </div>
        )}
      </div>

      {running && <div className="scanline" aria-hidden="true" />}

      <div className={`document-body ${repaired ? "is-corrected" : ""}`}>
        {segments.map((segment, i) => {
          if (segment.kind === "claim") {
            const c = segment.claim;
            return (
              <button
                key={`${c.id}-${i}`}
                id={`claim-${c.id}`}
                type="button"
                className={`claim claim--${c.verdict} ${selected === c.id ? "is-selected" : ""}`}
                onClick={() => onSelect(c.id)}
                aria-label={`Claim ${c.idx + 1}, ${verdictLabel(c.verdict)}`}
              >
                {segment.text}
                <sup>{c.idx + 1}</sup>
              </button>
            );
          }
          if (segment.kind === "corrected") {
            return (
              <mark key={`m-${i}`} className="corrected-mark">
                {segment.text}
              </mark>
            );
          }
          return <span key={`t-${i}`}>{segment.text}</span>;
        })}
      </div>

      <div className="document-foot">
        <span>
          {check.claim_count} claims · {check.changed_count} changed ·{" "}
          {check.stable_count} stable
        </span>
        <span style={{ display: "inline-flex", alignItems: "center", gap: 10 }}>
          <span>
            {repaired
              ? "Only contradicted sentences were replaced."
              : "Click a highlighted sentence to see its evidence."}
          </span>
          {repaired && (
            <button type="button" className="btn btn--small" onClick={copyCorrected}>
              {copied ? "Copied" : "Copy repaired"}
            </button>
          )}
        </span>
      </div>
    </section>
  );
}
