use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState};

/// GET /api/sensors?screen=X — list sensors, optionally filtered by screen
pub async fn list_sensors(
    State(state): State<AppState>,
    Query(params): Query<DataQuery>,
) -> Result<impl IntoResponse, AppError> {
    let screen_id = params.screen.as_deref();
    let sensors = state.storage.list_sensors(screen_id).await?;
    Ok(Json(sensors))
}

/// POST /api/sensors — create a new sensor
pub async fn create_sensor(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let key = body["key"].as_str().unwrap_or("sensor").to_string();
    let name = body["name"].as_str().unwrap_or("Sensor").to_string();
    let unit = body["unit"].as_str().unwrap_or("").to_string();
    let color = body["color"].as_str().unwrap_or("#f97316").to_string();
    let min_val = body["min_val"].as_f64().unwrap_or(0.0);
    let max_val = body["max_val"].as_f64().unwrap_or(100.0);
    let warn_val = body["warn_val"].as_f64().unwrap_or(0.0);
    let crit_val = body["crit_val"].as_f64().unwrap_or(0.0);
    let base_val = body["base_val"].as_f64().unwrap_or(50.0);
    let decimals = body["decimals"].as_i64().unwrap_or(1);
    let source_type = body["source_type"].as_str().unwrap_or("random_walk").to_string();
    let source_config = body
        .get("source_config")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());
    let device_id = body["device_id"].as_str().unwrap_or("default").to_string();
    let equipment_id = body["equipment_id"].as_str();
    let chart_type = body["chart_type"].as_str().unwrap_or("sparkline").to_string();

    // Get max sort_order
    let max_sort = state.storage.max_sensor_sort_order().await?;

    let id = state
        .storage
        .create_sensor(
            &key,
            &name,
            &unit,
            &color,
            min_val,
            max_val,
            warn_val,
            crit_val,
            base_val,
            max_sort + 1,
            decimals,
            &source_type,
            &source_config,
            &device_id,
            equipment_id,
            &chart_type,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"id": id})),
    ))
}

/// PUT /api/sensors/{id} — update sensor fields dynamically
pub async fn update_sensor(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.update_sensor(id, &body).await?;
    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/sensors/{id}
pub async fn delete_sensor(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.delete_sensor(id).await?;
    Ok(Json(json!({"ok": true})))
}

/// PATCH /api/sensors/reorder — accept {order: [id1, id2, ...]}
pub async fn reorder_sensors(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let order: Vec<i64> = body["order"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("order array is required"))?
        .iter()
        .map(|v| v.as_i64().ok_or_else(|| anyhow::anyhow!("order must contain integer IDs")))
        .collect::<Result<Vec<_>, _>>()?;

    state.storage.reorder_sensors(&order).await?;
    Ok(Json(json!({"ok": true})))
}
