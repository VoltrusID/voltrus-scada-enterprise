//! Single sign-on: OIDC bearer tokens, SAML 2.0 and LDAP.

pub mod auth_config;
pub mod handlers;
pub mod ldap;
pub mod oidc;
pub mod saml;

use serde_json::json;
use voltrus::edition::ExternalIdentity;

use crate::rt;

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

/// Provider entries appended to `GET /api/v1/auth/providers`.
pub fn auth_providers() -> Vec<serde_json::Value> {
    let rt = rt();
    let tier = rt.tier();
    let oidc_enabled = rt.oidc.enabled && tier.sso();
    // SAML and LDAP are reported as enabled together with OIDC, as before.
    let saml_ldap_enabled = tier.sso_saml() && rt.oidc.enabled;
    vec![
        json!({
            "provider": "oidc",
            "enabled": oidc_enabled,
            "label": "OpenID Connect",
            "issuer": if oidc_enabled { rt.oidc.issuer.as_str() } else { "" },
        }),
        json!({
            "provider": "saml",
            "enabled": saml_ldap_enabled,
            "label": "SAML 2.0",
        }),
        json!({
            "provider": "ldap",
            "enabled": saml_ldap_enabled,
            "label": "LDAP",
        }),
    ]
}
