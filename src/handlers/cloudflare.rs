use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use crate::config::AppState;
use crate::models::{CloudflarePayload, Notification};
use crate::services;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CloudflarePayload>,
) -> impl IntoResponse {
    let source_id = crate::config::AppConfig::source_id("cloudflare", &name);
    let Some(source) = state.config.sources.cloudflare.get(&name) else {
        warn!("Cloudflare source '{}' is not configured", name);
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Some(ref expected) = source.secret {
        let secret = headers.get("X-Webhook-Secret").and_then(|h| h.to_str().ok());

        if secret != Some(expected.as_str()) {
            warn!("Unauthorized Cloudflare webhook for source '{}'", name);
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    info!(
        "Received Cloudflare webhook ({}): {}",
        source_id,
        payload.policy_name.as_deref().unwrap_or("unknown")
    );

    let notification: Notification = payload.into();
    let destinations = state.config.destinations_for(&source_id);
    if !destinations.is_empty() {
        services::dispatch(
            &state.http_client,
            &state.config.targets,
            &destinations,
            notification,
        )
        .await;
    }

    StatusCode::OK.into_response()
}
