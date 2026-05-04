use super::*;

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
