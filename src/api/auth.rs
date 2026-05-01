use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use base64::Engine;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState};

const TOKEN_EXPIRY_MS: u64 = 24 * 60 * 60 * 1000;

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
    let exp = crate::timeutil::now_millis() as u64 + TOKEN_EXPIRY_MS;
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

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Rate limiting by username
    let rate_key = format!("login:{}", body.username);
    if !state.login_rate_limiter.is_allowed(&rate_key) {
        return Ok((
            jar,
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": "Too many login attempts. Please try again later."})),
            ),
        )
            .into_response());
    }

    let user = state.storage.get_user_by_username(&body.username).await?;
    let user = match user {
        Some(u) => u,
        None => {
            state.login_rate_limiter.record_attempt(&rate_key);
            return Ok((jar, (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"})))).into_response());
        }
    };
    if !verify_password(&body.password, &user.password_hash) {
        state.login_rate_limiter.record_attempt(&rate_key);
        return Ok((jar, (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"})))).into_response());
    }

    // Successful login — clear rate limit attempts for this user
    state.login_rate_limiter.cleanup();

    let secret = &state.auth_secret;
    let token = create_token(&body.username, &user.role, secret);
    let cookie = axum_extra::extract::cookie::Cookie::build((
        "voltrus_token",
        token.clone(),
    ))
    .http_only(true)
    .secure(state.cookie_secure)
    .same_site(axum_extra::extract::cookie::SameSite::Strict)
    .max_age(time::Duration::seconds(86400))
    .path("/")
    .build();
    let jar = jar.add(cookie);
    Ok((jar, Json(LoginResponse { ok: true, username: body.username, role: user.role.clone(), must_change_password: user.must_change_password })).into_response())
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = axum_extra::extract::cookie::Cookie::build(("voltrus_token", ""))
        .max_age(time::Duration::seconds(0))
        .path("/")
        .build();
    (jar.add(cookie), Json(json!({"ok": true})))
}

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

pub async fn seed(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    state.storage.seed_database().await?;
    Ok(Json(json!({"ok": true})))
}

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
