use serde::{Deserialize, Serialize};

use super::ldap::LdapConfig;
use crate::sso::oidc::OidcConfig;
use crate::sso::saml::SamlConfig;

/// Unified authentication provider types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    Local,
    Oidc,
    Saml,
    Ldap,
}

/// Combined authentication configuration for all providers.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub oidc: OidcConfig,
    #[serde(default)]
    pub saml: SamlConfig,
    #[serde(default)]
    pub ldap: LdapConfig,
}

impl AuthConfig {
    /// Returns a list of currently enabled auth providers.
    pub fn enabled_providers(&self) -> Vec<AuthProvider> {
        let mut providers = vec![AuthProvider::Local];
        if self.oidc.enabled {
            providers.push(AuthProvider::Oidc);
        }
        if self.saml.enabled {
            providers.push(AuthProvider::Saml);
        }
        if self.ldap.enabled {
            providers.push(AuthProvider::Ldap);
        }
        providers
    }

    /// Returns true if any external auth provider is enabled.
    pub fn has_external_provider(&self) -> bool {
        self.oidc.enabled || self.saml.enabled || self.ldap.enabled
    }
}
