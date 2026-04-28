use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use super::{AppState, AppError};

/// GET /api/settings/branding — get branding settings with defaults
pub async fn get_branding(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
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

    Ok(Json(json!({
        "title": title,
        "subtitle": subtitle,
        "logo": logo,
    })))
}

/// PUT /api/settings/branding — update branding settings
pub async fn update_branding(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
        state.storage.upsert_setting("brand_title", title).await?;
    }
    if let Some(subtitle) = body.get("subtitle").and_then(|v| v.as_str()) {
        state
            .storage
            .upsert_setting("brand_subtitle", subtitle)
            .await?;
    }
    if let Some(logo) = body.get("logo").and_then(|v| v.as_str()) {
        state.storage.upsert_setting("brand_logo", logo).await?;
    }

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
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let secs = body
        .get("secs")
        .and_then(|v| v.as_i64())
        .unwrap_or(300);
    let clamped = secs.clamp(0, 86400);
    state.storage.upsert_setting("alarm_cooldown_secs", &clamped.to_string()).await?;
    Ok(Json(json!({"ok": true, "secs": clamped})))
}
