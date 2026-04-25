use axum::extract::{Extension, Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState, AuthUser};

/// GET /api/alarms?status=active|acknowledged|all&limit=50
pub async fn list_alarms(
    State(state): State<AppState>,
    Query(params): Query<AlarmQuery>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.as_deref();
    let limit = params.limit.unwrap_or(50);
    let mut alarms = state.storage.list_alarms(status, limit).await?;

    // Resolve source_name from data_sources
    let sources = state.storage.list_data_sources(None).await?;
    let name_map: std::collections::HashMap<String, String> = sources
        .iter()
        .map(|s| (s.key.clone(), s.name.clone()))
        .collect();

    for alarm in &mut alarms {
        if let Some(name) = name_map.get(&alarm.source_key) {
            alarm.source_name = name.clone();
        } else {
            alarm.source_name = alarm.source_key.clone();
        }
    }

    Ok(Json(alarms))
}

/// POST /api/alarms/{id}/acknowledge
pub async fn acknowledge_alarm(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.acknowledge_alarm(id, &user.username).await?;
    Ok(Json(json!({"ok": true})))
}

/// POST /api/alarms/acknowledge-all
pub async fn acknowledge_all(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.acknowledge_all(&user.username).await?;
    Ok(Json(json!({"ok": true})))
}
