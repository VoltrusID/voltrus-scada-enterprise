use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::api::AppState;

/// GET /api/v1/health/detailed
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let (overall, subsystems) = state.health_monitor.get_health().await;
    let uptime = state.health_monitor.uptime_secs();

    let status_code = match overall {
        crate::ha::watchdog::HealthStatus::Healthy => StatusCode::OK,
        crate::ha::watchdog::HealthStatus::Degraded => StatusCode::OK,
        crate::ha::watchdog::HealthStatus::Critical => StatusCode::SERVICE_UNAVAILABLE,
        crate::ha::watchdog::HealthStatus::Stopped => StatusCode::SERVICE_UNAVAILABLE,
    };

    (
        status_code,
        Json(json!({
            "status": serde_json::to_value(overall).unwrap_or_default(),
            "uptime_secs": uptime,
            "subsystems": subsystems,
        })),
    )
}

/// GET /api/v1/ha/status
pub async fn ha_status(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.ha_state.get_status().await;
    Json(status)
}

/// POST /api/v1/ha/force-sync
pub async fn force_sync(State(state): State<AppState>) -> impl IntoResponse {
    if !state.ha_state.get_status().await.enabled {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "HA not enabled"})),
        );
    }
    (StatusCode::OK, Json(json!({"status": "sync_requested"})))
}

/// POST /api/v1/ha/force-failover
pub async fn force_failover(State(state): State<AppState>) -> impl IntoResponse {
    if !state.ha_state.get_status().await.enabled {
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
