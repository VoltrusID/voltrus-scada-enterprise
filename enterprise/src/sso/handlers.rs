//! HTTP endpoints for SAML, OIDC browser login and LDAP. All are public routes
//! (they create the session) and all check the licence tier first.
//!
//! * `GET  /api/v1/auth/saml/metadata`  SP metadata for the IdP administrator
//! * `GET  /api/v1/auth/saml/login`     start SP-initiated login (302 to the IdP)
//! * `POST /api/v1/auth/saml/acs`       assertion consumer service
//! * `GET  /api/v1/auth/oidc/login`     start authorization code flow (302)
//! * `GET  /api/v1/auth/oidc/callback`  authorization code redirect target
//! * `POST /api/v1/auth/ldap/login`     directory login (JSON)
//!
//! Every successful login ends in `voltrus::api::auth::issue_session`, the same
//! session local login creates, and the same security events.

use std::net::SocketAddr;

use axum::Form;
use axum::Json;
use axum::extract::{ConnectInfo, Query, State};
use axum::http::request::Parts;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::Deserialize;
use serde_json::json;

use voltrus::api::auth::issue_session;
use voltrus::api::{AppError, AppState};
use voltrus::models::CreateSecurityEvent;
use voltrus::storage::Storage;

use super::config::safe_return_to;
use super::ldap::LdapLoginError;
use crate::rt;

const SAML_BINDING_COOKIE: &str = "voltrus_saml_binding";
const OIDC_STATE_COOKIE: &str = "voltrus_oidc_state";
const FLOW_COOKIE_SECS: i64 = 600;

// ---------------------------------------------------------------------------
// Shared pieces
// ---------------------------------------------------------------------------

struct Client {
    ip: String,
    user_agent: Option<String>,
    /// Whether cookies for this request get the `Secure` attribute: the
    /// `cookie_secure` setting resolved from this request's peer and headers,
    /// exactly as local login does.
    secure: bool,
}

/// The `cookie_secure` setting resolved for this request's peer and headers.
fn request_secure(setting: voltrus::models::CookieSecure, parts: &Parts) -> bool {
    let peer = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(p)| *p);
    setting.resolve(voltrus::security::request_is_https(
        peer.as_ref(),
        &parts.headers,
    ))
}

fn client(parts: &Parts, state: &AppState) -> Client {
    let peer = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(p)| *p);
    Client {
        secure: request_secure(state.cookie_secure, parts),
        ip: voltrus::security::client_ip_from_parts(peer, &parts.headers),
        user_agent: parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string),
    }
}

async fn security_event(
    storage: &Storage,
    c: &Client,
    kind: &str,
    severity: &str,
    user: Option<&str>,
    details: String,
) {
    let _ = storage
        .insert_security_event(&CreateSecurityEvent {
            event_type: kind.into(),
            severity: severity.into(),
            source_ip: Some(c.ip.clone()),
            user_agent: c.user_agent.clone(),
            username: user.map(str::to_string),
            details: Some(details),
        })
        .await;
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn page(status: StatusCode, jar: CookieJar, head: &str, body: &str) -> Response {
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Voltrus sign-in</title>{head}</head><body style=\"font-family:system-ui,sans-serif;max-width:32rem;margin:15vh auto;padding:0 1rem\">{body}</body></html>"
    );
    (
        status,
        jar,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        html,
    )
        .into_response()
}

/// A page that continues to `to` from our own origin. Used instead of a 302
/// after the identity provider calls back: the session cookies are SameSite
/// Strict, and a redirect that began at another site would not carry them.
fn continue_page(jar: CookieJar, to: &str) -> Response {
    let to = html_escape(to);
    page(
        StatusCode::OK,
        jar,
        &format!("<meta http-equiv=\"refresh\" content=\"0;url={to}\">"),
        &format!("<p>Signed in. <a href=\"{to}\">Continue to Voltrus</a></p>"),
    )
}

fn error_page(status: StatusCode, jar: CookieJar, message: &str) -> Response {
    page(
        status,
        jar,
        "",
        &format!(
            "<h1>Sign-in failed</h1><p>{}</p><p><a href=\"/\">Back to sign in</a></p>",
            html_escape(message)
        ),
    )
}

fn tier_page(jar: CookieJar) -> Response {
    error_page(
        StatusCode::FORBIDDEN,
        jar,
        "Single sign-on requires an Enterprise licence.",
    )
}

fn flow_cookie(
    secure: bool,
    name: &'static str,
    value: String,
    path: &'static str,
    same_site: SameSite,
) -> Cookie<'static> {
    let mut cookie = flow_cookie_for(secure, same_site);
    cookie.set_name(name);
    cookie.set_value(value);
    cookie.set_path(path);
    cookie
}

fn flow_cookie_for(cookie_secure: bool, same_site: SameSite) -> Cookie<'static> {
    // SameSite=None needs Secure; without it the browser would drop the cookie.
    let (same_site, secure) = match same_site {
        SameSite::None if !cookie_secure => (SameSite::Lax, false),
        s => (s, cookie_secure),
    };
    Cookie::build(("", ""))
        .http_only(true)
        .secure(secure)
        .same_site(same_site)
        .max_age(time::Duration::seconds(FLOW_COOKIE_SECS))
        .build()
}

fn clear_cookie(name: &'static str, path: &'static str) -> Cookie<'static> {
    Cookie::build((name, ""))
        .max_age(time::Duration::seconds(0))
        .path(path)
        .build()
}

/// An identity an external provider vouched for.
struct ExternalLogin<'a> {
    provider: &'static str,
    id: &'a str,
    email: &'a str,
    display_name: &'a str,
    role: &'a str,
}

/// Provision or update the local account and open a session.
///
/// External accounts are named `<provider>:<id>`, so they can never collide
/// with or take over a local account. The role is re-applied on every login, so
/// a change in directory groups reaches Voltrus at the next sign-in.
async fn complete_login(
    storage: &Storage,
    auth_secret: &str,
    cookie_secure: bool,
    jar: CookieJar,
    c: &Client,
    login: ExternalLogin<'_>,
) -> Result<Result<(CookieJar, String), &'static str>, AppError> {
    if login.id.is_empty() || login.id.len() > 200 || login.id.chars().any(|ch| ch.is_control()) {
        return Ok(Err("The identity provider returned an unusable user name."));
    }
    let user = storage
        .get_or_create_user_by_external_id(login.provider, login.id, login.role)
        .await?;
    if user.disabled {
        security_event(
            storage,
            c,
            "login_disabled",
            "warning",
            Some(&user.username),
            format!("{} login on a disabled account", login.provider),
        )
        .await;
        return Ok(Err("This account is disabled."));
    }
    if user.role != login.role {
        storage.update_user_role(user.id, login.role).await?;
    }
    let meta = json!({
        "display_name": login.display_name,
        "email": login.email,
        "provisioned_via": login.provider,
    });
    let _ = storage
        .upsert_setting(&format!("user_meta:{}", user.username), &meta.to_string())
        .await;

    let jar = issue_session(
        storage,
        auth_secret,
        cookie_secure,
        jar,
        &user.username,
        login.role,
    )
    .await;
    security_event(
        storage,
        c,
        "login_success",
        "info",
        Some(&user.username),
        format!("{} login successful (role {})", login.provider, login.role),
    )
    .await;
    Ok(Ok((jar, user.username)))
}

// ---------------------------------------------------------------------------
// SAML
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct LoginQuery {
    #[serde(default)]
    pub return_to: Option<String>,
}

/// GET /api/v1/auth/saml/metadata
pub async fn saml_metadata(State(state): State<AppState>) -> Response {
    if !state.tier().sso_saml() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "SAML SSO requires an Enterprise licence"})),
        )
            .into_response();
    }
    let Some(saml) = &rt().sso.saml else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "SAML is not configured"})),
        )
            .into_response();
    };
    (
        [(header::CONTENT_TYPE, "application/samlmetadata+xml")],
        saml.metadata_xml(),
    )
        .into_response()
}

/// GET /api/v1/auth/saml/login
pub async fn saml_login(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Query(q): Query<LoginQuery>,
) -> Response {
    let c = client(&parts, &state);
    if !state.tier().sso_saml() {
        return tier_page(jar);
    }
    let Some(saml) = &rt().sso.saml else {
        return error_page(
            StatusCode::NOT_FOUND,
            jar,
            "SAML sign-in is not configured.",
        );
    };
    match saml.begin_login(&safe_return_to(q.return_to.as_deref())) {
        Ok((url, binding)) => {
            // The ACS is a cross-site POST, so the cookie must be SameSite=None.
            let jar = jar.add(flow_cookie(
                c.secure,
                SAML_BINDING_COOKIE,
                binding,
                "/api/v1/auth/saml",
                SameSite::None,
            ));
            (jar, Redirect::to(&url)).into_response()
        }
        Err(e) => {
            tracing::warn!("SAML login could not start: {e}");
            error_page(
                StatusCode::SERVICE_UNAVAILABLE,
                jar,
                "Single sign-on is unavailable right now.",
            )
        }
    }
}

#[derive(Deserialize)]
pub struct AcsForm {
    #[serde(rename = "SAMLResponse")]
    pub saml_response: String,
}

/// POST /api/v1/auth/saml/acs
pub async fn saml_acs(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Form(form): Form<AcsForm>,
) -> Result<Response, AppError> {
    let c = client(&parts, &state);
    if !state.tier().sso_saml() {
        return Ok(tier_page(jar));
    }
    let Some(saml) = &rt().sso.saml else {
        return Ok(error_page(
            StatusCode::NOT_FOUND,
            jar,
            "SAML sign-in is not configured.",
        ));
    };
    let binding = jar.get(SAML_BINDING_COOKIE).map(|c| c.value().to_string());
    let jar = jar.add(clear_cookie(SAML_BINDING_COOKIE, "/api/v1/auth/saml"));

    // Cookie binding needs SameSite=None, which browsers only keep over HTTPS.
    let (assertion, pending) = match saml
        .process_response(&form.saml_response, binding.as_deref(), c.secure)
        .await
    {
        Ok(ok) => ok,
        Err(e) => {
            tracing::warn!("SAML response refused: {e}");
            security_event(
                &state.storage,
                &c,
                "login_failed",
                "warning",
                None,
                format!("SAML response refused: {e}"),
            )
            .await;
            return Ok(error_page(
                StatusCode::UNAUTHORIZED,
                jar,
                "The identity provider's response was not accepted.",
            ));
        }
    };
    let identity = saml.identity(&assertion);
    let Some(role) = saml.cfg.role_mapping.resolve(&identity.groups) else {
        security_event(
            &state.storage,
            &c,
            "login_failed",
            "warning",
            Some(&identity.username),
            "SAML user matches no role mapping".into(),
        )
        .await;
        return Ok(error_page(
            StatusCode::FORBIDDEN,
            jar,
            "Your account has no access to Voltrus.",
        ));
    };
    match complete_login(
        &state.storage,
        &state.auth_secret,
        c.secure,
        jar.clone(),
        &c,
        ExternalLogin {
            provider: "saml",
            id: &identity.username,
            email: &identity.email,
            display_name: &identity.display_name,
            role: &role,
        },
    )
    .await?
    {
        Ok((jar, _)) => {
            let to = pending.map(|p| p.return_to).unwrap_or_else(|| "/".into());
            Ok(continue_page(jar, &safe_return_to(Some(&to))))
        }
        Err(msg) => Ok(error_page(StatusCode::FORBIDDEN, jar, msg)),
    }
}

// ---------------------------------------------------------------------------
// OIDC
// ---------------------------------------------------------------------------

/// GET /api/v1/auth/oidc/login
pub async fn oidc_login(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Query(q): Query<LoginQuery>,
) -> Response {
    let c = client(&parts, &state);
    if !state.tier().sso() {
        return tier_page(jar);
    }
    let Some(oidc) = &rt().sso.oidc else {
        return error_page(
            StatusCode::NOT_FOUND,
            jar,
            "OpenID Connect sign-in is not configured.",
        );
    };
    match oidc.begin(&safe_return_to(q.return_to.as_deref())).await {
        Ok((url, st)) => {
            // The provider redirects back with a GET, which Lax cookies accompany.
            let jar = jar.add(flow_cookie(
                c.secure,
                OIDC_STATE_COOKIE,
                st,
                "/api/v1/auth/oidc",
                SameSite::Lax,
            ));
            (jar, Redirect::to(&url)).into_response()
        }
        Err(e) => {
            tracing::warn!("OIDC login could not start: {e}");
            error_page(
                StatusCode::SERVICE_UNAVAILABLE,
                jar,
                "Single sign-on is unavailable right now.",
            )
        }
    }
}

#[derive(Deserialize)]
pub struct OidcCallback {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// GET /api/v1/auth/oidc/callback
pub async fn oidc_callback(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Query(q): Query<OidcCallback>,
) -> Result<Response, AppError> {
    let c = client(&parts, &state);
    if !state.tier().sso() {
        return Ok(tier_page(jar));
    }
    let Some(oidc) = &rt().sso.oidc else {
        return Ok(error_page(
            StatusCode::NOT_FOUND,
            jar,
            "OpenID Connect sign-in is not configured.",
        ));
    };
    let cookie_state = jar.get(OIDC_STATE_COOKIE).map(|c| c.value().to_string());
    let jar = jar.add(clear_cookie(OIDC_STATE_COOKIE, "/api/v1/auth/oidc"));

    if let Some(e) = &q.error {
        let detail = q.error_description.as_deref().unwrap_or("");
        security_event(
            &state.storage,
            &c,
            "login_failed",
            "warning",
            None,
            format!("OIDC provider returned {e}: {detail}"),
        )
        .await;
        return Ok(error_page(
            StatusCode::UNAUTHORIZED,
            jar,
            "The identity provider did not sign you in.",
        ));
    }
    let (Some(code), Some(st)) = (q.code.as_deref(), q.state.as_deref()) else {
        return Ok(error_page(
            StatusCode::BAD_REQUEST,
            jar,
            "The sign-in response is incomplete.",
        ));
    };
    let (identity, pending) = match oidc.finish(code, st, cookie_state.as_deref()).await {
        Ok(ok) => ok,
        Err(e) => {
            tracing::warn!("OIDC login refused: {e}");
            security_event(
                &state.storage,
                &c,
                "login_failed",
                "warning",
                None,
                format!("OIDC login refused: {e}"),
            )
            .await;
            return Ok(error_page(
                StatusCode::UNAUTHORIZED,
                jar,
                "The identity provider's response was not accepted.",
            ));
        }
    };
    let Some(role) = oidc.cfg.role_mapping.resolve(&identity.groups) else {
        security_event(
            &state.storage,
            &c,
            "login_failed",
            "warning",
            Some(&identity.username),
            "OIDC user matches no role mapping".into(),
        )
        .await;
        return Ok(error_page(
            StatusCode::FORBIDDEN,
            jar,
            "Your account has no access to Voltrus.",
        ));
    };
    match complete_login(
        &state.storage,
        &state.auth_secret,
        c.secure,
        jar.clone(),
        &c,
        ExternalLogin {
            provider: "oidc",
            id: &identity.username,
            email: &identity.email,
            display_name: &identity.display_name,
            role: &role,
        },
    )
    .await?
    {
        Ok((jar, _)) => Ok(continue_page(
            jar,
            &safe_return_to(Some(&pending.return_to)),
        )),
        Err(msg) => Ok(error_page(StatusCode::FORBIDDEN, jar, msg)),
    }
}

// ---------------------------------------------------------------------------
// LDAP
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct LdapLoginRequest {
    pub username: String,
    pub password: String,
}

/// POST /api/v1/auth/ldap/login
pub async fn ldap_login(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Json(body): Json<LdapLoginRequest>,
) -> Result<Response, AppError> {
    let c = client(&parts, &state);
    if !state.tier().sso_saml() {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "LDAP authentication requires an Enterprise licence"})),
        )
            .into_response());
    }
    let Some(ldap) = &rt().sso.ldap else {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "LDAP is not configured"})),
        )
            .into_response());
    };
    if body.username.is_empty() || body.password.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "username and password are required"})),
        )
            .into_response());
    }

    let started = std::time::Instant::now();
    // Brute-force limits, the same ones local login uses.
    let rate_key = format!("login:ldap:{}", body.username.to_lowercase());
    let limiter = &state.login_rate_limiter;
    if !limiter.is_allowed(&rate_key) || !limiter.is_ip_allowed(&c.ip) {
        return Ok((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "Too many login attempts. Please try again later."})),
        )
            .into_response());
    }

    let user = match ldap.authenticate(&body.username, &body.password).await {
        Ok(u) => u,
        Err(LdapLoginError::InvalidCredentials) => {
            limiter.record_attempt(&rate_key);
            limiter.record_ip_attempt(&c.ip);
            security_event(
                &state.storage,
                &c,
                "login_failed",
                "warning",
                Some(&body.username),
                "Invalid LDAP credentials".into(),
            )
            .await;
            // Same floor as local login, so timing does not tell a known user from an unknown one.
            if let Some(rest) = std::time::Duration::from_millis(300).checked_sub(started.elapsed())
            {
                tokio::time::sleep(rest).await;
            }
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid credentials"})),
            )
                .into_response());
        }
        Err(LdapLoginError::Directory(e)) => {
            tracing::error!("LDAP login failed: {e}");
            security_event(
                &state.storage,
                &c,
                "login_failed",
                "warning",
                Some(&body.username),
                format!("LDAP directory error: {e}"),
            )
            .await;
            return Ok((
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "The directory service is unavailable"})),
            )
                .into_response());
        }
    };

    let Some(role) = ldap.cfg.role_mapping.resolve(&user.groups) else {
        security_event(
            &state.storage,
            &c,
            "login_failed",
            "warning",
            Some(&user.username),
            "LDAP user matches no role mapping".into(),
        )
        .await;
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Your account has no access to Voltrus"})),
        )
            .into_response());
    };
    match complete_login(
        &state.storage,
        &state.auth_secret,
        c.secure,
        jar,
        &c,
        ExternalLogin {
            provider: "ldap",
            id: &user.username,
            email: &user.email,
            display_name: &user.display_name,
            role: &role,
        },
    )
    .await?
    {
        Ok((jar, username)) => Ok((
            jar,
            Json(json!({
                "ok": true,
                "username": username,
                "role": role,
                "must_change_password": false,
            })),
        )
            .into_response()),
        Err(msg) => Ok((StatusCode::FORBIDDEN, Json(json!({"error": msg}))).into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voltrus::api::auth::verify_token;

    const SECRET: &str = "test-secret";

    fn client() -> Client {
        Client {
            ip: "203.0.113.5".into(),
            user_agent: Some("test".into()),
            secure: true,
        }
    }

    fn login<'a>(provider: &'static str, id: &'a str, role: &'a str) -> ExternalLogin<'a> {
        ExternalLogin {
            provider,
            id,
            email: "a@example.com",
            display_name: "A",
            role,
        }
    }

    async fn sign_in(
        storage: &Storage,
        l: ExternalLogin<'_>,
    ) -> Result<(CookieJar, String), &'static str> {
        complete_login(storage, SECRET, true, CookieJar::new(), &client(), l)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn first_login_provisions_a_prefixed_account_and_opens_a_normal_session() {
        let storage = Storage::new_in_memory().unwrap();
        let (jar, username) = sign_in(&storage, login("saml", "alice@corp.example", "operator"))
            .await
            .unwrap();
        assert_eq!(username, "saml:alice@corp.example");

        let claims = verify_token(jar.get("voltrus_token").unwrap().value(), SECRET).unwrap();
        assert_eq!(
            (claims.username.as_str(), claims.role.as_str()),
            ("saml:alice@corp.example", "operator")
        );
        assert!(jar.get("voltrus_refresh").is_some());

        let user = storage
            .get_user_by_username(&username)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.role, "operator");
        assert!(!user.disabled);
    }

    #[tokio::test]
    async fn the_role_follows_the_provider_at_every_login() {
        let storage = Storage::new_in_memory().unwrap();
        let _ = sign_in(&storage, login("ldap", "bob", "admin"))
            .await
            .unwrap();
        let (jar, username) = sign_in(&storage, login("ldap", "bob", "viewer"))
            .await
            .unwrap();
        assert_eq!(
            storage
                .get_user_by_username(&username)
                .await
                .unwrap()
                .unwrap()
                .role,
            "viewer"
        );
        let claims = verify_token(jar.get("voltrus_token").unwrap().value(), SECRET).unwrap();
        assert_eq!(
            claims.role, "viewer",
            "a demoted directory user does not keep admin in the new session"
        );
    }

    #[tokio::test]
    async fn external_identities_never_resolve_to_local_accounts() {
        let storage = Storage::new_in_memory().unwrap();
        let hash = voltrus::api::auth::hash_password("local-pw");
        storage
            .create_user("admin", &hash, "", "admin")
            .await
            .unwrap();

        // A directory user literally named "admin" gets their own, lower account.
        let (_, username) = sign_in(&storage, login("ldap", "admin", "viewer"))
            .await
            .unwrap();
        assert_eq!(username, "ldap:admin");
        let local = storage
            .get_user_by_username("admin")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(local.role, "admin");
        assert_eq!(local.password_hash, hash, "the local account is untouched");
    }

    #[tokio::test]
    async fn a_disabled_account_cannot_sign_in_and_gets_no_session() {
        let storage = Storage::new_in_memory().unwrap();
        let (_, username) = sign_in(&storage, login("saml", "carol", "viewer"))
            .await
            .unwrap();
        let id = storage
            .get_user_by_username(&username)
            .await
            .unwrap()
            .unwrap()
            .id;
        storage.set_user_disabled(id, true).await.unwrap();

        let denied = sign_in(&storage, login("saml", "carol", "admin")).await;
        assert!(denied.is_err());
        let user = storage
            .get_user_by_username(&username)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            user.role, "viewer",
            "a refused login must not change the role either"
        );
    }

    #[tokio::test]
    async fn unusable_identifiers_are_refused() {
        let storage = Storage::new_in_memory().unwrap();
        let long = "x".repeat(201);
        for bad in ["", "a\nb", long.as_str()] {
            assert!(
                sign_in(&storage, login("oidc", bad, "viewer"))
                    .await
                    .is_err(),
                "{bad:?}"
            );
        }
    }

    async fn body_of(r: Response) -> String {
        let bytes = axum::body::to_bytes(r.into_body(), 1 << 20).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn pages_escape_what_they_echo() {
        let hostile = "/x\"><script>alert(1)</script>";
        let r = continue_page(CookieJar::new(), hostile);
        let body = body_of(r).await;
        assert!(!body.contains("<script>"), "{body}");
        let r = error_page(
            StatusCode::BAD_REQUEST,
            CookieJar::new(),
            "<img src=x onerror=1>",
        );
        assert!(!body_of(r).await.contains("<img"));
    }

    #[test]
    fn flow_cookies_degrade_to_lax_only_when_secure_is_off() {
        // SameSite=None is only legal with Secure; without HTTPS the cookie falls back to Lax.
        let secure = flow_cookie_for(true, SameSite::None);
        assert_eq!(
            (secure.same_site(), secure.secure()),
            (Some(SameSite::None), Some(true))
        );
        let plain = flow_cookie_for(false, SameSite::None);
        assert_eq!(
            (plain.same_site(), plain.secure()),
            (Some(SameSite::Lax), Some(false))
        );
        assert_eq!(
            flow_cookie_for(true, SameSite::Lax).same_site(),
            Some(SameSite::Lax)
        );
    }

    fn parts(peer: &str, headers: &[(&str, &str)]) -> Parts {
        let mut b = axum::http::Request::builder()
            .extension(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
        for (k, v) in headers {
            b = b.header(*k, *v);
        }
        b.body(()).unwrap().into_parts().0
    }

    #[test]
    fn auto_cookie_secure_follows_the_request_like_local_login() {
        use voltrus::models::CookieSecure::{Always, Auto, Never};
        let direct_http = parts("203.0.113.9:5000", &[]);
        let proxied_https = parts("127.0.0.1:5000", &[("x-forwarded-proto", "https")]);
        let proxied_http = parts("127.0.0.1:5000", &[("x-forwarded-proto", "http")]);
        // A remote peer cannot claim HTTPS with a header.
        let spoofed = parts("203.0.113.9:5000", &[("x-forwarded-proto", "https")]);

        assert!(!request_secure(Auto, &direct_http));
        assert!(request_secure(Auto, &proxied_https));
        assert!(!request_secure(Auto, &proxied_http));
        assert!(!request_secure(Auto, &spoofed));
        assert!(request_secure(Always, &direct_http));
        assert!(!request_secure(Never, &proxied_https));
    }

    #[test]
    fn saml_binding_cookie_is_none_and_secure_behind_an_https_proxy_but_lax_over_plain_http() {
        let proxied = flow_cookie(
            request_secure(
                voltrus::models::CookieSecure::Auto,
                &parts("127.0.0.1:1", &[("x-forwarded-proto", "https")]),
            ),
            SAML_BINDING_COOKIE,
            "v".into(),
            "/api/v1/auth/saml",
            SameSite::None,
        );
        assert_eq!(
            (proxied.same_site(), proxied.secure()),
            (Some(SameSite::None), Some(true))
        );
        let plain = flow_cookie(
            request_secure(
                voltrus::models::CookieSecure::Auto,
                &parts("203.0.113.9:1", &[]),
            ),
            SAML_BINDING_COOKIE,
            "v".into(),
            "/api/v1/auth/saml",
            SameSite::None,
        );
        assert_eq!(
            (plain.same_site(), plain.secure()),
            (Some(SameSite::Lax), Some(false))
        );
    }
}
