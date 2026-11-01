"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { createCheck, eventsUrl, getCheck } from "@/lib/api";
import type { Check, CheckEvent } from "@/lib/types";
import { scoreLine, timeAgo, verdictLabel } from "@/lib/format";
import { Header } from "./Header";
import { DocumentView } from "./DocumentView";
import { EvidenceCard } from "./EvidenceCard";
import { ProofReceipt } from "./ProofReceipt";
import { ChangesSummary } from "./ChangesSummary";

export function CheckRoom({ id }: { id: string }) {
  const router = useRouter();
  const [check, setCheck] = useState<Check | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [view, setView] = useState<"original" | "repaired">("original");
  const [drillBusy, setDrillBusy] = useState(false);
  const [recheckBusy, setRecheckBusy] = useState(false);
  const [copied, setCopied] = useState(false);
  const esRef = useRef<EventSource | null>(null);
  const pollRef = useRef<number | null>(null);

  useEffect(() => {
    let cancelled = false;
    setCheck(null);
    setError(null);

    function startPolling() {
      if (pollRef.current) return;
      pollRef.current = window.setInterval(async () => {
        try {
          const next = await getCheck(id);
          if (cancelled) return;
          setCheck(next);
          if (next.status === "done" || next.status === "failed") {
            if (pollRef.current) window.clearInterval(pollRef.current);
            pollRef.current = null;
          }
        } catch {
          // keep polling briefly; the initial error path handles hard failure
        }
      }, 2500);
    }

    function subscribe() {
      const es = new EventSource(eventsUrl(id));
      esRef.current = es;
      es.onmessage = (event) => {
        let data: CheckEvent;
        try {
          data = JSON.parse(event.data) as CheckEvent;
        } catch {
          return;
        }
        if (data.type === "status") {
          setCheck((prev) =>
            prev
              ? { ...prev, status: data.status, stage_message: data.message }
              : prev,
          );
        } else if (data.type === "claims") {
          setCheck((prev) =>
            prev
              ? {
                  ...prev,
                  claims: data.claims,
                  claim_count: data.claims.length,
                }
              : prev,
          );
        } else if (data.type === "claim_update") {
          setCheck((prev) => {
            if (!prev) return prev;
            const claims = prev.claims.map((c) =>
              c.id === data.claim.id ? data.claim : c,
            );
            return { ...prev, claims };
          });
        } else if (data.type === "done") {
          setCheck(data.check);
          es.close();
        } else if (data.type === "failed") {
          setError(data.message);
          es.close();
        }
      };
      es.onerror = () => {
        es.close();
        startPolling();
      };
    }

    getCheck(id)
      .then((initial) => {
        if (cancelled) return;
        setCheck(initial);
        if (initial.status !== "done" && initial.status !== "failed") {
          subscribe();
        }
      })
      .catch((err) => {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : "Could not load this check.");
      });

    return () => {
      cancelled = true;
      esRef.current?.close();
      if (pollRef.current) window.clearInterval(pollRef.current);
    };
  }, [id]);

  async function recheck() {
    if (!check) return;
    setRecheckBusy(true);
    setError(null);
    try {
      const { id: nextId } = await createCheck(check.input_text, "recheck", false);
      router.push(`/check/${nextId}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not start a new check.");
      setRecheckBusy(false);
    }
  }

  async function drill() {
    if (!check) return;
    setDrillBusy(true);
    setError(null);
    try {
      const { id: nextId } = await createCheck(check.input_text, "drill", true);
      router.push(`/check/${nextId}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not run the drill.");
      setDrillBusy(false);
    }
  }

  async function copyLink() {
    try {
      await navigator.clipboard.writeText(window.location.href);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    } catch {
      setCopied(false);
    }
  }

  function selectClaim(claimId: string) {
    setSelected(claimId);
    document
      .getElementById(`evidence-${claimId}`)
      ?.scrollIntoView({ behavior: "smooth", block: "center" });
  }

  function selectFromMargin(claimId: string) {
    setSelected(claimId);
    document
      .getElementById(`claim-${claimId}`)
      ?.scrollIntoView({ behavior: "smooth", block: "center" });
  }

  if (!check) {
    return (
      <>
        <Header compact />
        <main className="shell">
          <div className="loading-page">
            {error ? (
              <div className="failed-box" style={{ maxWidth: 520 }}>
                <h2>This check could not be loaded.</h2>
                <p>{error}</p>
                <p style={{ marginTop: 16 }}>
                  <Link className="btn" href="/">
                    Start a new check
                  </Link>
                </p>
              </div>
            ) : (
              <span>Opening the evidence room…</span>
            )}
          </div>
        </main>
      </>
    );
  }

  const claims = [...check.claims].sort((a, b) => a.idx - b.idx);
  const running = check.status !== "done" && check.status !== "failed";
  const score = check.freshness_score;
  const strip =
    claims.length > 0
      ? claims
      : Array.from({ length: Math.max(check.claim_count, 6) }).map((_, i) => ({
          id: `pending-${i}`,
          verdict: "pending" as const,
        }));

  return (
    <>
      <Header compact />
      <main className="shell check-page">
        <div className="check-topline">
          <span>
            Check {check.id.slice(0, 8)} · started {timeAgo(check.created_at)}
          </span>
          <span style={{ display: "flex", gap: 18 }}>
            <button
              type="button"
              onClick={copyLink}
              style={{
                background: "none",
                border: 0,
                padding: 0,
                font: "inherit",
                letterSpacing: "inherit",
                textTransform: "inherit",
                color: "inherit",
                cursor: "pointer",
              }}
            >
              {copied ? "Link copied" : "Copy link"}
            </button>
            <Link href="/">New check</Link>
          </span>
        </div>

        {check.status === "failed" && (
          <div className="failed-box">
            <h2>The check stopped before a verdict.</h2>
            <p>{check.error ?? "A source or model was unavailable."}</p>
            <p style={{ marginTop: 8 }}>
              Nothing was guessed. Try again, or inspect the receipt below for
              the failing route.
            </p>
          </div>
        )}

        <section className="verdict-bar" aria-label="Freshness summary">
          <div className="score-block">
            <div className={`score-num ${score === null ? "idle" : ""}`}>
              {score === null ? "—" : Math.round(score)}
            </div>
            <div className="score-meta">
              <div className="score-label">Freshness / 100</div>
              <p className="score-line">
                {scoreLine(score, check.changed_count)}
              </p>
            </div>
          </div>

          <div className="verdict-mid">
            <div className="claim-strip" aria-hidden="true">
              {strip.map((c) => (
                <span
                  key={c.id}
                  className={c.verdict}
                  title={verdictLabel(c.verdict)}
                />
              ))}
            </div>
            <div className="legend">
              <span>
                <i className="changed" /> {check.changed_count} changed
              </span>
              <span>
                <i className="supported" /> {check.supported_count} supported
              </span>
              <span>
                <i className="unverifiable" /> {check.unverifiable_count}{" "}
                unverifiable
              </span>
              <span>
                <i className="stable" /> {check.stable_count} stable
              </span>
            </div>
          </div>

          <div className="verdict-actions">
            {running ? (
              <span className="running-note">{check.stage_message}</span>
            ) : (
              <>
                <button
                  type="button"
                  className="btn btn--small btn--ghost"
                  onClick={recheck}
                  disabled={recheckBusy}
                >
                  {recheckBusy ? "Starting…" : "Re-check"}
                </button>
                <Link className="btn btn--small" href="/">
                  Check another
                </Link>
              </>
            )}
          </div>
        </section>

        <div className="room-grid">
          <div className="document-column">
            <DocumentView
              check={check}
              selected={selected}
              onSelect={selectClaim}
              view={view}
              onViewChange={setView}
            />
            <ProofReceipt
              check={check}
              onDrill={drill}
              drillBusy={drillBusy}
            />
            <p className="score-note">
              The freshness score is the importance-weighted average of cited
              verdicts: supported and stable count 1.0, unverifiable 0.5,
              changed 0.0. It summarizes evidence, not probability.{" "}
              <Link href="/method">Read the method →</Link>
            </p>
            <ChangesSummary claims={claims} />
          </div>

          <aside className="margin-panel" aria-label="Evidence margin">
            <div className="margin-head">
              <h2>Evidence margin</h2>
              <span>
                {claims.length} claims · {check.proof.tavily_searches} searches
              </span>
            </div>
            {claims.length === 0 && (
              <p className="empty-margin">
                {running
                  ? "Segmenting the answer. Claims will appear here with exact offsets."
                  : "No time-sensitive factual claims were found in this answer."}
              </p>
            )}
            {claims.map((claim) => (
              <EvidenceCard
                key={claim.id}
                claim={claim}
                selected={selected === claim.id}
                onSelect={selectFromMargin}
              />
            ))}
          </aside>
        </div>
      </main>
    </>
  );
}
