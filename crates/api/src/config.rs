use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub bifrost_url: String,
    pub bifrost_vk: String,
    pub bifrost_model: String,
    pub bifrost_fallback: Option<String>,
    pub drill_model: String,
    pub tavily_api_key: String,
    pub max_claims: usize,
    pub tavily_max_results: usize,
    pub extraction_mode: String,
    pub web_origin: String,
    pub rate_limit_per_hour: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let bifrost_url = env::var("BIFROST_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string())
            .trim_end_matches('/')
            .to_string();
        let tavily_api_key = env::var("TAVILY_API_KEY").unwrap_or_default();
        let cfg = Self {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8787),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://./data/stilltrue.db?mode=rwc".to_string()),
            bifrost_url,
            bifrost_vk: env::var("BIFROST_VK")
                .unwrap_or_else(|_| "sk-bf-stilltrue-dev".to_string()),
            bifrost_model: env::var("BIFROST_MODEL")
                .unwrap_or_else(|_| "pollinations/openai-fast".to_string()),
            bifrost_fallback: env::var("BIFROST_FALLBACK").ok().filter(|v| !v.is_empty()),
            drill_model: env::var("BIFROST_DRILL_MODEL")
                .unwrap_or_else(|_| "outage-lab/chat".to_string()),
            tavily_api_key,
            max_claims: env::var("MAX_CLAIMS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(6)
                .clamp(1, 10),
            tavily_max_results: env::var("TAVILY_MAX_RESULTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4)
                .clamp(1, 8),
            extraction_mode: env::var("EXTRACTION_MODE")
                .unwrap_or_else(|_| "hybrid".to_string()),
            rate_limit_per_hour: env::var("RATE_LIMIT_PER_HOUR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200)
                .clamp(10, 5000),
            web_origin: env::var("WEB_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        };
        Ok(cfg)
    }
}
