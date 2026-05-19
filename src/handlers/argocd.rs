use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use crate::config::AppState;
use crate::models::{ArgoCDPayload, Notification};
use crate::services;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ArgoCDPayload>,
) -> impl IntoResponse {
    let source_id = crate::config::AppConfig::source_id("argocd", &name);
    let Some(source) = state.config.sources.argocd.get(&name) else {
        warn!("ArgoCD source '{}' is not configured", name);
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Some(ref expected) = source.token {
        let token = headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(str::trim);

        if token != Some(expected.as_str()) {
            warn!("Unauthorized ArgoCD webhook for source '{}'", name);
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    info!(
        "Received ArgoCD webhook ({}): {:?}",
        source_id, payload.app_name
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
