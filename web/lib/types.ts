export type Verdict =
  | "pending"
  | "supported"
  | "changed"
  | "unverifiable"
  | "stable";

export type Status =
  | "queued"
  | "extracting"
  | "verifying"
  | "adjudicating"
  | "rewriting"
  | "done"
  | "failed";

export interface Evidence {
  id: string;
  title: string;
  url: string;
  quote: string;
  published_date: string | null;
  retrieved_at: string;
  score: number;
}

export interface Claim {
  id: string;
  idx: number;
  quote: string;
  text: string;
  volatility: "high" | "medium" | "low" | string;
  volatility_reason: string;
  query: string;
  verdict: Verdict;
  confidence: number;
  explanation: string;
  corrected_text: string | null;
  importance: number;
  start: number;
  end: number;
  evidence: Evidence[];
  cited_evidence_ids?: string[];
}

export interface Proof {
  provider: string | null;
  model: string | null;
  requested_model: string | null;
  fallback_used: boolean;
  fallback_index: number | null;
  prompt_tokens: number;
  completion_tokens: number;
  llm_calls: number;
  llm_attempts: number;
  fallback_reason: string | null;
  tavily_searches: number;
  tavily_credits: number;
  duration_ms: number;
  request_ids: string[];
  cache_hits: number;
  drill: boolean;
  cloud_equivalent_usd: number;
}

export interface Check {
  id: string;
  input_text: string;
  source: string;
  status: Status;
  stage_message: string;
  freshness_score: number | null;
  claim_count: number;
  changed_count: number;
  supported_count: number;
  unverifiable_count: number;
  stable_count: number;
  corrected_text: string | null;
  created_at: string;
  completed_at: string | null;
  error: string | null;
  claims: Claim[];
  proof: Proof;
  adjudicated_count?: number;
  coverage?: number | null;
  recheck_of?: string | null;
  source_url?: string | null;
}

export interface CheckSummary {
  id: string;
  preview: string;
  source: string;
  status: Status;
  freshness_score: number | null;
  claim_count: number;
  changed_count: number;
  created_at: string;
  recheck_of?: string | null;
}

export type CheckEvent =
  | { type: "status"; status: Status; message: string }
  | { type: "claims"; claims: Claim[] }
  | { type: "claim_update"; claim: Claim }
  | { type: "done"; check: Check }
  | { type: "failed"; message: string };

export interface DemoAnswer {
  title: string;
  subtitle: string;
  text: string;
}
