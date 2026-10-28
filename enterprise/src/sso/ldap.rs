//! LDAP and Active Directory sign-in.
//!
//! The password is verified with a bind as the user, never compared locally.
//! Connections use `ldaps://` or StartTLS with certificate and hostname
//! verification (system roots, or the CA configured here). Cleartext `ldap://`
//! is refused unless the operator explicitly allows it.
//!
//! Two modes:
//! * search (set `bind_dn`): bind as a service account, find the user with
//!   `user_filter`, then bind as the found DN with the typed password;
//! * direct (set `bind_dn_format` instead): bind as `{username}@corp.example.com`
//!   or `uid={username},ou=people,...`, then read the user's own entry.

use std::sync::Arc;
use std::time::Duration;

use ldap3::{Ldap, LdapConnAsync, LdapConnSettings, Scope, SearchEntry, dn_escape, ldap_escape};
use serde::{Deserialize, Serialize};

use super::config::{RoleMapping, secret};

/// Longest username accepted.
const MAX_USERNAME: usize = 256;
/// LDAP result code for invalidCredentials.
const RC_INVALID_CREDENTIALS: u32 = 49;
/// AD's transitive membership matching rule (members of nested groups).
const AD_IN_CHAIN: &str = "1.2.840.113556.1.4.1941";

/// `sso.ldap` in the configuration file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LdapConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Button text on the login page.
    #[serde(default = "default_label")]
    pub label: String,
    /// `ldaps://dc1.example.com` (port 636) or `ldap://dc1.example.com` with `starttls`.
    #[serde(default)]
    pub url: String,
    /// Upgrade an `ldap://` connection with StartTLS.
    #[serde(default)]
    pub starttls: bool,
    /// Allow cleartext `ldap://` without StartTLS. Sends passwords unencrypted.
    #[serde(default)]
    pub allow_plain_ldap: bool,
    /// PEM file with the CA that signed the server certificate. When set, only
    /// this CA is trusted; otherwise the system roots are used.
    #[serde(default)]
    pub ca_cert_file: String,
    #[serde(default)]
    pub ca_cert: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Service account for search mode.
    #[serde(default)]
    pub bind_dn: String,
    #[serde(default)]
    pub bind_password: String,
    #[serde(default)]
    pub bind_password_env: String,
    /// Direct mode bind name, `{username}` is replaced (DN-escaped).
    #[serde(default)]
    pub bind_dn_format: String,

    #[serde(default)]
    pub base_dn: String,
    /// `{username}` is replaced (filter-escaped). AD: `(sAMAccountName={username})`.
    #[serde(default = "default_user_filter")]
    pub user_filter: String,
    /// Attribute used as the local login name; empty uses the typed name.
    #[serde(default)]
    pub username_attribute: String,
    #[serde(default = "default_email_attribute")]
    pub email_attribute: String,
    #[serde(default = "default_display_name_attribute")]
    pub display_name_attribute: String,

    /// Attribute on the user entry listing group DNs. Empty to skip.
    #[serde(default = "default_member_of")]
    pub member_of_attribute: String,
    /// Also search for groups, including nested ones (AD transitive membership).
    #[serde(default)]
    pub nested_groups: bool,
    /// Group search filter with `{user_dn}` and `{username}`; overrides `nested_groups`.
    #[serde(default)]
    pub group_filter: String,
    #[serde(default)]
    pub group_base_dn: String,
    #[serde(default)]
    pub role_mapping: RoleMapping,
}

fn default_label() -> String {
    "LDAP".to_string()
}
fn default_timeout() -> u64 {
    10
}
fn default_user_filter() -> String {
    "(uid={username})".to_string()
}
fn default_email_attribute() -> String {
    "mail".to_string()
}
fn default_display_name_attribute() -> String {
    "displayName".to_string()
}
fn default_member_of() -> String {
    "memberOf".to_string()
}

impl Default for LdapConfig {
    fn default() -> Self {
        serde_yaml_neo::from_str("{}").expect("empty LDAP config deserializes")
    }
}

/// A directory user that proved their password.
#[derive(Debug, Clone)]
pub struct LdapUser {
    pub dn: String,
    /// Local login name (lowercase).
    pub username: String,
    pub email: String,
    pub display_name: String,
    /// Group DNs and their CNs, for role mapping.
    pub groups: Vec<String>,
}

#[derive(Debug)]
pub enum LdapLoginError {
    /// Unknown user or wrong password. The two are not distinguished.
    InvalidCredentials,
    /// Directory unreachable, TLS failure or misconfiguration.
    Directory(String),
}

impl std::fmt::Display for LdapLoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LdapLoginError::InvalidCredentials => f.write_str("invalid credentials"),
            LdapLoginError::Directory(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for LdapLoginError {}

fn directory_err(what: &str, e: impl std::fmt::Display) -> LdapLoginError {
    LdapLoginError::Directory(format!("{what}: {e}"))
}

pub struct LdapProvider {
    pub cfg: LdapConfig,
    service_password: String,
    tls: Arc<rustls::ClientConfig>,
}

fn parse_pem_certs(pem: &str) -> Result<Vec<rustls::pki_types::CertificateDer<'static>>, String> {
    rustls_pemfile::certs(&mut pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("invalid CA certificate PEM: {e}"))
}

/// TLS settings that verify the server: the configured CA only, or the system roots.
pub fn build_tls_config(cfg: &LdapConfig) -> Result<Arc<rustls::ClientConfig>, String> {
    let mut roots = rustls::RootCertStore::empty();
    let pem = if !cfg.ca_cert.trim().is_empty() {
        Some(cfg.ca_cert.clone())
    } else if !cfg.ca_cert_file.is_empty() {
        Some(
            std::fs::read_to_string(&cfg.ca_cert_file)
                .map_err(|e| format!("cannot read ca_cert_file {}: {e}", cfg.ca_cert_file))?,
        )
    } else {
        None
    };
    match pem {
        Some(pem) => {
            let certs = parse_pem_certs(&pem)?;
            if certs.is_empty() {
                return Err("ca_cert holds no certificate".into());
            }
            for c in certs {
                roots
                    .add(c)
                    .map_err(|e| format!("unusable CA certificate: {e}"))?;
            }
        }
        None => {
            let native = rustls_native_certs::load_native_certs();
            for c in native.certs {
                let _ = roots.add(c);
            }
            if roots.is_empty() {
                return Err("no system CA certificates found; set ldap.ca_cert_file".into());
            }
        }
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| e.to_string())?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

impl LdapProvider {
    pub fn new(cfg: LdapConfig) -> Result<Arc<Self>, String> {
        cfg.role_mapping
            .validate()
            .map_err(|e| format!("ldap.role_mapping: {e}"))?;
        let url = reqwest::Url::parse(&cfg.url).map_err(|e| format!("ldap.url: {e}"))?;
        match url.scheme() {
            "ldaps" => {}
            "ldap" if cfg.starttls || cfg.allow_plain_ldap => {}
            "ldap" => {
                return Err(
                    "ldap.url is cleartext ldap://: use ldaps://, set starttls: true, or allow_plain_ldap: true"
                        .into(),
                );
            }
            other => return Err(format!("ldap.url scheme '{other}' is not supported")),
        }
        if url.scheme() == "ldaps" && cfg.starttls {
            return Err("starttls applies to ldap:// URLs, not ldaps://".into());
        }
        if cfg.bind_dn.is_empty() && cfg.bind_dn_format.is_empty() {
            return Err("ldap needs bind_dn (search mode) or bind_dn_format (direct mode)".into());
        }
        if !cfg.bind_dn.is_empty() && cfg.base_dn.is_empty() {
            return Err("ldap search mode needs base_dn".into());
        }
        if !cfg.bind_dn_format.is_empty() && !cfg.bind_dn_format.contains("{username}") {
            return Err("ldap.bind_dn_format must contain {username}".into());
        }
        if !cfg.user_filter.contains("{username}") {
            return Err("ldap.user_filter must contain {username}".into());
        }
        let tls = build_tls_config(&cfg)?;
        let service_password = secret(&cfg.bind_password, &cfg.bind_password_env);
        Ok(Arc::new(Self {
            cfg,
            service_password,
            tls,
        }))
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.cfg.timeout_secs.max(1))
    }

    async fn connect(&self) -> Result<Ldap, LdapLoginError> {
        let settings = LdapConnSettings::new()
            .set_conn_timeout(self.timeout())
            .set_starttls(self.cfg.starttls)
            .set_config(self.tls.clone());
        let (conn, ldap) = LdapConnAsync::with_settings(settings, &self.cfg.url)
            .await
            .map_err(|e| directory_err("connecting to the directory failed", e))?;
        ldap3::drive!(conn);
        Ok(ldap)
    }

    /// Check a username and password against the directory.
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LdapUser, LdapLoginError> {
        validate_credentials(username, password)?;
        if self.cfg.bind_dn.is_empty() {
            self.authenticate_direct(username, password).await
        } else {
            self.authenticate_search(username, password).await
        }
    }

    async fn authenticate_search(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LdapUser, LdapLoginError> {
        let mut svc = self.connect().await?;
        let result = self.search_mode(&mut svc, username, password).await;
        let _ = svc.unbind().await;
        result
    }

    async fn search_mode(
        &self,
        svc: &mut Ldap,
        username: &str,
        password: &str,
    ) -> Result<LdapUser, LdapLoginError> {
        let t = self.timeout();
        svc.with_timeout(t)
            .simple_bind(&self.cfg.bind_dn, &self.service_password)
            .await
            .and_then(|r| r.success())
            .map_err(|e| directory_err("service account bind failed", e))?;

        let entry = self
            .find_user(svc, username)
            .await?
            .ok_or(LdapLoginError::InvalidCredentials)?;

        // Verify the password with a bind as the user on a separate connection,
        // so the service session stays authenticated for the group lookup.
        let mut user_conn = self.connect().await?;
        let bound = user_conn
            .with_timeout(t)
            .simple_bind(&entry.dn, password)
            .await;
        let _ = user_conn.unbind().await;
        match bound {
            Ok(r) if r.rc == 0 => {}
            Ok(r) if r.rc == RC_INVALID_CREDENTIALS => {
                return Err(LdapLoginError::InvalidCredentials);
            }
            Ok(r) => {
                return Err(directory_err(
                    "user bind failed",
                    format!("rc={} {}", r.rc, r.text),
                ));
            }
            Err(e) => return Err(directory_err("user bind failed", e)),
        }

        self.build_user(svc, username, entry).await
    }

    async fn authenticate_direct(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LdapUser, LdapLoginError> {
        let mut conn = self.connect().await?;
        let result = self.direct_mode(&mut conn, username, password).await;
        let _ = conn.unbind().await;
        result
    }

    async fn direct_mode(
        &self,
        conn: &mut Ldap,
        username: &str,
        password: &str,
    ) -> Result<LdapUser, LdapLoginError> {
        let t = self.timeout();
        let name = self
            .cfg
            .bind_dn_format
            .replace("{username}", &dn_escape(username));
        match conn.with_timeout(t).simple_bind(&name, password).await {
            Ok(r) if r.rc == 0 => {}
            Ok(r) if r.rc == RC_INVALID_CREDENTIALS => {
                return Err(LdapLoginError::InvalidCredentials);
            }
            Ok(r) => {
                return Err(directory_err(
                    "bind failed",
                    format!("rc={} {}", r.rc, r.text),
                ));
            }
            Err(e) => return Err(directory_err("bind failed", e)),
        }
        if self.cfg.base_dn.is_empty() {
            // No search base: no profile and no groups, the default role applies.
            return Ok(LdapUser {
                dn: name,
                username: username.to_lowercase(),
                email: String::new(),
                display_name: String::new(),
                groups: Vec::new(),
            });
        }
        let entry = self.find_user(conn, username).await?.ok_or_else(|| {
            LdapLoginError::Directory("authenticated, but the user filter finds no entry".into())
        })?;
        self.build_user(conn, username, entry).await
    }

    fn user_attrs(&self) -> Vec<String> {
        let mut a = Vec::new();
        for x in [
            &self.cfg.username_attribute,
            &self.cfg.email_attribute,
            &self.cfg.display_name_attribute,
            &self.cfg.member_of_attribute,
        ] {
            if !x.is_empty() {
                a.push(x.clone());
            }
        }
        a
    }

    async fn find_user(
        &self,
        ldap: &mut Ldap,
        username: &str,
    ) -> Result<Option<SearchEntry>, LdapLoginError> {
        let filter = self
            .cfg
            .user_filter
            .replace("{username}", &ldap_escape(username));
        let (entries, _) = ldap
            .with_timeout(self.timeout())
            .search(
                &self.cfg.base_dn,
                Scope::Subtree,
                &filter,
                self.user_attrs(),
            )
            .await
            .and_then(|r| r.success())
            .map_err(|e| directory_err("user search failed", e))?;
        let mut found: Vec<SearchEntry> = entries
            .into_iter()
            .map(SearchEntry::construct)
            // Search continuation references are not entries.
            .filter(|e| !e.dn.is_empty())
            .collect();
        match found.len() {
            0 => Ok(None),
            1 => Ok(found.pop()),
            n => {
                tracing::warn!(
                    "LDAP user filter matched {n} entries for one username; refusing the login"
                );
                Err(LdapLoginError::InvalidCredentials)
            }
        }
    }

    async fn build_user(
        &self,
        ldap: &mut Ldap,
        typed_username: &str,
        entry: SearchEntry,
    ) -> Result<LdapUser, LdapLoginError> {
        let first = |attr: &str| -> String {
            if attr.is_empty() {
                return String::new();
            }
            attr_values(&entry, attr)
                .into_iter()
                .next()
                .unwrap_or_default()
        };
        let username = {
            let from_dir = first(&self.cfg.username_attribute);
            if from_dir.is_empty() {
                typed_username.to_string()
            } else {
                from_dir
            }
        }
        .to_lowercase();

        let mut group_dns: Vec<String> = if self.cfg.member_of_attribute.is_empty() {
            Vec::new()
        } else {
            attr_values(&entry, &self.cfg.member_of_attribute)
        };
        let filter = if !self.cfg.group_filter.is_empty() {
            Some(self.cfg.group_filter.clone())
        } else if self.cfg.nested_groups {
            Some(format!("(member:{AD_IN_CHAIN}:={{user_dn}})"))
        } else {
            None
        };
        if let Some(filter) = filter {
            let filter = filter
                .replace("{user_dn}", &ldap_escape(&entry.dn))
                .replace("{username}", &ldap_escape(&username));
            let base = if self.cfg.group_base_dn.is_empty() {
                &self.cfg.base_dn
            } else {
                &self.cfg.group_base_dn
            };
            let (groups, _) = ldap
                .with_timeout(self.timeout())
                .search(base, Scope::Subtree, &filter, vec!["1.1"])
                .await
                .and_then(|r| r.success())
                .map_err(|e| directory_err("group search failed", e))?;
            group_dns.extend(groups.into_iter().map(|g| SearchEntry::construct(g).dn));
        }

        let mut groups = Vec::new();
        for dn in group_dns {
            if let Some(cn) = common_name(&dn) {
                groups.push(cn);
            }
            groups.push(dn);
        }
        groups.sort();
        groups.dedup();

        Ok(LdapUser {
            email: first(&self.cfg.email_attribute),
            display_name: first(&self.cfg.display_name_attribute),
            dn: entry.dn,
            username,
            groups,
        })
    }
}

/// Reject input that is empty (an empty password makes many servers accept an
/// unauthenticated bind as success) or malformed.
fn validate_credentials(username: &str, password: &str) -> Result<(), LdapLoginError> {
    let bad = username.trim().is_empty()
        || password.is_empty()
        || username.len() > MAX_USERNAME
        || username.chars().any(|c| c.is_control())
        || password.contains('\0');
    if bad {
        return Err(LdapLoginError::InvalidCredentials);
    }
    Ok(())
}

/// Attribute values, matching the attribute name case-insensitively.
fn attr_values(entry: &SearchEntry, name: &str) -> Vec<String> {
    entry
        .attrs
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

/// The first CN of a DN, with escapes resolved (`CN=Plant\, East,OU=...` -> `Plant, East`).
pub fn common_name(dn: &str) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut chars = dn.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(n) = chars.next() {
                    cur.push(n);
                }
            }
            ',' => parts.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    parts.push(cur);
    parts.into_iter().find_map(|p| {
        let (k, v) = p.split_once('=')?;
        k.trim()
            .eq_ignore_ascii_case("cn")
            .then(|| v.trim().to_string())
    })
}

#[cfg(test)]
mod tests;
