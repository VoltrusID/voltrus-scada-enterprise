use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState};

/// GET /api/v1/notifications/settings
pub async fn list_notification_settings(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let settings = state.storage.get_all_notification_settings().await?;
    Ok(Json(settings))
}

/// POST /api/v1/notifications/settings
pub async fn create_notification_setting(
    State(state): State<AppState>,
    Json(req): Json<CreateNotificationSettingRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate channel_type
    if req.channel_type != "email" && req.channel_type != "webhook" {
        return Err(AppError(anyhow::anyhow!(
            "channel_type must be 'email' or 'webhook'"
        )));
    }

    // Validate config is valid JSON array
    let targets: Vec<String> = match serde_json::from_str(&req.config) {
        Ok(v) => v,
        Err(_) => {
            return Err(AppError(anyhow::anyhow!(
                "config must be a JSON array of strings (email addresses or webhook URLs)"
            )));
        }
    };

    if targets.is_empty() {
        return Err(AppError(anyhow::anyhow!("config array must not be empty")));
    }

    // Validate individual entries based on channel type
    if req.channel_type == "email" {
        for entry in &targets {
            if !entry.contains('@') || !entry.contains('.') {
                return Err(AppError(anyhow::anyhow!(
                    "invalid email address: {}",
                    entry
                )));
            }
        }
    } else if req.channel_type == "webhook" {
        for entry in &targets {
            if !entry.starts_with("http://") && !entry.starts_with("https://") {
                return Err(AppError(anyhow::anyhow!(
                    "invalid webhook URL (must start with http:// or https://): {}",
                    entry
                )));
            }
        }
    }

    let id = state
        .storage
        .create_notification_setting(&req.source_key, &req.channel_type, &req.config, req.enabled)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "source_key": req.source_key,
            "channel_type": req.channel_type,
            "config": req.config,
            "enabled": req.enabled,
        })),
    ))
}

/// PUT /api/v1/notifications/settings/{id}
pub async fn update_notification_setting(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateNotificationSettingRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate channel_type if provided
    if let Some(ref ct) = req.channel_type {
        if ct != "email" && ct != "webhook" {
            return Err(AppError(anyhow::anyhow!(
                "channel_type must be 'email' or 'webhook'"
            )));
        }
    }

    // Validate config if provided
    if let Some(ref cfg) = req.config {
        let targets: Vec<String> = match serde_json::from_str(cfg) {
            Ok(v) => v,
            Err(_) => {
                return Err(AppError(anyhow::anyhow!(
                    "config must be a JSON array of strings"
                )));
            }
        };
        if targets.is_empty() {
            return Err(AppError(anyhow::anyhow!(
                "config array must not be empty"
            )));
        }
    }

    state
        .storage
        .update_notification_setting(
            id,
            req.source_key.as_deref(),
            req.channel_type.as_deref(),
            req.config.as_deref(),
            req.enabled,
        )
        .await?;

    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/v1/notifications/settings/{id}
pub async fn delete_notification_setting(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.delete_notification_setting(id).await?;
    Ok(Json(json!({"ok": true})))
}

/// POST /api/v1/notifications/test
///
/// Sends a test notification to validate configuration without creating an actual alarm.
pub async fn test_notification(
    State(state): State<AppState>,
    Json(req): Json<TestNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.channel_type != "email" && req.channel_type != "webhook" {
        return Err(AppError(anyhow::anyhow!(
            "channel_type must be 'email' or 'webhook'"
        )));
    }

    let targets: Vec<String> = match serde_json::from_str(&req.config) {
        Ok(v) => v,
        Err(_) => {
            return Err(AppError(anyhow::anyhow!(
                "config must be a JSON array of strings"
            )));
        }
    };

    if targets.is_empty() {
        return Err(AppError(anyhow::anyhow!("config array must not be empty")));
    }

    let source_key = req.source_key.as_deref().unwrap_or("test_source");
    let source_name = format!("Test Notification ({})", source_key);

    match req.channel_type.as_str() {
        "email" => {
            let alarm_data = crate::notification::email::AlarmEmailData {
                source_key: source_key.to_string(),
                source_name: source_name.clone(),
                severity: "warning".to_string(),
                value: 42.0,
                threshold: 40.0,
                message: "This is a test alarm notification".to_string(),
                unit: "test".to_string(),
                triggered_at: crate::notification::webhook::format_triggered_at(
                    crate::timeutil::now_secs(),
                ),
            };

            match &state.smtp_config {
                Some(smtp_cfg) => {
                    let notifier = crate::notification::email::EmailNotifier::new(smtp_cfg)?;
                    let mut results = Vec::new();
                    for to in &targets {
                        match notifier.send_alarm_email(to, &alarm_data).await {
                            Ok(_) => results.push(json!({"to": to, "status": "sent"})),
                            Err(e) => results.push(json!({"to": to, "status": "error", "error": e.to_string()})),
                        }
                    }
                    Ok(Json(json!({
                        "channel": "email",
                        "results": results,
                    })))
                }
                None => Err(AppError(anyhow::anyhow!(
                    "SMTP is not configured in config.yaml"
                ))),
            }
        }
        "webhook" => {
            let payload = crate::notification::webhook::WebhookPayload {
                severity: "warning".to_string(),
                source_key: source_key.to_string(),
                source_name: source_name.clone(),
                value: 42.0,
                threshold: 40.0,
                message: "This is a test alarm notification".to_string(),
                triggered_at: crate::notification::webhook::format_triggered_at(
                    crate::timeutil::now_secs(),
                ),
                device_info: source_key.to_string(),
            };

            let mut results = Vec::new();
            for url in &targets {
                match crate::notification::webhook::send_webhook_alarm(url, &payload).await {
                    Ok(_) => results.push(json!({"url": url, "status": "sent"})),
                    Err(e) => results.push(json!({"url": url, "status": "error", "error": e.to_string()})),
                }
            }

            Ok(Json(json!({
                "channel": "webhook",
                "results": results,
            })))
        }
        _ => unreachable!(),
    }
}
