use axum::Json;
use axum::extract::{Extension, State};
use axum::response::IntoResponse;
use serde_json::json;

use voltrus::api::{AppError, AppState, AuthUser};
use voltrus::security::MAX_BRAND_LENGTH;

/// Longest logo value stored, in bytes (a URL or a `data:` URL), the same limit the core
/// sanitizer cuts at. An uploaded image is base64 encoded, so this is about 36 KB of image.
/// Longer values are refused instead of cut, which would break the image.
pub const MAX_LOGO_BYTES: usize = 50_000;

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

    let settings = state
        .storage
        .get_settings(&[
            "brand_title",
            "brand_subtitle",
            "brand_logo",
            "brand_accent",
        ])
        .await?;

    let mut title = "Voltrus".to_string();
    let mut subtitle = "Monitoring Dashboard".to_string();
    let mut logo = "".to_string();
    let mut accent = "".to_string();

    for s in &settings {
        match s.key.as_str() {
            "brand_title" => title = s.value.clone(),
            "brand_subtitle" => subtitle = s.value.clone(),
            "brand_logo" => logo = s.value.clone(),
            "brand_accent" => accent = s.value.clone(),
            _ => {}
        }
    }

    let full_theme = state.tier().theme_full();
    let theme_level = if full_theme { "full" } else { "basic" };

    Ok(Json(json!({
        "title": title,
        "subtitle": subtitle,
        "logo": logo,
        // The accent colour is part of the theme: a logo-only grant shows the default.
        "accent": if full_theme { accent } else { String::new() },
        "theme_level": theme_level,
    })))
}

/// Cut `s` to at most `max` bytes on a character boundary. The core sanitizer cuts by
/// byte index, which panics inside a multi-byte character, so text reaches it short
/// enough already.
fn clamp_utf8(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// An accent colour is `#rrggbb` (case-insensitive), stored lower case; an empty
/// string resets it to the theme default. Anything else is refused, because the
/// value is written into a CSS variable.
fn parse_accent(value: &str) -> Result<String, AppError> {
    let v = value.trim();
    if v.is_empty() {
        return Ok(String::new());
    }
    let hex = v.strip_prefix('#').unwrap_or("");
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(format!("#{}", hex.to_ascii_lowercase()))
    } else {
        Err(AppError::bad_request(
            "accent must be a colour like #0055aa, or empty to reset it",
        ))
    }
}

/// PUT /api/settings/branding — update branding settings
pub async fn update_branding(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    if !state.white_label() {
        return Err(AppError::forbidden(
            "White-label branding requires an Integrator or Enterprise licence",
        ));
    }

    // Logo-only entitlements (legacy Starter/Edge keys) may update the logo; the
    // title, subtitle, accent colour and theme need theme_full (Integrator and above).

    let full_theme = state.tier().theme_full();

    if !full_theme
        && (body.get("title").is_some()
            || body.get("subtitle").is_some()
            || body.get("accent").is_some())
    {
        return Err(AppError::forbidden(
            "Full theme customization needs an Integrator or Enterprise licence.",
        ));
    }

    // Validate everything before writing anything, so a refused request changes nothing.
    let logo = body.get("logo").and_then(|v| v.as_str()).map(str::trim);
    if let Some(logo) = logo
        && logo.len() > MAX_LOGO_BYTES
    {
        return Err(AppError::bad_request(format!(
            "Logo is too large: at most {MAX_LOGO_BYTES} bytes (an uploaded image of about 36 KB). Use a smaller image or a URL."
        )));
    }
    let accent = match body.get("accent").and_then(|v| v.as_str()) {
        Some(a) => Some(parse_accent(a)?),
        None => None,
    };

    if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
        let sanitized =
            voltrus::security::sanitize_brand_input(clamp_utf8(title, MAX_BRAND_LENGTH));
        state
            .storage
            .upsert_setting("brand_title", &sanitized)
            .await?;
    }
    if let Some(subtitle) = body.get("subtitle").and_then(|v| v.as_str()) {
        let sanitized =
            voltrus::security::sanitize_brand_input(clamp_utf8(subtitle, MAX_BRAND_LENGTH));
        state
            .storage
            .upsert_setting("brand_subtitle", &sanitized)
            .await?;
    }
    if let Some(logo) = logo {
        let sanitized = voltrus::security::sanitize_logo_input(logo)
            .map_err(|e| AppError::bad_request(e.1))?;
        state
            .storage
            .upsert_setting("brand_logo", &sanitized)
            .await?;
    }
    if let Some(accent) = accent {
        state
            .storage
            .upsert_setting("brand_accent", &accent)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_never_cuts_inside_a_character() {
        let title = format!("a{}", "Ä".repeat(150));
        let cut = clamp_utf8(&title, MAX_BRAND_LENGTH);
        assert!(cut.len() <= MAX_BRAND_LENGTH);
        assert_eq!(cut.len(), MAX_BRAND_LENGTH - 1);
        // The core sanitizer must not panic on what we hand it.
        let sanitized = voltrus::security::sanitize_brand_input(cut);
        assert!(sanitized.starts_with('a'));
        assert_eq!(clamp_utf8("Acme", MAX_BRAND_LENGTH), "Acme");
    }

    #[test]
    fn accent_accepts_hex_colours_and_reset_only() {
        assert_eq!(parse_accent("#0055AA").unwrap(), "#0055aa");
        assert_eq!(parse_accent(" #a1b2c3 ").unwrap(), "#a1b2c3");
        assert_eq!(parse_accent("").unwrap(), "");
        for bad in [
            "red",
            "#fff",
            "0055aa",
            "#0055aa;background:url(x)",
            "#gggggg",
            "#0055aa00",
        ] {
            assert!(parse_accent(bad).is_err(), "{bad}");
        }
    }
}
