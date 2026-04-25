use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Config types ──

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
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

fn default_modbus_port() -> u16 {
    502
}
fn default_unit_id() -> u8 {
    1
}
fn default_interval() -> u64 {
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
}

fn default_reg_type() -> RegisterType {
    RegisterType::Float32
}
fn default_scale() -> f64 {
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

// ── Runtime types ──

#[derive(Debug, Clone, Serialize)]
pub struct DeviceSnapshot {
    pub device_id: String,
    pub status: DeviceStatus,
    pub timestamp: i64,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
}

// ── API response types ──

#[derive(Debug, Serialize)]
pub struct LiveResponse {
    pub devices: Vec<DeviceSnapshot>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub devices: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DataPoint {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub device_id: String,
    pub metric: String,
    pub from: Option<i64>,
    pub to: Option<i64>,
}

// ── System response types ──

#[derive(Debug, serde::Serialize)]
pub struct SystemResponse {
    pub hostname: String,
    pub os: String,
    pub uptime_secs: u64,
    pub cpu_usage: f32,
    pub memory_total_mb: f64,
    pub memory_used_mb: f64,
    pub memory_percent: f64,
    pub disk_total_gb: f64,
    pub disk_used_gb: f64,
    pub disk_percent: f64,
    pub process: ProcessInfo,
}

#[derive(Debug, serde::Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub memory_mb: f64,
    pub cpu_percent: f32,
    pub uptime_secs: u64,
    pub binary_size_mb: f64,
}

// ── Application entities ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    #[serde(skip_serializing)]
    pub salt: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataSource {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub unit: String,
    pub color: String,
    pub min_val: f64,
    pub max_val: f64,
    pub warn_val: f64,
    pub crit_val: f64,
    pub base_val: f64,
    pub sort_order: i64,
    pub decimals: i64,
    pub source_type: String,
    pub source_config: String,
    pub device_id: String,
    pub equipment_id: Option<String>,
    pub chart_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenDataSource {
    pub screen_id: String,
    pub data_source_id: i64,
    pub sort_order: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenLayoutRow {
    pub screen_id: String,
    pub breakpoint: String,
    pub layout: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TwinLayout {
    pub screen_id: String,
    pub equipment_id: String,
    pub equipment_type: String,
    pub position_x: f64,
    pub position_z: f64,
    pub rotation_y: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TwinConnection {
    pub screen_id: String,
    pub from_id: String,
    pub to_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanvasLayoutItem {
    pub screen_id: String,
    pub data_source_id: i64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserShortcut {
    pub user_id: i64,
    pub data_source_id: i64,
    pub shortcut: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

// ── Request/Response types ──

#[derive(Debug, serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LoginResponse {
    pub ok: bool,
    pub username: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenClaims {
    pub username: String,
    pub exp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrandingSettings {
    pub title: String,
    pub subtitle: String,
    pub logo: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DataSourceData {
    pub id: i64,
    pub key: String,
    pub value: f64,
    pub warn: f64,
    pub crit: f64,
    pub source_type: String,
}

#[derive(Debug, serde::Serialize)]
pub struct HistoryResponse {
    pub sensors: Vec<DataSourceSummary>,
    pub data: serde_json::Value,
    pub range: String,
    pub interval: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DataSourceSummary {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub unit: String,
    pub color: String,
    pub warn_val: f64,
    pub crit_val: f64,
    pub decimals: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct DataQuery {
    pub screen: Option<String>,
    pub device: Option<String>,
    pub range: Option<String>,
    pub sensor: Option<String>,
    pub device_id: Option<String>,
    pub metric: Option<String>,
    pub from: Option<i64>,
    pub to: Option<i64>,
}

// ── Alarm types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Alarm {
    pub id: i64,
    pub source_key: String,
    pub source_name: String,
    pub severity: String,
    pub value: f64,
    pub threshold: f64,
    pub message: String,
    pub triggered_at: i64,
    pub resolved_at: Option<i64>,
    pub acknowledged_at: Option<i64>,
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AlarmQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

// ── Push subscription (groundwork for Web Push) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PushSubscription {
    pub id: i64,
    pub user: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth_key: String,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct PushSubscribeRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth_key: String,
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_count() {
        let r = RegisterConfig {
            name: "v".into(),
            address: 40001,
            reg_type: RegisterType::Uint16,
            scale: 1.0,
        };
        assert_eq!(r.register_count(), 1);

        let r = RegisterConfig {
            name: "v".into(),
            address: 40001,
            reg_type: RegisterType::Float32,
            scale: 1.0,
        };
        assert_eq!(r.register_count(), 2);
    }

    #[test]
    fn test_protocol_address() {
        let r = RegisterConfig {
            name: "v".into(),
            address: 40001,
            reg_type: RegisterType::Float32,
            scale: 1.0,
        };
        assert_eq!(r.protocol_address(), 0);

        let r = RegisterConfig {
            name: "v".into(),
            address: 40003,
            reg_type: RegisterType::Float32,
            scale: 1.0,
        };
        assert_eq!(r.protocol_address(), 2);

        let r = RegisterConfig {
            name: "v".into(),
            address: 30005,
            reg_type: RegisterType::Float32,
            scale: 1.0,
        };
        assert_eq!(r.protocol_address(), 4);

        let r = RegisterConfig {
            name: "v".into(),
            address: 10,
            reg_type: RegisterType::Uint16,
            scale: 1.0,
        };
        assert_eq!(r.protocol_address(), 10);
    }

    #[test]
    fn test_default_scale() {
        assert_eq!(default_scale(), 1.0);
    }

    #[test]
    fn test_data_point_serde() {
        let dp = DataPoint {
            timestamp: 1710000000,
            value: 220.5,
        };
        let json = serde_json::to_string(&dp).unwrap();
        assert!(json.contains("\"timestamp\":1710000000"));
        assert!(json.contains("\"value\":220.5"));

        let back: DataPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(back, dp);
    }
}
