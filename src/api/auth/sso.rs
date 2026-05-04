use base64::Engine;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use super::*;

fn extract_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
        })
}

fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

// -- SSO request types --

#[derive(Debug, serde::Deserialize)]
pub struct SamlAcsRequest {
    pub saml_response: String,
    #[serde(default)]
    pub relay_state: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct LdapLoginRequest {
    pub username: String,
    pub password: String,
}

// -- SSO / SAML / LDAP endpoints (Enterprise-tier gated) --

/// GET /api/v1/auth/providers -- lists enabled auth providers.
pub async fn list_auth_providers(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let mut providers = vec![json!({
        "provider": "local",
        "enabled": true,
        "label": "Local Authentication",
    })];

    // OIDC provider info
    let oidc_enabled = state.oidc_config.enabled;
    let empty_issuer = String::new();
    let issuer = if oidc_enabled { &state.oidc_config.issuer } else { &empty_issuer };
    providers.push(json!({
        "provider": "oidc",
        "enabled": oidc_enabled,
        "label": "OpenID Connect",
        "issuer": issuer,
    }));

    // SAML provider info (Enterprise only)
    let saml_enabled = state.license_tier.sso_saml()
        && state.oidc_config.enabled; // reuse oidc_config check as proxy
    providers.push(json!({
        "provider": "saml",
        "enabled": saml_enabled,
        "label": "SAML 2.0",
    }));

    // LDAP provider info (Enterprise only)
    let ldap_enabled = state.license_tier.sso_saml()
        && state.oidc_config.enabled; // same proxy
    providers.push(json!({
        "provider": "ldap",
        "enabled": ldap_enabled,
        "label": "LDAP",
    }));

    Ok(Json(json!({"providers": providers})))
}

/// POST /api/v1/auth/saml/metadata -- returns SP metadata XML.
pub async fn saml_metadata(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    if !state.license_tier.sso_saml() {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "SAML SSO requires Enterprise license"})),
        )
            .into_response());
    }

    let config = crate::auth::saml::SamlConfig::default();
    // In production, load from settings KV
    let metadata_xml = crate::auth::saml::generate_sp_metadata(&config);

    Ok((
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "application/samlmetadata+xml",
        )],
        metadata_xml,
    )
        .into_response())
}

/// POST /api/v1/auth/saml/acs -- SAML Assertion Consumer Service.
pub async fn saml_acs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SamlAcsRequest>,
) -> Result<impl IntoResponse, AppError> {
    let source_ip = extract_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    if !state.license_tier.sso_saml() {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "SAML SSO requires Enterprise license"})),
        )
            .into_response());
    }

    // Parse the SAML response
    let assertion = crate::auth::saml::parse_saml_response(&body.saml_response)
        .map_err(|e| AppError(anyhow::anyhow!("SAML response parse error: {}", e)))?;

    // Validate the SAML response
    let decoded = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(&body.saml_response)
            .unwrap_or_default(),
    )
    .unwrap_or_default();
    crate::auth::saml::validate_saml_response(&decoded, "")
        .map_err(|e| AppError(anyhow::anyhow!("SAML validation error: {}", e)))?;

    tracing::info!("SAML ACS: authenticated user={}", assertion.name_id);

    // JIT provision the user
    let user = state
        .storage
        .get_or_create_user_by_external_id("saml", &assertion.name_id, "user")
        .await?;

    let _ = state.storage.insert_security_event(&crate::models::CreateSecurityEvent {
        event_type: "login_success".into(),
        severity: "info".into(),
        source_ip: source_ip.clone(),
        user_agent: user_agent.clone(),
        username: Some(user.username.clone()),
        details: Some("SAML login successful".into()),
    }).await;

    // Create a local session token
    let secret = &state.auth_secret;
    let access_token = create_token(&user.username, &user.role, secret);

    Ok(Json(json!({
        "ok": true,
        "username": user.username,
        "role": user.role,
        "token": access_token,
    }))
    .into_response())
}

/// POST /api/v1/auth/ldap/login -- LDAP authentication endpoint.
pub async fn ldap_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LdapLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let source_ip = extract_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    if !state.license_tier.sso_saml() {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "LDAP authentication requires Enterprise license"})),
        )
            .into_response());
    }

    if body.username.is_empty() || body.password.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "username and password are required"})),
        )
            .into_response());
    }

    // Construct the bind DN
    // Load LDAP config from settings KV in production
    let config = crate::auth::ldap::LdapConfig::default();
    let bind_dn = if config.bind_dn_format.is_empty() {
        format!("uid={},{}", body.username, config.base_dn)
    } else {
        config.bind_dn_format.replace("{username}", &body.username)
    };

    let server = if config.server_url.is_empty() {
        "localhost".to_string()
    } else {
        config.server_url.clone()
    };
    let port = if config.port == 0 { 389 } else { config.port };

    // Attempt LDAP authentication
    let result = crate::auth::ldap::ldap_authenticate(
        &server,
        port,
        &bind_dn,
        &body.password,
        config.use_tls,
    )
    .await
    .map_err(|e| AppError(anyhow::anyhow!("LDAP auth error: {}", e)))?;

    if !result.success {
        tracing::warn!("LDAP login failed for user: {}", body.username);
        let _ = state.storage.insert_security_event(&crate::models::CreateSecurityEvent {
            event_type: "login_failed".into(),
            severity: "warning".into(),
            source_ip: source_ip.clone(),
            user_agent: user_agent.clone(),
            username: Some(body.username.clone()),
            details: Some("Invalid LDAP credentials".into()),
        }).await;
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid LDAP credentials"})),
        )
            .into_response());
    }

    // JIT provision the user
    let user = state
        .storage
        .get_or_create_user_by_external_id("ldap", &body.username, "user")
        .await?;

    // Create a local session token
    let secret = &state.auth_secret;
    let access_token = create_token(&user.username, &user.role, secret);

    tracing::info!("LDAP login successful: user={}", body.username);

    let _ = state.storage.insert_security_event(&crate::models::CreateSecurityEvent {
        event_type: "login_success".into(),
        severity: "info".into(),
        source_ip: source_ip.clone(),
        user_agent: user_agent.clone(),
        username: Some(body.username.clone()),
        details: Some("LDAP login successful".into()),
    }).await;

    Ok(Json(json!({
        "ok": true,
        "username": user.username,
        "role": user.role,
        "token": access_token,
    }))
    .into_response())
}
