use serde::{Deserialize, Serialize};
use super::{DeviceConfig, Endianness, RegisterConfig, RegisterType,
    default_modbus_port, default_unit_id, default_interval, default_scale,
    default_register_area, default_interval_ms, default_true};
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
            readback: None,
            network: None,
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

// ── OPC-UA config types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OpcUaSecurityPolicy {
    None,
    #[serde(rename = "basic256sha256")]
    Basic256Sha256,
}

impl Default for OpcUaSecurityPolicy {
    fn default() -> Self { Self::None }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpcUaNode {
    pub id: i64,
    pub device_id: i64,
    pub node_id: String,
    pub name: String,
    pub display_name: String,
    pub data_type: String,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
pub struct UpdateS7DeviceRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub rack: Option<u8>,
    pub slot: Option<u8>,
    pub interval_ms: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
pub struct UpdateEnipDeviceRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub slot: Option<u8>,
    pub interval_ms: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
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
