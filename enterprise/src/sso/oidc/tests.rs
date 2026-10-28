//! OIDC browser login against a mock provider on localhost: discovery, JWKS,
//! authorization request parameters, PKCE verification at the token endpoint,
//! and ID token validation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use axum::extract::{Form, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use jsonwebtoken::{EncodingKey, Header};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::traits::PublicKeyParts;
use serde_json::{Value, json};

use super::*;
use crate::sso::config::GroupRole;

const CLIENT_ID: &str = "voltrus";

struct TestKey {
    private_pem: String,
    jwks: Value,
}

/// One RSA key for the whole test binary: generating it dominates the runtime.
fn test_key() -> &'static TestKey {
    static KEY: OnceLock<TestKey> = OnceLock::new();
    KEY.get_or_init(|| {
        let key = rsa::RsaPrivateKey::new(&mut rand_core::OsRng, 2048).unwrap();
        let b64 = |b: Vec<u8>| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b);
        TestKey {
            private_pem: key
                .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
                .unwrap()
                .to_string(),
            jwks: json!({"keys": [{
                "kty": "RSA", "use": "sig", "alg": "RS256", "kid": "k1",
                "n": b64(key.n().to_bytes_be()), "e": b64(key.e().to_bytes_be()),
            }]}),
        }
    })
}

fn sign(claims: &Value, kid: Option<&str>, alg: jsonwebtoken::Algorithm) -> String {
    let mut header = Header::new(alg);
    header.kid = kid.map(str::to_string);
    let key = match alg {
        jsonwebtoken::Algorithm::HS256 => EncodingKey::from_secret(b"anything"),
        _ => EncodingKey::from_rsa_pem(test_key().private_pem.as_bytes()).unwrap(),
    };
    jsonwebtoken::encode(&header, claims, &key).unwrap()
}

fn now() -> u64 {
    time::OffsetDateTime::now_utc().unix_timestamp() as u64
}

fn good_claims(issuer: &str) -> Value {
    json!({
        "iss": issuer, "aud": CLIENT_ID, "sub": "user-1", "nonce": "n-1",
        "exp": now() + 300, "iat": now(),
        "preferred_username": "alice", "email": "alice@example.com", "name": "Alice",
        "groups": ["scada-admins"], "realm_access": {"roles": ["ops"]},
    })
}

async fn jwks_manager() -> JwksManager {
    let m = JwksManager::new("");
    m.set_keys_from_jwks(&test_key().jwks).await;
    m
}

const ISS: &str = "https://op.example.com";

async fn check(claims: Value, kid: Option<&str>) -> Result<Value, String> {
    let token = sign(&claims, kid, jsonwebtoken::Algorithm::RS256);
    jwks_manager()
        .await
        .validate_id_token(&token, ISS, CLIENT_ID, "n-1")
        .await
}

#[test]
fn pkce_challenge_matches_the_rfc_7636_vector() {
    assert_eq!(
        pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}

#[tokio::test]
async fn a_well_formed_id_token_is_accepted() {
    let claims = check(good_claims(ISS), Some("k1")).await.unwrap();
    assert_eq!(claims["sub"], "user-1");
}

#[tokio::test]
async fn id_token_checks_reject_each_deviation() {
    let mut cases: Vec<(&str, Value)> = Vec::new();
    let mut c = good_claims(ISS);
    c["iss"] = json!("https://evil.example.com");
    cases.push(("issuer", c));
    let mut c = good_claims(ISS);
    c["aud"] = json!("another-client");
    cases.push(("audience", c));
    let mut c = good_claims(ISS);
    c["aud"] = json!([CLIENT_ID, "other"]);
    cases.push(("azp missing with several audiences", c));
    let mut c = good_claims(ISS);
    c["exp"] = json!(now() - 3600);
    cases.push(("expired", c));
    let mut c = good_claims(ISS);
    c["nbf"] = json!(now() + 3600);
    cases.push(("not yet valid", c));
    let mut c = good_claims(ISS);
    c["nonce"] = json!("replayed-nonce");
    cases.push(("nonce", c));
    let mut c = good_claims(ISS);
    c.as_object_mut().unwrap().remove("nonce");
    cases.push(("missing nonce", c));
    let mut c = good_claims(ISS);
    c.as_object_mut().unwrap().remove("sub");
    cases.push(("missing sub", c));
    for (what, claims) in cases {
        assert!(
            check(claims, Some("k1")).await.is_err(),
            "accepted a token with a bad {what}"
        );
    }
}

#[tokio::test]
async fn token_signed_by_an_unknown_key_id_is_rejected() {
    assert!(check(good_claims(ISS), Some("other-key")).await.is_err());
}

#[tokio::test]
async fn hmac_token_using_the_public_key_as_secret_is_rejected() {
    // Algorithm confusion: an HS256 token must never validate against an RSA key.
    let token = sign(
        &good_claims(ISS),
        Some("k1"),
        jsonwebtoken::Algorithm::HS256,
    );
    assert!(
        jwks_manager()
            .await
            .validate_id_token(&token, ISS, CLIENT_ID, "n-1")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn tampered_payload_is_rejected() {
    let token = sign(
        &good_claims(ISS),
        Some("k1"),
        jsonwebtoken::Algorithm::RS256,
    );
    let parts: Vec<&str> = token.split('.').collect();
    let mut claims = good_claims(ISS);
    claims["sub"] = json!("admin");
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let forged = format!("{}.{}.{}", parts[0], payload, parts[2]);
    assert!(
        jwks_manager()
            .await
            .validate_id_token(&forged, ISS, CLIENT_ID, "n-1")
            .await
            .is_err()
    );
}

#[test]
fn claim_paths_reach_nested_values() {
    let c = json!({"realm_access": {"roles": ["a", "b"]}, "groups": "single", "a.b": 1});
    assert_eq!(
        claim_at(&c, "realm_access.roles").unwrap(),
        &json!(["a", "b"])
    );
    assert_eq!(
        claim_at(&c, "a.b").unwrap(),
        &json!(1),
        "a literal dotted key wins"
    );
    assert!(claim_at(&c, "realm_access.missing").is_none());
    assert_eq!(
        claim_strings(claim_at(&c, "groups").unwrap()),
        vec!["single"]
    );
}

fn login_cfg() -> OidcLoginConfig {
    OidcLoginConfig {
        client_id: CLIENT_ID.into(),
        client_secret: "s3cret".into(),
        username_claim: "preferred_username".into(),
        groups_claim: "realm_access.roles".into(),
        role_mapping: RoleMapping {
            default_role: "viewer".into(),
            groups: vec![GroupRole {
                group: "ops".into(),
                role: "operator".into(),
            }],
        },
        ..OidcLoginConfig::default()
    }
}

#[test]
fn identity_comes_from_the_configured_claims() {
    let l = OidcLogin::new(login_cfg(), ISS, "https://scada.example.com").unwrap();
    assert_eq!(
        l.cfg.redirect_uri,
        "https://scada.example.com/api/v1/auth/oidc/callback"
    );
    let id = l.identity(&good_claims(ISS)).unwrap();
    assert_eq!(id.username, "alice");
    assert_eq!(id.subject, "user-1");
    assert_eq!(id.groups, vec!["ops"]);
    assert_eq!(
        l.cfg.role_mapping.resolve(&id.groups).as_deref(),
        Some("operator")
    );

    // A missing username claim is an error, not a silent fallback to someone else.
    let mut c = good_claims(ISS);
    c.as_object_mut().unwrap().remove("preferred_username");
    assert!(l.identity(&c).is_err());
}

#[test]
fn login_configuration_is_validated() {
    assert!(
        OidcLogin::new(login_cfg(), "", "https://x").is_err(),
        "needs an issuer"
    );
    assert!(
        OidcLogin::new(login_cfg(), "http://op.example.com", "https://x").is_err(),
        "issuer must be https"
    );
    assert!(
        OidcLogin::new(login_cfg(), ISS, "").is_err(),
        "needs public_url or redirect_uri"
    );
    let mut cfg = login_cfg();
    cfg.client_auth = "magic".into();
    assert!(OidcLogin::new(cfg, ISS, "https://x").is_err());
    let mut cfg = login_cfg();
    cfg.role_mapping.default_role = "root".into();
    assert!(OidcLogin::new(cfg, ISS, "https://x").is_err());
}

// --- the whole code flow against a mock provider ------------------------------

#[derive(Default)]
struct OpState {
    issuer: String,
    /// What the token endpoint puts in the ID token and checks.
    nonce: Mutex<String>,
    challenge: Mutex<String>,
    token_requests: Mutex<Vec<HashMap<String, String>>>,
    fail_token: Mutex<bool>,
}

async fn discovery(State(s): State<Arc<OpState>>) -> Json<Value> {
    Json(json!({
        "issuer": s.issuer,
        "authorization_endpoint": format!("{}/authorize", s.issuer),
        "token_endpoint": format!("{}/token", s.issuer),
        "jwks_uri": format!("{}/jwks", s.issuer),
    }))
}

async fn jwks() -> Json<Value> {
    Json(test_key().jwks.clone())
}

async fn token(
    State(s): State<Arc<OpState>>,
    Form(form): Form<HashMap<String, String>>,
) -> (axum::http::StatusCode, Json<Value>) {
    s.token_requests.lock().unwrap().push(form.clone());
    let verifier = form.get("code_verifier").cloned().unwrap_or_default();
    let pkce_ok = pkce_challenge(&verifier) == *s.challenge.lock().unwrap();
    let secret_ok = form.get("client_secret").map(String::as_str) == Some("s3cret");
    if *s.fail_token.lock().unwrap()
        || !pkce_ok
        || !secret_ok
        || form.get("code").map(String::as_str) != Some("good-code")
    {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_grant"})),
        );
    }
    let mut claims = good_claims(&s.issuer);
    claims["nonce"] = json!(s.nonce.lock().unwrap().clone());
    let id_token = sign(&claims, Some("k1"), jsonwebtoken::Algorithm::RS256);
    (
        axum::http::StatusCode::OK,
        Json(json!({"id_token": id_token, "access_token": "x", "token_type": "Bearer"})),
    )
}

async fn mock_op() -> (Arc<OpState>, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let state = Arc::new(OpState {
        issuer: issuer.clone(),
        ..Default::default()
    });
    let app = Router::new()
        .route("/.well-known/openid-configuration", get(discovery))
        .route("/jwks", get(jwks))
        .route("/token", post(token))
        .with_state(state.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (state, issuer)
}

fn query(url: &str) -> HashMap<String, String> {
    reqwest::Url::parse(url)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}

#[tokio::test]
async fn authorization_code_flow_with_pkce_end_to_end() {
    let (op, issuer) = mock_op().await;
    let login = OidcLogin::new(login_cfg(), &issuer, "https://scada.example.com").unwrap();

    let (url, state) = login.begin("/screens/4").await.unwrap();
    assert!(url.starts_with(&format!("{issuer}/authorize?")));
    let q = query(&url);
    assert_eq!(q["response_type"], "code");
    assert_eq!(q["client_id"], CLIENT_ID);
    assert_eq!(
        q["redirect_uri"],
        "https://scada.example.com/api/v1/auth/oidc/callback"
    );
    assert_eq!(q["code_challenge_method"], "S256");
    assert_eq!(q["scope"], "openid profile email");
    assert_eq!(q["state"], state);
    *op.nonce.lock().unwrap() = q["nonce"].clone();
    *op.challenge.lock().unwrap() = q["code_challenge"].clone();

    let (identity, pending) = login
        .finish("good-code", &state, Some(&state))
        .await
        .unwrap();
    assert_eq!(identity.username, "alice");
    assert_eq!(identity.email, "alice@example.com");
    assert_eq!(pending.return_to, "/screens/4");
    // The verifier, not the challenge, went to the token endpoint.
    let sent = op.token_requests.lock().unwrap()[0].clone();
    assert_eq!(sent["grant_type"], "authorization_code");
    assert_eq!(sent["redirect_uri"], q["redirect_uri"]);
    assert_ne!(sent["code_verifier"], q["code_challenge"]);

    // A state is single use.
    assert!(
        login
            .finish("good-code", &state, Some(&state))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn callback_from_another_browser_is_refused_and_burns_nothing_at_the_provider() {
    let (op, issuer) = mock_op().await;
    let login = OidcLogin::new(login_cfg(), &issuer, "https://scada.example.com").unwrap();
    let (url, state) = login.begin("/").await.unwrap();
    let q = query(&url);
    *op.nonce.lock().unwrap() = q["nonce"].clone();
    *op.challenge.lock().unwrap() = q["code_challenge"].clone();

    // Login CSRF: the callback arrives without our state cookie, or with a different one.
    assert!(login.finish("good-code", &state, None).await.is_err());
    assert!(
        login
            .finish("good-code", &state, Some("attacker-state"))
            .await
            .is_err()
    );
    assert!(
        op.token_requests.lock().unwrap().is_empty(),
        "the code must not be redeemed"
    );
    // The genuine browser can still complete.
    assert!(
        login
            .finish("good-code", &state, Some(&state))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn unknown_state_and_rejected_codes_fail() {
    let (op, issuer) = mock_op().await;
    let login = OidcLogin::new(login_cfg(), &issuer, "https://scada.example.com").unwrap();
    assert!(
        login
            .finish("good-code", "forged", Some("forged"))
            .await
            .is_err()
    );

    let (url, state) = login.begin("/").await.unwrap();
    let q = query(&url);
    *op.nonce.lock().unwrap() = q["nonce"].clone();
    *op.challenge.lock().unwrap() = "wrong-challenge".into();
    let err = login
        .finish("good-code", &state, Some(&state))
        .await
        .unwrap_err();
    assert!(err.contains("refused the code"), "{err}");
}

#[tokio::test]
async fn id_token_with_a_replayed_nonce_is_refused() {
    let (op, issuer) = mock_op().await;
    let login = OidcLogin::new(login_cfg(), &issuer, "https://scada.example.com").unwrap();
    let (url, state) = login.begin("/").await.unwrap();
    *op.challenge.lock().unwrap() = query(&url)["code_challenge"].clone();
    *op.nonce.lock().unwrap() = "a-nonce-from-another-login".into();
    let err = login
        .finish("good-code", &state, Some(&state))
        .await
        .unwrap_err();
    assert!(err.contains("nonce"), "{err}");
}

#[tokio::test]
async fn discovery_for_a_different_issuer_is_refused() {
    let (_op, issuer) = mock_op().await;
    // Same server, but we claim a different issuer string: the document disagrees.
    let login = OidcLogin::new(
        login_cfg(),
        &format!("{issuer}/"),
        "https://scada.example.com",
    )
    .unwrap();
    assert!(
        login.begin("/").await.is_ok(),
        "a trailing slash is not a different issuer"
    );
    let (_op2, issuer2) = mock_op().await;
    let mismatched = OidcLogin::new(
        login_cfg(),
        &issuer2.replace("127.0.0.1", "localhost"),
        "https://x",
    )
    .unwrap();
    assert!(mismatched.begin("/").await.is_err());
}
