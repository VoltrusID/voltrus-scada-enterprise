use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::api::{AppError, AppState, AuthUser};
use crate::models::*;

// ── Camera CRUD ──

/// GET /api/v1/cameras
pub async fn list_cameras(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.rtsp_video() {
        return Err(AppError(anyhow::anyhow!(
            "RTSP video requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

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

/// POST /api/v1/cameras
pub async fn create_camera(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<CreateCameraRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.rtsp_video() {
        return Err(AppError(anyhow::anyhow!(
            "RTSP video requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

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
    if let Some(ref proxy) = state.stream_proxy {
        let camera = Camera {
            id,
            name: req.name.clone(),
            url: req.url.clone(),
            enabled,
            snapshot_on_alarm,
            retention_hours,
            created_at: crate::timeutil::now_secs(),
        };
        proxy.add_camera(&camera).await;
    }

    state.audit_logger.log(crate::security::AuditEvent {
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

    if !state.license_tier.rtsp_video() {
        return Err(AppError(anyhow::anyhow!(
            "RTSP video requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

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

    state.audit_logger.log(crate::security::AuditEvent {
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

    if !state.license_tier.rtsp_video() {
        return Err(AppError(anyhow::anyhow!(
            "RTSP video requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Remove from stream proxy
    if let Some(ref proxy) = state.stream_proxy {
        proxy.remove_camera(id).await;
    }

    state.storage.delete_camera(id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
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

    if !state.license_tier.rtsp_video() {
        return Err(AppError(anyhow::anyhow!(
            "RTSP video requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

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

    // Use stream proxy if available
    let snapshot = if let Some(ref proxy) = state.stream_proxy {
        proxy.capture_snapshot(&camera, None).await?
    } else {
        // Fallback: direct RTSP capture without proxy
        let mut client = crate::video::RtspClient::new(&camera.url);
        let data = client.capture_snapshot().await?;

        let timestamp = crate::timeutil::now_secs();
        let filename = format!("cam{}_{}.jpg", camera.id, timestamp);
        let file_path = format!("./snapshots/{}", filename);

        let _ = std::fs::create_dir_all("./snapshots");
        std::fs::write(&file_path, &data)
            .map_err(|e| AppError(anyhow::anyhow!("Failed to write snapshot: {}", e)))?;

        CameraSnapshot {
            id: 0,
            camera_id: camera.id,
            timestamp,
            alarm_source: None,
            file_path: file_path.clone(),
            file_size: data.len() as u64,
        }
    };

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

    state.audit_logger.log(crate::security::AuditEvent {
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

    if !state.license_tier.rtsp_video() {
        return Err(AppError(anyhow::anyhow!(
            "RTSP video requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

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

    if !state.license_tier.rtsp_video() {
        return Err(AppError(anyhow::anyhow!(
            "RTSP video requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    state.storage.delete_camera_snapshot(snapshot_id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
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

    if !state.license_tier.rtsp_video() {
        return Err(AppError(anyhow::anyhow!(
            "RTSP video requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let camera = state
        .storage
        .get_camera(id)
        .await?
        .ok_or_else(|| AppError(anyhow::anyhow!("Camera not found")))?;

    Ok(Json(json!({
        "camera_id": camera.id,
        "name": camera.name,
        "stream_url": camera.url,
        "enabled": camera.enabled,
        "protocol": "rtsp",
    })))
}
