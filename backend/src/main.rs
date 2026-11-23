mod api;
mod auth;
mod config;
mod state;

use crate::config::Config;
use crate::state::AppState;
use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let db = sqlx::PgPool::connect(&config.database_url).await?;

    let app = Router::new()
        .route("/healthz", get(healthz))
        .nest("/v1/auth", auth::router())
        .with_state(AppState { db });

    let listener = tokio::net::TcpListener::bind(config.bind_addr.as_str()).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
