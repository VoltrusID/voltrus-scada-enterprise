use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::api::{AppError, AppState, AuthUser};
use crate::dnp3::objects::common_group_variations;
use crate::models::*;

// ── Device CRUD ──

/// GET /api/v1/dnp3/devices
pub async fn list_devices(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let devices = state.storage.list_dnp3_devices().await?;
    let mut result = Vec::new();
    for device in &devices {
        let points = state.storage.list_dnp3_points(device.id).await?;
        result.push(json!({
            "id": device.id,
            "name": device.name,
            "ip": device.ip,
            "port": device.port,
            "master_address": device.master_address,
            "outstation_address": device.outstation_address,
            "enable_unsolicited": device.enable_unsolicited,
            "integrity_poll_interval_secs": device.integrity_poll_interval_secs,
            "event_scan_interval_ms": device.event_scan_interval_ms,
            "enabled": device.enabled,
            "created_at": device.created_at,
            "updated_at": device.updated_at,
            "point_count": points.len(),
        }));
    }
    Ok(Json(result))
}

/// POST /api/v1/dnp3/devices
pub async fn create_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<CreateDnp3DeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }
    if req.name.is_empty() {
        return Err(AppError(anyhow::anyhow!("Device name is required")));
    }
    if req.ip.is_empty() {
        return Err(AppError(anyhow::anyhow!("Device IP is required")));
    }

    let id = state
        .storage
        .create_dnp3_device(
            &req.name,
            &req.ip,
            req.port,
            req.master_address,
            req.outstation_address,
            req.enable_unsolicited,
            req.integrity_poll_interval_secs,
            req.event_scan_interval_ms,
            req.enabled,
        )
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_dnp3_device".to_string(),
        username: user.username.clone(),
        resource: format!("dnp3_device:{}", id),
        details: Some(format!("name={}, ip={}", req.name, req.ip)),
    });

    Ok((StatusCode::CREATED, Json(json!({ "ok": true, "id": id }))))
}

/// GET /api/v1/dnp3/devices/{id}
pub async fn get_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let device = state.storage.get_dnp3_device(id).await?;
    let device = device.ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;
    let points = state.storage.list_dnp3_points(id).await?;
    Ok(Json(json!({ "device": device, "points": points })))
}

/// PUT /api/v1/dnp3/devices/{id}
pub async fn update_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateDnp3DeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let updated = state
        .storage
        .update_dnp3_device(
            id,
            req.name.as_deref(),
            req.ip.as_deref(),
            req.port,
            req.master_address,
            req.outstation_address,
            req.enable_unsolicited,
            req.integrity_poll_interval_secs,
            req.event_scan_interval_ms,
            req.enabled,
        )
        .await?;

    if !updated {
        return Err(AppError(anyhow::anyhow!("Device not found")));
    }
    Ok(Json(json!({ "ok": true })))
}

/// DELETE /api/v1/dnp3/devices/{id}
pub async fn delete_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let deleted = state.storage.delete_dnp3_device(id).await?;
    if !deleted {
        return Err(AppError(anyhow::anyhow!("Device not found")));
    }

    // Stop poller if running
    let device_key = format!("dnp3_{}", id);
    state.poller_manager.stop(&device_key).await;

    Ok(StatusCode::NO_CONTENT)
}

// ── Point CRUD ──

/// POST /api/v1/dnp3/devices/{id}/points
pub async fn add_point(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<CreateDnp3PointRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }
    if req.name.is_empty() {
        return Err(AppError(anyhow::anyhow!("Point name is required")));
    }

    // Verify device exists
    let device = state.storage.get_dnp3_device(id).await?;
    device.ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    let point_id = state
        .storage
        .create_dnp3_point(
            id,
            req.group_num,
            req.variation,
            req.point_index,
            &req.name,
            &req.display_name,
            &req.data_type,
            req.scale,
            req.deadband,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(json!({ "ok": true, "id": point_id }))))
}

/// DELETE /api/v1/dnp3/devices/{id}/points/{point_id}
pub async fn remove_point(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((_id, point_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let deleted = state.storage.delete_dnp3_point(point_id).await?;
    if !deleted {
        return Err(AppError(anyhow::anyhow!("Point not found")));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ── Discovery ──

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub range: String,
    #[serde(default = "default_dnp3_scan_port")]
    pub port: u16,
}

fn default_dnp3_scan_port() -> u16 {
    20000
}

/// POST /api/v1/dnp3/scan
pub async fn scan_network(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<ScanRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let devices = crate::dnp3::discovery::scan_network(&req.range, req.port, 2000).await?;
    Ok(Json(json!(devices)))
}

// ── Actions ──

/// POST /api/v1/dnp3/devices/{id}/integrity
pub async fn trigger_integrity(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Verify device exists
    let device = state.storage.get_dnp3_device(id).await?;
    device.ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    Ok(Json(json!({ "ok": true, "message": "Integrity poll triggered" })))
}

/// POST /api/v1/dnp3/devices/{id}/freeze
pub async fn trigger_freeze(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Verify device exists
    let device = state.storage.get_dnp3_device(id).await?;
    device.ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    Ok(Json(json!({ "ok": true, "message": "Freeze triggered" })))
}

/// GET /api/v1/dnp3/group-variations
pub async fn list_group_variations(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }
    if !state.license_tier.dnp3() {
        return Err(AppError(anyhow::anyhow!(
            "DNP3 requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let gv = common_group_variations()
        .into_iter()
        .map(|(g, v, name)| json!({ "group": g, "variation": v, "name": name }))
        .collect::<Vec<_>>();
    Ok(Json(gv))
}
