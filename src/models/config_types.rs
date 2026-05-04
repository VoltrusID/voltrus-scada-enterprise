use serde::Deserialize;
use utoipa::ToSchema;

// ── Config types ──

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub devices: Vec<DeviceConfig>,
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default = "default_db_path")]
    pub db_path: String,
    #[serde(default = "default_persist_interval")]
    pub persist_interval_secs: u64,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub auth_secret: Option<String>,
    #[serde(default)]
    pub cors_origin: Option<String>,
    #[serde(default = "default_cookie_secure")]
    pub cookie_secure: bool,
    #[serde(default)]
    pub retention: RetentionConfig,
    #[serde(default)]
    pub oidc_enabled: bool,
    #[serde(default)]
    pub oidc_issuer: String,
    #[serde(default)]
    pub oidc_audience: String,
    #[serde(default)]
    pub oidc_jwks_url: String,
    #[serde(default)]
    pub oidc_role_claim: String,
    #[serde(default)]
    pub oidc_admin_role: String,
    #[serde(default)]
    pub db_encryption_key: Option<String>,
    #[serde(default)]
    pub smtp: Option<super::notification_types::SmtpConfig>,
    #[serde(default)]
    pub push: Option<super::notification_types::PushConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetentionConfig {
    #[serde(default = "default_retention_enabled")]
    pub enabled: bool,
    #[serde(default = "default_run_interval")]
    pub run_interval_secs: u64,
    #[serde(default = "default_archive_dir")]
    pub archive_dir: String,
    #[serde(default = "default_tiers")]
    pub tiers: Vec<RetentionTier>,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            enabled: default_retention_enabled(),
            run_interval_secs: default_run_interval(),
            archive_dir: default_archive_dir(),
            tiers: default_tiers(),
        }
    }
}

fn default_retention_enabled() -> bool { true }
fn default_run_interval() -> u64 { 21600 }
fn default_archive_dir() -> String { "./archive".to_string() }

fn default_tiers() -> Vec<RetentionTier> {
    vec![
        RetentionTier {
            name: "raw".into(),
            keep: Some(604800),
            downsample_to: Some("hourly".into()),
            archive: true,
        },
        RetentionTier {
            name: "hourly".into(),
            keep: Some(7776000),
            downsample_to: Some("daily".into()),
            archive: false,
        },
        RetentionTier {
            name: "daily".into(),
            keep: None,
            downsample_to: None,
            archive: false,
        },
    ]
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetentionTier {
    pub name: String,
    pub keep: Option<u64>,
    pub downsample_to: Option<String>,
    #[serde(default)]
    pub archive: bool,
}

fn default_cookie_secure() -> bool {
    true
}

fn default_server_port() -> u16 {
    3000
}
fn default_db_path() -> String {
    "data.db".to_string()
}
fn default_persist_interval() -> u64 {
    5
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceConfig {
    pub id: String,
    pub ip: String,
    #[serde(default = "default_modbus_port")]
    pub port: u16,
    #[serde(default = "default_unit_id")]
    pub unit_id: u8,
    #[serde(default = "default_interval")]
    pub interval_ms: u64,
    #[serde(default)]
    pub endianness: Endianness,
    pub registers: Vec<RegisterConfig>,
}

pub fn default_modbus_port() -> u16 {
    502
}
pub fn default_unit_id() -> u8 {
    1
}
pub fn default_interval() -> u64 {
    1000
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Endianness {
    #[default]
    Big,
    Little,
    Swapped,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RegisterConfig {
    pub name: String,
    pub address: u16,
    #[serde(default = "default_reg_type", rename = "type")]
    pub reg_type: RegisterType,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default = "default_register_area")]
    pub register_area: String,
}

pub fn default_register_area() -> String {
    "holding".to_string()
}

pub fn default_reg_type() -> RegisterType {
    RegisterType::Float32
}
pub fn default_scale() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RegisterType {
    Int16,
    Uint16,
    Int32,
    Float32,
}

impl RegisterConfig {
    pub fn register_count(&self) -> u16 {
        match self.reg_type {
            RegisterType::Int16 | RegisterType::Uint16 => 1,
            RegisterType::Int32 | RegisterType::Float32 => 2,
        }
    }

    /// Convert Modbus 5-digit address (e.g. 40001) to 0-based protocol address.
    pub fn protocol_address(&self) -> u16 {
        if self.address >= 40001 {
            self.address - 40001
        } else if self.address >= 30001 {
            self.address - 30001
        } else {
            self.address
        }
    }
}
