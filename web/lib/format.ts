import type { Verdict } from "./types";

export function domain(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

export function timeAgo(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const seconds = Math.max(1, Math.round((Date.now() - then) / 1000));
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.round(hours / 24);
  return `${days}d ago`;
}

export function shortDate(iso: string | null | undefined): string {
  if (!iso) return "date unknown";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "date unknown";
  return d.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function verdictLabel(v: Verdict | string): string {
  switch (v) {
    case "changed":
      return "Changed";
    case "supported":
      return "Supported";
    case "unverifiable":
      return "Unverifiable";
    case "stable":
      return "Stable";
    case "pending":
      return "Checking";
    default:
      return v;
  }
}

export function scoreLine(score: number | null, changed: number): string {
  if (score === null) return "Awaiting evidence";
  if (changed === 0) return "Every checked claim still holds";
  if (changed === 1) return "One claim is no longer true";
  return `${changed} claims are no longer true`;
}

export function scoreTone(score: number | null): "good" | "mixed" | "bad" | "idle" {
  if (score === null) return "idle";
  if (score >= 80) return "good";
  if (score >= 50) return "mixed";
  return "bad";
}

export function truncate(s: string, n: number): string {
  if (s.length <= n) return s;
  return `${s.slice(0, n).trimEnd()}…`;
}

export function initials(s: string): string {
  return s
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? "")
    .join("");
}

/** Only allow http(s) links from untrusted search results. */
export function safeHref(url: string): string {
  try {
    const parsed = new URL(url);
    if (parsed.protocol === "http:" || parsed.protocol === "https:") {
      return parsed.toString();
    }
  } catch {
    // fall through
  }
  return "#";
}
