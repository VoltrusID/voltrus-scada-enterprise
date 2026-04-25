use axum::extract::{Extension, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState, AuthUser};

/// POST /api/push/subscribe
pub async fn subscribe(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<PushSubscribeRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .storage
        .save_push_subscription(&user.username, &body.endpoint, &body.p256dh, &body.auth_key)
        .await?;
    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/push/subscribe
pub async fn unsubscribe(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let endpoint = body["endpoint"].as_str().unwrap_or("");
    state
        .storage
        .remove_push_subscription(&user.username, endpoint)
        .await?;
    Ok(Json(json!({"ok": true})))
}
