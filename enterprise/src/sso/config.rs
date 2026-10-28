//! The `sso:` section of the configuration file.
//!
//! ```yaml
//! sso:
//!   public_url: https://scada.example.com
//!   oidc: { client_id: ..., client_secret_env: VOLTRUS_OIDC_SECRET }
//!   saml: { idp_metadata_url: ..., role_mapping: { groups: [...] } }
//!   ldap: { url: ldaps://dc1.example.com, ... }
//! ```
//!
//! The OIDC issuer, audience and JWKS URL for bearer tokens stay at the top
//! level (`oidc_enabled`, `oidc_issuer`, ...); browser sign-in reuses the issuer.

use serde::{Deserialize, Serialize};

use super::ldap::LdapConfig;
use super::oidc::OidcLoginConfig;
use super::saml::SamlConfig;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SsoConfig {
    /// Externally visible base URL of this server, no trailing slash. Redirect
    /// and ACS URLs are built from it, so it must be the address browsers use.
    #[serde(default)]
    pub public_url: String,
    #[serde(default)]
    pub oidc: OidcLoginConfig,
    #[serde(default)]
    pub saml: SamlConfig,
    #[serde(default)]
    pub ldap: LdapConfig,
}

#[derive(Deserialize, Default)]
struct FileConfig {
    #[serde(default)]
    sso: SsoConfig,
}

impl SsoConfig {
    /// Read the `sso:` section of the config file. A missing file or section
    /// means SSO browser/directory sign-in is not configured.
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(Self::default()),
        };
        Self::from_yaml(&content)
    }

    pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        let file: FileConfig = serde_yaml_neo::from_str(yaml)
            .map_err(|e| anyhow::anyhow!("invalid sso configuration: {e}"))?;
        let mut cfg = file.sso;
        cfg.public_url = cfg.public_url.trim().trim_end_matches('/').to_string();
        Ok(cfg)
    }
}

/// Roles a directory or IdP login may map to, lowest privilege first.
const ROLES: [&str; 5] = ["viewer", "demo", "user", "operator", "admin"];

pub fn is_known_role(role: &str) -> bool {
    ROLES.contains(&role)
}

fn rank(role: &str) -> usize {
    ROLES.iter().position(|r| *r == role).unwrap_or(0)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroupRole {
    /// A group name, a group DN (LDAP) or an IdP group/role value. Compared
    /// case-insensitively; LDAP DNs also ignore spaces around `,` and `=`.
    pub group: String,
    pub role: String,
}

/// Maps the groups an identity provider reports to a local role.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoleMapping {
    /// Role for a user who matches no mapping. Empty refuses the login instead.
    #[serde(default = "default_role")]
    pub default_role: String,
    #[serde(default)]
    pub groups: Vec<GroupRole>,
}

fn default_role() -> String {
    "viewer".to_string()
}

impl Default for RoleMapping {
    fn default() -> Self {
        Self {
            default_role: default_role(),
            groups: Vec::new(),
        }
    }
}

/// Lowercase and drop spaces around `,` and `=` so DNs written differently match.
pub fn normalize_group(g: &str) -> String {
    let lower = g.trim().to_lowercase();
    if !lower.contains('=') {
        return lower;
    }
    lower
        .split(',')
        .map(|part| {
            part.split_once('=')
                .map(|(k, v)| format!("{}={}", k.trim(), v.trim()))
                .unwrap_or_else(|| part.trim().to_string())
        })
        .collect::<Vec<_>>()
        .join(",")
}

impl RoleMapping {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.default_role.is_empty() && !is_known_role(&self.default_role) {
            anyhow::bail!("unknown default_role '{}'", self.default_role);
        }
        for g in &self.groups {
            if !is_known_role(&g.role) {
                anyhow::bail!("unknown role '{}' for group '{}'", g.role, g.group);
            }
            if g.group.trim().is_empty() {
                anyhow::bail!("empty group in role_mapping");
            }
        }
        Ok(())
    }

    /// The highest-privilege role any of `groups` maps to, else the default
    /// role. `None` when nothing matches and the default is empty: refuse.
    pub fn resolve<S: AsRef<str>>(&self, groups: &[S]) -> Option<String> {
        let have: Vec<String> = groups.iter().map(|g| normalize_group(g.as_ref())).collect();
        let best = self
            .groups
            .iter()
            .filter(|m| have.contains(&normalize_group(&m.group)))
            .map(|m| m.role.as_str())
            .max_by_key(|r| rank(r));
        match best {
            Some(role) => Some(role.to_string()),
            None if self.default_role.is_empty() => None,
            None => Some(self.default_role.clone()),
        }
    }
}

/// A secret from the config file or, when `env_name` is set, the environment.
pub fn secret(value: &str, env_name: &str) -> String {
    if !env_name.is_empty()
        && let Ok(v) = std::env::var(env_name)
        && !v.is_empty()
    {
        return v;
    }
    value.to_string()
}

/// Local redirect target after sign-in. Only same-site absolute paths pass, so
/// a crafted `return_to` cannot turn the login flow into an open redirect.
pub fn safe_return_to(candidate: Option<&str>) -> String {
    match candidate {
        Some(p)
            if p.starts_with('/')
                && !p.starts_with("//")
                && !p.contains('\\')
                && !p.chars().any(|c| c.is_control()) =>
        {
            p.to_string()
        }
        _ => "/".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapping() -> RoleMapping {
        RoleMapping {
            default_role: "viewer".into(),
            groups: vec![
                GroupRole {
                    group: "CN=SCADA Operators,OU=Groups,DC=corp,DC=example".into(),
                    role: "operator".into(),
                },
                GroupRole {
                    group: "scada-admins".into(),
                    role: "admin".into(),
                },
            ],
        }
    }

    #[test]
    fn highest_privilege_role_wins() {
        let groups = [
            "scada-admins",
            "cn=scada operators, ou=groups, dc=corp, dc=example",
        ];
        assert_eq!(mapping().resolve(&groups).as_deref(), Some("admin"));
    }

    #[test]
    fn dn_matching_ignores_case_and_spacing() {
        let groups = ["cn=SCADA Operators ,ou=Groups,dc=corp,dc=EXAMPLE"];
        assert_eq!(mapping().resolve(&groups).as_deref(), Some("operator"));
    }

    #[test]
    fn unmatched_user_gets_default_or_is_refused() {
        let none: [&str; 0] = [];
        assert_eq!(mapping().resolve(&none).as_deref(), Some("viewer"));
        let mut strict = mapping();
        strict.default_role.clear();
        assert_eq!(strict.resolve(&["other"]), None);
    }

    #[test]
    fn unknown_roles_are_rejected_at_load() {
        let mut m = mapping();
        m.groups[0].role = "root".into();
        assert!(m.validate().is_err());
    }

    #[test]
    fn return_to_only_accepts_local_paths() {
        assert_eq!(safe_return_to(Some("/screens/3?x=1")), "/screens/3?x=1");
        for bad in [
            "//evil.example",
            "https://evil.example",
            "/\\evil.example",
            "evil",
            "/a\nb",
        ] {
            assert_eq!(safe_return_to(Some(bad)), "/", "{bad}");
        }
        assert_eq!(safe_return_to(None), "/");
    }

    #[test]
    fn sso_section_parses_from_the_main_config_file() {
        let cfg = SsoConfig::from_yaml(
            r#"
server_port: 8080
sso:
  public_url: "https://scada.example.com/"
  saml:
    enabled: true
    idp_metadata_url: https://idp.example.com/metadata
    role_mapping:
      default_role: viewer
      groups:
        - { group: scada-admins, role: admin }
"#,
        )
        .unwrap();
        assert_eq!(cfg.public_url, "https://scada.example.com");
        assert!(cfg.saml.enabled);
        assert_eq!(cfg.saml.role_mapping.groups[0].role, "admin");
        assert!(!cfg.ldap.enabled);
    }
}
