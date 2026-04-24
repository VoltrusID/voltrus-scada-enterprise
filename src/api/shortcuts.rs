use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::User;
use super::{AppState, AppError};

/// GET /api/shortcuts — get shortcuts for the current user
pub async fn get_shortcuts(
    State(state): State<AppState>,
    axum::Extension(user): axum::Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    let shortcuts = state.storage.get_user_shortcuts(user.id).await?;
    // Return as map: {sensor_id: shortcut}
    let mut map = serde_json::Map::new();
    for s in &shortcuts {
        map.insert(s.sensor_id.to_string(), json!(s.shortcut));
    }
    Ok(Json(serde_json::Value::Object(map)))
}

/// PUT /api/shortcuts — save shortcuts, accept map of {sensor_id: shortcut}
/// If value is null or empty string, delete the entry
pub async fn save_shortcuts(
    State(state): State<AppState>,
    axum::Extension(user): axum::Extension<User>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let obj = body
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Expected a JSON object"))?;

    // Collect as owned strings first
    let mut owned_entries: Vec<(i64, Option<String>)> = Vec::new();
    for (sensor_id_str, value) in obj {
        let sensor_id: i64 = sensor_id_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid sensor_id: {}", sensor_id_str))?;
        match value {
            serde_json::Value::Null => {
                owned_entries.push((sensor_id, None));
            }
            serde_json::Value::String(s) if s.is_empty() => {
                owned_entries.push((sensor_id, None));
            }
            serde_json::Value::String(s) => {
                owned_entries.push((sensor_id, Some(s.clone())));
            }
            _ => {
                owned_entries.push((sensor_id, Some(value.to_string())));
            }
        }
    }

    // Convert to references for the storage method
    let entries: Vec<(i64, Option<&str>)> = owned_entries
        .iter()
        .map(|(id, opt)| (*id, opt.as_deref()))
        .collect();

    state.storage.save_user_shortcuts(user.id, &entries).await?;
    Ok(Json(json!({"ok": true})))
}
