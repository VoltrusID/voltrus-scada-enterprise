use axum::extract::{Extension, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState, AuthUser};

/// GET /api/v1/push/vapid-key
///
/// Returns the VAPID public key for frontend push subscription.
/// Public endpoint (no auth required) — the key is not a secret.
#[utoipa::path(
    get,
    path = "/api/v1/push/vapid-key",
    responses((status = 200, description = "VAPID public key for push subscriptions")),
    tag = "Notifications"
)]
pub async fn vapid_public_key(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    match &state.vapid_public_key {
        Some(key) => Ok(Json(json!({"publicKey": key}))),
        None => Err(AppError(anyhow::anyhow!("Push notifications are not configured"))),
    }
}

/// POST /api/v1/push/subscribe
#[utoipa::path(
    post,
    path = "/api/v1/push/subscribe",
    request_body = PushSubscribeRequest,
    responses((status = 200, description = "Push subscription saved")),
    tag = "Notifications",
    security(("api_key" = []), ("cookie_auth" = []))
)]
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

/// DELETE /api/v1/push/subscribe
#[utoipa::path(
    delete,
    path = "/api/v1/push/subscribe",
    responses((status = 200, description = "Push subscription removed")),
    tag = "Notifications",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn unsubscribe(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let endpoint = body["endpoint"].as_str().unwrap_or("");
    if endpoint.is_empty() {
        return Err(AppError(anyhow::anyhow!("endpoint is required")));
    }
    state
        .storage
        .remove_push_subscription(&user.username, endpoint)
        .await?;
    Ok(Json(json!({"ok": true})))
}
