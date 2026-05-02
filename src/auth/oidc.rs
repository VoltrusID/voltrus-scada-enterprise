use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

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

/// Group-to-role mapping configuration.
/// Maps OIDC group names to local application roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRoleMapping {
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub role: String,
}

/// Resolves a user's role based on OIDC groups and a configurable mapping.
/// Falls back to the default `extract_role` logic if no mapping matches.
pub fn extract_role_with_mapping(
    claims: &OidcClaims,
    config: &OidcConfig,
    group_mappings: &[GroupRoleMapping],
) -> String {
    // Collect all groups the user belongs to
    let user_groups: Vec<&str> = claims.groups
        .as_ref()
        .map(|g| g.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();

    // Check group mappings in order (first match wins)
    for mapping in group_mappings {
        if user_groups.iter().any(|g| *g == mapping.group) {
            return mapping.role.clone();
        }
    }

    // Fall back to standard role extraction
    extract_role(claims, config)
}

/// OIDC provider presets for common identity providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProviderPreset {
    pub name: String,
    pub issuer_format: String,
    pub default_scope: String,
    pub default_role_claim: String,
}

/// Returns preset configurations for well-known OIDC providers.
pub fn get_provider_presets() -> Vec<OidcProviderPreset> {
    vec![
        OidcProviderPreset {
            name: "azure-ad".to_string(),
            issuer_format: "https://login.microsoftonline.com/{tenant-id}/v2.0".to_string(),
            default_scope: "openid profile email".to_string(),
            default_role_claim: "groups".to_string(),
        },
        OidcProviderPreset {
            name: "okta".to_string(),
            issuer_format: "https://{domain}/oauth2/default".to_string(),
            default_scope: "openid profile email groups".to_string(),
            default_role_claim: "groups".to_string(),
        },
        OidcProviderPreset {
            name: "google-workspace".to_string(),
            issuer_format: "https://accounts.google.com".to_string(),
            default_scope: "openid profile email".to_string(),
            default_role_claim: "hd".to_string(),
        },
        OidcProviderPreset {
            name: "keycloak".to_string(),
            issuer_format: "https://{domain}/realms/{realm}".to_string(),
            default_scope: "openid profile email".to_string(),
            default_role_claim: "realm_access.roles".to_string(),
        },
    ]
}

/// JIT (Just-in-Time) user provisioning info extracted from OIDC claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitUserInfo {
    pub sub: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
}

/// Extracts JIT user provisioning info from OIDC claims.
pub fn extract_jit_user_info(claims: &OidcClaims, config: &OidcConfig) -> JitUserInfo {
    let role = extract_role(claims, config);

    let email = claims.sub.clone();
    let display_name = claims.sub.clone();

    JitUserInfo {
        sub: claims.sub.clone(),
        email,
        display_name,
        role,
    }
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
