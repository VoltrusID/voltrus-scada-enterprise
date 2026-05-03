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
    pub compression: CompressionConfig,
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
    #[serde(default)]
    pub push: Option<PushConfig>,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompressionConfig {
    #[serde(default = "default_compression_enabled")]
    pub enabled: bool,
    #[serde(default = "default_hot_retention_secs")]
    pub hot_retention_secs: u64,
    #[serde(default = "default_warm_retention_secs")]
    pub warm_retention_secs: u64,
    #[serde(default = "default_cold_retention_secs")]
    pub cold_retention_secs: u64,
    #[serde(default = "default_compress_interval_secs")]
    pub hot_to_warm_interval_secs: u64,
    #[serde(default = "default_compress_cold_interval_secs")]
    pub warm_to_cold_interval_secs: u64,
    #[serde(default)]
    pub edge_mode: bool,
}

fn default_compression_enabled() -> bool { true }
fn default_hot_retention_secs() -> u64 { 86400 }
fn default_warm_retention_secs() -> u64 { 2592000 }
fn default_cold_retention_secs() -> u64 { 31536000 }
fn default_compress_interval_secs() -> u64 { 3600 }
fn default_compress_cold_interval_secs() -> u64 { 86400 }

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: default_compression_enabled(),
            hot_retention_secs: default_hot_retention_secs(),
            warm_retention_secs: default_warm_retention_secs(),
            cold_retention_secs: default_cold_retention_secs(),
            hot_to_warm_interval_secs: default_compress_interval_secs(),
            warm_to_cold_interval_secs: default_compress_cold_interval_secs(),
            edge_mode: false,
        }
    }
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ScannedRegister {
    pub address: u16,
    pub raw_value: Option<u16>,
    pub raw_value2: Option<u16>,
    pub changed: bool,
    pub readable: bool,
    pub likely_type: String,
    pub interpreted_value: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct PollerStatus {
    pub device_id: String,
    pub running: bool,
    pub status: String,
}

// ── OPC-UA config types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum OpcUaSecurityPolicy {
    None,
    #[serde(rename = "basic256sha256")]
    Basic256Sha256,
}

impl Default for OpcUaSecurityPolicy {
    fn default() -> Self { Self::None }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum OpcUaSecurityMode {
    None,
    Sign,
    #[serde(rename = "signandencrypt")]
    SignAndEncrypt,
}

impl Default for OpcUaSecurityMode {
    fn default() -> Self { Self::None }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpcUaNodeConfig {
    pub node_id: String,
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub data_type: String,
}

/// Runtime config for an OPC-UA device, used by the poller.
#[derive(Debug, Clone)]
pub struct OpcUaDeviceConfig {
    pub id: String,
    pub url: String,
    pub security_policy: OpcUaSecurityPolicy,
    pub security_mode: OpcUaSecurityMode,
    pub username: Option<String>,
    pub password: Option<String>,
    pub interval_ms: u64,
    pub nodes: Vec<OpcUaNodeConfig>,
}

/// Database row for opcua_devices table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OpcUaDevice {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub security_policy: OpcUaSecurityPolicy,
    pub security_mode: OpcUaSecurityMode,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub created_at: i64,
}

/// Database row for opcua_nodes table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OpcUaNode {
    pub id: i64,
    pub device_id: i64,
    pub node_id: String,
    pub name: String,
    pub display_name: String,
    pub data_type: String,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateOpcUaDeviceRequest {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub security_policy: OpcUaSecurityPolicy,
    #[serde(default)]
    pub security_mode: OpcUaSecurityMode,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateOpcUaDeviceRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub security_policy: Option<OpcUaSecurityPolicy>,
    pub security_mode: Option<OpcUaSecurityMode>,
    pub username: Option<Option<String>>,
    pub password: Option<Option<String>>,
    pub interval_ms: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateOpcUaNodeRequest {
    pub node_id: String,
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub data_type: String,
}

// ── S7 (Siemens) device types ──

/// Database row for s7_devices table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct S7Device {
    pub id: i64,
    pub name: String,
    pub url: String,
    #[serde(default = "default_s7_rack")]
    pub rack: u8,
    #[serde(default = "default_s7_slot")]
    pub slot: u8,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub created_at: i64,
}

fn default_s7_rack() -> u8 { 0 }
fn default_s7_slot() -> u8 { 1 }

/// Database row for s7_tags table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct S7TagRow {
    pub id: i64,
    pub device_id: i64,
    pub tag_address: String,
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub data_type: String,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateS7DeviceRequest {
    pub name: String,
    pub url: String,
    #[serde(default = "default_s7_rack")]
    pub rack: u8,
    #[serde(default = "default_s7_slot")]
    pub slot: u8,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateS7DeviceRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub rack: Option<u8>,
    pub slot: Option<u8>,
    pub interval_ms: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateS7TagRequest {
    pub tag_address: String,
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub data_type: String,
}

// ── EtherNet/IP (Allen-Bradley) device types ──

/// Database row for enip_devices table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnipDevice {
    pub id: i64,
    pub name: String,
    pub url: String,
    #[serde(default = "default_enip_slot")]
    pub slot: u8,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub created_at: i64,
}

fn default_enip_slot() -> u8 { 0 }

/// Database row for enip_tags table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct EnipTag {
    pub id: i64,
    pub device_id: i64,
    pub tag_name: String,
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub data_type: String,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateEnipDeviceRequest {
    pub name: String,
    pub url: String,
    #[serde(default = "default_enip_slot")]
    pub slot: u8,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateEnipDeviceRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub slot: Option<u8>,
    pub interval_ms: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateEnipTagRequest {
    pub tag_name: String,
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub data_type: String,
}

/// Runtime config for an EtherNet/IP device, used by the poller.
#[derive(Debug, Clone)]
pub struct EnipDeviceConfig {
    pub id: String,
    pub url: String,
    pub slot: u8,
    pub interval_ms: u64,
    pub tags: Vec<crate::ethernet_ip::EnipTagConfig>,
}

// ── BACnet/IP device types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BacnetDevice {
    pub id: i64,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub device_instance: u32,
    pub interval_ms: u64,
    pub cov_enabled: bool,
    pub cov_lifetime: u32,
    pub enabled: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BacnetObject {
    pub id: i64,
    pub device_db_id: i64,
    pub object_type: String,
    pub object_instance: u32,
    pub property: String,
    pub name: String,
    pub display_name: String,
    pub cov_increment: f64,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateBacnetDeviceRequest {
    pub name: String,
    pub ip: String,
    #[serde(default = "default_bacnet_port")]
    pub port: u16,
    #[serde(default)]
    pub device_instance: u32,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default)]
    pub cov_enabled: bool,
    #[serde(default = "default_cov_lifetime")]
    pub cov_lifetime: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_bacnet_port() -> u16 { 47808 }
fn default_cov_lifetime() -> u32 { 300 }

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateBacnetDeviceRequest {
    pub name: Option<String>,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub device_instance: Option<u32>,
    pub interval_ms: Option<u64>,
    pub cov_enabled: Option<bool>,
    pub cov_lifetime: Option<u32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateBacnetObjectRequest {
    pub object_type: String,
    pub object_instance: u32,
    #[serde(default = "default_bacnet_property")]
    pub property: String,
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub cov_increment: f64,
}

fn default_bacnet_property() -> String { "presentValue".to_string() }

#[derive(Debug, Clone)]
pub struct BacnetDeviceConfig {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub device_instance: u32,
    pub interval_ms: u64,
    pub cov_enabled: bool,
    pub cov_lifetime: u32,
    pub objects: Vec<BacnetObjectConfig>,
}

#[derive(Debug, Clone)]
pub struct BacnetObjectConfig {
    pub object_type: String,
    pub object_instance: u32,
    pub property: String,
    pub name: String,
    pub cov_increment: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct BacnetDiscoveredDevice {
    pub device_instance: u32,
    pub ip: String,
    pub port: u16,
    pub vendor_name: Option<String>,
    pub model_name: Option<String>,
    pub firmware_version: Option<String>,
    pub object_list_count: Option<u32>,
}

// ── Runtime types ──

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct DeviceSnapshot {
    pub device_id: String,
    pub status: DeviceStatus,
    pub timestamp: i64,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
}

// ── API response types ──

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LiveResponse {
    pub devices: Vec<DeviceSnapshot>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
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

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
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

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Device {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ScreenDataSource {
    pub screen_id: String,
    pub data_source_id: i64,
    pub sort_order: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ScreenLayoutRow {
    pub screen_id: String,
    pub breakpoint: String,
    pub layout: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct TwinLayout {
    pub screen_id: String,
    pub equipment_id: String,
    pub equipment_type: String,
    pub equipment_name: String,
    pub position_x: f64,
    pub position_z: f64,
    pub rotation_y: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct TwinConnection {
    pub screen_id: String,
    pub from_id: String,
    pub to_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CanvasLayoutItem {
    pub screen_id: String,
    pub data_source_id: i64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// ── SCADA 2D view types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
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

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct BrandingSettings {
    pub title: String,
    pub subtitle: String,
    pub logo: String,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DataSourceData {
    pub id: i64,
    pub key: String,
    pub value: f64,
    pub warn: f64,
    pub crit: f64,
    pub source_type: String,
    pub status: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HistoryResponse {
    pub sensors: Vec<DataSourceSummary>,
    pub data: serde_json::Value,
    pub range: String,
    pub interval: u64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
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

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
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
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TelemetryExportRow {
    pub timestamp: i64,
    pub device: String,
    pub metric: String,
    pub value: f64,
}

/// Paginated meta information for export responses.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ExportMeta {
    pub total: i64,
    pub limit: usize,
    pub offset: usize,
    pub aggregation: String,
    pub interval: String,
}

/// The standard envelope for JSON export responses.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
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

// ── Alarm types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Alarm {
    pub id: i64,
    pub source_key: String,
    pub source_name: String,
    pub severity: String,
    pub priority: String,
    pub value: f64,
    pub threshold: f64,
    pub message: String,
    pub triggered_at: i64,
    pub resolved_at: Option<i64>,
    pub acknowledged_at: Option<i64>,
    pub acknowledged_by: Option<String>,
    pub cooldown_until: Option<i64>,
    pub shelved_until: Option<i64>,
    pub shelved_by: Option<String>,
    pub shelved_reason: Option<String>,
    pub deadband: f64,
    pub state: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AlarmQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

// ── Alarm Management Overhaul types (v0.23.0) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlarmAuditEntry {
    pub id: i64,
    pub alarm_id: i64,
    pub action: String,
    pub performed_by: String,
    pub details: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ShelfAlarmRequest {
    pub duration_secs: u64,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlarmSummary {
    pub active_critical: i64,
    pub active_warning: i64,
    pub active_info: i64,
    pub shelved: i64,
    pub recent_24h: i64,
    pub avg_ack_time_secs: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AlarmAnalyticsEntry {
    pub source_key: String,
    pub source_name: String,
    pub trigger_count: i64,
    pub avg_time_to_ack_secs: f64,
    pub last_triggered_at: i64,
}

// ── Notification settings ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct NotificationSetting {
    pub id: i64,
    pub source_key: String,
    pub channel_type: String,
    pub config: String,
    pub enabled: bool,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateNotificationSettingRequest {
    pub source_key: String,
    pub channel_type: String,
    pub config: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_interval_ms() -> u64 { 1000 }

fn default_true() -> bool {
    true
}

fn default_retention_hours() -> u32 { 72 }

// ── RTSP Video / Camera types ──

/// Database row for cameras table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Camera {
    pub id: i64,
    pub name: String,
    pub url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub snapshot_on_alarm: bool,
    #[serde(default = "default_retention_hours")]
    pub retention_hours: u32,
    pub created_at: i64,
}

/// Database row for camera_snapshots table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CameraSnapshot {
    pub id: i64,
    pub camera_id: i64,
    pub timestamp: i64,
    pub alarm_source: Option<String>,
    pub file_path: String,
    pub file_size: u64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateCameraRequest {
    pub name: String,
    pub url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub snapshot_on_alarm: bool,
    #[serde(default = "default_retention_hours")]
    pub retention_hours: u32,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateCameraRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub enabled: Option<bool>,
    pub snapshot_on_alarm: Option<bool>,
    pub retention_hours: Option<u32>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateNotificationSettingRequest {
    pub source_key: Option<String>,
    pub channel_type: Option<String>,
    pub config: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct TestNotificationRequest {
    pub channel_type: String,
    pub config: String,
    pub source_key: Option<String>,
}

// ── Push subscription (groundwork for Web Push) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PushSubscription {
    pub id: i64,
    pub user: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth_key: String,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct PushSubscribeRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth_key: String,
}

// ── Flow types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Flow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger_config: String,
    pub nodes: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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

    #[test]
    fn test_machine_state_serde() {
        let state = MachineState::Running;
        let json = serde_json::to_string(&state).unwrap();
        let expected = r#""running""#;
        assert_eq!(json, expected);
        let back: MachineState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, MachineState::Running);
        assert!(MachineState::from_str("running").unwrap() == MachineState::Running);
        assert!(MachineState::from_str("idle").unwrap() == MachineState::Idle);
        assert!(MachineState::from_str("faulted").unwrap() == MachineState::Faulted);
        assert!(MachineState::from_str("setup").unwrap() == MachineState::Setup);
        assert!(MachineState::from_str("maintenance").unwrap() == MachineState::Maintenance);
        assert!(MachineState::from_str("unknown").is_none());
    }

    #[test]
    fn test_create_downtime_event_request_deserialize() {
        let json = r#"{"device_id":"dev1","state":"faulted"}"#;
        let req: CreateDowntimeEventRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.device_id, "dev1");
        assert_eq!(req.state, "faulted");
    }

    #[test]
    fn test_oee_metrics_structure() {
        let metrics = OeeMetrics {
            availability: 0.85,
            performance: 0.90,
            quality: 0.95,
            oee: 0.85 * 0.90 * 0.95,
            total_running_secs: 3600,
            total_planned_secs: 4235,
            downtime_count: 3,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["availability"], 0.85);
        assert_eq!(parsed["total_running_secs"], 3600);
    }

    #[test]
    fn test_permission_constants() {
        assert_eq!(PERM_SCREENS_READ, "screens:read");
        assert_eq!(PERM_SCREENS_WRITE, "screens:write");
        assert_eq!(ALL_PERMISSIONS.len(), 18);
        assert!(ALL_PERMISSIONS.contains(&PERM_USERS_MANAGE));
    }

    #[test]
    fn test_role_serde() {
        let role = Role {
            id: 1,
            name: "Admin".to_string(),
            permissions: r#"["screens:read"]"#.to_string(),
            created_at: 1000,
        };
        let json = serde_json::to_string(&role).unwrap();
        assert!(json.contains("\"name\":\"Admin\""));
        let back: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 1);
        assert_eq!(back.name, "Admin");
    }

    #[test]
    fn test_create_role_request_deserialize() {
        let json = r#"{"name":"Operator","permissions":"[\"screens:read\"]"}"#;
        let req: CreateRoleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Operator");
        assert_eq!(req.permissions, r#"["screens:read"]"#);
    }

    #[test]
    fn test_command_status_roundtrip() {
        for status in [
            CommandStatus::Pending, CommandStatus::Queued, CommandStatus::Sent,
            CommandStatus::Acked, CommandStatus::Failed, CommandStatus::Expired,
            CommandStatus::Cancelled,
        ] {
            let s = status.as_str();
            assert_eq!(CommandStatus::from_str(s), Some(status.clone()));
            // serde round-trip
            let json = serde_json::to_string(&status).unwrap();
            let parsed: CommandStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
        assert!(CommandStatus::from_str("unknown").is_none());
        assert!(CommandStatus::Acked.is_terminal());
        assert!(!CommandStatus::Pending.is_terminal());
    }

    #[test]
    fn test_write_policy_update_serde() {
        let json = r#"{"writable": true, "write_min": null}"#;
        let parsed: WritePolicyUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.writable, Some(true));
        assert_eq!(parsed.write_min, Some(None));
        // Fields not in JSON should be None (not provided)
        assert_eq!(parsed.write_max, None);
    }
}

// -- OEE / Downtime types --

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineState {
    Running,
    Idle,
    Faulted,
    Setup,
    Maintenance,
}

impl MachineState {
    pub fn as_str(&self) -> &'static str {
        match self {
            MachineState::Running => "running",
            MachineState::Idle => "idle",
            MachineState::Faulted => "faulted",
            MachineState::Setup => "setup",
            MachineState::Maintenance => "maintenance",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "running" => Some(MachineState::Running),
            "idle" => Some(MachineState::Idle),
            "faulted" => Some(MachineState::Faulted),
            "setup" => Some(MachineState::Setup),
            "maintenance" => Some(MachineState::Maintenance),
            _ => None,
        }
    }
}

// ── Command & Control types (v0.24.0) ──

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Pending,
    Queued,
    Sent,
    Acked,
    Failed,
    Expired,
    Cancelled,
}

impl CommandStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Queued => "queued",
            Self::Sent => "sent",
            Self::Acked => "acked",
            Self::Failed => "failed",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Acked | Self::Failed | Self::Expired | Self::Cancelled)
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "queued" => Some(Self::Queued),
            "sent" => Some(Self::Sent),
            "acked" => Some(Self::Acked),
            "failed" => Some(Self::Failed),
            "expired" => Some(Self::Expired),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Command {
    pub id: String,
    pub data_source_id: i64,
    pub device_id: i64,
    pub protocol: String,
    pub value: String,
    pub status: String,
    pub policy: String,
    pub scheduled_at: Option<i64>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub sent_at: Option<i64>,
    pub acked_at: Option<i64>,
    pub resolved_at: Option<i64>,
    pub created_by: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CommandAuditEntry {
    pub id: i64,
    pub command_id: String,
    pub old_status: String,
    pub new_status: String,
    pub changed_by: String,
    pub reason: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WritePolicy {
    pub writable: bool,
    pub write_min: Option<f64>,
    pub write_max: Option<f64>,
    pub write_max_rate: Option<i64>,
    pub write_requires_reason: bool,
    pub write_deadband: Option<f64>,
}

impl Default for WritePolicy {
    fn default() -> Self {
        Self {
            writable: false,
            write_min: None,
            write_max: None,
            write_max_rate: None,
            write_requires_reason: false,
            write_deadband: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct WriteRequest {
    pub value: serde_json::Value,
    #[serde(default = "default_write_policy")]
    pub policy: String,
    pub scheduled_at: Option<i64>,
    pub reason: Option<String>,
}

fn default_write_policy() -> String {
    "immediate".to_string()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct CommandQuery {
    pub tag: Option<String>,
    pub status: Option<String>,
    pub device: Option<String>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Serde helper: deserialize missing JSON field as `None`, JSON `null` as `Some(None)`,
/// and a present value as `Some(Some(value))`. This enables three-state semantics for
/// `Option<Option<T>>` fields (absent / explicitly cleared / explicitly set).
mod option_option {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<T: serde::Serialize, S: Serializer>(
        val: &Option<Option<T>>, s: S,
    ) -> Result<S::Ok, S::Error> {
        match val {
            None => s.serialize_none(),
            Some(inner) => s.serialize_some(inner),
        }
    }

    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<Option<T>>, D::Error> {
        Ok(Some(Option::<T>::deserialize(d)?))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct WritePolicyUpdate {
    pub writable: Option<bool>,
    #[serde(default, with = "option_option")]
    pub write_min: Option<Option<f64>>,
    #[serde(default, with = "option_option")]
    pub write_max: Option<Option<f64>>,
    #[serde(default, with = "option_option")]
    pub write_max_rate: Option<Option<i64>>,
    pub write_requires_reason: Option<bool>,
    #[serde(default, with = "option_option")]
    pub write_deadband: Option<Option<f64>>,
}

#[derive(Debug, Clone)]
pub struct WriteResult {
    pub success: bool,
    pub device_response: Option<String>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DowntimeEvent {
    pub id: i64,
    pub device_id: String,
    pub state: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration_secs: Option<i64>,
    pub reason: Option<String>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateDowntimeEventRequest {
    pub device_id: String,
    pub state: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CloseDowntimeEventRequest {
    pub reason: Option<String>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OeeMetrics {
    pub availability: f64,
    pub performance: f64,
    pub quality: f64,
    pub oee: f64,
    pub total_running_secs: i64,
    pub total_planned_secs: i64,
    pub downtime_count: i64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ShiftConfig {
    pub id: i64,
    pub name: String,
    pub start_hour: u32,
    pub start_minute: u32,
    pub duration_hours: u32,
    pub days_mask: u32,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateShiftRequest {
    pub name: String,
    pub start_hour: u32,
    pub start_minute: u32,
    pub duration_hours: u32,
    pub days_mask: u32,
}

// ── Report types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Report {
    pub id: i64,
    pub name: String,
    pub schedule: String,
    pub enabled: bool,
    pub device_ids: String,
    pub email_recipients: String,
    pub last_generated: Option<i64>,
    pub created_at: i64,
    #[serde(default = "default_report_type")]
    pub report_type: Option<String>,
    pub analytics_config: Option<String>,
}

fn default_report_type() -> Option<String> {
    Some("pdf".to_string())
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateReportRequest {
    pub name: String,
    pub schedule: String,
    pub device_ids: Option<String>,
    pub email_recipients: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateReportRequest {
    pub name: Option<String>,
    pub schedule: Option<String>,
    pub device_ids: Option<String>,
    pub email_recipients: Option<String>,
    pub enabled: Option<bool>,
}

// ── Hub (Multi-Site) types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct HubSite {
    pub id: i64,
    pub name: String,
    pub url: String,
    #[serde(skip_serializing)]
    pub api_key: String,
    pub enabled: bool,
    pub last_seen: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateHubSiteRequest {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateHubSiteRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub enabled: Option<bool>,
}

// ── RBAC types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub permissions: String, // JSON array: ["screens:read", "screens:write", ...]
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateRoleRequest {
    pub name: String,
    pub permissions: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub permissions: Option<String>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct AssignRoleRequest {
    pub role_id: i64,
}

// ── Permission constants ──

pub const PERM_SCREENS_READ: &str = "screens:read";
pub const PERM_SCREENS_WRITE: &str = "screens:write";
pub const PERM_DEVICES_READ: &str = "devices:read";
pub const PERM_DEVICES_WRITE: &str = "devices:write";
pub const PERM_ALARMS_READ: &str = "alarms:read";
pub const PERM_ALARMS_ACK: &str = "alarms:ack";
pub const PERM_SETTINGS_READ: &str = "settings:read";
pub const PERM_SETTINGS_WRITE: &str = "settings:write";
pub const PERM_USERS_MANAGE: &str = "users:manage";
pub const PERM_FLOWS_MANAGE: &str = "flows:manage";
pub const PERM_NOTIFICATIONS_MANAGE: &str = "notifications:manage";
pub const PERM_HUB_MANAGE: &str = "hub:manage";
pub const PERM_REPORTS_MANAGE: &str = "reports:manage";
pub const PERM_RECIPES_MANAGE: &str = "recipes:manage";
pub const PERM_COMMANDS_WRITE: &str = "commands:write";
pub const PERM_COMMANDS_READ: &str = "commands:read";
pub const PERM_COMMANDS_CANCEL: &str = "commands:cancel";
pub const PERM_TAGS_WRITE_POLICY: &str = "tags:write_policy";

/// All available permission constants for seeding.
pub const ALL_PERMISSIONS: &[&str] = &[
    PERM_SCREENS_READ, PERM_SCREENS_WRITE,
    PERM_DEVICES_READ, PERM_DEVICES_WRITE,
    PERM_ALARMS_READ, PERM_ALARMS_ACK,
    PERM_SETTINGS_READ, PERM_SETTINGS_WRITE,
    PERM_USERS_MANAGE,
    PERM_FLOWS_MANAGE,
    PERM_NOTIFICATIONS_MANAGE,
    PERM_HUB_MANAGE,
    PERM_REPORTS_MANAGE,
    PERM_RECIPES_MANAGE,
    PERM_COMMANDS_WRITE, PERM_COMMANDS_READ,
    PERM_COMMANDS_CANCEL,
    PERM_TAGS_WRITE_POLICY,
];

/// Read-only permissions for the Viewer role.
pub const VIEWER_PERMISSIONS: &[&str] = &[
    PERM_SCREENS_READ, PERM_DEVICES_READ, PERM_ALARMS_READ,
    PERM_SETTINGS_READ,
];

/// Read + ack permissions for the Operator role.
pub const OPERATOR_PERMISSIONS: &[&str] = &[
    PERM_SCREENS_READ, PERM_DEVICES_READ, PERM_ALARMS_READ,
    PERM_ALARMS_ACK, PERM_SETTINGS_READ,
    PERM_COMMANDS_WRITE, PERM_COMMANDS_READ,
];

// ── Recipe types ──

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub category: Option<String>,
    pub device_id: String,
    pub values: String,
    pub version: i32,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateRecipeRequest {
    pub name: String,
    pub category: Option<String>,
    pub device_id: String,
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct LoadRecipeRequest {
    pub recipe_id: i64,
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

// ── Security Audit types (v0.22.0 IEC 62443) ──

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SecurityEvent {
    pub id: i64,
    pub event_type: String,
    pub severity: String,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub username: Option<String>,
    pub details: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSecurityEvent {
    pub event_type: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub username: Option<String>,
    pub details: Option<String>,
}

fn default_severity() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SecurityThresholds {
    pub failed_login_threshold: u32,
    pub failed_login_window_secs: u64,
    pub suspicious_401_threshold: u32,
    pub suspicious_401_window_secs: u64,
    pub rate_limit_threshold: u32,
    pub rate_limit_window_secs: u64,
}

impl Default for SecurityThresholds {
    fn default() -> Self {
        Self {
            failed_login_threshold: 5,
            failed_login_window_secs: 60,
            suspicious_401_threshold: 10,
            suspicious_401_window_secs: 60,
            rate_limit_threshold: 100,
            rate_limit_window_secs: 60,
        }
    }
}

// ── Annotation types (v0.23.0) ──

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Annotation {
    pub id: i64,
    pub tag_id: String,
    pub annotation_text: String,
    pub data_timestamp: i64,
    pub author_id: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateAnnotationRequest {
    pub tag_id: String,
    pub annotation_text: String,
    pub data_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateAnnotationRequest {
    pub annotation_text: Option<String>,
}

// ── Shift Handover types (v0.23.0) ──

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ShiftHandover {
    pub id: i64,
    pub author_id: i64,
    pub shift_date: String,
    pub shift_period: String,
    pub what_happened: String,
    pub whats_pending: String,
    pub what_to_watch: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateShiftHandoverRequest {
    pub shift_date: String,
    pub shift_period: String,
    pub what_happened: Option<String>,
    pub whats_pending: Option<String>,
    pub what_to_watch: Option<String>,
}

// ── User Notification types (v0.23.0) ──

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserNotification {
    pub id: i64,
    pub user_id: i64,
    pub from_user_id: i64,
    pub notification_type: String,
    pub message: String,
    pub reference_id: Option<i64>,
    pub read: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: i64,
    pub from_user_id: i64,
    pub notification_type: String,
    pub message: String,
    pub reference_id: Option<i64>,
}

/// Extract `@username` patterns from text, returning a deduplicated list of usernames.
pub fn parse_mentions(text: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'@' {
            let start = i + 1;
            let mut end = start;
            while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                let name = text[start..end].to_string();
                if seen.insert(name.clone()) {
                    result.push(name);
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
    result
}

// ── Analytics v0.28.0 models ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CustomMetric {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub formula: String,
    pub unit: Option<String>,
    pub tag_refs: String,
    pub validation_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateCustomMetricRequest {
    pub name: String,
    pub description: String,
    pub formula: String,
    pub unit: Option<String>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateCustomMetricRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub formula: Option<String>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct CustomMetricValue {
    pub id: i64,
    pub name: String,
    pub value: f64,
    pub unit: Option<String>,
    pub computed_at: i64,
    pub tag_values: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct SpcResult {
    pub chart_type: String,
    pub data_points: Vec<SpcPoint>,
    pub control_limits: ControlLimits,
    pub violations: Vec<usize>,
    pub in_control_percent: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct SpcPoint {
    pub index: usize,
    pub value: f64,
    pub timestamp: i64,
    pub subgroup_values: Option<Vec<f64>>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ControlLimits {
    pub ucl: f64,
    pub cl: f64,
    pub lcl: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct HistogramResult {
    pub bins: Vec<HistogramBin>,
    pub normal_curve: Vec<CurvePoint>,
    pub stats: DistributionStats,
    pub capability: Option<CapabilityIndices>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct HistogramBin {
    pub lower: f64,
    pub upper: f64,
    pub count: u64,
    pub frequency: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DistributionStats {
    pub mean: f64,
    pub stddev: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub count: u64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct CapabilityIndices {
    pub cp: f64,
    pub cpk: f64,
    pub pp: f64,
    pub ppk: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct RunChartResult {
    pub points: Vec<RunChartPoint>,
    pub median: f64,
    pub signals: Vec<Signal>,
    pub runs_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct RunChartPoint {
    pub timestamp: i64,
    pub value: f64,
    pub above_median: bool,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct Signal {
    pub rule: String,
    pub start_idx: usize,
    pub end_idx: usize,
    pub description: String,
}

// ── AI Copilot types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AiProposal {
    pub id: i64,
    pub proposal_type: String,
    pub status: String,
    pub title: String,
    pub input_summary: String,
    pub proposed_config: serde_json::Value,
    pub applied_config: Option<serde_json::Value>,
    pub target_entity_id: Option<String>,
    pub error_message: Option<String>,
    pub model_used: Option<String>,
    pub tokens_used: Option<i64>,
    pub created_at: i64,
    pub reviewed_at: Option<i64>,
    pub reviewed_by: Option<String>,
    pub applied_at: Option<i64>,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct AiSettingsUpdate {
    pub provider: String,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct AiSettingsResponse {
    pub provider: String,
    pub api_key_set: bool,
    pub api_key_preview: String,
    pub default_model: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct EquipmentTemplate {
    pub id: i64,
    pub industry: String,
    pub equipment_type: String,
    pub name: String,
    pub tag_template: serde_json::Value,
    pub alarm_template: serde_json::Value,
    pub dashboard_template: serde_json::Value,
    pub flow_template: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct RegisterMapRequest {
    pub file_content: String,
    pub file_type: String,
    pub device_name: String,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct DashboardLayoutRequest {
    pub description: String,
    pub screen_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct AlarmThresholdRequest {
    pub device_type: String,
    pub tags: Vec<AlarmThresholdTag>,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct AlarmThresholdTag {
    pub source_key: String,
    pub name: String,
    pub unit: Option<String>,
    pub min_val: Option<f64>,
    pub max_val: Option<f64>,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct TagRenameRequest {
    pub tags: Vec<TagRenameEntry>,
    pub convention: String,
    pub custom_pattern: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct TagRenameEntry {
    pub current: String,
    pub proposed: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct FlowRuleRequest {
    pub description: String,
    pub available_tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct TemplateInstantiateRequest {
    pub template_id: i64,
    pub device_id: Option<String>,
    pub prefix: Option<String>,
}
