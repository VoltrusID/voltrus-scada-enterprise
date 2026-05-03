use axum::extract::{Extension, Json, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

use super::{AppError, AppState, AuthUser};

// ── Query params ──

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub unread_only: Option<bool>,
}

// ── Handlers ──

/// GET /api/v1/user-notifications
#[utoipa::path(
    get,
    path = "/api/v1/user-notifications",
    responses((status = 200, description = "List user notifications")),
    tag = "Notifications",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_notifications(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<ListNotificationsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Resolve current user ID from authenticated username
    let user = state
        .storage
        .get_user_by_username(&auth_user.username)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Authenticated user not found"))?;
    let user_id = user.id;

    let unread_only = params.unread_only.unwrap_or(false);

    // Auto-cleanup old notifications (best-effort, ignore errors)
    let _ = state.storage.cleanup_old_notifications().await;

    let notifications = state
        .storage
        .list_user_notifications(user_id, unread_only)
        .await?;

    Ok(Json(json!({
        "notifications": notifications,
        "count": notifications.len(),
    })))
}

/// PUT /api/v1/user-notifications/{id}/read
#[utoipa::path(
    put,
    path = "/api/v1/user-notifications/{id}/read",
    params(("id" = i64, Path, description = "Notification ID")),
    responses((status = 200, description = "Notification marked as read")),
    tag = "Notifications",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn mark_read(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    // Resolve current user ID from authenticated username
    let user = state
        .storage
        .get_user_by_username(&auth_user.username)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Authenticated user not found"))?;
    let user_id = user.id;

    // Verify ownership: list user's notifications and find the target
    let user_notifications = state
        .storage
        .list_user_notifications(user_id, false)
        .await?;

    let owned = user_notifications
        .iter()
        .any(|n| n.id == id);

    if !owned {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Notification not found"})),
        )
            .into_response());
    }

    state.storage.mark_notification_read(id).await?;

    Ok(Json(json!({ "ok": true })).into_response())
}

/// DELETE /api/v1/user-notifications/{id}
#[utoipa::path(
    delete,
    path = "/api/v1/user-notifications/{id}",
    params(("id" = i64, Path, description = "Notification ID")),
    responses((status = 204, description = "Notification dismissed")),
    tag = "Notifications",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn dismiss_notification(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    // Resolve current user ID from authenticated username
    let user = state
        .storage
        .get_user_by_username(&auth_user.username)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Authenticated user not found"))?;
    let user_id = user.id;

    // Verify ownership: list user's notifications and find the target
    let user_notifications = state
        .storage
        .list_user_notifications(user_id, false)
        .await?;

    let owned = user_notifications
        .iter()
        .any(|n| n.id == id);

    if !owned {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Notification not found"})),
        )
            .into_response());
    }

    state.storage.dismiss_notification(id).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

// ── Tests ─
