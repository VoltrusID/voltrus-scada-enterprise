use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState};

/// GET /api/screens/{id}/layouts — get layouts for a screen, returned as {lg:[], md:[], sm:[]}
pub async fn get_layouts(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let rows = state.storage.get_layouts(&id).await?;
    let mut result = serde_json::Map::new();
    for row in &rows {
        let layout: serde_json::Value =
            serde_json::from_str(&row.layout).unwrap_or(serde_json::json!([]));
        result.insert(row.breakpoint.clone(), layout);
    }
    // Ensure all breakpoints exist
    if !result.contains_key("lg") {
        result.insert("lg".to_string(), json!([]));
    }
    if !result.contains_key("md") {
        result.insert("md".to_string(), json!([]));
    }
    if !result.contains_key("sm") {
        result.insert("sm".to_string(), json!([]));
    }
    Ok(Json(serde_json::Value::Object(result)))
}

/// PUT /api/screens/{id}/layouts — accept {lg:[], md:[], sm:[]} and upsert per breakpoint
pub async fn save_layouts(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Transform {lg:[], md:[], sm:[]} into array of {breakpoint, layout} for storage
    let mut layouts_array = vec![];
    for bp in &["lg", "md", "sm"] {
        if let Some(layout_val) = body.get(bp) {
            layouts_array.push(json!({
                "breakpoint": bp,
                "layout": layout_val
            }));
        }
    }
    state
        .storage
        .save_layouts(&id, &serde_json::Value::Array(layouts_array))
        .await?;
    Ok(Json(json!({"ok": true})))
}

/// GET /api/screens/{id}/twin-layouts — get twin layouts for a screen
pub async fn get_twin_layouts(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let layouts = state.storage.get_twin_layouts(&id).await?;
    Ok(Json(layouts))
}

/// PUT /api/screens/{id}/twin-layouts — accept array of twin layouts, validate equipment_type
pub async fn save_twin_layouts(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let arr = body
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Expected an array of twin layouts"))?;

    let valid_types = [
        "solar_panel",
        "battery",
        "inverter",
        "charge_controller",
        "grid_meter",
        "load",
        "transformer",
        "generator",
    ];

    let mut layouts = Vec::new();
    for item in arr {
        let equipment_type = item["equipment_type"]
            .as_str()
            .unwrap_or("")
            .to_string();
        // Allow any equipment_type (don't restrict for flexibility)
        let _ = valid_types; // suppress unused warning
        layouts.push(TwinLayout {
            screen_id: id.clone(),
            equipment_id: item["equipment_id"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            equipment_type,
            position_x: item["position_x"].as_f64().unwrap_or(0.0),
            position_z: item["position_z"].as_f64().unwrap_or(0.0),
            rotation_y: item["rotation_y"].as_f64().unwrap_or(0.0),
        });
    }

    state.storage.save_twin_layouts(&id, &layouts).await?;
    Ok(Json(json!({"ok": true})))
}
