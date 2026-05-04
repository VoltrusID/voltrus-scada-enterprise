use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::api::{AppError, AppState, AuthUser};
use crate::ethernet_ip::{EthernetIpClient, validate_tag_name};
use crate::models::*;

// ── Device CRUD ──

/// GET /api/v1/enip/devices
pub async fn list_devices(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let devices = state.storage.list_enip_devices().await?;
    let mut result = Vec::new();
    for device in &devices {
        let tags = state.storage.list_enip_tags(device.id).await?;
        result.push(json!({
            "id": device.id,
            "name": device.name,
            "url": device.url,
            "slot": device.slot,
            "interval_ms": device.interval_ms,
            "enabled": device.enabled,
            "created_at": device.created_at,
            "tag_count": tags.len(),
        }));
    }
    Ok(Json(result))
}

/// POST /api/v1/enip/devices
pub async fn create_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<CreateEnipDeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    if req.name.is_empty() {
        return Err(AppError(anyhow::anyhow!("Device name is required")));
    }
    if req.url.is_empty() {
        return Err(AppError(anyhow::anyhow!("Device URL is required")));
    }

    let id = state
        .storage
        .create_enip_device(
            &req.name,
            &req.url,
            req.slot,
            req.interval_ms,
            req.enabled,
        )
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_enip_device".to_string(),
        username: user.username.clone(),
        resource: format!("enip_device:{}", id),
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

/// PUT /api/v1/enip/devices/{id}
pub async fn update_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateEnipDeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    state
        .storage
        .update_enip_device(
            id,
            req.name.as_deref(),
            req.url.as_deref(),
            req.slot,
            req.interval_ms,
            req.enabled,
        )
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_enip_device".to_string(),
        username: user.username.clone(),
        resource: format!("enip_device:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/v1/enip/devices/{id}
pub async fn delete_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Stop poller if running
    let device_key = format!("enip_{}", id);
    state.poller_manager.stop(&device_key).await;

    state.storage.delete_enip_device(id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "delete_enip_device".to_string(),
        username: user.username.clone(),
        resource: format!("enip_device:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

// ── Tag CRUD ──

/// GET /api/v1/enip/devices/{id}/tags
pub async fn list_tags(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let tags = state.storage.list_enip_tags(id).await?;
    Ok(Json(tags))
}

/// POST /api/v1/enip/devices/{id}/tags
pub async fn add_tag(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(device_id): Path<i64>,
    Json(req): Json<CreateEnipTagRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    if req.tag_name.is_empty() {
        return Err(AppError(anyhow::anyhow!("Tag name is required")));
    }
    if req.name.is_empty() {
        return Err(AppError(anyhow::anyhow!("Name is required")));
    }

    // Validate tag name format
    if let Err(e) = validate_tag_name(&req.tag_name) {
        return Err(AppError(anyhow::anyhow!("Invalid tag name: {}", e)));
    }

    // Look up device name for data source key generation
    let devices = state.storage.list_enip_devices().await?;
    let device = devices
        .iter()
        .find(|d| d.id == device_id)
        .ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    let id = state
        .storage
        .create_enip_tag(
            device_id,
            &device.name,
            &req.tag_name,
            &req.name,
            &req.display_name,
            &req.data_type,
        )
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                AppError(anyhow::anyhow!("Tag already exists on this device"))
            } else {
                AppError(e)
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "ok": true,
            "id": id,
        })),
    ))
}

/// DELETE /api/v1/enip/devices/{id}/tags/{tag_id}
pub async fn remove_tag(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((_device_id, tag_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    state.storage.delete_enip_tag(tag_id).await?;

    Ok(Json(json!({"ok": true})))
}

// ── Browse ──

/// POST /api/v1/enip/devices/{id}/browse
///
/// Connects to the PLC and browses available tags using GetAttributeAll on Symbol class.
pub async fn browse_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Load device from DB
    let devices = state.storage.list_enip_devices().await?;
    let device = devices
        .iter()
        .find(|d| d.id == id)
        .ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    // Connect and browse
    let mut client = EthernetIpClient::new(&device.url);
    client.connect().await.map_err(|e| {
        AppError(anyhow::anyhow!("EtherNet/IP connection failed: {}", e))
    })?;

    let tags = client.browse_tags().await.map_err(|e| {
        AppError(anyhow::anyhow!("EtherNet/IP browse failed: {}", e))
    })?;

    client.disconnect().await.ok();

    let tag_list: Vec<serde_json::Value> = tags
        .iter()
        .map(|tag| {
            json!({
                "tag_name": tag.name,
                "data_type": tag.data_type.as_str(),
                "element_count": tag.element_count,
            })
        })
        .collect();

    Ok(Json(json!({
        "tags": tag_list,
        "count": tag_list.len(),
    })))
}

// ── Start / Stop ──

/// POST /api/v1/enip/devices/{id}/start
///
/// Starts the EtherNet/IP poller for the device.
pub async fn start_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Load device from DB
    let devices = state.storage.list_enip_devices().await?;
    let device = devices
        .iter()
        .find(|d| d.id == id)
        .ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    if !device.enabled {
        return Err(AppError(anyhow::anyhow!(
            "Device is disabled. Enable it first before starting."
        )));
    }

    // Load tags for device
    let tags = state.storage.list_enip_tags(device.id).await?;

    if tags.is_empty() {
        return Err(AppError(anyhow::anyhow!(
            "Device has no tags configured. Add tags before starting."
        )));
    }

    // Convert to runtime config
    let device_key = format!("enip_{}", device.id);
    let config = EnipDeviceConfig {
        id: device_key.clone(),
        url: device.url.clone(),
        slot: device.slot,
        interval_ms: device.interval_ms,
        tags: tags
            .iter()
            .map(|t| crate::ethernet_ip::EnipTagConfig {
                tag_name: t.tag_name.clone(),
                name: t.name.clone(),
                data_type: t.data_type.clone(),
            })
            .collect(),
    };

    state.poller_manager.start_ethernet_ip(config).await;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "start_enip_device".to_string(),
        username: user.username.clone(),
        resource: format!("enip_device:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

/// POST /api/v1/enip/devices/{id}/stop
pub async fn stop_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.ethernet_ip() {
        return Err(AppError(anyhow::anyhow!(
            "EtherNet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let device_key = format!("enip_{}", id);
    state.poller_manager.stop(&device_key).await;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "stop_enip_device".to_string(),
        username: user.username.clone(),
        resource: format!("enip_device:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}
