use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use voltrus::api::{AppError, AppState, AuthUser};
use voltrus::models::*;

/// RTSP video is an Enterprise feature: refuse with 403 without the entitlement.
fn require_video(state: &AppState) -> Result<(), AppError> {
    if !state.tier().rtsp_video() {
        return Err(AppError::forbidden(
            "RTSP video requires an Enterprise licence. See https://voltrus.id/voltrus-scada",
        ));
    }
    Ok(())
}

// ── Camera CRUD ──

/// GET /api/v1/cameras
pub async fn list_cameras(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    require_video(&state)?;

    let cameras = state.storage.list_cameras().await?;
    let mut result = Vec::new();
    for camera in &cameras {
        let snapshot_count = state
            .storage
            .list_camera_snapshots(camera.id, 1)
            .await
            .map(|s| if s.is_empty() { 0 } else { 1 })
            .unwrap_or(0);
        result.push(json!({
            "id": camera.id,
            "name": camera.name,
            "url": camera.url,
            "enabled": camera.enabled,
            "snapshot_on_alarm": camera.snapshot_on_alarm,
            "retention_hours": camera.retention_hours,
            "created_at": camera.created_at,
            "has_snapshots": snapshot_count > 0,
        }));
    }
    Ok(Json(result))
}

/// GET /api/v1/cameras/available
///
/// The cameras an operator may pick for a screen widget: id, name and whether
/// it is on. The RTSP address (which may carry the camera's password) stays
/// in the admin list above.
pub async fn list_camera_names(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    require_video(&state)?;
    let cameras = state.storage.list_cameras().await?;
    Ok(Json(
        cameras
            .iter()
            .map(|c| json!({"id": c.id, "name": c.name, "enabled": c.enabled}))
            .collect::<Vec<_>>(),
    ))
}

/// POST /api/v1/cameras
pub async fn create_camera(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<CreateCameraRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    require_video(&state)?;

    if req.name.is_empty() {
        return Err(AppError(anyhow::anyhow!("Camera name is required")));
    }
    if !req.url.starts_with("rtsp://") {
        return Err(AppError(anyhow::anyhow!(
            "Camera URL must start with rtsp://"
        )));
    }

    // Validate URL can be parsed
    if let Err(e) = crate::video::parse_rtsp_url(&req.url) {
        return Err(AppError(anyhow::anyhow!("Invalid RTSP URL: {}", e)));
    }

    let enabled = req.enabled;
    let snapshot_on_alarm = req.snapshot_on_alarm;
    let retention_hours = req.retention_hours;

    let id = state
        .storage
        .create_camera(
            &req.name,
            &req.url,
            enabled,
            snapshot_on_alarm,
            retention_hours,
        )
        .await?;

    // Add to stream proxy if available
    {
        let proxy = &crate::rt().stream_proxy;
        let camera = Camera {
            id,
            name: req.name.clone(),
            url: req.url.clone(),
            enabled,
            snapshot_on_alarm,
            retention_hours,
            created_at: voltrus::timeutil::now_secs(),
        };
        proxy.add_camera(&camera).await;
    }

    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "create_camera".to_string(),
        username: user.username.clone(),
        resource: format!("camera:{}", id),
        details: Some(format!("name={}, url={}", req.name, req.url)),
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "ok": true,
            "id": id,
        })),
    ))
}

/// PUT /api/v1/cameras/{id}
pub async fn update_camera(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateCameraRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    require_video(&state)?;

    // Validate URL if provided
    if let Some(ref url) = req.url {
        if !url.starts_with("rtsp://") {
            return Err(AppError(anyhow::anyhow!(
                "Camera URL must start with rtsp://"
            )));
        }
        if let Err(e) = crate::video::parse_rtsp_url(url) {
            return Err(AppError(anyhow::anyhow!("Invalid RTSP URL: {}", e)));
        }
    }

    state
        .storage
        .update_camera(
            id,
            req.name.as_deref(),
            req.url.as_deref(),
            req.enabled,
            req.snapshot_on_alarm,
            req.retention_hours,
        )
        .await?;

    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "update_camera".to_string(),
        username: user.username.clone(),
        resource: format!("camera:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/v1/cameras/{id}
pub async fn delete_camera(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    require_video(&state)?;

    // Remove from stream proxy
    {
        let proxy = &crate::rt().stream_proxy;
        proxy.remove_camera(id).await;
    }

    state.storage.delete_camera(id).await?;

    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "delete_camera".to_string(),
        username: user.username.clone(),
        resource: format!("camera:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

// ── Snapshot endpoints ──

/// POST /api/v1/cameras/{id}/snapshot
pub async fn trigger_snapshot(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    require_video(&state)?;

    let camera = state
        .storage
        .get_camera(id)
        .await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Camera not found")))?;

    if !camera.enabled {
        return Err(AppError(anyhow::anyhow!(
            "Camera is disabled. Enable it before capturing snapshots."
        )));
    }

    let snapshot = crate::rt()
        .stream_proxy
        .capture_snapshot(&camera, None)
        .await?;

    // Persist snapshot metadata
    let snapshot_id = state
        .storage
        .create_camera_snapshot(
            snapshot.camera_id,
            snapshot.timestamp,
            snapshot.alarm_source.as_deref(),
            &snapshot.file_path,
            snapshot.file_size,
        )
        .await?;

    // Cleanup old snapshots based on retention policy
    let _ = state
        .storage
        .cleanup_expired_snapshots(camera.id, camera.retention_hours)
        .await;

    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "capture_snapshot".to_string(),
        username: user.username.clone(),
        resource: format!("camera:{}", id),
        details: Some(format!(
            "snapshot_id={}, file_size={}",
            snapshot_id, snapshot.file_size
        )),
    });

    Ok(Json(json!({
        "ok": true,
        "snapshot_id": snapshot_id,
        "camera_id": snapshot.camera_id,
        "timestamp": snapshot.timestamp,
        "file_size": snapshot.file_size,
    })))
}

/// GET /api/v1/cameras/{id}/snapshots
pub async fn list_snapshots(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    require_video(&state)?;

    let snapshots = state.storage.list_camera_snapshots(id, 100).await?;
    Ok(Json(snapshots))
}

/// DELETE /api/v1/cameras/{id}/snapshots/{snapshot_id}
pub async fn delete_snapshot(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((_camera_id, snapshot_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    require_video(&state)?;

    state.storage.delete_camera_snapshot(snapshot_id).await?;

    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "delete_snapshot".to_string(),
        username: user.username.clone(),
        resource: format!("camera_snapshot:{}", snapshot_id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

/// GET /api/v1/cameras/{id}/stream
///
/// Returns the RTSP stream URL for the camera (client-side playback).
pub async fn get_stream_info(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    require_video(&state)?;

    let camera = state
        .storage
        .get_camera(id)
        .await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Camera not found")))?;

    Ok(Json(json!({
        "camera_id": camera.id,
        "name": camera.name,
        // The camera password stays on the server.
        "stream_url": crate::video::ffmpeg::redact(&camera.url),
        "live_url": format!("/api/v1/cameras/{}/live", camera.id),
        "enabled": camera.enabled,
        "protocol": "rtsp",
    })))
}

/// GET /api/v1/cameras/{id}/snapshots/{snapshot_id}/file
///
/// The JPEG of one snapshot.
pub async fn snapshot_file(
    State(state): State<AppState>,
    Path((id, snapshot_id)): Path<(i64, i64)>,
) -> Result<axum::response::Response, AppError> {
    require_video(&state)?;
    let snapshot = state
        .storage
        .list_camera_snapshots(id, 10_000)
        .await?
        .into_iter()
        .find(|s| s.id == snapshot_id)
        .ok_or_else(|| AppError::not_found("Snapshot not found"))?;
    let bytes = tokio::fs::read(&snapshot.file_path)
        .await
        .map_err(|_| AppError::not_found("Snapshot file is missing on this node"))?;
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "image/jpeg"),
            (axum::http::header::CACHE_CONTROL, "private, max-age=3600"),
        ],
        bytes,
    )
        .into_response())
}

/// GET /api/v1/cameras/{id}/live
///
/// The camera as MJPEG (`multipart/x-mixed-replace`), transcoded by ffmpeg on
/// the server, so a browser shows it in an `<img>` without any plug-in and the
/// camera itself is never exposed to the browser.
pub async fn live_view(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    require_video(&state)?;
    let camera = state
        .storage
        .get_camera(id)
        .await?
        .ok_or_else(|| AppError::not_found("Camera not found"))?;
    if !camera.enabled {
        return Err(AppError::bad_request("Camera is disabled"));
    }
    let Some(slot) = crate::video::ffmpeg::take_live_slot() else {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!(
                "{} live views are open already; close one and try again",
                crate::video::ffmpeg::MAX_LIVE_VIEWS
            )})),
        )
            .into_response());
    };
    let child = crate::video::ffmpeg::spawn_live(&camera.url)?;
    let body = axum::body::Body::from_stream(crate::video::ffmpeg::live_stream(child, slot));
    Ok((
        [
            (
                axum::http::header::CONTENT_TYPE,
                crate::video::ffmpeg::LIVE_CONTENT_TYPE,
            ),
            (axum::http::header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response())
}
