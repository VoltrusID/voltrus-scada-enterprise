use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::api::{AppError, AppState, AuthUser};
use crate::models::{
    BacnetDeviceConfig, BacnetObjectConfig, CreateBacnetDeviceRequest, CreateBacnetObjectRequest,
    UpdateBacnetDeviceRequest,
};

// ── Device CRUD ──

/// GET /api/v1/bacnet/devices
#[utoipa::path(
    get,
    path = "/api/v1/bacnet/devices",
    responses(
        (status = 200, body = Vec<serde_json::Value>),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_devices(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let devices = state.storage.list_bacnet_devices().await?;
    let mut result = Vec::new();
    for device in &devices {
        let objects = state.storage.list_bacnet_objects(device.id).await?;
        result.push(json!({
            "id": device.id,
            "name": device.name,
            "ip": device.ip,
            "port": device.port,
            "device_instance": device.device_instance,
            "interval_ms": device.interval_ms,
            "cov_enabled": device.cov_enabled,
            "cov_lifetime": device.cov_lifetime,
            "enabled": device.enabled,
            "created_at": device.created_at,
            "object_count": objects.len(),
        }));
    }
    Ok(Json(result))
}

/// POST /api/v1/bacnet/devices
#[utoipa::path(
    post,
    path = "/api/v1/bacnet/devices",
    request_body = CreateBacnetDeviceRequest,
    responses(
        (status = 201, body = serde_json::Value),
        (status = 400),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn create_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<CreateBacnetDeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
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
        .create_bacnet_device(
            &req.name,
            &req.ip,
            req.port,
            req.device_instance,
            req.interval_ms,
            req.cov_enabled,
            req.cov_lifetime,
            req.enabled,
        )
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_bacnet_device".to_string(),
        username: user.username.clone(),
        resource: format!("bacnet_device:{}", id),
        details: Some(format!("name={}, ip={}", req.name, req.ip)),
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "ok": true,
            "id": id,
        })),
    ))
}

/// PUT /api/v1/bacnet/devices/{id}
#[utoipa::path(
    put,
    path = "/api/v1/bacnet/devices/{id}",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    request_body = UpdateBacnetDeviceRequest,
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateBacnetDeviceRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    state
        .storage
        .update_bacnet_device(
            id,
            req.name.as_deref(),
            req.ip.as_deref(),
            req.port,
            req.device_instance,
            req.interval_ms,
            req.cov_enabled,
            req.cov_lifetime,
            req.enabled,
        )
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_bacnet_device".to_string(),
        username: user.username.clone(),
        resource: format!("bacnet_device:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/v1/bacnet/devices/{id}
#[utoipa::path(
    delete,
    path = "/api/v1/bacnet/devices/{id}",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn delete_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Stop poller if running
    let device_key = format!("bacnet_{}", id);
    state.poller_manager.stop(&device_key).await;

    state.storage.delete_bacnet_device(id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "delete_bacnet_device".to_string(),
        username: user.username.clone(),
        resource: format!("bacnet_device:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

// ── Object CRUD ──

/// GET /api/v1/bacnet/devices/{id}/objects
#[utoipa::path(
    get,
    path = "/api/v1/bacnet/devices/{id}/objects",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    responses(
        (status = 200, body = Vec<serde_json::Value>),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_objects(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let objects = state.storage.list_bacnet_objects(id).await?;
    Ok(Json(objects))
}

/// POST /api/v1/bacnet/devices/{id}/objects
#[utoipa::path(
    post,
    path = "/api/v1/bacnet/devices/{id}/objects",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    request_body = CreateBacnetObjectRequest,
    responses(
        (status = 201, body = serde_json::Value),
        (status = 400),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn add_object(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(device_id): Path<i64>,
    Json(req): Json<CreateBacnetObjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Validate object type
    let valid_types = [
        "analogInput",
        "analogOutput",
        "analogValue",
        "binaryInput",
        "binaryOutput",
        "binaryValue",
        "multiStateInput",
        "multiStateOutput",
        "multiStateValue",
    ];
    if !valid_types.contains(&req.object_type.as_str()) {
        return Err(AppError(anyhow::anyhow!(
            "Invalid object_type '{}'. Must be one of: {}",
            req.object_type,
            valid_types.join(", ")
        )));
    }

    // Load device name for data source key generation
    let devices = state.storage.list_bacnet_devices().await?;
    let device = devices
        .iter()
        .find(|d| d.id == device_id)
        .ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    let id = state
        .storage
        .create_bacnet_object(
            device_id,
            &device.name,
            &req.object_type,
            req.object_instance,
            &req.property,
            &req.name,
            &req.display_name,
            req.cov_increment,
        )
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                AppError(anyhow::anyhow!("Object already exists on this device"))
            } else {
                AppError(e)
            }
        })?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_bacnet_object".to_string(),
        username: user.username.clone(),
        resource: format!("bacnet_object:{}", id),
        details: Some(format!(
            "device_id={}, object_type={}, object_instance={}",
            device_id, req.object_type, req.object_instance
        )),
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "ok": true,
            "id": id,
        })),
    ))
}

/// DELETE /api/v1/bacnet/devices/{id}/objects/{object_id}
#[utoipa::path(
    delete,
    path = "/api/v1/bacnet/devices/{id}/objects/{object_id}",
    params(
        ("id" = i64, Path, description = "Device ID"),
        ("object_id" = i64, Path, description = "Object ID")
    ),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn remove_object(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((_device_id, object_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    state.storage.delete_bacnet_object(object_id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "delete_bacnet_object".to_string(),
        username: user.username.clone(),
        resource: format!("bacnet_object:{}", object_id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

// ── Browse ──

/// POST /api/v1/bacnet/devices/{id}/browse
#[utoipa::path(
    post,
    path = "/api/v1/bacnet/devices/{id}/browse",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
        (status = 500),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn browse_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Load device from DB
    let devices = state.storage.list_bacnet_devices().await?;
    let device = devices
        .iter()
        .find(|d| d.id == id)
        .ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    let objects = crate::bacnet::browse_objects(&device.ip, device.port, device.device_instance)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("BACnet browse failed: {}", e)))?;

    let object_list: Vec<serde_json::Value> = objects
        .iter()
        .map(|obj| {
            json!({
                "object_type": obj.object_type,
                "object_instance": obj.object_instance,
                "object_name": obj.object_name,
            })
        })
        .collect();

    Ok(Json(json!({
        "objects": object_list,
        "count": object_list.len(),
    })))
}

// ── Discover ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DiscoverRequest {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    5
}

/// POST /api/v1/bacnet/discover
#[utoipa::path(
    post,
    path = "/api/v1/bacnet/discover",
    request_body = Option<DiscoverRequest>,
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
        (status = 500),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn discover_network(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<Option<DiscoverRequest>>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let timeout_secs = body.map(|b| b.timeout_secs).unwrap_or(5);

    let discovered = crate::bacnet::discover_devices(timeout_secs)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("BACnet discover failed: {}", e)))?;

    let device_list: Vec<serde_json::Value> = discovered
        .iter()
        .map(|d| {
            json!({
                "device_instance": d.device_instance,
                "ip": d.ip,
                "port": d.port,
                "vendor_name": d.vendor_name,
                "model_name": d.model_name,
                "firmware_version": d.firmware_version,
                "object_list_count": d.object_list_count,
            })
        })
        .collect();

    Ok(Json(json!({
        "devices": device_list,
        "count": device_list.len(),
    })))
}

// ── Start / Stop ──

/// POST /api/v1/bacnet/devices/{id}/start
#[utoipa::path(
    post,
    path = "/api/v1/bacnet/devices/{id}/start",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 400),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn start_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    // Load device from DB
    let devices = state.storage.list_bacnet_devices().await?;
    let device = devices
        .iter()
        .find(|d| d.id == id)
        .ok_or_else(|| AppError(anyhow::anyhow!("Device not found")))?;

    if !device.enabled {
        return Err(AppError(anyhow::anyhow!(
            "Device is disabled. Enable it first before starting."
        )));
    }

    // Load objects for device
    let objects = state.storage.list_bacnet_objects(device.id).await?;

    if objects.is_empty() {
        return Err(AppError(anyhow::anyhow!(
            "Device has no objects configured. Add objects before starting."
        )));
    }

    // Build runtime config
    let device_key = format!("bacnet_{}", device.id);
    let config = BacnetDeviceConfig {
        id: device_key.clone(),
        ip: device.ip.clone(),
        port: device.port,
        device_instance: device.device_instance,
        interval_ms: device.interval_ms,
        cov_enabled: device.cov_enabled,
        cov_lifetime: device.cov_lifetime,
        objects: objects
            .iter()
            .map(|o| BacnetObjectConfig {
                object_type: o.object_type.clone(),
                object_instance: o.object_instance,
                property: o.property.clone(),
                name: o.name.clone(),
                cov_increment: o.cov_increment,
            })
            .collect(),
    };

    state.poller_manager.start_bacnet(config).await;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "start_bacnet_device".to_string(),
        username: user.username.clone(),
        resource: format!("bacnet_device:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

/// POST /api/v1/bacnet/devices/{id}/stop
#[utoipa::path(
    post,
    path = "/api/v1/bacnet/devices/{id}/stop",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
    ),
    tag = "BACnet",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn stop_device(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if user.role != "admin" {
        return Err(AppError(anyhow::anyhow!("Admin access required")));
    }

    if !state.license_tier.bacnet() {
        return Err(AppError(anyhow::anyhow!(
            "BACnet/IP requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }

    let device_key = format!("bacnet_{}", id);
    state.poller_manager.stop(&device_key).await;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "stop_bacnet_device".to_string(),
        username: user.username.clone(),
        resource: format!("bacnet_device:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}
