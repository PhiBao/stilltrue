use crate::models::{Check, CheckSummary, Claim, Proof};
use anyhow::Context;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;

pub async fn connect(database_url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(database_url)
        .await
        .context("connect sqlite")?;
    migrate(&pool).await?;
    Ok(pool)
}

async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS checks (
            id TEXT PRIMARY KEY,
            input_text TEXT NOT NULL,
            source TEXT NOT NULL,
            status TEXT NOT NULL,
            stage_message TEXT NOT NULL DEFAULT '',
            freshness_score REAL,
            claim_count INTEGER NOT NULL DEFAULT 0,
            changed_count INTEGER NOT NULL DEFAULT 0,
            supported_count INTEGER NOT NULL DEFAULT 0,
            unverifiable_count INTEGER NOT NULL DEFAULT 0,
            stable_count INTEGER NOT NULL DEFAULT 0,
            corrected_text TEXT,
            created_at TEXT NOT NULL,
            completed_at TEXT,
            error TEXT,
            claims_json TEXT NOT NULL DEFAULT '[]',
            proof_json TEXT NOT NULL DEFAULT '{}'
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_checks_created ON checks(created_at DESC);")
        .execute(pool)
        .await?;

    // Additive columns for newer builds; idempotent via PRAGMA check.
    let cols: Vec<String> = sqlx::query("PRAGMA table_info(checks)")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|r| {
            use sqlx::Row;
            r.try_get::<String, _>("name").unwrap_or_default()
        })
        .collect();
    if !cols.iter().any(|c| c == "recheck_of") {
        sqlx::query("ALTER TABLE checks ADD COLUMN recheck_of TEXT")
            .execute(pool)
            .await?;
    }
    if !cols.iter().any(|c| c == "source_url") {
        sqlx::query("ALTER TABLE checks ADD COLUMN source_url TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn save_check(pool: &SqlitePool, check: &Check) -> anyhow::Result<()> {
    let claims_json = serde_json::to_string(&check.claims)?;
    let proof_json = serde_json::to_string(&check.proof)?;
    sqlx::query(
        r#"
        INSERT INTO checks (
            id, input_text, source, status, stage_message, freshness_score,
            claim_count, changed_count, supported_count, unverifiable_count, stable_count,
            corrected_text, created_at, completed_at, error, claims_json, proof_json,
            recheck_of, source_url
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            status = excluded.status,
            stage_message = excluded.stage_message,
            freshness_score = excluded.freshness_score,
            claim_count = excluded.claim_count,
            changed_count = excluded.changed_count,
            supported_count = excluded.supported_count,
            unverifiable_count = excluded.unverifiable_count,
            stable_count = excluded.stable_count,
            corrected_text = excluded.corrected_text,
            completed_at = excluded.completed_at,
            error = excluded.error,
            claims_json = excluded.claims_json,
            proof_json = excluded.proof_json,
            recheck_of = excluded.recheck_of,
            source_url = excluded.source_url
        "#,
    )
    .bind(&check.id)
    .bind(&check.input_text)
    .bind(&check.source)
    .bind(&check.status)
    .bind(&check.stage_message)
    .bind(check.freshness_score)
    .bind(check.claim_count)
    .bind(check.changed_count)
    .bind(check.supported_count)
    .bind(check.unverifiable_count)
    .bind(check.stable_count)
    .bind(&check.corrected_text)
    .bind(&check.created_at)
    .bind(&check.completed_at)
    .bind(&check.error)
    .bind(claims_json)
    .bind(proof_json)
    .bind(&check.recheck_of)
    .bind(&check.source_url)
    .execute(pool)
    .await?;
    Ok(())
}

fn row_to_check(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<Check> {
    let claims_json: String = row.try_get("claims_json")?;
    let proof_json: String = row.try_get("proof_json")?;
    let claims: Vec<Claim> = serde_json::from_str(&claims_json).unwrap_or_default();
    let proof: Proof = serde_json::from_str(&proof_json).unwrap_or_default();
    Ok(Check {
        id: row.try_get("id")?,
        input_text: row.try_get("input_text")?,
        source: row.try_get("source")?,
        status: row.try_get("status")?,
        stage_message: row.try_get("stage_message")?,
        freshness_score: row.try_get("freshness_score")?,
        claim_count: row.try_get("claim_count")?,
        changed_count: row.try_get("changed_count")?,
        supported_count: row.try_get("supported_count")?,
        unverifiable_count: row.try_get("unverifiable_count")?,
        stable_count: row.try_get("stable_count")?,
        corrected_text: row.try_get("corrected_text")?,
        created_at: row.try_get("created_at")?,
        completed_at: row.try_get("completed_at")?,
        error: row.try_get("error")?,
        adjudicated_count: crate::models::adjudicated_count(&claims),
        coverage: crate::models::coverage(&claims),
        claims,
        proof,
        recheck_of: row.try_get::<Option<String>, _>("recheck_of").unwrap_or(None),
        source_url: row.try_get::<Option<String>, _>("source_url").unwrap_or(None),
    })
}

pub async fn get_check(pool: &SqlitePool, id: &str) -> anyhow::Result<Option<Check>> {
    let row = sqlx::query("SELECT * FROM checks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    match row {
        Some(row) => Ok(Some(row_to_check(&row)?)),
        None => Ok(None),
    }
}

pub async fn list_checks(pool: &SqlitePool, limit: i64) -> anyhow::Result<Vec<CheckSummary>> {
    let rows = sqlx::query(
        "SELECT id, input_text, source, status, freshness_score, claim_count, changed_count, created_at, recheck_of
         FROM checks ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit.clamp(1, 50))
    .fetch_all(pool)
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let input: String = row.try_get("input_text")?;
        out.push(CheckSummary {
            id: row.try_get("id")?,
            preview: input.chars().take(150).collect(),
            source: row.try_get("source")?,
            status: row.try_get("status")?,
            freshness_score: row.try_get("freshness_score")?,
            claim_count: row.try_get("claim_count")?,
            changed_count: row.try_get("changed_count")?,
            created_at: row.try_get("created_at")?,
            recheck_of: row.try_get::<Option<String>, _>("recheck_of").unwrap_or(None),
        });
    }
    Ok(out)
}
