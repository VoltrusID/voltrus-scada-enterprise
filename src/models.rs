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
