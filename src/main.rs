mod config;
mod handlers;
mod models;
mod services;

use crate::config::{AppConfig, AppState};
use axum::{routing::{get, post}, Router};
use reqwest::Client;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::new().expect("Failed to load configuration");
    let addr = config.server.addr.clone();
    let shared_state = Arc::new(AppState {
        config,
        http_client: Client::new(),
    });

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/webhook/argocd/{name}", post(handlers::argocd::handle))
        .route("/webhook/cloudflare/{name}", post(handlers::cloudflare::handle))
        .route("/webhook/alertmanager/{name}", post(handlers::alertmanager::handle))
        // 限制请求体最大 1 MiB，防止超大 payload 耗尽内存
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .with_state(shared_state);

    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
