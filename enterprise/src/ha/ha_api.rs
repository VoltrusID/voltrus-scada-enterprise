use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

use voltrus::api::AppState;

fn requires_enterprise() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"error": "Hot standby requires an Enterprise licence"})),
    )
}

/// GET /api/v1/ha/status
pub async fn ha_status() -> impl IntoResponse {
    let status = crate::rt().ha_state.get_status().await;
    Json(status)
}

/// POST /api/v1/ha/force-sync
pub async fn force_sync(State(state): State<AppState>) -> impl IntoResponse {
    if !state.tier().ha() {
        return requires_enterprise();
    }
    if !crate::rt().ha_state.get_status().await.enabled {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "HA not enabled"})),
        );
    }
    (StatusCode::OK, Json(json!({"status": "sync_requested"})))
}

/// POST /api/v1/ha/force-failover
pub async fn force_failover(State(state): State<AppState>) -> impl IntoResponse {
    if !state.tier().ha() {
        return requires_enterprise();
    }
    if !crate::rt().ha_state.get_status().await.enabled {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "HA not enabled"})),
        );
    }
    (
        StatusCode::OK,
        Json(json!({"status": "failover_initiated"})),
    )
}
