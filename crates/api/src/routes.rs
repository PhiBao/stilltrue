use crate::db;
use crate::models::*;
use crate::pipeline;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use futures::stream::{self, BoxStream, StreamExt};
use serde_json::json;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

pub fn router(state: Arc<AppState>) -> Router {
    let origin = state.cfg.web_origin.clone();
    let cors = CorsLayer::new()
        .allow_origin(
            origin
                .parse::<axum::http::HeaderValue>()
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("http://localhost:3000")),
        )
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/health", get(health))
        .route("/api/demo", get(demo))
        .route("/api/checks", post(create_check).get(list_checks))
        .route("/api/checks/{id}", get(get_check))
        .route("/api/checks/{id}/events", get(events))
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(json!({"ok": true, "service": "stilltrue-api"}))
}

async fn ready(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let db_ok = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();
    let bifrost_ok = if state.cfg.bifrost_url.is_empty() {
        false
    } else {
        state
            .http
            .get(format!("{}/health", state.cfg.bifrost_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    };
    let tavily_configured = !state.cfg.tavily_api_key.is_empty();
    let ok = db_ok && bifrost_ok && tavily_configured;
    Ok(Json(json!({
        "ok": db_ok && bifrost_ok,
        "ready": ok,
        "db_ok": db_ok,
        "bifrost_ok": bifrost_ok,
        "tavily_configured": tavily_configured,
        "model": state.cfg.bifrost_model,
    })))
}

async fn demo() -> impl IntoResponse {
    Json(pipeline::demo_answer())
}

async fn create_check(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateCheckRequest>,
) -> Result<impl IntoResponse, AppError> {
    let (text, source_url) = if let Some(url) = req.url.clone().filter(|u| !u.trim().is_empty()) {
        let url = url.trim().to_string();
        let parsed = url::Url::parse(&url)
            .map_err(|_| AppError::bad_request("That URL is not valid. Paste an https:// link or paste answer text instead."))?;
        if parsed.scheme() != "https" && parsed.scheme() != "http" {
            return Err(AppError::bad_request("Only http(s) page URLs are supported."));
        }
        match crate::clients::tavily_extract(&state.http, &state.cfg, &url).await {
            Ok(content) => (content, Some(url)),
            Err(e) => {
                tracing::warn!(error = %e, "tavily extract failed");
                return Err(AppError::bad_request("Could not extract that page. It may be blocked or requires login. Paste the answer text instead."));
            }
        }
    } else {
        (req.text.trim().to_string(), None)
    };
    if text.chars().count() < 40 {
        return Err(AppError::bad_request(
            "Paste at least 40 characters, or paste one https:// page URL.",
        ));
    }
    if text.chars().count() > 12_000 {
        return Err(AppError::bad_request(
            "Answers are limited to 12,000 characters.",
        ));
    }
    let recheck_of = req.recheck_of.clone().filter(|v| !v.trim().is_empty());
    if let Some(parent_id) = recheck_of.as_ref() {
        if db::get_check(&state.db, parent_id).await?.is_none() {
            return Err(AppError::bad_request("The check to re-run was not found."));
        }
    }

    let client_key = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "local".to_string());

    if !state.allow_check(&client_key).await {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit reached. Try again in a little while.",
        ));
    }

    let id = Uuid::new_v4().to_string();
    let mut check = Check {
        id: id.clone(),
        input_text: text,
        source: req.source.unwrap_or_else(|| "pasted".to_string()),
        status: "queued".into(),
        stage_message: "Queued".into(),
        freshness_score: None,
        claim_count: 0,
        changed_count: 0,
        supported_count: 0,
        unverifiable_count: 0,
        stable_count: 0,
        corrected_text: None,
        created_at: Utc::now().to_rfc3339(),
        completed_at: None,
        error: None,
        claims: vec![],
        adjudicated_count: 0,
        coverage: None,
        recheck_of,
        source_url,
        proof: Proof {
            drill: req.drill,
            ..Default::default()
        },
    };
    db::save_check(&state.db, &check).await?;
    state.register_check(&id).await;

    let state_bg = state.clone();
    let check_bg = check.clone();
    tokio::spawn(async move {
        let _ = pipeline::run_check(state_bg, check_bg).await;
    });

    check.status = "queued".into();
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id": id, "status": "queued"})),
    ))
}

async fn list_checks(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let checks = db::list_checks(&state.db, 24).await?;
    Ok(Json(json!({"checks": checks})))
}

async fn get_check(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let check = db::get_check(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::not_found("Check not found"))?;
    Ok(Json(check))
}

async fn events(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, AppError> {
    let check = db::get_check(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::not_found("Check not found"))?;

    let terminal = check.status == "done" || check.status == "failed";
    let stream: BoxStream<'static, Result<Event, Infallible>> = if terminal {
        let event = if check.status == "done" {
            CheckEvent::Done {
                check: Box::new(check),
            }
        } else {
            CheckEvent::Failed {
                message: check
                    .error
                    .clone()
                    .unwrap_or_else(|| "Verification failed".into()),
            }
        };
        Box::pin(stream::once(async move { Ok(to_sse(event)) }))
    } else {
        let mut snapshot = vec![CheckEvent::Status {
            status: check.status.clone(),
            message: check.stage_message.clone(),
        }];
        if !check.claims.is_empty() {
            snapshot.push(CheckEvent::Claims {
                claims: check.claims.clone(),
            });
        }
        let initial = stream::iter(
            snapshot
                .into_iter()
                .map(|e| Ok::<Event, Infallible>(to_sse(e)))
                .collect::<Vec<_>>(),
        );

        match state.subscribe(&id).await {
            Some(rx) => {
                let live = BroadcastStream::new(rx).filter_map(|item| async move {
                    item.ok().map(|e| Ok::<Event, Infallible>(to_sse(e)))
                });
                Box::pin(initial.chain(live))
            }
            None => Box::pin(initial.chain(stream::once(async move {
                Ok(to_sse(CheckEvent::Failed {
                    message: "This check is no longer live. Reload to see its saved result.".into(),
                }))
            }))),
        }
    };

    Ok(Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(std::time::Duration::from_secs(12))
                .text("keep-alive"),
        )
        .into_response())
}

fn to_sse(event: CheckEvent) -> Event {
    let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
    Event::default().data(data)
}

pub struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }
    fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(error = %err, "request failed");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong on our side.",
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(json!({"error": self.message}))).into_response()
    }
}
