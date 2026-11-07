use crate::config::Config;
use crate::models::CheckEvent;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, Mutex};

pub struct AppState {
    pub cfg: Config,
    pub db: SqlitePool,
    pub http: reqwest::Client,
    pub events: Arc<Mutex<HashMap<String, broadcast::Sender<CheckEvent>>>>,
    /// Very small in-memory abuse guard: timestamps of recent checks per client key.
    pub rate: Mutex<HashMap<String, Vec<Instant>>>,
}

impl AppState {
    pub fn new(cfg: Config, db: SqlitePool) -> Arc<Self> {
        let http = reqwest::Client::builder()
            .user_agent("StillTrue/0.1 (+https://stilltrue.app)")
            .build()
            .expect("build http client");
        Arc::new(Self {
            cfg,
            db,
            http,
            events: Arc::new(Mutex::new(HashMap::new())),
            rate: Mutex::new(HashMap::new()),
        })
    }

    pub async fn register_check(&self, id: &str) {
        let (tx, _) = broadcast::channel(128);
        self.events.lock().await.insert(id.to_string(), tx);
    }

    pub async fn emit(&self, id: &str, event: CheckEvent) {
        if let Some(tx) = self.events.lock().await.get(id) {
            let _ = tx.send(event);
        }
    }

    pub async fn finish_check(&self, id: &str) {
        // Keep the channel briefly so a late SSE client can still replay, then drop it.
        let id_owned = id.to_string();
        let events = self.events.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(120)).await;
            events.lock().await.remove(&id_owned);
        });
    }

    pub async fn subscribe(&self, id: &str) -> Option<broadcast::Receiver<CheckEvent>> {
        self.events.lock().await.get(id).map(|tx| tx.subscribe())
    }

    /// Returns true if the caller is within budget.
    pub async fn allow_check(&self, key: &str) -> bool {
        let mut map = self.rate.lock().await;
        let now = Instant::now();
        let entry = map.entry(key.to_string()).or_default();
        entry.retain(|t| now.duration_since(*t) < std::time::Duration::from_secs(3600));
        if entry.len() >= self.cfg.rate_limit_per_hour {
            return false;
        }
        entry.push(now);
        if map.len() > 5_000 {
            map.retain(|_, v| {
                v.iter()
                    .any(|t| now.duration_since(*t) < std::time::Duration::from_secs(3600))
            });
        }
        true
    }
}
