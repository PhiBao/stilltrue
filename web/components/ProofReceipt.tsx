"use client";

import { useState } from "react";
import type { Check } from "@/lib/types";

export function ProofReceipt({
  check,
  onDrill,
  drillBusy,
}: {
  check: Check;
  onDrill: () => void;
  drillBusy: boolean;
}) {
  const [copied, setCopied] = useState(false);
  const p = check.proof;
  const route =
    p.provider && p.model ? `${p.provider} / ${p.model}` : "not yet routed";

  async function copy() {
    try {
      await navigator.clipboard.writeText(
        JSON.stringify(
          {
            check_id: check.id,
            route,
            requested_model: p.requested_model,
            fallback_used: p.fallback_used,
            fallback_reason: p.fallback_reason,
            tokens: { input: p.prompt_tokens, output: p.completion_tokens },
            tavily: { searches: p.tavily_searches, credits: p.tavily_credits },
            duration_ms: p.duration_ms,
            request_ids: p.request_ids,
            drill: p.drill,
          },
          null,
          2,
        ),
      );
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    } catch {
      setCopied(false);
    }
  }

  return (
    <div className="receipt-wrap">
      <div className="receipt">
        <div className="receipt-head">
          <b>Gateway receipt</b>
          <span>Bifrost · Tavily</span>
        </div>
        <div className="receipt-row">
          <span>route</span>
          <span>{route}</span>
        </div>
        <div className="receipt-row">
          <span>requested</span>
          <span>{p.requested_model ?? "—"}</span>
        </div>
        <div className={`receipt-row ${p.fallback_used ? "takeover" : ""}`}>
          <span>failover</span>
          <span>
            {p.fallback_used
              ? `yes · hop ${p.fallback_index ?? 1}`
              : "no · primary held"}
          </span>
        </div>
        {p.fallback_reason && (
          <div className="receipt-row">
            <span>reason</span>
            <span>{p.fallback_reason}</span>
          </div>
        )}
        {p.drill && (
          <div className="receipt-row takeover">
            <span>drill</span>
            <span>forced primary outage</span>
          </div>
        )}
        <div className="receipt-row">
          <span>tokens</span>
          <span>
            {p.prompt_tokens.toLocaleString()} in /{" "}
            {p.completion_tokens.toLocaleString()} out
          </span>
        </div>
        <div className="receipt-row">
          <span>tavily</span>
          <span>
            {p.tavily_searches} searches · {p.tavily_credits} credits
          </span>
        </div>
        <div className="receipt-row">
          <span>llm calls</span>
          <span>
            {p.llm_calls} ({p.llm_attempts} attempts)
          </span>
        </div>
        <div className="receipt-row">
          <span>wall time</span>
          <span>{(p.duration_ms / 1000).toFixed(1)}s</span>
        </div>
        <div className="receipt-row">
          <span>request id</span>
          <span>{p.request_ids[0] ?? "—"}</span>
        </div>
        <div className="receipt-row">
          <span>budget</span>
          <span>virtual key · $20 / month cap</span>
        </div>
        <div className="receipt-row">
          <span>cloud equiv.</span>
          <span>${p.cloud_equivalent_usd.toFixed(6)}</span>
        </div>
        <p className="receipt-note">
          Route, failover and usage are read from Bifrost response headers and
          usage counters. Cloud-equivalent is a benchmark at GPT-4o mini list
          rates, not the TokenFactory bill.
        </p>
        <div className="receipt-actions">
          <button className="btn btn--small" onClick={copy} type="button">
            {copied ? "Copied" : "Copy receipt"}
          </button>
          <button
            className="btn btn--small btn--ghost"
            onClick={onDrill}
            disabled={drillBusy}
            type="button"
          >
            {drillBusy ? "Drilling…" : "Test provider failover"}
          </button>
        </div>
      </div>
    </div>
  );
}
