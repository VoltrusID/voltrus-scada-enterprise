use axum::Json;
use axum::extract::{Extension, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use serde_json::json;

use voltrus::api::{AppError, AppState, AuthUser};
use voltrus::security::MAX_BRAND_LENGTH;

use super::theme::{self, MAX_FONT_BYTES, Theme};

const KEYS: &[&str] = &[
    "brand_title",
    "brand_subtitle",
    "brand_logo",
    "brand_accent",
    "brand_theme",
    "brand_font_name",
    "brand_font_mime",
    "brand_font_sha",
];

/// Where the web client loads an uploaded font from (same origin, so the CSP's
/// `font-src 'self'` covers it). `v` changes with the file, so it can be cached.
const FONT_PATH: &str = "/api/v1/settings/branding/font";

fn stored_theme(raw: &str) -> Theme {
    serde_json::from_str(raw).unwrap_or_default()
}

/// GET /api/settings/branding — get branding settings with defaults
pub async fn get_branding(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    // Without white-label entitlement, return the hardcoded defaults.
    if !state.white_label() {
        return Ok(Json(json!({
            "title": "Voltrus",
            "subtitle": "Monitoring Dashboard",
            "logo": "",
            "accent": "",
            "theme_level": "none",
        })));
    }

    let settings = state.storage.get_settings(KEYS).await?;
    let get = |key: &str| {
        settings
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.value.clone())
            .unwrap_or_default()
    };
    let title = Some(get("brand_title"))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Voltrus".into());
    let subtitle = Some(get("brand_subtitle"))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Monitoring Dashboard".into());

    let full_theme = state.tier().theme_full();
    if !full_theme {
        // A logo-only grant shows the Voltrus colours and font.
        return Ok(Json(json!({
            "title": title,
            "subtitle": subtitle,
            "logo": get("brand_logo"),
            "accent": "",
            "theme_level": "basic",
        })));
    }
    let theme = stored_theme(&get("brand_theme"));
    let sha = get("brand_font_sha");
    let font_url =
        (theme.font == "custom" && !sha.is_empty()).then(|| format!("{FONT_PATH}?v={sha}"));
    Ok(Json(json!({
        "title": title,
        "subtitle": subtitle,
        "logo": get("brand_logo"),
        "accent": get("brand_accent"),
        "theme_level": "full",
        "theme": {
            "dark": theme.dark,
            "light": theme.light,
            // An uploaded font that was removed falls back to the default.
            "font": if theme.font == "custom" && font_url.is_none() { String::new() } else { theme.font.clone() },
        },
        "font_url": font_url,
        "font_name": if sha.is_empty() { String::new() } else { get("brand_font_name") },
        "fonts": theme::FONTS.iter().map(|(id, label)| json!({"id": id, "label": label})).collect::<Vec<_>>(),
    })))
}

/// Cut `s` to at most `max` characters. Kept for callers that cut before the core
/// sanitizer (which now cuts by characters itself).
fn clamp_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// PUT /api/settings/branding — update branding settings
pub async fn update_branding(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    if !state.white_label() {
        return Err(AppError::forbidden(
            "White-label branding requires an Integrator or Enterprise licence",
        ));
    }

    // Logo-only entitlements (legacy Starter/Edge keys) may update the logo; the
    // title, subtitle, colours, theme and font need theme_full (Integrator and above).
    let full_theme = state.tier().theme_full();
    if !full_theme
        && ["title", "subtitle", "accent", "theme"]
            .iter()
            .any(|k| body.get(*k).is_some())
    {
        return Err(AppError::forbidden(
            "Full theme customization needs an Integrator or Enterprise licence.",
        ));
    }

    // Validate everything before writing anything, so a refused request changes nothing.
    let logo = match body.get("logo").and_then(|v| v.as_str()) {
        Some(logo) => Some(voltrus::security::sanitize_logo_input(logo)?),
        None => None,
    };
    let current = state
        .storage
        .get_settings(&["brand_accent", "brand_theme"])
        .await?;
    let current_of = |key: &str| {
        current
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.value.clone())
            .unwrap_or_default()
    };
    let accent = match body.get("accent").and_then(|v| v.as_str()) {
        Some(a) => Some(
            theme::normalize_colour(&voltrus::security::sanitize_brand_color(a)?)
                .map_err(AppError::bad_request)?,
        ),
        None => None,
    };
    let new_theme = match body.get("theme") {
        Some(v) if v.is_null() => Some(Theme::default()),
        Some(v) => Some(
            serde_json::from_value::<Theme>(v.clone())
                .map_err(|e| AppError::bad_request(format!("theme: {e}")))?,
        ),
        None => None,
    };
    // The accent and the theme are judged together: a new accent against the stored
    // theme, a new theme against the stored (or new) accent.
    let effective_accent = accent.clone().unwrap_or_else(|| current_of("brand_accent"));
    let new_theme = match new_theme {
        Some(t) => Some(
            t.validated(&effective_accent)
                .map_err(AppError::bad_request)?,
        ),
        None => {
            if accent.is_some() {
                stored_theme(&current_of("brand_theme"))
                    .validated(&effective_accent)
                    .map_err(AppError::bad_request)?;
            }
            None
        }
    };

    if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
        let sanitized =
            voltrus::security::sanitize_brand_input(&clamp_chars(title, MAX_BRAND_LENGTH));
        state
            .storage
            .upsert_setting("brand_title", &sanitized)
            .await?;
    }
    if let Some(subtitle) = body.get("subtitle").and_then(|v| v.as_str()) {
        let sanitized =
            voltrus::security::sanitize_brand_input(&clamp_chars(subtitle, MAX_BRAND_LENGTH));
        state
            .storage
            .upsert_setting("brand_subtitle", &sanitized)
            .await?;
    }
    if let Some(logo) = logo {
        state.storage.upsert_setting("brand_logo", &logo).await?;
    }
    if let Some(accent) = &accent {
        state.storage.upsert_setting("brand_accent", accent).await?;
    }
    if let Some(theme) = &new_theme {
        let value = if theme.is_empty() {
            String::new()
        } else {
            serde_json::to_string(theme).unwrap_or_default()
        };
        state.storage.upsert_setting("brand_theme", &value).await?;
    }

    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "update_branding".to_string(),
        username: auth.username.clone(),
        resource: "settings:branding".to_string(),
        details: None,
    });

    Ok(Json(json!({"ok": true})).into_response())
}

/// GET /api/v1/settings/branding/font — the uploaded font file (public: the
/// login page uses it).
pub async fn get_font(State(state): State<AppState>) -> Result<Response, AppError> {
    let not_found = || {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "no brand font"})),
        )
            .into_response()
    };
    if !state.white_label() || !state.tier().theme_full() {
        return Ok(not_found());
    }
    let settings = state
        .storage
        .get_settings(&["brand_font_file", "brand_font_mime"])
        .await?;
    let get = |key: &str| {
        settings
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.value.clone())
            .unwrap_or_default()
    };
    let Ok(data) = base64::engine::general_purpose::STANDARD.decode(get("brand_font_file")) else {
        return Ok(not_found());
    };
    // Served only as one of the font types its signature was checked against.
    let Some(mime) = theme::font_mime(&data) else {
        return Ok(not_found());
    };
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime.to_string()),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
            (
                header::CACHE_CONTROL,
                "public, max-age=31536000, immutable".to_string(),
            ),
        ],
        data,
    )
        .into_response())
}

/// PUT /api/v1/settings/branding/font — upload a font file: `{"name": "Acme Sans",
/// "data": "<base64>"}`. WOFF2, WOFF, TTF or OTF, at most 256 KB. Selects it.
pub async fn put_font(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    if !state.white_label() || !state.tier().theme_full() {
        return Err(AppError::forbidden(
            "A brand font needs an Integrator or Enterprise licence.",
        ));
    }
    let b64 = body.get("data").and_then(|v| v.as_str()).unwrap_or("");
    let b64 = b64.split_once("base64,").map_or(b64, |(_, d)| d).trim();
    // Refuse an oversized upload before decoding it.
    if b64.len() > MAX_FONT_BYTES.div_ceil(3) * 4 + 4 {
        return Err(AppError::bad_request(format!(
            "Font file is too large: at most {} KB",
            MAX_FONT_BYTES / 1024
        )));
    }
    let data = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|_| AppError::bad_request("data must be the font file, base64 encoded"))?;
    if data.len() > MAX_FONT_BYTES {
        return Err(AppError::bad_request(format!(
            "Font file is too large: at most {} KB",
            MAX_FONT_BYTES / 1024
        )));
    }
    let Some(mime) = theme::font_mime(&data) else {
        return Err(AppError::bad_request(
            "Not a font file: upload a WOFF2, WOFF, TTF or OTF file",
        ));
    };
    let name = voltrus::security::sanitize_brand_input(&clamp_chars(
        body.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Custom font"),
        60,
    ));
    let sha = {
        use sha2::{Digest, Sha256};
        hex::encode(&Sha256::digest(&data)[..8])
    };
    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
    state
        .storage
        .upsert_setting("brand_font_file", &encoded)
        .await?;
    state
        .storage
        .upsert_setting("brand_font_mime", mime)
        .await?;
    state
        .storage
        .upsert_setting(
            "brand_font_name",
            if name.is_empty() {
                "Custom font"
            } else {
                &name
            },
        )
        .await?;
    state.storage.upsert_setting("brand_font_sha", &sha).await?;
    // Uploading a font selects it.
    let current = state.storage.get_settings(&["brand_theme"]).await?;
    let mut theme = stored_theme(&current.first().map(|s| s.value.as_str()).unwrap_or(""));
    theme.font = "custom".into();
    state
        .storage
        .upsert_setting(
            "brand_theme",
            &serde_json::to_string(&theme).unwrap_or_default(),
        )
        .await?;
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "update_branding_font".to_string(),
        username: auth.username.clone(),
        resource: "settings:branding".to_string(),
        details: Some(format!("{mime}, {} bytes, sha256 {sha}", data.len())),
    });
    Ok(
        Json(json!({"ok": true, "font_url": format!("{FONT_PATH}?v={sha}"), "mime": mime}))
            .into_response(),
    )
}

/// DELETE /api/v1/settings/branding/font — remove the uploaded font; the theme
/// falls back to the default font.
pub async fn delete_font(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Response, AppError> {
    if !state.white_label() || !state.tier().theme_full() {
        return Err(AppError::forbidden(
            "A brand font needs an Integrator or Enterprise licence.",
        ));
    }
    for key in [
        "brand_font_file",
        "brand_font_mime",
        "brand_font_name",
        "brand_font_sha",
    ] {
        state.storage.upsert_setting(key, "").await?;
    }
    let current = state.storage.get_settings(&["brand_theme"]).await?;
    let mut theme = stored_theme(&current.first().map(|s| s.value.as_str()).unwrap_or(""));
    if theme.font == "custom" {
        theme.font = String::new();
        let value = if theme.is_empty() {
            String::new()
        } else {
            serde_json::to_string(&theme).unwrap_or_default()
        };
        state.storage.upsert_setting("brand_theme", &value).await?;
    }
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "delete_branding_font".to_string(),
        username: auth.username.clone(),
        resource: "settings:branding".to_string(),
        details: None,
    });
    Ok(Json(json!({"ok": true})).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_never_cuts_inside_a_character() {
        let title = format!("a{}", "Ä".repeat(250));
        let cut = clamp_chars(&title, MAX_BRAND_LENGTH);
        assert_eq!(cut.chars().count(), MAX_BRAND_LENGTH);
        let sanitized = voltrus::security::sanitize_brand_input(&cut);
        assert!(sanitized.starts_with('a'));
        assert_eq!(clamp_chars("Acme", MAX_BRAND_LENGTH), "Acme");
    }

    #[test]
    fn a_stored_theme_that_does_not_parse_is_the_default() {
        assert_eq!(stored_theme(""), Theme::default());
        assert_eq!(stored_theme("{not json"), Theme::default());
        let t = stored_theme(r##"{"dark":{"background":"#000000"},"font":"serif"}"##);
        assert_eq!(t.dark.background, "#000000");
        assert_eq!(t.font, "serif");
    }
}
