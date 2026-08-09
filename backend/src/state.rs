use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub pool: PgPool,
}

impl AppState {
    pub async fn connect(config: AppConfig) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("PostgreSQL connection failed: {e}"))?;
        Ok(Self { config, pool })
    }
}
