use axum::{extract::State, Json, response::IntoResponse, http::{HeaderMap, StatusCode}};
use crate::models::{AlertmanagerPayload, Notification};
use crate::services;
use crate::config::AppState;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<AlertmanagerPayload>,
) -> impl IntoResponse {
    if let Some(ref expected_token) = state.config.auth.alertmanager_token {
        let token_str: Option<&str> = headers.get("X-Token")
            .and_then(|h| h.to_str().ok())
            .or_else(|| {
                headers.get("Authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.strip_prefix("Bearer "))
                    .map(str::trim)
            });

        if token_str != Some(expected_token) {
            warn!("Unauthorized Alertmanager webhook attempt");
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    info!("Received Alertmanager webhook: status={}", payload.status);

    let notification: Notification = payload.into();

    if let Some(route) = state.config.routes.get("alertmanager") {
        services::dispatch(&state.http_client, &state.config.targets, route, notification).await;
    }

    StatusCode::OK.into_response()
}
