use axum::extract::State;
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

pub fn create_token(username: &str, secret: &str) -> String {
    let exp = chrono::Utc::now().timestamp_millis() as u64 + TOKEN_EXPIRY_MS;
    let payload =
        serde_json::to_string(&TokenClaims {
            username: username.to_string(),
            exp,
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
    let now = chrono::Utc::now().timestamp_millis() as u64;
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
    let user = state.storage.get_user_by_username(&body.username).await?;
    let user = match user {
        Some(u) => u,
        None => {
            return Ok((jar, (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"})))).into_response());
        }
    };
    if !verify_password(&body.password, &user.password_hash) {
        return Ok((jar, (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"})))).into_response());
    }
    let secret = &state.auth_secret;
    let token = create_token(&body.username, secret);
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
    Ok((jar, Json(LoginResponse { ok: true, username: body.username })).into_response())
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
        Some(claims) => Json(json!({"authenticated": true, "username": claims.username})),
        None => Json(json!({"authenticated": false})),
    }
}

pub async fn seed(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    state.storage.seed_database().await?;
    Ok(Json(json!({"ok": true})))
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
        let token = create_token("admin", secret);
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.username, "admin");
        assert!(claims.exp > chrono::Utc::now().timestamp_millis() as u64);
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let token = create_token("admin", "secret1");
        assert!(verify_token(&token, "secret2").is_none());
    }

    #[test]
    fn test_verify_token_expired() {
        use base64::engine::general_purpose::STANDARD;
        let claims = TokenClaims {
            username: "admin".to_string(),
            exp: 0, // expired
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
