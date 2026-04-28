use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
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

fn default_role_claim() -> String { "realm_access.roles".to_string() }
fn default_admin_role() -> String { "voltrus-admin".to_string() }

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

#[derive(Clone)]
pub struct JwksManager {
    keys: Arc<RwLock<Option<jsonwebtoken::DecodingKey>>>,
    jwks_url: String,
}

impl JwksManager {
    pub fn new(jwks_url: &str) -> Self {
        Self {
            keys: Arc::new(RwLock::new(None)),
            jwks_url: jwks_url.to_string(),
        }
    }

    pub async fn refresh(&self) -> anyhow::Result<()> {
        if self.jwks_url.is_empty() {
            return Ok(());
        }

        let resp = reqwest::get(&self.jwks_url).await?;
        let jwks: serde_json::Value = resp.json().await?;

        if let Some(keys) = jwks.get("keys").and_then(|k| k.as_array()) {
            for key in keys {
                if let (Some(kty), Some(n), Some(e)) = (
                    key.get("kty").and_then(|v| v.as_str()),
                    key.get("n").and_then(|v| v.as_str()),
                    key.get("e").and_then(|v| v.as_str()),
                ) {
                    if kty == "RSA" {
                        match jsonwebtoken::DecodingKey::from_rsa_components(n, e) {
                            Ok(decoding_key) => {
                                *self.keys.write().await = Some(decoding_key);
                                tracing::info!("OIDC JWKS key refreshed successfully");
                                return Ok(());
                            }
                            Err(err) => {
                                tracing::warn!("Failed to create decoding key from JWKS: {}", err);
                            }
                        }
                    }
                }
            }
        }

        anyhow::bail!("No valid RSA key found in JWKS response")
    }

    pub async fn validate_token(&self, token: &str, config: &OidcConfig) -> Option<OidcClaims> {
        if !config.enabled {
            return None;
        }

        let key = self.keys.read().await;
        let key = key.as_ref()?;

        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        if !config.issuer.is_empty() {
            validation.set_issuer(&[&config.issuer]);
        }
        if !config.audience.is_empty() {
            validation.set_audience(&[&config.audience]);
        } else {
            validation.validate_aud = false;
        }

        match jsonwebtoken::decode::<OidcClaims>(token, key, &validation) {
            Ok(data) => Some(data.claims),
            Err(err) => {
                tracing::debug!("OIDC token validation failed: {}", err);
                None
            }
        }
    }
}

pub fn extract_role(claims: &OidcClaims, config: &OidcConfig) -> String {
    if config.role_claim.starts_with("realm_access.") {
        if let Some(ref ra) = claims.realm_access {
            if ra.roles.contains(&config.admin_role) {
                return "admin".to_string();
            }
        }
    }

    if let Some(ref groups) = claims.groups {
        if groups.contains(&config.admin_role) {
            return "admin".to_string();
        }
    }

    "user".to_string()
}

pub fn spawn_jwks_refresh(manager: JwksManager) {
    tokio::spawn(async move {
        if let Err(err) = manager.refresh().await {
            tracing::warn!("Initial JWKS fetch failed: {}", err);
        }

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400));
        loop {
            interval.tick().await;
            if let Err(err) = manager.refresh().await {
                tracing::warn!("JWKS refresh failed: {}", err);
            }
        }
    });
}
