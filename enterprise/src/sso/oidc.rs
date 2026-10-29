//! OpenID Connect: bearer-token validation for API clients, and browser sign-in
//! with the authorization code flow and PKCE.

use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use dashmap::DashMap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;

use super::config::{RoleMapping, secret};

// ---------------------------------------------------------------------------
// Bearer tokens (top-level `oidc_*` settings)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OidcConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub audience: String,
    #[serde(default)]
    pub jwks_url: String,
    #[serde(default = "default_role_claim")]
    pub role_claim: String,
    #[serde(default = "default_admin_role")]
    pub admin_role: String,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            issuer: String::new(),
            audience: String::new(),
            jwks_url: String::new(),
            role_claim: default_role_claim(),
            admin_role: default_admin_role(),
        }
    }
}

fn default_role_claim() -> String {
    "realm_access.roles".to_string()
}
fn default_admin_role() -> String {
    "voltrus-admin".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcClaims {
    pub sub: String,
    pub exp: u64,
    pub iss: String,
    pub aud: serde_json::Value,
    #[serde(default)]
    pub realm_access: Option<RealmAccess>,
    #[serde(default)]
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    #[serde(default)]
    pub roles: Vec<String>,
}

pub fn extract_role(claims: &OidcClaims, config: &OidcConfig) -> String {
    if config.role_claim.starts_with("realm_access.")
        && let Some(ra) = &claims.realm_access
        && ra.roles.contains(&config.admin_role)
    {
        return "admin".to_string();
    }

    if let Some(groups) = &claims.groups
        && groups.contains(&config.admin_role)
    {
        return "admin".to_string();
    }

    "user".to_string()
}

// ---------------------------------------------------------------------------
// JWKS
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct JwkKey {
    kid: Option<String>,
    key: DecodingKey,
    /// Algorithms this key may verify (an RSA key never verifies an ES256 token).
    algs: &'static [Algorithm],
}

const RSA_ALGS: &[Algorithm] = &[Algorithm::RS256, Algorithm::RS384, Algorithm::RS512];
const EC_ALGS: &[Algorithm] = &[Algorithm::ES256];

/// The provider's signing keys by `kid`, refetched when a token names a key we lack.
#[derive(Clone)]
pub struct JwksManager {
    keys: Arc<RwLock<Vec<JwkKey>>>,
    jwks_url: String,
    last_fetch: Arc<std::sync::Mutex<Option<Instant>>>,
    http: reqwest::Client,
}

/// Minimum gap between refetches triggered by unknown key IDs.
const JWKS_MIN_REFETCH: Duration = Duration::from_secs(60);

fn parse_jwks(jwks: &serde_json::Value) -> Vec<JwkKey> {
    let mut out = Vec::new();
    let Some(keys) = jwks.get("keys").and_then(|k| k.as_array()) else {
        return out;
    };
    let s = |k: &serde_json::Value, f: &str| k.get(f).and_then(|v| v.as_str()).map(str::to_string);
    for key in keys {
        if s(key, "use").is_some_and(|u| u != "sig") {
            continue;
        }
        let kid = s(key, "kid");
        match s(key, "kty").as_deref() {
            Some("RSA") => {
                if let (Some(n), Some(e)) = (s(key, "n"), s(key, "e"))
                    && let Ok(k) = DecodingKey::from_rsa_components(&n, &e)
                {
                    out.push(JwkKey {
                        kid,
                        key: k,
                        algs: RSA_ALGS,
                    });
                }
            }
            Some("EC") if s(key, "crv").as_deref() == Some("P-256") => {
                if let (Some(x), Some(y)) = (s(key, "x"), s(key, "y"))
                    && let Ok(k) = DecodingKey::from_ec_components(&x, &y)
                {
                    out.push(JwkKey {
                        kid,
                        key: k,
                        algs: EC_ALGS,
                    });
                }
            }
            _ => {}
        }
    }
    out
}

impl JwksManager {
    pub fn new(jwks_url: &str) -> Self {
        Self::with_client(
            jwks_url,
            reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        )
    }

    pub fn with_client(jwks_url: &str, http: reqwest::Client) -> Self {
        Self {
            keys: Arc::new(RwLock::new(Vec::new())),
            jwks_url: jwks_url.to_string(),
            last_fetch: Arc::new(std::sync::Mutex::new(None)),
            http,
        }
    }

    pub async fn refresh(&self) -> anyhow::Result<()> {
        if self.jwks_url.is_empty() {
            return Ok(());
        }
        *self.last_fetch.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
        let jwks: serde_json::Value = self
            .http
            .get(&self.jwks_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let keys = parse_jwks(&jwks);
        if keys.is_empty() {
            anyhow::bail!("No usable signing key in JWKS response");
        }
        tracing::info!("OIDC JWKS refreshed: {} key(s)", keys.len());
        *self.keys.write().await = keys;
        Ok(())
    }

    /// Keys loaded directly, for tests.
    #[cfg(test)]
    pub async fn set_keys_from_jwks(&self, jwks: &serde_json::Value) {
        *self.keys.write().await = parse_jwks(jwks);
    }

    async fn find(&self, kid: Option<&str>, alg: Algorithm) -> Option<DecodingKey> {
        let keys = self.keys.read().await;
        let usable: Vec<&JwkKey> = keys.iter().filter(|k| k.algs.contains(&alg)).collect();
        match kid {
            Some(kid) => usable
                .iter()
                .find(|k| k.kid.as_deref() == Some(kid))
                .map(|k| k.key.clone()),
            // No kid in the token header: only unambiguous with a single key.
            None if usable.len() == 1 => Some(usable[0].key.clone()),
            None => None,
        }
    }

    /// The key for a token header. A kid we do not know triggers one refetch
    /// (rate limited), which is how provider key rotation is picked up.
    async fn key_for(&self, kid: Option<&str>, alg: Algorithm) -> Option<DecodingKey> {
        if let Some(k) = self.find(kid, alg).await {
            return Some(k);
        }
        let due = self
            .last_fetch
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_none_or(|t| t.elapsed() >= JWKS_MIN_REFETCH);
        if due && self.refresh().await.is_ok() {
            return self.find(kid, alg).await;
        }
        None
    }

    /// Validate an `Authorization: Bearer` access token (RS256).
    pub async fn validate_token(&self, token: &str, config: &OidcConfig) -> Option<OidcClaims> {
        if !config.enabled {
            return None;
        }
        let header = jsonwebtoken::decode_header(token).ok()?;
        if header.alg != Algorithm::RS256 {
            return None;
        }
        let key = self.key_for(header.kid.as_deref(), header.alg).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        if !config.issuer.is_empty() {
            validation.set_issuer(&[&config.issuer]);
        }
        if !config.audience.is_empty() {
            validation.set_audience(&[&config.audience]);
        } else {
            validation.validate_aud = false;
        }

        match jsonwebtoken::decode::<OidcClaims>(token, &key, &validation) {
            Ok(data) => Some(data.claims),
            Err(err) => {
                tracing::debug!("OIDC token validation failed: {}", err);
                None
            }
        }
    }

    /// Validate an ID token: signature, issuer, audience (our client ID),
    /// expiry, not-before, nonce, and `azp` when the audience is several.
    pub async fn validate_id_token(
        &self,
        token: &str,
        issuer: &str,
        client_id: &str,
        nonce: &str,
    ) -> Result<serde_json::Value, String> {
        let header =
            jsonwebtoken::decode_header(token).map_err(|e| format!("bad ID token header: {e}"))?;
        if !matches!(
            header.alg,
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 | Algorithm::ES256
        ) {
            return Err(format!(
                "ID token algorithm {:?} is not accepted",
                header.alg
            ));
        }
        let key = self
            .key_for(header.kid.as_deref(), header.alg)
            .await
            .ok_or("no matching signing key for the ID token")?;

        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[client_id]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        validation.validate_nbf = true;
        validation.leeway = 60;
        let claims = jsonwebtoken::decode::<serde_json::Value>(token, &key, &validation)
            .map_err(|e| format!("ID token rejected: {e}"))?
            .claims;

        if claims.get("nonce").and_then(|n| n.as_str()) != Some(nonce) {
            return Err("ID token nonce does not match".into());
        }
        if let Some(aud) = claims.get("aud").and_then(|a| a.as_array())
            && aud.len() > 1
            && claims.get("azp").and_then(|a| a.as_str()) != Some(client_id)
        {
            return Err("ID token azp does not match the client".into());
        }
        Ok(claims)
    }
}

pub fn spawn_jwks_refresh(manager: JwksManager) {
    tokio::spawn(async move {
        if let Err(err) = manager.refresh().await {
            tracing::warn!("Initial JWKS fetch failed: {}", err);
        }

        let mut interval = tokio::time::interval(Duration::from_secs(86400));
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = manager.refresh().await {
                tracing::warn!("JWKS refresh failed: {}", err);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Browser sign-in (authorization code + PKCE)
// ---------------------------------------------------------------------------

/// `sso.oidc` in the configuration file. The issuer comes from the top-level
/// `oidc_issuer`; the client ID is the audience of the ID token.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OidcLoginConfig {
    /// Setting a `client_id` turns browser sign-in on.
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    /// Environment variable holding the client secret (wins over `client_secret`).
    #[serde(default)]
    pub client_secret_env: String,
    /// CA (PEM text or file) for an identity provider behind a private CA: discovery,
    /// token and JWKS requests. Added to the built-in public roots.
    #[serde(default)]
    pub ca_cert: String,
    #[serde(default)]
    pub ca_cert_file: String,
    /// Default `<public_url>/api/v1/auth/oidc/callback`; register it at the provider.
    #[serde(default)]
    pub redirect_uri: String,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    /// How the secret reaches the token endpoint: `post` (body) or `basic` (header).
    #[serde(default = "default_client_auth")]
    pub client_auth: String,
    /// Claim used as the login name; empty uses `sub`.
    #[serde(default)]
    pub username_claim: String,
    /// Claim holding groups or roles (dotted path, e.g. `realm_access.roles`).
    #[serde(default = "default_groups_claim")]
    pub groups_claim: String,
    #[serde(default)]
    pub role_mapping: RoleMapping,
}

fn default_scopes() -> Vec<String> {
    vec!["openid".into(), "profile".into(), "email".into()]
}
fn default_client_auth() -> String {
    "post".to_string()
}
fn default_groups_claim() -> String {
    "groups".to_string()
}

impl Default for OidcLoginConfig {
    fn default() -> Self {
        serde_yaml_neo::from_str("{}").expect("empty OIDC login config deserializes")
    }
}

#[derive(Debug, Clone)]
struct Discovery {
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
}

#[derive(Debug, Clone)]
pub struct PendingOidc {
    nonce: String,
    verifier: String,
    created: i64,
    pub return_to: String,
}

#[derive(Debug, Clone)]
pub struct OidcIdentity {
    pub subject: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub groups: Vec<String>,
}

const STATE_TTL_SECS: i64 = 600;
const MAX_PENDING: usize = 10_000;

pub struct OidcLogin {
    pub cfg: OidcLoginConfig,
    pub issuer: String,
    client_secret: String,
    http: reqwest::Client,
    jwks_http: reqwest::Client,
    discovery: RwLock<Option<(Discovery, JwksManager)>>,
    pending: DashMap<String, PendingOidc>,
}

fn now_secs() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

fn random_url_token(bytes: usize) -> String {
    let b: Vec<u8> = (0..bytes).map(|_| rand::rng().random()).collect();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b)
}

/// PKCE S256 code challenge for a verifier.
pub fn pkce_challenge(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// `a.b.c` lookup into a JSON value.
pub fn claim_at<'a>(claims: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if let Some(v) = claims.get(path) {
        return Some(v);
    }
    path.split('.').try_fold(claims, |v, part| v.get(part))
}

fn claim_strings(v: &serde_json::Value) -> Vec<String> {
    match v {
        serde_json::Value::String(s) => vec![s.clone()],
        serde_json::Value::Array(a) => a
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn is_loopback_http(url: &str) -> bool {
    reqwest::Url::parse(url)
        .map(|u| matches!(u.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
        .unwrap_or(false)
}

fn require_https(what: &str, url: &str) -> Result<(), String> {
    if url.starts_with("https://") || (url.starts_with("http://") && is_loopback_http(url)) {
        Ok(())
    } else {
        Err(format!("{what} must be an https URL, got {url}"))
    }
}

impl OidcLogin {
    pub fn new(cfg: OidcLoginConfig, issuer: &str, public_url: &str) -> Result<Arc<Self>, String> {
        let mut cfg = cfg;
        cfg.role_mapping
            .validate()
            .map_err(|e| format!("oidc.role_mapping: {e}"))?;
        if issuer.is_empty() {
            return Err("oidc browser sign-in needs oidc_issuer".into());
        }
        require_https("oidc_issuer", issuer)?;
        if cfg.redirect_uri.is_empty() {
            if public_url.is_empty() {
                return Err(
                    "oidc browser sign-in needs sso.public_url (or oidc.redirect_uri)".into(),
                );
            }
            cfg.redirect_uri = format!("{public_url}/api/v1/auth/oidc/callback");
        }
        if !matches!(cfg.client_auth.as_str(), "post" | "basic") {
            return Err("oidc.client_auth must be 'post' or 'basic'".into());
        }
        let client_secret = secret(&cfg.client_secret, &cfg.client_secret_env);
        let client = |redirect: reqwest::redirect::Policy| {
            super::config::with_extra_ca(
                reqwest::Client::builder()
                    .timeout(Duration::from_secs(15))
                    .redirect(redirect),
                &cfg.ca_cert,
                &cfg.ca_cert_file,
            )
            .map_err(|e| format!("oidc: {e}"))?
            .build()
            .map_err(|e| e.to_string())
        };
        let http = client(reqwest::redirect::Policy::none())?;
        let jwks_http = client(reqwest::redirect::Policy::default())?;
        Ok(Arc::new(Self {
            cfg,
            issuer: issuer.to_string(),
            client_secret,
            http,
            jwks_http,
            discovery: RwLock::new(None),
            pending: DashMap::new(),
        }))
    }

    async fn discover(&self) -> Result<(Discovery, JwksManager), String> {
        if let Some(d) = self.discovery.read().await.clone() {
            return Ok(d);
        }
        let url = format!(
            "{}/.well-known/openid-configuration",
            self.issuer.trim_end_matches('/')
        );
        let doc: serde_json::Value = self
            .http
            .get(&url)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("OIDC discovery failed: {e}"))?
            .json()
            .await
            .map_err(|e| format!("OIDC discovery is not JSON: {e}"))?;
        let field = |name: &str| -> Result<String, String> {
            let v = doc
                .get(name)
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("OIDC discovery lacks {name}"))?;
            require_https(name, v)?;
            Ok(v.to_string())
        };
        let issuer = doc.get("issuer").and_then(|v| v.as_str()).unwrap_or("");
        if issuer.trim_end_matches('/') != self.issuer.trim_end_matches('/') {
            return Err("OIDC discovery issuer does not match oidc_issuer".into());
        }
        let d = Discovery {
            authorization_endpoint: field("authorization_endpoint")?,
            token_endpoint: field("token_endpoint")?,
            jwks_uri: field("jwks_uri")?,
        };
        let jwks = JwksManager::with_client(&d.jwks_uri, self.jwks_http.clone());
        let _ = jwks.refresh().await;
        *self.discovery.write().await = Some((d.clone(), jwks.clone()));
        Ok((d, jwks))
    }

    /// Start a login: returns the provider URL and the `state` to bind to the browser.
    pub async fn begin(&self, return_to: &str) -> Result<(String, String), String> {
        let (d, _) = self.discover().await?;
        let now = now_secs();
        self.pending.retain(|_, p| now - p.created < STATE_TTL_SECS);
        if self.pending.len() >= MAX_PENDING {
            return Err("too many logins in progress".into());
        }
        let state = random_url_token(24);
        let nonce = random_url_token(24);
        let verifier = random_url_token(48);
        let mut url = reqwest::Url::parse(&d.authorization_endpoint).map_err(|e| e.to_string())?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.cfg.client_id)
            .append_pair("redirect_uri", &self.cfg.redirect_uri)
            .append_pair("scope", &self.cfg.scopes.join(" "))
            .append_pair("state", &state)
            .append_pair("nonce", &nonce)
            .append_pair("code_challenge", &pkce_challenge(&verifier))
            .append_pair("code_challenge_method", "S256");
        self.pending.insert(
            state.clone(),
            PendingOidc {
                nonce,
                verifier,
                created: now,
                return_to: return_to.to_string(),
            },
        );
        Ok((url.to_string(), state))
    }

    /// Finish a login: check `state` against the browser cookie, redeem the
    /// code, validate the ID token and read the identity from it.
    pub async fn finish(
        &self,
        code: &str,
        state: &str,
        cookie_state: Option<&str>,
    ) -> Result<(OidcIdentity, PendingOidc), String> {
        if cookie_state != Some(state) {
            return Err("sign-in was not started by this browser".into());
        }
        // Single use: whatever happens next, this state is spent.
        let (_, pending) = self
            .pending
            .remove(state)
            .ok_or("unknown or already used state")?;
        if now_secs() - pending.created >= STATE_TTL_SECS {
            return Err("sign-in took too long".into());
        }
        let (d, jwks) = self.discover().await?;

        let mut form: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.cfg.redirect_uri),
            ("client_id", &self.cfg.client_id),
            ("code_verifier", &pending.verifier),
        ];
        let mut req = self.http.post(&d.token_endpoint);
        if !self.client_secret.is_empty() {
            if self.cfg.client_auth == "basic" {
                let enc = |s: &str| {
                    url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>()
                };
                req = req.basic_auth(enc(&self.cfg.client_id), Some(enc(&self.client_secret)));
                form.retain(|(k, _)| *k != "client_id");
            } else {
                form.push(("client_secret", &self.client_secret));
            }
        }
        let resp = req
            .form(&form)
            .send()
            .await
            .map_err(|e| format!("token request failed: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::debug!("OIDC token endpoint answered {status}: {body}");
            return Err(format!("token endpoint refused the code ({status})"));
        }
        let tokens: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("token response: {e}"))?;
        let id_token = tokens
            .get("id_token")
            .and_then(|t| t.as_str())
            .ok_or("token response has no id_token")?;

        let claims = jwks
            .validate_id_token(id_token, &self.issuer, &self.cfg.client_id, &pending.nonce)
            .await?;
        let identity = self.identity(&claims)?;
        Ok((identity, pending))
    }

    pub fn identity(&self, claims: &serde_json::Value) -> Result<OidcIdentity, String> {
        let subject = claims
            .get("sub")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .ok_or("ID token has no sub")?
            .to_string();
        let text = |name: &str| {
            claims
                .get(name)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        let username = if self.cfg.username_claim.is_empty() {
            subject.clone()
        } else {
            claim_at(claims, &self.cfg.username_claim)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .ok_or_else(|| format!("ID token has no {} claim", self.cfg.username_claim))?
        };
        let groups = claim_at(claims, &self.cfg.groups_claim)
            .map(claim_strings)
            .unwrap_or_default();
        Ok(OidcIdentity {
            subject,
            username,
            email: text("email"),
            display_name: text("name"),
            groups,
        })
    }
}

#[cfg(test)]
mod tests;
