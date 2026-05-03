use axum::extract::{Extension, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use super::{AppState, AppError, AuthUser};

/// GET /api/settings/branding — get branding settings with defaults
pub async fn get_branding(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Trial tier cannot use custom branding — return hardcoded defaults
    if !state.license_tier.white_label() {
        return Ok(Json(json!({
            "title": "Voltrus Lite",
            "subtitle": "Monitoring Dashboard",
            "logo": "",
            "theme_level": "none",
        })));
    }

    let settings = state
        .storage
        .get_settings(&["brand_title", "brand_subtitle", "brand_logo"])
        .await?;

    let mut title = "Voltrus Lite".to_string();
    let mut subtitle = "Monitoring Dashboard".to_string();
    let mut logo = "".to_string();

    for s in &settings {
        match s.key.as_str() {
            "brand_title" => title = s.value.clone(),
            "brand_subtitle" => subtitle = s.value.clone(),
            "brand_logo" => logo = s.value.clone(),
            _ => {}
        }
    }

    let theme_level = if state.license_tier.theme_full() {
        "full"
    } else {
        "basic"
    };

    Ok(Json(json!({
        "title": title,
        "subtitle": subtitle,
        "logo": logo,
        "theme_level": theme_level,
    })))
}

/// PUT /api/settings/branding — update branding settings
pub async fn update_branding(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.white_label() {
        return Err(AppError(anyhow::anyhow!("White-label branding requires Starter tier or above. Upgrade at https://voltrus.id")));
    }

    // Starter tier can only update logo (basic branding).
    // Professional+ can update title, subtitle, and logo (full theme).
    let full_theme = state.license_tier.theme_full();

    if !full_theme {
        // Starter: only logo updates allowed
        if body.get("title").is_some() || body.get("subtitle").is_some() {
            return Err(AppError(anyhow::anyhow!(
                "Full theme customization requires Professional tier. Starter can only update the logo."
            )));
        }
    }

    if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
        let sanitized = crate::security::sanitize_brand_input(title);
        state.storage.upsert_setting("brand_title", &sanitized).await?;
    }
    if let Some(subtitle) = body.get("subtitle").and_then(|v| v.as_str()) {
        let sanitized = crate::security::sanitize_brand_input(subtitle);
        state
            .storage
            .upsert_setting("brand_subtitle", &sanitized)
            .await?;
    }
    if let Some(logo) = body.get("logo").and_then(|v| v.as_str()) {
        let sanitized = crate::security::sanitize_logo_input(logo);
        state.storage.upsert_setting("brand_logo", &sanitized).await?;
    }

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_branding".to_string(),
        username: auth.username.clone(),
        resource: "settings:branding".to_string(),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}

/// GET /api/settings/alarm-cooldown — get alarm cooldown in seconds
pub async fn get_alarm_cooldown(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let settings = state.storage.get_settings(&["alarm_cooldown_secs"]).await?;
    let secs: i64 = settings
        .into_iter()
        .next()
        .and_then(|s| s.value.parse().ok())
        .unwrap_or(300);
    Ok(Json(json!({ "secs": secs })))
}

/// PUT /api/settings/alarm-cooldown — set alarm cooldown in seconds
pub async fn update_alarm_cooldown(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let secs = body
        .get("secs")
        .and_then(|v| v.as_i64())
        .unwrap_or(300);
    let clamped = secs.clamp(0, 86400);
    state.storage.upsert_setting("alarm_cooldown_secs", &clamped.to_string()).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_alarm_cooldown".to_string(),
        username: auth.username.clone(),
        resource: "settings:alarm-cooldown".to_string(),
        details: Some(format!("secs={}", clamped)),
    });

    Ok(Json(json!({"ok": true, "secs": clamped})))
}
