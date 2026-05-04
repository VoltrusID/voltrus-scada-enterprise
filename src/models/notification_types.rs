use serde::Deserialize;

// ── SMTP notification config ──

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    pub username: String,
    pub password: String,
    #[serde(default = "default_smtp_from_address")]
    pub from_address: String,
    #[serde(default = "default_smtp_from_name")]
    pub from_name: String,
    #[serde(default = "default_smtp_encryption")]
    pub encryption: SmtpEncryption,
}

fn default_smtp_port() -> u16 {
    587
}

fn default_smtp_from_address() -> String {
    "voltrus@example.com".to_string()
}

fn default_smtp_from_name() -> String {
    "Voltrus Lite".to_string()
}

fn default_smtp_encryption() -> SmtpEncryption {
    SmtpEncryption::StartTls
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SmtpEncryption {
    StartTls,
    ImplicitTls,
    None,
}

// ── Push notification config ──

#[derive(Debug, Clone, Deserialize)]
pub struct PushConfig {
    pub vapid_private_key: String,
    pub vapid_public_key: String,
    #[serde(default = "default_push_subject")]
    pub vapid_subject: String,
}

fn default_push_subject() -> String {
    "mailto:admin@example.com".to_string()
}
