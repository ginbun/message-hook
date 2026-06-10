use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use crate::config::AppState;
use crate::models::{AlertmanagerPayload, Notification};
use crate::services;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<AlertmanagerPayload>,
) -> impl IntoResponse {
    let source_id = crate::config::AppConfig::source_id("alertmanager", &name);
    let Some(source) = state.config.sources.alertmanager.get(&name) else {
        warn!("Alertmanager source '{}' is not configured", name);
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Some(ref expected) = source.token {
        let token_str: Option<&str> = headers
            .get("X-Token")
            .and_then(|h| h.to_str().ok())
            .or_else(|| {
                headers
                    .get("Authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.strip_prefix("Bearer "))
                    .map(str::trim)
            });

        if token_str != Some(expected.as_str()) {
            warn!("Unauthorized Alertmanager webhook for source '{}'", name);
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    info!(
        "Received Alertmanager webhook ({}): status={}",
        source_id, payload.status
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
