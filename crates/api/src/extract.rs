use crate::models::{Claim, VERDICT_PENDING};
use once_cell::sync::Lazy;
use regex::Regex;

static MULTI_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

/// Split prose into sentence spans while preserving byte offsets into the original.
/// Deliberately conservative: a boundary needs terminal punctuation followed by
/// whitespace and an uppercase letter, a quote, or end-of-text.
fn is_abbrev_word(text: &str, dot: usize) -> bool {
    // Last 1-4 chars before the dot are letters (Mr. Dr. U.S. No. vs. etc.)
    let start = dot.saturating_sub(4);
    let frag = &text[start..dot];
    let letters: String = frag.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    matches!(
        letters.to_lowercase().as_str(),
        "mr" | "mrs" | "dr" | "us" | "uk" | "no" | "st" | "sr" | "jr" | "vs" | "etc" | "fig" | "dept" | "est"
    )
}

fn is_decimal_dot(text: &str, dot: usize) -> bool {
    let bytes = text.as_bytes();
    dot > 0 && dot + 1 < bytes.len() && bytes[dot - 1].is_ascii_digit() && bytes[dot + 1].is_ascii_digit()
}

fn is_list_or_table_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("- ") || t.starts_with("* ") || t.starts_with("• ") || t.starts_with("| ")
        || (t.len() > 2 && t.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && (t[1..].starts_with(". ") || t[1..].starts_with(") ")))
}

pub fn sentence_spans(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        // Markdown bullets and table rows are atomic spans (one claim per row).
        if i == start {
            let line_end = text[i..].find('\n').map(|p| i + p).unwrap_or(text.len());
            let line = &text[i..line_end];
            if is_list_or_table_line(line) && line.trim().len() > 14 {
                spans.push((start, start + line.trim_end().len()));
                start += line.trim_end().len();
                let mut j = start;
                while j < bytes.len() && (bytes[j] == b'\n' || bytes[j] == b'\r' || bytes[j] == b' ') {
                    j += 1;
                }
                start = j;
                i = j;
                continue;
            }
        }
        if bytes[i] == b'\n' {
            let line = &text[start..i];
            let is_paragraph = i + 1 < bytes.len() && bytes[i + 1] == b'\n';
            if is_paragraph || line.trim_end().ends_with(':') {
                if i > start {
                    spans.push((start, i));
                }
                let mut j = i + 1;
                while j < bytes.len()
                    && (bytes[j] == b' '
                        || bytes[j] == b'\n'
                        || bytes[j] == b'\t'
                        || bytes[j] == b'\r')
                {
                    j += 1;
                }
                start = j;
                i = j;
                continue;
            }
        }
        if bytes[i] == b'.' && (is_decimal_dot(text, i) || is_abbrev_word(text, i)) {
            i += 1;
            continue;
        }
        if bytes[i] == b'.' || bytes[i] == b'!' || bytes[i] == b'?' {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\n' || bytes[j] == b'\t') {
                j += 1;
            }
            let next_starts_sentence = j >= bytes.len()
                || bytes[j].is_ascii_uppercase()
                || bytes[j] == b'"'
                || bytes[j] == b'\''
                || bytes[j] == b'#';
            if next_starts_sentence {
                let end = i + 1;
                if end > start {
                    spans.push((start, end));
                }
                start = j;
                i = j;
                continue;
            }
        }
        i += 1;
    }
    if start < bytes.len() {
        spans.push((start, bytes.len()));
    }
    // Trim leading/trailing whitespace from each span.
    spans
        .into_iter()
        .filter_map(|(s, e)| {
            let slice = &text[s..e];
            let leading = slice.len() - slice.trim_start().len();
            let trailing = slice.len() - slice.trim_end().len();
            let ns = s + leading;
            let ne = e.saturating_sub(trailing);
            if ne > ns {
                Some((ns, ne))
            } else {
                None
            }
        })
        .collect()
}

fn has_digit(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit())
}

fn has_money(s: &str) -> bool {
    s.contains('$') || s.contains('€') || s.contains('£') || s.contains('%')
}

fn has_year(s: &str) -> bool {
    let b = s.as_bytes();
    b.windows(4)
        .any(|w| w.iter().all(|c| c.is_ascii_digit()) && (w[0] == b'1' || w[0] == b'2'))
}

fn has_version(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.contains("version")
        || lower.contains(" v1")
        || lower.contains(" v2")
        || lower.contains("api ")
        || lower.contains("model")
}

fn factual_verb(s: &str) -> bool {
    let lower = format!(" {} ", s.to_lowercase());
    [
        " is ",
        " are ",
        " was ",
        " were ",
        " costs ",
        " cost ",
        " offers ",
        " includes ",
        " supports ",
        " requires ",
        " charges ",
        " released ",
        " founded ",
        " headquartered ",
        " recommended ",
        " available ",
        " allows ",
        " provides ",
        " uses ",
        " runs ",
        " limits ",
        " became ",
        " launched ",
        " acquired ",
        " deprecated ",
        " replaced ",
        " serves as ",
        " leads ",
        " heads ",
        " appointed ",
        " promoted ",
        " joined ",
        " chairs ",
        " chief ",
        " ceo ",
        " director ",
        " serves ",
        " was named ",
    ]
    .iter()
    .any(|v| lower.contains(v))
}

fn instruction(s: &str) -> bool {
    let lower = s.to_lowercase();
    [
        "you can ",
        "you should ",
        "try ",
        "click ",
        "use the ",
        "see the ",
        "read the ",
        "contact ",
        "learn more",
        "for example",
        "note:",
        "if you ",
        "to get ",
        "to use ",
        "please ",
        "remember to ",
        "make sure",
    ]
    .iter()
    .any(|p| lower.starts_with(p))
}

fn question(s: &str) -> bool {
    s.trim_end().ends_with('?')
}

fn proper_nouns(s: &str) -> usize {
    let mut count = 0;
    let mut prev_cap = false;
    for word in s.split_whitespace() {
        let first = word.chars().next().unwrap_or(' ');
        let cap = first.is_uppercase();
        if cap && prev_cap {
            count += 1;
        }
        prev_cap = cap;
    }
    count
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Volatility {
    High,
    Medium,
    Low,
}

impl Volatility {
    fn as_str(self) -> &'static str {
        match self {
            Volatility::High => "high",
            Volatility::Medium => "medium",
            Volatility::Low => "low",
        }
    }
    fn rank(self) -> i64 {
        match self {
            Volatility::High => 3,
            Volatility::Medium => 2,
            Volatility::Low => 1,
        }
    }
    fn reason(self) -> &'static str {
        match self {
            Volatility::High => "prices, versions, and availability change often",
            Volatility::Medium => "product packaging changes during the year",
            Volatility::Low => "historical fact with a fixed date",
        }
    }
}

fn classify(s: &str) -> Volatility {
    let lower = s.to_lowercase();
    if [
        "founded",
        "headquartered",
        "released in",
        "launched in",
        "was born",
    ]
    .iter()
    .any(|m| lower.contains(m))
    {
        return Volatility::Low;
    }
    if has_money(s)
        || has_version(s)
        || [
            "current",
            "latest",
            "recommended",
            "available",
            "limit",
            "as of",
            "now ",
        ]
        .iter()
        .any(|m| lower.contains(m))
        || (has_year(s) && !lower.contains("released"))
    {
        return Volatility::High;
    }
    if [
        "ceo", "chief ", "director", "leads ", "heads ", "appointed", "chair", "founder of",
    ]
    .iter()
    .any(|m| lower.contains(m))
    {
        return Volatility::High;
    }
    if [
        "includes", "offers", "requires", "policy", "plan", "tier", "allows", "provides",
        "supports",
    ]
    .iter()
    .any(|m| lower.contains(m))
    {
        return Volatility::Medium;
    }
    Volatility::Medium
}

fn claiminess(s: &str) -> i64 {
    if question(s) || instruction(s) || s.trim_end().ends_with(':') || s.chars().count() < 18 {
        return -10;
    }
    let mut score = 0;
    if has_digit(s) {
        score += 2;
    }
    if has_money(s) {
        score += 2;
    }
    if has_year(s) {
        score += 1;
    }
    if factual_verb(s) {
        score += 2;
    } else if proper_nouns(s) >= 2 && s.chars().count() > 30 {
        score += 1;
    } else {
        score -= 3;
    }
    if proper_nouns(s) >= 1 {
        score += 1;
    }
    if s.chars().count() > 45 {
        score += 1;
    }
    if s.contains("recommended") || s.contains("deprecated") || s.contains("no longer") {
        score += 2;
    }
    if s.contains("ceo") || s.contains("chief") || s.contains("appointed") {
        score += 1;
    }
    score
}

fn clean_query(sentence: &str) -> String {
    let mut q = sentence
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .replace(['"', '\u{201c}', '\u{201d}'], "")
        .replace('$', "");
    q = MULTI_SPACE.replace_all(&q, " ").to_string();
    if q.chars().count() > 170 {
        q = q.chars().take(170).collect();
    }
    if !has_year(&q) && !q.to_lowercase().contains("2026") {
        q.push_str(" 2026");
    }
    q
}

/// Deterministic claim segmentation. It is intentionally conservative and auditable:
/// the judge can inspect exactly which sentences were selected and why. The LLM is
/// reserved for the part that needs judgement — weighing live evidence.
pub fn extract_claims(text: &str, max_claims: usize) -> Vec<Claim> {
    let spans = sentence_spans(text);
    let mut candidates: Vec<(i64, i64, usize, usize, String, Volatility)> = Vec::new();
    for (start, end) in spans {
        let quote = &text[start..end];
        let score = claiminess(quote);
        if score < 3 {
            continue;
        }
        let vol = classify(quote);
        let importance = vol.rank();
        candidates.push((importance, vol.rank(), start, end, quote.to_string(), vol));
    }

    // Prefer decisions that matter: high-volatility, high-importance claims first.
    candidates.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
    candidates.truncate(max_claims);
    candidates.sort_by_key(|c| c.2);

    candidates
        .into_iter()
        .enumerate()
        .map(|(i, (importance, _, start, end, quote, vol))| {
            let query = clean_query(&quote);
            Claim {
                id: format!("c{}", i + 1),
                idx: i as i64,
                quote: quote.clone(),
                text: quote,
                volatility: vol.as_str().to_string(),
                volatility_reason: vol.reason().to_string(),
                query,
                verdict: VERDICT_PENDING.into(),
                confidence: 0.0,
                explanation: String::new(),
                corrected_text: None,
                importance,
                start: start as i64,
                end: end as i64,
                evidence: vec![],
                cited_evidence_ids: vec![],
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEMO: &str = "OpenAI API pricing and platform notes (support macro, 2024):\n\nGPT-4o costs $5 per million input tokens and $15 per million output tokens. GPT-4o mini costs $0.15 per million input tokens and $0.60 per million output tokens. The Batch API offers a 50% discount on both models. GPT-4o was released in May 2024. The Assistants API is the recommended way to build agents on the OpenAI platform. GPT-4 Turbo costs $10 per million input tokens and $30 per million output tokens.";

    #[test]
    fn extracts_the_demo_claims() {
        let claims = extract_claims(DEMO, 6);
        assert!(
            claims.len() >= 5,
            "got {} claims: {:?}",
            claims.len(),
            claims
        );
        let quotes: Vec<&str> = claims.iter().map(|c| c.quote.as_str()).collect();
        assert!(quotes.iter().any(|q| q.contains("GPT-4o costs $5")));
        assert!(quotes.iter().any(|q| q.contains("Assistants API")));
        assert!(quotes.iter().any(|q| q.contains("GPT-4 Turbo")));
    }

    #[test]
    fn header_is_not_merged_into_first_claim() {
        let claims = extract_claims(DEMO, 6);
        assert!(!claims[0].quote.contains("platform notes"));
        assert!(claims[0].quote.starts_with("GPT-4o costs"));
    }

    #[test]
    fn quotes_are_exact_slices() {
        for c in extract_claims(DEMO, 6) {
            assert_eq!(&DEMO[c.start as usize..c.end as usize], c.quote);
        }
    }

    #[test]
    fn skips_instructions_and_questions() {
        let text =
            "You can click the button to continue. What is the price? The API costs $5 per call.";
        let claims = extract_claims(text, 5);
        assert_eq!(claims.len(), 1);
        assert!(claims[0].quote.contains("costs $5"));
    }

    #[test]
    fn classifies_historical_facts_as_low() {
        let text = "Acme was founded in 2010 by Jane Doe. Acme charges $20 per seat.";
        let claims = extract_claims(text, 5);
        let founded = claims.iter().find(|c| c.quote.contains("founded")).unwrap();
        assert_eq!(founded.volatility, "low");
    }

    #[test]
    fn extracts_ceo_claim_without_digits() {
        let text = "Sarah Chen is the chief executive officer of Northwind. Northwind charges $40 per seat per month.";
        let claims = extract_claims(text, 5);
        assert!(claims.iter().any(|c| c.quote.contains("Sarah Chen")), "got {:?}", claims.iter().map(|c| c.quote.clone()).collect::<Vec<_>>());
    }

    #[test]
    fn splits_bullets_and_tables() {
        let text = "- Starter costs $20 per seat.\n- Pro costs $60 per seat.\n| Enterprise | $150 per seat |";
        let claims = extract_claims(text, 6);
        assert!(claims.len() >= 2, "got {:?}", claims.iter().map(|c| c.quote.clone()).collect::<Vec<_>>());
    }

    #[test]
    fn decimal_price_is_not_split() {
        let text = "The plan costs $2.50 per user. It includes storage.";
        let claims = extract_claims(text, 4);
        assert!(claims.iter().any(|c| c.quote.contains("2.50")), "got {:?}", claims.iter().map(|c| c.quote.clone()).collect::<Vec<_>>());
    }

    #[test]
    fn query_gets_current_year() {
        let claims = extract_claims("Acme charges $20 per seat per month.", 3);
        assert!(claims[0].query.contains("2026"));
    }
}
