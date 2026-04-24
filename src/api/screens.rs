use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState};

/// GET /api/screens — list all screens (devices)
pub async fn list_screens(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let devices = state.storage.list_devices().await?;
    Ok(Json(devices))
}

/// POST /api/screens — create a new screen
pub async fn create_screen(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let name = body["name"]
        .as_str()
        .unwrap_or("New Screen")
        .to_string();

    // Generate slug id from name
    let slug = name
        .to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
    let slug = slug.trim_matches('_').to_string();
    let id = if slug.is_empty() {
        format!("screen_{}", chrono::Utc::now().timestamp_millis())
    } else {
        slug
    };

    // Get max sort_order
    let devices = state.storage.list_devices().await?;
    let max_sort = devices.iter().map(|d| d.sort_order).max().unwrap_or(-1);

    state
        .storage
        .create_device(&id, &name, max_sort + 1)
        .await?;

    let device = Device {
        id,
        name,
        device_type: "screen".to_string(),
        sort_order: max_sort + 1,
    };

    Ok((StatusCode::CREATED, Json(device)))
}

/// PUT /api/screens/{id} — update a screen
pub async fn update_screen(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let name = body["name"].as_str();
    let sort_order = body["sort_order"].as_i64();

    state
        .storage
        .update_device(&id, name, sort_order)
        .await?;

    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/screens/{id} — delete a screen and its sensor associations
pub async fn delete_screen(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Delete screen_sensors first
    state
        .storage
        .delete_screen_sensors_by_screen(&id)
        .await?;
    state.storage.delete_device(&id).await?;

    Ok(Json(json!({"ok": true})))
}

/// GET /api/screens/{id}/sensors — list sensors for a screen
pub async fn list_screen_sensors(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let sensors = state.storage.list_sensors(Some(&id)).await?;
    Ok(Json(sensors))
}

/// POST /api/screens/{id}/sensors — add a sensor to a screen
pub async fn add_screen_sensor(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let sensor_id = body["sensor_id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("sensor_id is required"))?;

    // Get max sort_order for this screen's sensors
    let existing = state.storage.list_sensors(Some(&id)).await?;
    let max_sort = existing.iter().map(|s| s.sort_order).max().unwrap_or(-1);

    state
        .storage
        .add_screen_sensor(&id, sensor_id, max_sort + 1)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"ok": true})),
    ))
}

/// PUT /api/screens/{id}/sensors/{sensor_id} — update screen sensor sort order
pub async fn update_screen_sensor(
    State(state): State<AppState>,
    Path((id, sensor_id)): Path<(String, i64)>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let sort_order = body["sort_order"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("sort_order is required"))?;

    state
        .storage
        .update_screen_sensor_sort(&id, sensor_id, sort_order)
        .await?;

    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/screens/{id}/sensors/{sensor_id} — remove sensor from screen
pub async fn remove_screen_sensor(
    State(state): State<AppState>,
    Path((id, sensor_id)): Path<(String, i64)>,
) -> Result<impl IntoResponse, AppError> {
    state
        .storage
        .remove_screen_sensor(&id, sensor_id)
        .await?;

    Ok(Json(json!({"ok": true})))
}
