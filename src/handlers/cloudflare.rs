use axum::{extract::State, Json, response::IntoResponse, http::{HeaderMap, StatusCode}};
use crate::models::{CloudflarePayload, Notification};
use crate::services;
use crate::config::AppState;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CloudflarePayload>,
) -> impl IntoResponse {
    if let Some(ref expected_secret) = state.config.auth.cloudflare_secret {
        let secret = headers.get("X-Webhook-Secret")
            .and_then(|h| h.to_str().ok());

        if secret != Some(expected_secret) {
            warn!("Unauthorized Cloudflare webhook attempt");
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    info!("Received Cloudflare webhook: {}", payload.policy_name.as_deref().unwrap_or("unknown"));

    let notification: Notification = payload.into();

    if let Some(route) = state.config.routes.get("cloudflare") {
        services::dispatch(&state.http_client, &state.config.targets, route, notification).await;
    }

    StatusCode::OK.into_response()
}
