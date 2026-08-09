use crm_backend::{config::AppConfig, db, routes, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = AppConfig::from_env()?;
    let state = AppState::connect(config.clone()).await?;
    db::run_migrations(&state.pool).await?;

    tracing::info!(port = config.server_port, "starting crm-backend");

    let app = routes::build_router(state);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.server_port)).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
