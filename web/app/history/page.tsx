import Link from "next/link";
import { Footer } from "@/components/Footer";
import { Header } from "@/components/Header";
import type { CheckSummary } from "@/lib/types";
import { timeAgo } from "@/lib/format";

export const dynamic = "force-dynamic";

async function loadChecks(): Promise<{
  checks: CheckSummary[];
  failed: boolean;
}> {
  const base =
    process.env.API_ORIGIN ||
    process.env.NEXT_PUBLIC_API_BASE ||
    "http://localhost:8788";
  try {
    const res = await fetch(`${base}/api/checks`, { cache: "no-store" });
    if (!res.ok) return { checks: [], failed: true };
    const body = (await res.json()) as { checks: CheckSummary[] };
    return { checks: body.checks, failed: false };
  } catch {
    return { checks: [], failed: true };
  }
}

export default async function HistoryPage() {
  const { checks, failed } = await loadChecks();
  return (
    <>
      <Header />
      <main className="shell history-page">
        <h1>Checks, kept.</h1>
        <p>
          Every freshness check is stored with its claims, sources, and gateway
          receipt. Nothing here is regenerated from a prompt. Re-checks link
          back to their parent so drift over time is visible.
        </p>
        {failed ? (
          <div className="failed-box">
            <h2>Check history is unavailable.</h2>
            <p>
              The API could not be reached. This is not an empty history — it
              is an outage. Retry in a moment.
            </p>
            <p style={{ marginTop: 16 }}>
              <Link className="btn" href="/">
                Start a new check
              </Link>
            </p>
          </div>
        ) : checks.length === 0 ? (
          <div className="history-empty">
            No checks yet. <Link href="/">Run the first one →</Link>
          </div>
        ) : (
          <div className="history-list">
            {checks.map((c) => (
              <Link key={c.id} href={`/check/${c.id}`} className="history-row">
                <span className={`history-score ${scoreToneClass(c.freshness_score)}`}>
                  {c.freshness_score === null
                    ? "—"
                    : Math.round(c.freshness_score)}
                </span>
                <span className="history-preview">
                  {c.preview}
                  {c.recheck_of && (
                    <span className="history-recheck">
                      re-check of {c.recheck_of.slice(0, 8)}
                    </span>
                  )}
                </span>
                <span className="history-meta">
                  {c.changed_count} changed
                </span>
                <span className="history-meta">{c.claim_count} claims</span>
                <span className="history-meta">{timeAgo(c.created_at)}</span>
              </Link>
            ))}
          </div>
        )}
      </main>
      <Footer />
    </>
  );
}

function scoreToneClass(score: number | null): string {
  if (score === null) return "idle";
  if (score >= 80) return "good";
  if (score >= 50) return "mixed";
  return "bad";
}
