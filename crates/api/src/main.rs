mod clients;
mod config;
mod db;
mod extract;
mod models;
mod pipeline;
mod routes;
mod state;

use config::Config;
use state::AppState;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("stilltrue_api=info,tower_http=info")),
        )
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    let cfg = Config::from_env()?;
    if cfg.tavily_api_key.is_empty() {
        tracing::warn!("TAVILY_API_KEY is empty — live verification will fail");
    }

    if cfg.database_url.starts_with("sqlite://./") {
        std::fs::create_dir_all("./data").ok();
    }

    let pool = db::connect(&cfg.database_url).await?;
    let state = AppState::new(cfg.clone(), pool);
    let app = routes::router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "StillTrue API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
