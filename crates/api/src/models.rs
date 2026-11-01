use serde::{Deserialize, Serialize};

pub const VERDICT_SUPPORTED: &str = "supported";
pub const VERDICT_CHANGED: &str = "changed";
pub const VERDICT_UNVERIFIABLE: &str = "unverifiable";
pub const VERDICT_STABLE: &str = "stable";
pub const VERDICT_PENDING: &str = "pending";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub title: String,
    pub url: String,
    pub quote: String,
    pub published_date: Option<String>,
    pub retrieved_at: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub idx: i64,
    pub quote: String,
    pub text: String,
    pub volatility: String,
    pub volatility_reason: String,
    pub query: String,
    pub verdict: String,
    pub confidence: f64,
    pub explanation: String,
    pub corrected_text: Option<String>,
    pub importance: i64,
    pub start: i64,
    pub end: i64,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

impl Claim {
    pub fn is_time_sensitive(&self) -> bool {
        self.volatility == "high" || self.volatility == "medium"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Proof {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub requested_model: Option<String>,
    pub fallback_used: bool,
    pub fallback_index: Option<i64>,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub llm_calls: i64,
    pub llm_attempts: i64,
    pub fallback_reason: Option<String>,
    pub tavily_searches: i64,
    pub tavily_credits: i64,
    pub duration_ms: i64,
    pub request_ids: Vec<String>,
    pub cache_hits: i64,
    pub drill: bool,
    pub cloud_equivalent_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub input_text: String,
    pub source: String,
    pub status: String,
    pub stage_message: String,
    pub freshness_score: Option<f64>,
    pub claim_count: i64,
    pub changed_count: i64,
    pub supported_count: i64,
    pub unverifiable_count: i64,
    pub stable_count: i64,
    pub corrected_text: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
    pub claims: Vec<Claim>,
    pub proof: Proof,
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckRequest {
    pub text: String,
    pub source: Option<String>,
    #[serde(default)]
    pub drill: bool,
}

#[derive(Debug, Serialize)]
pub struct CheckSummary {
    pub id: String,
    pub preview: String,
    pub source: String,
    pub status: String,
    pub freshness_score: Option<f64>,
    pub claim_count: i64,
    pub changed_count: i64,
    pub created_at: String,
}

impl From<&Check> for CheckSummary {
    fn from(c: &Check) -> Self {
        let preview: String = c.input_text.chars().take(150).collect();
        Self {
            id: c.id.clone(),
            preview,
            source: c.source.clone(),
            status: c.status.clone(),
            freshness_score: c.freshness_score,
            claim_count: c.claim_count,
            changed_count: c.changed_count,
            created_at: c.created_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum CheckEvent {
    Status { status: String, message: String },
    Claims { claims: Vec<Claim> },
    ClaimUpdate { claim: Claim },
    Done { check: Box<Check> },
    Failed { message: String },
}

#[derive(Debug, Deserialize)]
pub struct LlmClaim {
    pub quote: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub volatility: String,
    #[serde(default)]
    pub volatility_reason: String,
    #[serde(default)]
    pub query: String,
    #[serde(default = "default_importance")]
    pub importance: i64,
}

fn default_importance() -> i64 {
    2
}

#[derive(Debug, Deserialize)]
pub struct LlmClaimList {
    pub claims: Vec<LlmClaim>,
}

#[derive(Debug, Deserialize)]
pub struct LlmVerdict {
    pub claim_id: String,
    pub verdict: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub corrected_text: Option<String>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

fn default_confidence() -> f64 {
    0.5
}

#[derive(Debug, Deserialize)]
pub struct LlmVerdictList {
    pub verdicts: Vec<LlmVerdict>,
}

/// Deterministic freshness score. Weighted by how much a reader would rely on a claim.
/// supported/stable = 1.0, unverifiable = 0.5, changed = 0.0.
pub fn freshness_score(claims: &[Claim]) -> Option<f64> {
    let scored: Vec<&Claim> = claims
        .iter()
        .filter(|c| c.verdict != VERDICT_PENDING)
        .collect();
    if scored.is_empty() {
        return None;
    }
    let mut weight_sum = 0.0;
    let mut value_sum = 0.0;
    for c in scored {
        let w = c.importance.clamp(1, 3) as f64;
        let v = match c.verdict.as_str() {
            VERDICT_SUPPORTED | VERDICT_STABLE => 1.0,
            VERDICT_UNVERIFIABLE => 0.5,
            VERDICT_CHANGED => 0.0,
            _ => 0.5,
        };
        weight_sum += w;
        value_sum += w * v;
    }
    if weight_sum == 0.0 {
        return None;
    }
    Some(((value_sum / weight_sum) * 100.0).round())
}

pub fn recompute_counts(check: &mut Check) {
    check.claim_count = check.claims.len() as i64;
    check.changed_count = check
        .claims
        .iter()
        .filter(|c| c.verdict == VERDICT_CHANGED)
        .count() as i64;
    check.supported_count = check
        .claims
        .iter()
        .filter(|c| c.verdict == VERDICT_SUPPORTED)
        .count() as i64;
    check.unverifiable_count = check
        .claims
        .iter()
        .filter(|c| c.verdict == VERDICT_UNVERIFIABLE)
        .count() as i64;
    check.stable_count = check
        .claims
        .iter()
        .filter(|c| c.verdict == VERDICT_STABLE)
        .count() as i64;
    check.freshness_score = freshness_score(&check.claims);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(verdict: &str, importance: i64) -> Claim {
        Claim {
            id: "c".into(),
            idx: 0,
            quote: "q".into(),
            text: "t".into(),
            volatility: "high".into(),
            volatility_reason: String::new(),
            query: String::new(),
            verdict: verdict.into(),
            confidence: 1.0,
            explanation: String::new(),
            corrected_text: None,
            importance,
            start: 0,
            end: 1,
            evidence: vec![],
        }
    }

    #[test]
    fn score_weights_important_changed_claims_harder() {
        let claims = vec![claim(VERDICT_CHANGED, 3), claim(VERDICT_SUPPORTED, 1)];
        // (3*0 + 1*1) / 4 = 25
        assert_eq!(freshness_score(&claims), Some(25.0));
    }

    #[test]
    fn unverifiable_is_half_credit() {
        let claims = vec![claim(VERDICT_UNVERIFIABLE, 2), claim(VERDICT_SUPPORTED, 2)];
        assert_eq!(freshness_score(&claims), Some(75.0));
    }

    #[test]
    fn stable_counts_as_supported() {
        let claims = vec![claim(VERDICT_STABLE, 3)];
        assert_eq!(freshness_score(&claims), Some(100.0));
    }

    #[test]
    fn pending_claims_are_excluded() {
        let claims = vec![claim(VERDICT_PENDING, 3), claim(VERDICT_SUPPORTED, 1)];
        assert_eq!(freshness_score(&claims), Some(100.0));
    }
}
