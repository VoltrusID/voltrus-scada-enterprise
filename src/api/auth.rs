use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use base64::Engine;
use rand::Rng;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState};

const ACCESS_TOKEN_EXPIRY_MS: u64 = 15 * 60 * 1000; // 15 minutes
const REFRESH_TOKEN_EXPIRY_SECS: i64 = 7 * 24 * 60 * 60; // 7 days

pub fn hash_password(password: &str) -> String {
    use argon2::password_hash::{SaltString, PasswordHasher};
    use rand::Rng;
    let salt_bytes: [u8; 32] = rand::rng().random();
    let salt = SaltString::encode_b64(&salt_bytes).unwrap();
    let params = argon2::Params::new(19456, 2, 1, None).unwrap();
    let hasher = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let hash = hasher.hash_password(password.as_bytes(), &salt).unwrap();
    hash.to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::password_hash::PasswordVerifier;
    let parsed = argon2::PasswordHash::new(hash).ok();
    match parsed {
        Some(parsed) => argon2::Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        None => false,
    }
}

fn constant_time_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut r = 0u8;
    for (ca, cb) in a.bytes().zip(b.bytes()) {
        r |= ca ^ cb;
    }
    r == 0
}

// ── Refresh token helpers ──

fn generate_refresh_token() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_refresh_token(token: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn hmac_sign(payload: &str, secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn create_token(username: &str, role: &str, secret: &str) -> String {
    let exp = crate::timeutil::now_millis() as u64 + ACCESS_TOKEN_EXPIRY_MS;
    let payload =
        serde_json::to_string(&TokenClaims {
            username: username.to_string(),
            exp,
            role: role.to_string(),
        })
        .unwrap();
    let sig = hmac_sign(&payload, secret);
    format!(
        "{}.{}",
        base64::engine::general_purpose::STANDARD.encode(&payload),
        sig
    )
}

pub fn verify_token(token: &str, secret: &str) -> Option<TokenClaims> {
    let (payload_b64, sig) = token.split_once('.')?;
    let payload = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(payload_b64)
            .ok()?,
    )
    .ok()?;
    // Verify HMAC signature
    let expected = hmac_sign(&payload, secret);
    if !constant_time_equal(sig, &expected) {
        return None;
    }
    let claims: TokenClaims = serde_json::from_str(&payload).ok()?;
    let now = crate::timeutil::now_millis() as u64;
    if claims.exp < now {
        return None;
    }
    Some(claims)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = crate::models::LoginRequest,
    responses((status = 200, body = crate::models::LoginResponse), (status = 401)),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let rate_key_user = format!("login:{}", body.username);

    // Check username-based rate limit
    if !state.login_rate_limiter.is_allowed(&rate_key_user) {
        return Ok((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "Too many login attempts. Please try again later."})),
        )
            .into_response());
    }

    let start_time = std::time::Instant::now();
    let target_duration = std::time::Duration::from_millis(300);

    // Perform login logic
    let user = state.storage.get_user_by_username(&body.username).await?;
    let (response_status, response_json, new_jar) = match user {
        Some(user) => {
            // Check account lockout
            let now_secs = crate::timeutil::now_secs();
            if let Some(locked_until) = user.locked_until {
                if locked_until > now_secs {
                    let _ = state.storage.insert_security_event(&crate::models::CreateSecurityEvent {
                        event_type: "login_locked".into(),
                        severity: "warning".into(),
                        source_ip: None,
                        user_agent: None,
                        username: Some(body.username.clone()),
                        details: Some("Attempted login on locked account".into()),
                    }).await;
                    (StatusCode::FORBIDDEN, json!({"error": "Account locked. Please try again later."}), jar)
                } else {
                    check_password_and_login(state, body, user, jar, &rate_key_user, now_secs).await
                }
            } else {
                check_password_and_login(state, body, user, jar, &rate_key_user, now_secs).await
            }
        }
        None => {
            state.login_rate_limiter.record_attempt(&rate_key_user);
            let _ = state.storage.insert_security_event(&crate::models::CreateSecurityEvent {
                event_type: "login_failed".into(),
                severity: "warning".into(),
                source_ip: None,
                user_agent: None,
                username: Some(body.username.clone()),
                details: Some("Unknown username".into()),
            }).await;
            (StatusCode::UNAUTHORIZED, json!({"error": "Invalid credentials"}), jar)
        }
    };

    // Timing attack mitigation: ensure response takes at least target_duration
    let elapsed = start_time.elapsed();
    if elapsed < target_duration {
        tokio::time::sleep(target_duration - elapsed).await;
    }

    Ok((new_jar, (response_status, Json(response_json))).into_response())
}

async fn check_password_and_login(
    state: AppState,
    body: LoginRequest,
    user: User,
    jar: CookieJar,
    rate_key_user: &str,
    now_secs: i64,
) -> (StatusCode, serde_json::Value, CookieJar) {
    if !verify_password(&body.password, &user.password_hash) {
        state.login_rate_limiter.record_attempt(rate_key_user);
        let _ = state.storage.increment_failed_attempts(&body.username).await;

        // Lock account after 10 failed attempts
        if user.failed_attempts + 1 >= 10 {
            let lock_until = now_secs + 900; // 15 minutes
            let _ = state.storage.lock_account(&body.username, lock_until).await;
            tracing::warn!("Account locked for user {} due to too many failed attempts", body.username);
        }

        let _ = state.storage.insert_security_event(&crate::models::CreateSecurityEvent {
            event_type: "login_failed".into(),
            severity: "warning".into(),
            source_ip: None,
            user_agent: None,
            username: Some(body.username.clone()),
            details: Some(format!("Invalid password (attempt {})", user.failed_attempts + 1)),
        }).await;

        return (StatusCode::UNAUTHORIZED, json!({"error": "Invalid credentials"}), jar);
    }

    // Successful login — reset failed attempts and revoke old refresh tokens
    let _ = state.storage.reset_failed_attempts(&body.username).await;
    let _ = state.storage.revoke_all_user_refresh_tokens(&body.username).await;
    state.login_rate_limiter.cleanup();

    let secret = &state.auth_secret;
    let access_token = create_token(&body.username, &user.role, secret);

    // Generate refresh token
    let refresh_token_raw = generate_refresh_token();
    let refresh_hash = hash_refresh_token(&refresh_token_raw);
    let refresh_expires = crate::timeutil::now_secs() + REFRESH_TOKEN_EXPIRY_SECS;
    let _ = state.storage.create_refresh_token(&body.username, &refresh_hash, refresh_expires, crate::timeutil::now_secs()).await;

    let access_cookie = axum_extra::extract::cookie::Cookie::build((
        "voltrus_token",
        access_token.clone(),
    ))
    .http_only(true)
    .secure(state.cookie_secure)
    .same_site(axum_extra::extract::cookie::SameSite::Strict)
    .max_age(time::Duration::seconds(900)) // 15 minutes
    .path("/")
    .build();

    let refresh_cookie = axum_extra::extract::cookie::Cookie::build((
        "voltrus_refresh",
        refresh_token_raw,
    ))
    .http_only(true)
    .secure(state.cookie_secure)
    .same_site(axum_extra::extract::cookie::SameSite::Strict)
    .max_age(time::Duration::seconds(REFRESH_TOKEN_EXPIRY_SECS))
    .path("/")
    .build();

    let jar = jar.add(access_cookie).add(refresh_cookie);
    (
        StatusCode::OK,
        json!({
            "ok": true,
            "username": body.username,
            "role": user.role,
            "must_change_password": user.must_change_password,
        }),
        jar,
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses((status = 200)),
    tag = "Auth",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Revoke refresh token if present
    if let Some(refresh_cookie) = jar.get("voltrus_refresh") {
        let refresh_hash = hash_refresh_token(refresh_cookie.value());
        let _ = state.storage.revoke_refresh_token(&refresh_hash).await;
    }

    let access_cookie = axum_extra::extract::cookie::Cookie::build(("voltrus_token", ""))
        .max_age(time::Duration::seconds(0))
        .path("/")
        .build();
    let refresh_cookie = axum_extra::extract::cookie::Cookie::build(("voltrus_refresh", ""))
        .max_age(time::Duration::seconds(0))
        .path("/")
        .build();
    (jar.add(access_cookie).add(refresh_cookie), Json(json!({"ok": true})))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses((status = 200)),
    tag = "Auth",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn auth_me(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let token = jar
        .get("voltrus_token")
        .map(|c| c.value())
        .unwrap_or("");
    if token.is_empty() {
        return Json(json!({"authenticated": false}));
    }
    match verify_token(token, &state.auth_secret) {
        Some(claims) => Json(json!({"authenticated": true, "username": claims.username, "role": claims.role})),
        None => Json(json!({"authenticated": false})),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/csrf-token",
    responses((status = 200)),
    tag = "Auth",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_csrf_token(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let token = jar
        .get("voltrus_token")
        .map(|c| c.value())
        .unwrap_or("");
    if token.is_empty() {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "unauthorized"}))).into_response();
    }
    match verify_token(token, &state.auth_secret) {
        Some(_claims) => {
            let csrf = crate::security::generate_csrf_token();
            Json(json!({"csrf_token": csrf})).into_response()
        }
        None => (StatusCode::UNAUTHORIZED, Json(json!({"error": "unauthorized"}))).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    responses((status = 200, body = crate::models::LoginResponse), (status = 401)),
    tag = "Auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let refresh_token = jar.get("voltrus_refresh").map(|c| c.value()).unwrap_or("");
    if refresh_token.is_empty() {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        )
            .into_response());
    }

    let refresh_hash = hash_refresh_token(refresh_token);
    match state.storage.get_refresh_token_by_hash(&refresh_hash).await? {
        Some(rt) if rt.revoked_at.is_none() && rt.expires_at > crate::timeutil::now_secs() => {
            // Revoke old token (rotation)
            let _ = state.storage.revoke_refresh_token(&rt.token_hash).await;

            // Get user
            let user = state
                .storage
                .get_user_by_username(&rt.username)
                .await?
                .ok_or_else(|| anyhow::anyhow!("User not found"))?;

            // Create new tokens
            let new_access = create_token(&user.username, &user.role, &state.auth_secret);
            let new_refresh_raw = generate_refresh_token();
            let new_refresh_hash = hash_refresh_token(&new_refresh_raw);
            let refresh_expires = crate::timeutil::now_secs() + REFRESH_TOKEN_EXPIRY_SECS;
            let _ = state
                .storage
                .create_refresh_token(&user.username, &new_refresh_hash, refresh_expires, crate::timeutil::now_secs())
                .await;

            let access_cookie = axum_extra::extract::cookie::Cookie::build(("voltrus_token", new_access))
                .http_only(true)
                .secure(state.cookie_secure)
                .same_site(axum_extra::extract::cookie::SameSite::Strict)
                .max_age(time::Duration::seconds(900))
                .path("/")
                .build();

            let refresh_cookie = axum_extra::extract::cookie::Cookie::build(("voltrus_refresh", new_refresh_raw))
                .http_only(true)
                .secure(state.cookie_secure)
                .same_site(axum_extra::extract::cookie::SameSite::Strict)
                .max_age(time::Duration::seconds(REFRESH_TOKEN_EXPIRY_SECS))
                .path("/")
                .build();

            let jar = jar.add(access_cookie).add(refresh_cookie);
            Ok((jar, Json(json!({"ok": true}))).into_response())
        }
        _ => {
            // Clear cookies on invalid refresh
            let access_cookie = axum_extra::extract::cookie::Cookie::build(("voltrus_token", ""))
                .max_age(time::Duration::seconds(0))
                .path("/")
                .build();
            let refresh_cookie = axum_extra::extract::cookie::Cookie::build(("voltrus_refresh", ""))
                .max_age(time::Duration::seconds(0))
                .path("/")
                .build();
            let jar = jar.add(access_cookie).add(refresh_cookie);
            Ok((
                jar,
                (StatusCode::UNAUTHORIZED, Json(json!({"error": "unauthorized"}))),
            )
                .into_response())
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/seed",
    responses((status = 200), (status = 409)),
    tag = "Auth",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn seed(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    state.storage.seed_database().await?;
    Ok(Json(json!({"ok": true})))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    responses((status = 200), (status = 401)),
    tag = "Auth",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth): Extension<super::AuthUser>,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.new_password.len() < 8 {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"error": "Password must be at least 8 characters"}))).into_response());
    }
    let user = state.storage.get_user_by_username(&auth.username).await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;
    let hash = hash_password(&body.new_password);
    state.storage.update_user_password(user.id, &hash).await?;
    Ok(Json(json!({"ok": true})).into_response())
}

// ── User management handlers ──

pub async fn list_users(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let users = state.storage.list_users().await?;
    Ok(Json(users))
}

pub async fn create_user_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<super::AuthUser>,
    Json(body): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.username.is_empty() || body.password.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"error": "username and password are required"}))).into_response());
    }
    if body.role != "admin" && body.role != "user" {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"error": "role must be 'admin' or 'user'"}))).into_response());
    }
    if state.storage.get_user_by_username(&body.username).await?.is_some() {
        return Ok((StatusCode::CONFLICT, Json(json!({"error": "username already exists"}))).into_response());
    }
    let password_hash = hash_password(&body.password);
    let id = state.storage.create_user(&body.username, &password_hash, "", &body.role).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_user".to_string(),
        username: auth.username.clone(),
        resource: format!("user:{}", body.username),
        details: Some(format!("role={}", body.role)),
    });

    Ok(Json(json!({"ok": true, "id": id})).into_response())
}

pub async fn update_user_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<super::AuthUser>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.storage.get_user_by_id(id).await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;
    let mut details = Vec::new();
    if let Some(role) = body.role {
        if role != "admin" && role != "user" {
            return Ok((StatusCode::BAD_REQUEST, Json(json!({"error": "role must be 'admin' or 'user'"}))).into_response());
        }
        state.storage.update_user_role(id, &role).await?;
        details.push(format!("role={}", role));
    }
    if let Some(password) = body.password {
        if password.is_empty() {
            return Ok((StatusCode::BAD_REQUEST, Json(json!({"error": "password cannot be empty"}))).into_response());
        }
        let hash = hash_password(&password);
        state.storage.update_user_password(id, &hash).await?;
        details.push("password_changed".to_string());
    }

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_user".to_string(),
        username: auth.username.clone(),
        resource: format!("user:{}", user.username),
        details: Some(details.join(",")),
    });

    Ok(Json(json!({"ok": true})).into_response())
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<super::AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.storage.get_user_by_id(id).await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;
    if user.username == auth.username {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"error": "cannot delete yourself"}))).into_response());
    }
    state.storage.delete_user(id).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "delete_user".to_string(),
        username: auth.username.clone(),
        resource: format!("user:{}", user.username),
        details: None,
    });

    Ok(Json(json!({"ok": true})).into_response())
}

// ── SSO / SAML / LDAP endpoints (Enterprise-tier gated) ──

/// GET /api/v1/auth/providers — lists enabled auth providers.
#[utoipa::path(
    get,
    path = "/api/v1/auth/providers",
    responses((status = 200)),
    tag = "Auth"
)]
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

/// POST /api/v1/auth/saml/metadata — returns SP metadata XML.
#[utoipa::path(
    post,
    path = "/api/v1/auth/saml/metadata",
    responses((status = 200)),
    tag = "Auth"
)]
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

/// POST /api/v1/auth/saml/acs — SAML Assertion Consumer Service.
#[utoipa::path(
    post,
    path = "/api/v1/auth/saml/acs",
    responses((status = 200), (status = 302)),
    tag = "Auth"
)]
pub async fn saml_acs(
    State(state): State<AppState>,
    Json(body): Json<SamlAcsRequest>,
) -> Result<impl IntoResponse, AppError> {
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

/// POST /api/v1/auth/ldap/login — LDAP authentication endpoint.
#[utoipa::path(
    post,
    path = "/api/v1/auth/ldap/login",
    responses((status = 200, body = crate::models::LoginResponse), (status = 401)),
    tag = "Auth"
)]
pub async fn ldap_login(
    State(state): State<AppState>,
    Json(body): Json<LdapLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
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

    Ok(Json(json!({
        "ok": true,
        "username": user.username,
        "role": user.role,
        "token": access_token,
    }))
    .into_response())
}

// ── SSO request types ──

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct SamlAcsRequest {
    pub saml_response: String,
    #[serde(default)]
    pub relay_state: Option<String>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct LdapLoginRequest {
    pub username: String,
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let hash = hash_password("admin");
        assert!(verify_password("admin", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn test_hash_password_unique_salts() {
        let a = hash_password("admin");
        let b = hash_password("admin");
        assert_ne!(a, b); // different salts each time
    }

    #[test]
    fn test_create_and_verify_token() {
        let secret = "test-secret";
        let token = create_token("admin", "admin", secret);
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.username, "admin");
        assert_eq!(claims.role, "admin");
        assert!(claims.exp > crate::timeutil::now_millis() as u64);
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let token = create_token("admin", "admin", "secret1");
        assert!(verify_token(&token, "secret2").is_none());
    }

    #[test]
    fn test_verify_token_expired() {
        use base64::engine::general_purpose::STANDARD;
        let claims = TokenClaims {
            username: "admin".to_string(),
            exp: 0, // expired
            role: "admin".to_string(),
        };
        let payload = serde_json::to_string(&claims).unwrap();
        let sig = hmac_sign(&payload, "secret");
        let token = format!("{}.{}", STANDARD.encode(&payload), sig);
        assert!(verify_token(&token, "secret").is_none());
    }

    #[test]
    fn test_verify_token_malformed() {
        assert!(verify_token("not-a-token", "secret").is_none());
        assert!(verify_token("", "secret").is_none());
    }

    #[test]
    fn test_constant_time_equal() {
        assert!(constant_time_equal("abc", "abc"));
        assert!(!constant_time_equal("abc", "abd"));
        assert!(!constant_time_equal("abc", "ab"));
    }
}
