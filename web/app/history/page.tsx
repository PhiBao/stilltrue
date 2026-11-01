import Link from "next/link";
import { Footer } from "@/components/Footer";
import { Header } from "@/components/Header";
import type { CheckSummary } from "@/lib/types";
import { scoreTone, timeAgo } from "@/lib/format";

export const dynamic = "force-dynamic";

async function loadChecks(): Promise<CheckSummary[]> {
  const base =
    process.env.API_ORIGIN ||
    process.env.NEXT_PUBLIC_API_BASE ||
    "http://localhost:8788";
  try {
    const res = await fetch(`${base}/api/checks`, { cache: "no-store" });
    if (!res.ok) return [];
    const body = (await res.json()) as { checks: CheckSummary[] };
    return body.checks;
  } catch {
    return [];
  }
}

export default async function HistoryPage() {
  const checks = await loadChecks();
  return (
    <>
      <Header />
      <main className="shell history-page">
        <h1>Checks, kept.</h1>
        <p>
          Every freshness check is stored with its claims, sources, and gateway
          receipt. Nothing here is regenerated from a prompt.
        </p>
        {checks.length === 0 ? (
          <div className="history-empty">
            No checks yet. <Link href="/">Run the first one →</Link>
          </div>
        ) : (
          <div className="history-list">
            {checks.map((c) => (
              <Link key={c.id} href={`/check/${c.id}`} className="history-row">
                <span
                  className={`history-score ${scoreTone(c.freshness_score)}`}
                >
                  {c.freshness_score === null
                    ? "—"
                    : Math.round(c.freshness_score)}
                </span>
                <span className="history-preview">{c.preview}</span>
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
