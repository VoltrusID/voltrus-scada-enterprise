use axum::Json;
use axum::extract::{Extension, State};
use axum::response::IntoResponse;
use serde_json::json;

use voltrus::api::{AppError, AppState, AuthUser};

/// GET /api/settings/branding — get branding settings with defaults
pub async fn get_branding(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    // Engineering tier cannot use custom branding — return hardcoded defaults
    if !state.tier().white_label() {
        return Ok(Json(json!({
            "title": "Voltrus",
            "subtitle": "Monitoring Dashboard",
            "logo": "",
            "theme_level": "none",
        })));
    }

    let settings = state
        .storage
        .get_settings(&["brand_title", "brand_subtitle", "brand_logo"])
        .await?;

    let mut title = "Voltrus".to_string();
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

    let theme_level = if state.tier().theme_full() {
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
    if !state.tier().white_label() {
        return Err(AppError(anyhow::anyhow!(
            "White-label branding requires an Integrator or Enterprise licence"
        )));
    }

    // Integrator licences may update the logo only; Enterprise gets the full theme.

    let full_theme = state.tier().theme_full();

    if !full_theme {
        // logo only
        if body.get("title").is_some() || body.get("subtitle").is_some() {
            return Err(AppError(anyhow::anyhow!(
                "Full theme customization needs an Integrator or Enterprise licence."
            )));
        }
    }

    if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
        let sanitized = voltrus::security::sanitize_brand_input(title);
        state
            .storage
            .upsert_setting("brand_title", &sanitized)
            .await?;
    }
    if let Some(subtitle) = body.get("subtitle").and_then(|v| v.as_str()) {
        let sanitized = voltrus::security::sanitize_brand_input(subtitle);
        state
            .storage
            .upsert_setting("brand_subtitle", &sanitized)
            .await?;
    }
    if let Some(logo) = body.get("logo").and_then(|v| v.as_str()) {
        let sanitized = voltrus::security::sanitize_logo_input(logo);
        state
            .storage
            .upsert_setting("brand_logo", &sanitized)
            .await?;
    }

    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "update_branding".to_string(),
        username: auth.username.clone(),
        resource: "settings:branding".to_string(),
        details: None,
    });

    Ok(Json(json!({"ok": true})))
}
