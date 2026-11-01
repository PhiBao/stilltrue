import type { Check, CheckSummary, DemoAnswer } from "./types";

export const API_BASE = process.env.NEXT_PUBLIC_API_BASE ?? "";

async function json<T>(res: Response): Promise<T> {
  if (!res.ok) {
    let message = `Request failed (${res.status})`;
    try {
      const body = (await res.json()) as { error?: string };
      if (body.error) message = body.error;
    } catch {
      // keep the status message
    }
    throw new Error(message);
  }
  return (await res.json()) as T;
}

export async function createCheck(
  text: string,
  source = "pasted",
  drill = false,
): Promise<{ id: string }> {
  const res = await fetch(`${API_BASE}/api/checks`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ text, source, drill }),
  });
  return json<{ id: string }>(res);
}

export async function getCheck(id: string): Promise<Check> {
  const res = await fetch(`${API_BASE}/api/checks/${id}`, { cache: "no-store" });
  return json<Check>(res);
}

export async function listChecks(): Promise<CheckSummary[]> {
  const res = await fetch(`${API_BASE}/api/checks`, { cache: "no-store" });
  const body = await json<{ checks: CheckSummary[] }>(res);
  return body.checks;
}

export async function getDemo(): Promise<DemoAnswer> {
  const res = await fetch(`${API_BASE}/api/demo`, { cache: "no-store" });
  return json<DemoAnswer>(res);
}

export function eventsUrl(id: string): string {
  return `${API_BASE}/api/checks/${id}/events`;
}
