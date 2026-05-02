use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub smtp: Option<SmtpConfig>,
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
    #[serde(default = "default_register_area")]
    pub register_area: String,
}

fn default_register_area() -> String {
    "holding".to_string()
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

// ── DB-persisted Modbus types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModbusDevice {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub unit_id: u8,
    pub interval_ms: u64,
    pub endianness: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ModbusDevice {
    pub fn to_device_config(&self, registers: &[ModbusRegister]) -> DeviceConfig {
        DeviceConfig {
            id: self.id.clone(),
            ip: self.ip.clone(),
            port: self.port,
            unit_id: self.unit_id,
            interval_ms: self.interval_ms,
            endianness: match self.endianness.as_str() {
                "little" => Endianness::Little,
                "swapped" => Endianness::Swapped,
                _ => Endianness::Big,
            },
            registers: registers.iter().map(|r| RegisterConfig {
                name: r.name.clone(),
                address: r.address,
                reg_type: match r.register_type.as_str() {
                    "int16" => RegisterType::Int16,
                    "uint16" => RegisterType::Uint16,
                    "int32" => RegisterType::Int32,
                    _ => RegisterType::Float32,
                },
                scale: r.scale,
                register_area: r.register_area.clone(),
            }).collect(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModbusRegister {
    pub id: i64,
    pub device_id: String,
    pub name: String,
    pub address: u16,
    pub register_type: String,
    pub register_area: String,
    pub scale: f64,
    pub sort_order: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateModbusDeviceRequest {
    pub id: String,
    pub name: String,
    pub ip: String,
    #[serde(default = "default_modbus_port")]
    pub port: u16,
    #[serde(default = "default_unit_id")]
    pub unit_id: u8,
    #[serde(default = "default_interval")]
    pub interval_ms: u64,
    #[serde(default)]
    pub endianness: String,
    pub registers: Vec<ModbusRegisterInput>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ModbusRegisterInput {
    pub name: String,
    pub address: u16,
    #[serde(default = "default_reg_type_str")]
    pub register_type: String,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default = "default_register_area")]
    pub register_area: String,
}

fn default_reg_type_str() -> String {
    "float32".to_string()
}

#[derive(Debug, serde::Deserialize)]
pub struct ScanRegisterRequest {
    pub ip: String,
    #[serde(default = "default_modbus_port")]
    pub port: u16,
    #[serde(default = "default_unit_id")]
    pub unit_id: u8,
    pub start_addr: u16,
    pub end_addr: u16,
    #[serde(default)]
    pub endianness: String,
    #[serde(default = "default_scan_register_area")]
    pub register_area: String,
    #[serde(default = "default_scan_delay")]
    pub scan_delay_ms: u64,
}

fn default_scan_delay() -> u64 {
    500
}

fn default_scan_register_area() -> String {
    "holding".to_string()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScannedRegister {
    pub address: u16,
    pub raw_value: Option<u16>,
    pub raw_value2: Option<u16>,
    pub changed: bool,
    pub readable: bool,
    pub likely_type: String,
    pub interpreted_value: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PollerStatus {
    pub device_id: String,
    pub running: bool,
    pub status: String,
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
    pub db_size_mb: f64,
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
    #[serde(default = "default_user_role")]
    pub role: String,
    #[serde(default)]
    pub must_change_password: bool,
    #[serde(skip_serializing)]
    pub failed_attempts: i64,
    #[serde(skip_serializing)]
    pub locked_until: Option<i64>,
}

fn default_user_role() -> String {
    "user".to_string()
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
    pub equipment_name: String,
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

// ── SCADA 2D view types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScadaEquipment {
    pub screen_id: String,
    pub equipment_id: String,
    pub equipment_type: String,
    pub equipment_name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub sensor_bindings: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScadaConnection {
    pub screen_id: String,
    pub connection_id: String,
    pub from_equipment: String,
    pub to_equipment: String,
    pub waypoints: String,
    pub connection_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserShortcut {
    pub user_id: i64,
    pub data_source_id: i64,
    pub shortcut: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NavShortcut {
    pub user_id: i64,
    pub target: String,
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
    pub role: String,
    #[serde(default)]
    pub must_change_password: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    #[serde(default = "default_user_role")]
    pub role: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChangePasswordRequest {
    pub new_password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenClaims {
    pub username: String,
    pub exp: u64,
    #[serde(default = "default_user_role")]
    pub role: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefreshToken {
    pub id: i64,
    pub token_hash: String,
    pub username: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub revoked_at: Option<i64>,
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
    pub status: String,
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
    pub source_type: String,
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
    pub detail: Option<String>,
    pub agg: Option<String>,
}

/// Query parameters for the /api/v1/data/export endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct ExportQuery {
    /// Start time: ISO 8601 string or unix timestamp (seconds).
    /// Parsed to i64 unix timestamp.
    pub from: Option<String>,
    /// End time: ISO 8601 string or unix timestamp (seconds).
    pub to: Option<String>,
    /// Filter by device ID.
    pub device: Option<String>,
    /// Filter by metric name.
    pub metric: Option<String>,
    /// Aggregation function: avg, min, max, count, sum. Default: raw (no aggregation).
    pub aggregation: Option<String>,
    /// Aggregation window: 1m, 5m, 15m, 1h, 1d.
    pub interval: Option<String>,
    /// Pagination: max rows to return (default 1000, max 10000).
    pub limit: Option<usize>,
    /// Pagination: rows to skip (default 0).
    pub offset: Option<usize>,
    /// Response format: json (default) or csv.
    pub format: Option<String>,
}

/// A single telemetry row for export responses.
#[derive(Debug, serde::Serialize)]
pub struct TelemetryExportRow {
    pub timestamp: i64,
    pub device: String,
    pub metric: String,
    pub value: f64,
}

/// Paginated meta information for export responses.
#[derive(Debug, serde::Serialize)]
pub struct ExportMeta {
    pub total: i64,
    pub limit: usize,
    pub offset: usize,
    pub aggregation: String,
    pub interval: String,
}

/// The standard envelope for JSON export responses.
#[derive(Debug, serde::Serialize)]
pub struct ExportResponse {
    pub data: Vec<TelemetryExportRow>,
    pub meta: ExportMeta,
}

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
    pub cooldown_until: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AlarmQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

// ── Notification settings ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NotificationSetting {
    pub id: i64,
    pub source_key: String,
    pub channel_type: String,
    pub config: String,
    pub enabled: bool,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateNotificationSettingRequest {
    pub source_key: String,
    pub channel_type: String,
    pub config: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateNotificationSettingRequest {
    pub source_key: Option<String>,
    pub channel_type: Option<String>,
    pub config: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TestNotificationRequest {
    pub channel_type: String,
    pub config: String,
    pub source_key: Option<String>,
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

// ── Flow types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Flow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger_config: String,
    pub nodes: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlowRun {
    pub id: i64,
    pub flow_id: String,
    pub started_at: i64,
    pub duration_ms: i64,
    pub success: bool,
    pub error: Option<String>,
    pub output: Option<String>,
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
            register_area: "holding".into(),
        };
        assert_eq!(r.register_count(), 1);

        let r = RegisterConfig {
            name: "v".into(),
            address: 40001,
            reg_type: RegisterType::Float32,
            scale: 1.0,
            register_area: "holding".into(),
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
            register_area: "holding".into(),
        };
        assert_eq!(r.protocol_address(), 0);

        let r = RegisterConfig {
            name: "v".into(),
            address: 40003,
            reg_type: RegisterType::Float32,
            scale: 1.0,
            register_area: "holding".into(),
        };
        assert_eq!(r.protocol_address(), 2);

        let r = RegisterConfig {
            name: "v".into(),
            address: 30005,
            reg_type: RegisterType::Float32,
            scale: 1.0,
            register_area: "input".into(),
        };
        assert_eq!(r.protocol_address(), 4);

        let r = RegisterConfig {
            name: "v".into(),
            address: 10,
            reg_type: RegisterType::Uint16,
            scale: 1.0,
            register_area: "holding".into(),
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

#[derive(Debug, Clone)]
pub struct LicenseActivationRow {
    pub license_key: String,
    pub fingerprint_hash: String,
    pub activation_blob: Vec<u8>,
    pub fingerprint_hostname: String,
    pub fingerprint_cpu: String,
    pub fingerprint_disk_serial: String,
    pub fingerprint_mac: String,
    pub fingerprint_machine_id: String,
    pub activated_at: String,
    pub expires_at: Option<String>,
    pub tier: Option<String>,
    pub device_limit: Option<u32>,
}
