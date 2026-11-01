use crate::config::Config;
use crate::models::Evidence;
use anyhow::{anyhow, Context};
use chrono::Utc;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct LlmMeta {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub requested_model: Option<String>,
    pub fallback_used: bool,
    pub fallback_index: Option<i64>,
    pub primary_provider: Option<String>,
    pub primary_model: Option<String>,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub request_id: Option<String>,
    pub cache_hit: bool,
    pub drill: bool,
}

pub struct LlmResponse {
    pub value: Value,
    pub meta: LlmMeta,
    pub raw: String,
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub async fn chat_json(
    http: &reqwest::Client,
    cfg: &Config,
    system: &str,
    user: &str,
    max_tokens: u32,
    drill: bool,
) -> anyhow::Result<LlmResponse> {
    chat_json_model(http, cfg, system, user, max_tokens, drill, None).await
}

pub async fn chat_json_model(
    http: &reqwest::Client,
    cfg: &Config,
    system: &str,
    user: &str,
    max_tokens: u32,
    drill: bool,
    model_override: Option<&str>,
) -> anyhow::Result<LlmResponse> {
    if cfg.bifrost_url.is_empty() {
        return Err(anyhow!("BIFROST_URL is not configured"));
    }

    let (model, fallbacks) = if drill {
        (
            cfg.drill_model.clone(),
            Some(vec![cfg.bifrost_model.clone()]),
        )
    } else {
        (
            model_override
                .map(|s| s.to_string())
                .unwrap_or_else(|| cfg.bifrost_model.clone()),
            None,
        )
    };

    let mut body = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.1,
        "max_tokens": max_tokens,
        "stream": false
    });
    if let Some(fallbacks) = fallbacks {
        body["fallbacks"] = json!(fallbacks);
    }

    let resp = http
        .post(format!("{}/v1/chat/completions", cfg.bifrost_url))
        .header("x-bf-vk", &cfg.bifrost_vk)
        .header("x-bf-cache-key", "stilltrue-pipeline-v1")
        .json(&body)
        .timeout(Duration::from_secs(180))
        .send()
        .await
        .context("call Bifrost gateway")?;

    let status = resp.status();
    let headers = resp.headers().clone();
    let text = resp.text().await.context("read Bifrost body")?;

    let meta = LlmMeta {
        provider: header_str(&headers, "x-bifrost-provider"),
        model: header_str(&headers, "x-bifrost-resolved-model"),
        requested_model: header_str(&headers, "x-bifrost-original-model"),
        fallback_used: header_str(&headers, "x-bifrost-routing-info-is-fallback")
            .map(|v| v == "true")
            .unwrap_or(false),
        fallback_index: header_str(&headers, "x-bifrost-fallback-index")
            .and_then(|v| v.parse().ok()),
        primary_provider: header_str(&headers, "x-bifrost-routing-info-primary-provider"),
        primary_model: header_str(&headers, "x-bifrost-routing-info-primary-model"),
        prompt_tokens: 0,
        completion_tokens: 0,
        request_id: header_str(&headers, "x-bifrost-request-id")
            .or_else(|| header_str(&headers, "x-request-id")),
        cache_hit: header_str(&headers, "x-bifrost-cache-hit")
            .map(|v| v == "true")
            .unwrap_or(false),
        drill,
    };

    if !status.is_success() {
        let detail = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| text.chars().take(300).collect());
        tracing::warn!(
            status = status.as_u16(),
            body = %text.chars().take(1500).collect::<String>(),
            "Bifrost upstream error"
        );
        return Err(anyhow!("Bifrost returned {}: {}", status.as_u16(), detail));
    }

    let parsed: Value = serde_json::from_str(&text).context("parse Bifrost envelope")?;
    let content = parsed
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Bifrost response has no message content"))?;

    let mut meta = meta;
    meta.prompt_tokens = parsed
        .pointer("/usage/prompt_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    meta.completion_tokens = parsed
        .pointer("/usage/completion_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let value = parse_json_relaxed(content).ok_or_else(|| {
        anyhow!(
            "model did not return valid JSON: {}",
            truncate(content, 240)
        )
    })?;

    Ok(LlmResponse {
        value,
        meta,
        raw: content.to_string(),
    })
}

/// Models sometimes wrap JSON in prose or markdown fences. Find the first complete
/// JSON object/array without being fooled by braces inside strings.
pub fn parse_json_relaxed(input: &str) -> Option<Value> {
    let trimmed = input.trim();
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        return Some(v);
    }
    let start = trimmed
        .char_indices()
        .find(|(_, c)| *c == '{' || *c == '[')
        .map(|(i, _)| i)?;
    let bytes = trimmed.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in trimmed.char_indices().skip_while(|(i, _)| *i < start) {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth -= 1;
                if depth == 0 {
                    let candidate = &trimmed[start..=i];
                    return serde_json::from_str::<Value>(candidate).ok();
                }
            }
            _ => {}
        }
    }
    None
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[derive(Debug, Clone)]
pub struct TavilyMeta {
    pub searches: i64,
    pub credits: i64,
}

pub async fn tavily_search(
    http: &reqwest::Client,
    cfg: &Config,
    query: &str,
    claim_id: &str,
) -> anyhow::Result<(Vec<Evidence>, TavilyMeta)> {
    if cfg.tavily_api_key.is_empty() {
        return Err(anyhow!("TAVILY_API_KEY is not configured"));
    }
    let body = json!({
        "query": query,
        "search_depth": "basic",
        "max_results": cfg.tavily_max_results,
        "include_answer": false,
        "include_raw_content": false,
        "topic": "general"
    });
    let resp = http
        .post("https://api.tavily.com/search")
        .bearer_auth(&cfg.tavily_api_key)
        .json(&body)
        .timeout(Duration::from_secs(35))
        .send()
        .await
        .context("call Tavily search")?;
    let status = resp.status();
    let text = resp.text().await.context("read Tavily body")?;
    if !status.is_success() {
        return Err(anyhow!(
            "Tavily returned {}: {}",
            status.as_u16(),
            truncate(&text, 240)
        ));
    }
    let parsed: Value = serde_json::from_str(&text).context("parse Tavily envelope")?;
    let retrieved_at = Utc::now().to_rfc3339();
    let mut out = Vec::new();
    if let Some(results) = parsed.get("results").and_then(|v| v.as_array()) {
        for (i, r) in results.iter().enumerate() {
            let url = r.get("url").and_then(|v| v.as_str()).unwrap_or("");
            if url.is_empty() {
                continue;
            }
            let title = r
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled source");
            let content = r.get("content").and_then(|v| v.as_str()).unwrap_or("");
            if content.trim().is_empty() {
                continue;
            }
            out.push(Evidence {
                id: format!("{}-E{}", claim_id, i + 1),
                title: title.to_string(),
                url: url.to_string(),
                quote: truncate(content.trim(), 700),
                published_date: r
                    .get("published_date")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                retrieved_at: retrieved_at.clone(),
                score: r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
            });
        }
    }
    Ok((
        out,
        TavilyMeta {
            searches: 1,
            credits: 1,
        },
    ))
}
