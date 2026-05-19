use axum::{extract::State, Json, response::IntoResponse, http::{HeaderMap, StatusCode}};
use crate::models::{ArgoCDPayload, Notification};
use crate::services;
use crate::config::AppState;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ArgoCDPayload>,
) -> impl IntoResponse {
    if let Some(ref expected_token) = state.config.auth.argocd_token {
        let token = headers.get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(str::trim);

        if token != Some(expected_token) {
            warn!("Unauthorized ArgoCD webhook attempt");
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    info!("Received ArgoCD webhook: {:?}", payload.app_name);

    let notification: Notification = payload.into();

    if let Some(route) = state.config.routes.get("argocd") {
        services::dispatch(&state.http_client, &state.config.targets, route, notification).await;
    }

    StatusCode::OK.into_response()
}
