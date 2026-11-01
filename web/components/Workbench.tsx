"use client";

import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";
import { createCheck, getDemo } from "@/lib/api";

export function Workbench() {
  const router = useRouter();
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [demoBusy, setDemoBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const count = useMemo(() => text.trim().length, [text]);
  const canSubmit = count >= 40 && count <= 12000 && !busy && !demoBusy;

  async function submit(value: string, source: string) {
    setError(null);
    setBusy(true);
    try {
      const { id } = await createCheck(value, source);
      router.push(`/check/${id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not start the check.");
      setBusy(false);
    }
  }

  async function runDemo() {
    setError(null);
    setDemoBusy(true);
    try {
      const demo = await getDemo();
      setText(demo.text);
      const { id } = await createCheck(demo.text, "demo");
      router.push(`/check/${id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not load the demo.");
      setDemoBusy(false);
    }
  }

  return (
    <section className="shell workbench" id="check">
      <div className="composer">
        <div className="panel-label">
          <strong>Paste an answer you are about to trust</strong>
          <span>{count.toLocaleString()} / 12,000</span>
        </div>
        <label className="sr-only" htmlFor="answer">
          AI answer to check
        </label>
        <textarea
          id="answer"
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder="Paste a support macro, a docs answer, a sales brief — anything an AI wrote that a person might act on."
          spellCheck={false}
        />
        <div className="composer-actions">
          <button
            className="btn btn--primary"
            disabled={!canSubmit}
            onClick={() => submit(text, "pasted")}
          >
            {busy ? "Starting…" : "Check freshness"}
            <span aria-hidden="true">→</span>
          </button>
          <button
            className="btn btn--ghost"
            onClick={runDemo}
            disabled={busy || demoBusy}
          >
            {demoBusy ? "Loading…" : "Load the 2024 support macro"}
          </button>
        </div>
        <div className="composer-foot">
          <span>Live Tavily sources · evidence attached</span>
          <span>No sign-up</span>
        </div>
        {error && <p className="error-note">{error}</p>}
      </div>

      <div className="specimen" aria-label="Example freshness check">
        <div className="specimen-head">
          <div>
            <div className="specimen-kicker">Specimen 01 · support macro</div>
            <h2 className="specimen-title">OpenAI API pricing</h2>
            <p className="specimen-sub">Written in 2024. Still in the help center.</p>
          </div>
          <div className="specimen-score">
            <b>44</b>
            <span>freshness</span>
          </div>
        </div>
        <div className="specimen-body">
          <p>
            <span className="stale">GPT-4o costs $5 per million input tokens</span>{" "}
            and <span className="stale">$15 per million output tokens</span>.{" "}
            <span className="ok">GPT-4o mini costs $0.15 / $0.60 per million tokens</span>.{" "}
            The Batch API offers a 50% discount.{" "}
            <span className="stale">
              The Assistants API is the recommended way to build agents.
            </span>{" "}
            GPT-4 Turbo costs $10 / $30 per million tokens.
          </p>
        </div>
        <div className="specimen-stamp">3 claims expired</div>
        <div className="specimen-cta">
          <p>Checked against 5 live sources</p>
          <button
            className="btn btn--small"
            onClick={runDemo}
            disabled={busy || demoBusy}
          >
            {demoBusy ? "Opening…" : "Run this check →"}
          </button>
        </div>
      </div>
    </section>
  );
}

