//! Single sign-on: OIDC (bearer tokens and browser login), SAML 2.0 and LDAP.
//!
//! Configuration is the `sso:` section of the config file ([`config`]). Every
//! entry point checks the licence tier at request time, so an activation or a
//! lapse takes effect without a restart.

pub mod config;
pub mod handlers;
pub mod ldap;
pub mod oidc;
pub mod saml;
pub mod xmldsig;

use std::sync::Arc;

use serde_json::json;
use voltrus::edition::ExternalIdentity;

use crate::rt;
use config::SsoConfig;
use ldap::LdapProvider;
use oidc::{OidcConfig, OidcLogin};
use saml::SamlProvider;

/// The configured sign-in providers. A provider whose configuration is invalid
/// is left out and the reason logged; the others keep working.
pub struct SsoRuntime {
    pub saml: Option<Arc<SamlProvider>>,
    pub oidc: Option<Arc<OidcLogin>>,
    pub ldap: Option<Arc<LdapProvider>>,
}

impl SsoRuntime {
    pub fn none() -> Self {
        Self {
            saml: None,
            oidc: None,
            ldap: None,
        }
    }

    /// Read `sso:` from the config file and set the providers up.
    pub async fn start(config_path: &str, bearer: &OidcConfig) -> Self {
        let cfg = match SsoConfig::load(config_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("SSO disabled: {e:#}");
                return Self::none();
            }
        };
        let mut rt = Self::none();

        if cfg.saml.enabled {
            match SamlProvider::new(cfg.saml.clone(), &cfg.public_url).await {
                Ok(p) => {
                    p.spawn_refresh();
                    tracing::info!("SAML sign-in enabled, ACS {}", p.cfg.acs_url);
                    rt.saml = Some(p);
                }
                Err(e) => tracing::error!("SAML sign-in disabled: {e}"),
            }
        }
        if !cfg.oidc.client_id.is_empty() {
            match OidcLogin::new(cfg.oidc.clone(), &bearer.issuer, &cfg.public_url) {
                Ok(p) => {
                    tracing::info!(
                        "OIDC browser sign-in enabled, redirect {}",
                        p.cfg.redirect_uri
                    );
                    rt.oidc = Some(p);
                }
                Err(e) => tracing::error!("OIDC browser sign-in disabled: {e}"),
            }
        }
        if cfg.ldap.enabled {
            match LdapProvider::new(cfg.ldap.clone()) {
                Ok(p) => {
                    tracing::info!("LDAP sign-in enabled, directory {}", p.cfg.url);
                    rt.ldap = Some(p);
                }
                Err(e) => tracing::error!("LDAP sign-in disabled: {e}"),
            }
        }
        rt
    }
}

/// Validate an `Authorization: Bearer` token from the configured OIDC provider.
/// Only Enterprise licences accept external identities.
pub async fn authenticate_bearer(token: &str) -> Option<ExternalIdentity> {
    let rt = rt();
    if !rt.oidc.enabled || !rt.tier().sso() {
        return None;
    }
    let claims = rt.jwks.validate_token(token, &rt.oidc).await?;
    let role = oidc::extract_role(&claims, &rt.oidc);
    Some(ExternalIdentity {
        username: claims.sub,
        role,
    })
}

/// Provider entries appended to `GET /api/v1/auth/providers`. A provider is
/// enabled when it is configured and the licence includes it, which is exactly
/// when its login routes answer.
pub fn auth_providers() -> Vec<serde_json::Value> {
    let rt = rt();
    let tier = rt.tier();
    let oidc_enabled = rt.sso.oidc.is_some() && tier.sso();
    vec![
        json!({
            "provider": "oidc",
            "enabled": oidc_enabled,
            "label": "OpenID Connect",
            "issuer": if oidc_enabled { rt.oidc.issuer.as_str() } else { "" },
        }),
        json!({
            "provider": "saml",
            "enabled": rt.sso.saml.is_some() && tier.sso_saml(),
            "label": rt.sso.saml.as_ref().map(|p| p.cfg.label.as_str()).unwrap_or("SAML 2.0"),
        }),
        json!({
            "provider": "ldap",
            "enabled": rt.sso.ldap.is_some() && tier.sso_saml(),
            "label": rt.sso.ldap.as_ref().map(|p| p.cfg.label.as_str()).unwrap_or("LDAP"),
        }),
    ]
}
