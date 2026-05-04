use axum::Json;
use axum::body::Body;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use super::{AppError, AppState, AuthUser};

/// GET /api/settings/branding — get branding settings with defaults
#[utoipa::path(
    get,
    path = "/api/v1/settings/branding",
    responses((status = 200, body = crate::models::BrandingSettings)),
    tag = "Settings"
)]
pub async fn get_branding(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
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
#[utoipa::path(
    put,
    path = "/api/v1/settings/branding",
    responses((status = 200)),
    tag = "Settings",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_branding(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.white_label() {
        return Err(AppError(anyhow::anyhow!(
            "White-label branding requires Starter tier or above. Upgrade at https://voltrus.id"
        )));
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
        state
            .storage
            .upsert_setting("brand_title", &sanitized)
            .await?;
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
        state
            .storage
            .upsert_setting("brand_logo", &sanitized)
            .await?;
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
#[utoipa::path(
    get,
    path = "/api/v1/settings/alarm-cooldown",
    responses((status = 200)),
    tag = "Settings",
    security(("api_key" = []), ("cookie_auth" = []))
)]
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
#[utoipa::path(
    put,
    path = "/api/v1/settings/alarm-cooldown",
    responses((status = 200)),
    tag = "Settings",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_alarm_cooldown(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let secs = body.get("secs").and_then(|v| v.as_i64()).unwrap_or(300);
    let clamped = secs.clamp(0, 86400);
    state
        .storage
        .upsert_setting("alarm_cooldown_secs", &clamped.to_string())
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_alarm_cooldown".to_string(),
        username: auth.username.clone(),
        resource: "settings:alarm-cooldown".to_string(),
        details: Some(format!("secs={}", clamped)),
    });

    Ok(Json(json!({"ok": true, "secs": clamped})))
}

/// GET /api/v1/settings/ui-theme
#[utoipa::path(
    get,
    path = "/api/v1/settings/ui-theme",
    responses((status = 200)),
    tag = "Settings",
    security(("cookie_auth" = []))
)]
pub async fn get_ui_theme(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Response<Body>, AppError> {
    let theme = state
        .storage
        .get_user_preference(&auth.username, "ui_theme")
        .await?
        .unwrap_or_else(|| "dark".to_string());
    Ok(Json(json!({ "theme": theme })).into_response())
}

/// PUT /api/v1/settings/ui-theme
#[utoipa::path(
    put,
    path = "/api/v1/settings/ui-theme",
    responses((status = 200)),
    tag = "Settings",
    security(("cookie_auth" = []))
)]
pub async fn update_ui_theme(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response<Body>, AppError> {
    let theme = body.get("theme").and_then(|v| v.as_str()).unwrap_or("dark");
    let valid = ["light", "dark", "system"].contains(&theme);
    if !valid {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid theme. Must be 'light', 'dark', or 'system'."})),
        )
            .into_response());
    }
    state
        .storage
        .set_user_preference(&auth.username, "ui_theme", theme)
        .await?;
    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_ui_theme".to_string(),
        username: auth.username.clone(),
        resource: "settings:ui-theme".to_string(),
        details: Some(format!("theme={}", theme)),
    });
    Ok(Json(json!({ "ok": true, "theme": theme })).into_response())
}

/// GET /api/v1/settings/ui-lang
#[utoipa::path(
    get,
    path = "/api/v1/settings/ui-lang",
    responses((status = 200)),
    tag = "Settings",
    security(("cookie_auth" = []))
)]
pub async fn get_ui_lang(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Response<Body>, AppError> {
    let lang = state
        .storage
        .get_user_preference(&auth.username, "ui_lang")
        .await?
        .unwrap_or_else(|| "en".to_string());
    Ok(Json(json!({ "lang": lang })).into_response())
}

/// PUT /api/v1/settings/ui-lang
#[utoipa::path(
    put,
    path = "/api/v1/settings/ui-lang",
    responses((status = 200)),
    tag = "Settings",
    security(("cookie_auth" = []))
)]
pub async fn update_ui_lang(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response<Body>, AppError> {
    let lang = body.get("lang").and_then(|v| v.as_str()).unwrap_or("en");
    let valid = ["en", "id", "zh", "de"].contains(&lang);
    if !valid {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid language. Must be one of: en, id, zh, de."})),
        )
            .into_response());
    }
    state
        .storage
        .set_user_preference(&auth.username, "ui_lang", lang)
        .await?;
    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_ui_lang".to_string(),
        username: auth.username.clone(),
        resource: "settings:ui-lang".to_string(),
        details: Some(format!("lang={}", lang)),
    });
    Ok(Json(json!({ "ok": true, "lang": lang })).into_response())
}
