# Modbus → HTTP Service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust service that polls Modbus TCP devices, caches metrics in memory, persists to SQLite, and exposes live/history data via HTTP API.

**Architecture:** Per-device async polling loops (tokio) write to an in-memory cache (Arc<RwLock<HashMap>>). A periodic persistence task batches cache snapshots into SQLite (rusqlite). Axum serves /api/live from cache and /api/history from SQLite.

**Tech Stack:** Tokio, tokio-modbus (TCP), axum 0.8, rusqlite (bundled), serde/serde_yaml, chrono, tracing

---

## File Structure

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies |
| `config.yaml` | Device/register configuration |
| `src/main.rs` | Entry point, wires all modules |
| `src/models.rs` | Config types, runtime types, API types |
| `src/config.rs` | YAML config loader + validation |
| `src/cache.rs` | Thread-safe in-memory latest-values cache |
| `src/modbus.rs` | Register parsing, Modbus TCP polling loop |
| `src/storage.rs` | SQLite schema, batch insert, history query |
| `src/api.rs` | Axum routes: /api/live, /api/health, /api/history |

---

## API Contract

### GET /api/live

```json
{
  "devices": [
    {
      "device_id": "plts_1",
      "status": "online",
      "timestamp": 1710000000,
      "metrics": { "voltage": 220.5, "current": 5.2, "power": 1146.0 }
    }
  ]
}
```

### GET /api/health

```json
{
  "status": "ok",
  "devices": { "plts_1": "online" }
}
```

### GET /api/history?device_id=plts_1&metric=voltage&from=1710000000&to=1710003600

```json
[
  { "timestamp": 1710000000, "value": 220.1 },
  { "timestamp": 1710000060, "value": 221.0 }
]
```

---

## Task 1: Project Setup

**Files:**
- Modify: `Cargo.toml`
- Create: `src/models.rs`, `src/config.rs`, `src/cache.rs`, `src/modbus.rs`, `src/storage.rs`, `src/api.rs`
- Create: `config.yaml`

- [ ] **Step 1: Write Cargo.toml with all dependencies**

```toml
[package]
name = "voltrus-lite-rust-service"
version = "0.1.0"
edition = "2024"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-modbus = { version = "0.16", default-features = false, features = ["tcp"] }
axum = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
rusqlite = { version = "0.34", features = ["bundled"] }
chrono = "0.4"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tower-http = { version = "0.6", features = ["cors"] }

[dev-dependencies]
http = "1"
tower = { version = "0.5", features = ["util"] }
tempfile = "3"
```

- [ ] **Step 2: Create module stubs**

Each file gets an empty placeholder — they'll be filled in subsequent tasks:

`src/models.rs`:
```rust
// Task 2
```

`src/config.rs`:
```rust
// Task 3
```

`src/cache.rs`:
```rust
// Task 4
```

`src/modbus.rs`:
```rust
// Task 5
```

`src/storage.rs`:
```rust
// Task 6
```

`src/api.rs`:
```rust
// Task 7
```

`src/main.rs`:
```rust
mod models;
mod config;
mod cache;
mod modbus;
mod storage;
mod api;

fn main() {
    println!("Hello, world!");
}
```

- [ ] **Step 3: Create config.yaml**

```yaml
server_port: 3000
db_path: "data.db"
persist_interval_secs: 5

devices:
  - id: "plts_1"
    ip: "192.168.1.10"
    port: 502
    unit_id: 1
    interval_ms: 1000
    endianness: "big"
    registers:
      - name: "voltage"
        address: 40001
        type: "float32"
        scale: 0.1
      - name: "current"
        address: 40003
        type: "float32"
        scale: 0.01
      - name: "power"
        address: 40005
        type: "float32"
      - name: "frequency"
        address: 40007
        type: "float32"
      - name: "power_factor"
        address: 40009
        type: "float32"
      - name: "energy_total"
        address: 40011
        type: "float32"

  - id: "plts_2"
    ip: "192.168.1.11"
    port: 502
    unit_id: 1
    interval_ms: 1000
    endianness: "big"
    registers:
      - name: "voltage"
        address: 40001
        type: "float32"
        scale: 0.1
      - name: "current"
        address: 40003
        type: "float32"
        scale: 0.01
      - name: "power"
        address: 40005
        type: "float32"
      - name: "frequency"
        address: 40007
        type: "float32"
      - name: "power_factor"
        address: 40009
        type: "float32"
      - name: "energy_total"
        address: 40011
        type: "float32"
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles (warnings about unused modules are fine)

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock config.yaml src/ .gitignore prd.md
git commit -m "feat: project skeleton with dependencies and config"
```

---

## Task 2: Data Models

**Files:**
- Modify: `src/models.rs`

- [ ] **Step 1: Write models.rs with all types and inline tests**

```rust
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
    #[serde(default = "default_reg_type")]
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
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib models`
Expected: 4 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/models.rs
git commit -m "feat: data models for config, runtime, and API types"
```

---

## Task 3: Config Loader

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: Write config.rs with tests**

```rust
use anyhow::{Context, Result};
use std::fs;
use crate::models::AppConfig;

pub fn load_config(path: &str) -> Result<AppConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {path}"))?;

    let config: AppConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {path}"))?;

    validate(&config)?;

    Ok(config)
}

fn validate(config: &AppConfig) -> Result<()> {
    anyhow::ensure!(!config.devices.is_empty(), "No devices configured");

    let mut ids = std::collections::HashSet::new();
    for device in &config.devices {
        anyhow::ensure!(!device.id.is_empty(), "Device ID cannot be empty");
        anyhow::ensure!(
            ids.insert(device.id.clone()),
            "Duplicate device ID: {}",
            device.id
        );
        anyhow::ensure!(
            !device.registers.is_empty(),
            "Device {} has no registers",
            device.id
        );

        let mut reg_names = std::collections::HashSet::new();
        for reg in &device.registers {
            anyhow::ensure!(
                reg_names.insert(reg.name.clone()),
                "Duplicate register name '{}' in device {}",
                reg.name,
                device.id
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
server_port: 8080
db_path: "test.db"
persist_interval_secs: 10
devices:
  - id: "dev1"
    ip: "192.168.1.10"
    registers:
      - name: "voltage"
        address: 40001
        type: "float32"
        scale: 0.1
"#;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let config = load_config(f.path().to_str().unwrap()).unwrap();

        assert_eq!(config.server_port, 8080);
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].id, "dev1");
        assert_eq!(config.devices[0].ip, "192.168.1.10");
        assert_eq!(config.devices[0].port, 502); // default
        assert_eq!(config.devices[0].registers.len(), 1);
        assert_eq!(config.devices[0].registers[0].scale, 0.1);
    }

    #[test]
    fn test_defaults_applied() {
        let yaml = r#"
devices:
  - id: "dev1"
    ip: "10.0.0.1"
    registers:
      - name: "temp"
        address: 100
"#;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let config = load_config(f.path().to_str().unwrap()).unwrap();

        assert_eq!(config.server_port, 3000);
        assert_eq!(config.db_path, "data.db");
        assert_eq!(config.persist_interval_secs, 5);
        assert_eq!(config.devices[0].port, 502);
        assert_eq!(config.devices[0].unit_id, 1);
        assert_eq!(config.devices[0].interval_ms, 1000);
        assert_eq!(config.devices[0].registers[0].scale, 1.0); // default
    }

    #[test]
    fn test_no_devices_fails() {
        let yaml = "devices: []\n";
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let result = load_config(f.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No devices"));
    }

    #[test]
    fn test_duplicate_device_id_fails() {
        let yaml = r#"
devices:
  - id: "dev1"
    ip: "10.0.0.1"
    registers:
      - name: "temp"
        address: 100
  - id: "dev1"
    ip: "10.0.0.2"
    registers:
      - name: "temp"
        address: 100
"#;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let result = load_config(f.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate device ID"));
    }

    #[test]
    fn test_duplicate_register_name_fails() {
        let yaml = r#"
devices:
  - id: "dev1"
    ip: "10.0.0.1"
    registers:
      - name: "temp"
        address: 100
      - name: "temp"
        address: 102
"#;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let result = load_config(f.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate register name"));
    }

    #[test]
    fn test_file_not_found() {
        let result = load_config("/nonexistent/config.yaml");
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib config`
Expected: 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/config.rs
git commit -m "feat: YAML config loader with validation"
```

---

## Task 4: In-Memory Cache

**Files:**
- Modify: `src/cache.rs`

- [ ] **Step 1: Write cache.rs with tests**

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::{DeviceSnapshot, DeviceStatus};

#[derive(Debug, Clone)]
pub struct DeviceCache {
    devices: Arc<RwLock<HashMap<String, DeviceSnapshot>>>,
}

impl DeviceCache {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update(&self, device_id: &str, metrics: HashMap<String, f64>) {
        let mut devices = self.devices.write().await;
        let timestamp = chrono::Utc::now().timestamp();

        devices
            .entry(device_id.to_string())
            .and_modify(|e| {
                e.status = DeviceStatus::Online;
                e.timestamp = timestamp;
                e.metrics = metrics.clone();
            })
            .or_insert_with(|| DeviceSnapshot {
                device_id: device_id.to_string(),
                status: DeviceStatus::Online,
                timestamp,
                metrics,
            });
    }

    pub async fn set_status(&self, device_id: &str, status: DeviceStatus) {
        let mut devices = self.devices.write().await;
        if let Some(entry) = devices.get_mut(device_id) {
            entry.status = status;
        }
    }

    pub async fn get_all(&self) -> Vec<DeviceSnapshot> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    pub async fn get_statuses(&self) -> HashMap<String, String> {
        let devices = self.devices.read().await;
        devices
            .iter()
            .map(|(id, snap)| {
                let status_str = match snap.status {
                    DeviceStatus::Online => "online",
                    DeviceStatus::Offline => "offline",
                };
                (id.clone(), status_str.to_string())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_and_get_all() {
        let cache = DeviceCache::new();

        let mut metrics = HashMap::new();
        metrics.insert("voltage".to_string(), 220.5);
        metrics.insert("current".to_string(), 5.2);

        cache.update("plts_1", metrics).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].device_id, "plts_1");
        assert_eq!(all[0].status, DeviceStatus::Online);
        assert_eq!(all[0].metrics.get("voltage"), Some(&220.5));
        assert_eq!(all[0].metrics.get("current"), Some(&5.2));
    }

    #[tokio::test]
    async fn test_multiple_devices() {
        let cache = DeviceCache::new();

        let m1 = HashMap::from([("voltage".into(), 220.0)]);
        let m2 = HashMap::from([("voltage".into(), 380.0)]);
        cache.update("dev1", m1).await;
        cache.update("dev2", m2).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_update_overwrites() {
        let cache = DeviceCache::new();

        let m1 = HashMap::from([("voltage".into(), 220.0)]);
        let m2 = HashMap::from([("voltage".into(), 225.0)]);
        cache.update("dev1", m1).await;
        cache.update("dev1", m2).await;

        let all = cache.get_all().await;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].metrics.get("voltage"), Some(&225.0));
    }

    #[tokio::test]
    async fn test_set_status_offline() {
        let cache = DeviceCache::new();

        let metrics = HashMap::from([("voltage".into(), 220.0)]);
        cache.update("dev1", metrics).await;
        cache.set_status("dev1", DeviceStatus::Offline).await;

        let all = cache.get_all().await;
        assert_eq!(all[0].status, DeviceStatus::Offline);

        let statuses = cache.get_statuses().await;
        assert_eq!(statuses.get("dev1"), Some(&"offline".to_string()));
    }

    #[tokio::test]
    async fn test_get_statuses() {
        let cache = DeviceCache::new();

        let m1 = HashMap::from([("voltage".into(), 220.0)]);
        let m2 = HashMap::from([("voltage".into(), 380.0)]);
        cache.update("dev1", m1).await;
        cache.update("dev2", m2).await;
        cache.set_status("dev2", DeviceStatus::Offline).await;

        let statuses = cache.get_statuses().await;
        assert_eq!(statuses.get("dev1"), Some(&"online".to_string()));
        assert_eq!(statuses.get("dev2"), Some(&"offline".to_string()));
    }

    #[tokio::test]
    async fn test_set_status_nonexistent() {
        let cache = DeviceCache::new();
        cache.set_status("ghost", DeviceStatus::Offline).await;

        let all = cache.get_all().await;
        assert!(all.is_empty());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib cache`
Expected: 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/cache.rs
git commit -m "feat: thread-safe in-memory device cache"
```

---

## Task 5: Modbus Register Parsing & Poller

**Files:**
- Modify: `src/modbus.rs`

- [ ] **Step 1: Write modbus.rs with register parsing, poller, and tests**

```rust
use std::collections::HashMap;

use anyhow::{Context, Result};
use tokio_modbus::client::tcp;
use tokio_modbus::prelude::*;

use crate::cache::DeviceCache;
use crate::models::*;

// ── Register value conversion ──

pub fn parse_register(words: &[u16], reg_type: &RegisterType, endianness: &Endianness) -> f64 {
    match reg_type {
        RegisterType::Uint16 => words[0] as f64,
        RegisterType::Int16 => (words[0] as i16) as f64,
        RegisterType::Int32 => {
            let raw = combine_u16(words[0], words[1], endianness);
            raw as i32 as f64
        }
        RegisterType::Float32 => {
            let raw = combine_u16(words[0], words[1], endianness);
            f32::from_bits(raw) as f64
        }
    }
}

fn combine_u16(high: u16, low: u16, endianness: &Endianness) -> u32 {
    match endianness {
        Endianness::Big => ((high as u32) << 16) | (low as u32),
        Endianness::Little => ((low as u32) << 16) | (high as u32),
        Endianness::Swapped => {
            let h = high.swap_bytes();
            let l = low.swap_bytes();
            ((l as u32) << 16) | (h as u32)
        }
    }
}

// ── Polling engine ──

pub async fn poll_device(config: DeviceConfig, cache: DeviceCache) {
    let device_id = config.id.clone();
    let mut backoff = Backoff::new();

    loop {
        match try_poll(&config, &cache).await {
            Ok(()) => backoff.reset(),
            Err(e) => {
                tracing::warn!("Device {device_id} error: {e}");
                cache.set_status(&device_id, DeviceStatus::Offline).await;
                let delay = backoff.next();
                tracing::info!("Device {device_id} retrying in {delay}ms");
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
        }
    }
}

async fn try_poll(config: &DeviceConfig, cache: &DeviceCache) -> Result<()> {
    let addr = format!("{}:{}", config.ip, config.port);
    let socket_addr: std::net::SocketAddr = addr.parse().context("Invalid address")?;

    let mut ctx = tcp::connect_slave(socket_addr, Slave(config.unit_id))
        .await
        .context("Modbus connection failed")?;

    tracing::info!("Connected to device {} at {}", config.id, addr);

    loop {
        let mut metrics = HashMap::new();

        for reg in &config.registers {
            let addr = reg.protocol_address();
            let count = reg.register_count();

            let response = ctx
                .read_holding_registers(addr, count)
                .await
                .context("Modbus read failed")?;

            if response.len() < count as usize {
                anyhow::bail!(
                    "Short read for register {}: expected {count}, got {}",
                    reg.name,
                    response.len()
                );
            }

            let value = parse_register(&response, &reg.reg_type, &config.endianness) * reg.scale;
            metrics.insert(reg.name.clone(), value);
        }

        cache.update(&config.id, metrics).await;
        tracing::debug!("Device {} polled successfully", config.id);

        tokio::time::sleep(std::time::Duration::from_millis(config.interval_ms)).await;
    }
}

struct Backoff {
    attempt: u32,
    max_delay_ms: u64,
}

impl Backoff {
    fn new() -> Self {
        Self {
            attempt: 0,
            max_delay_ms: 30_000,
        }
    }

    fn reset(&mut self) {
        self.attempt = 0;
    }

    fn next(&mut self) -> u64 {
        let delay = 1000 * 2u64.pow(self.attempt);
        self.attempt += 1;
        delay.min(self.max_delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_register tests ──

    #[test]
    fn test_parse_uint16() {
        let words = [1000u16];
        let val = parse_register(&words, &RegisterType::Uint16, &Endianness::Big);
        assert_eq!(val, 1000.0);
    }

    #[test]
    fn test_parse_int16_positive() {
        let words = [1000u16];
        let val = parse_register(&words, &RegisterType::Int16, &Endianness::Big);
        assert_eq!(val, 1000.0);
    }

    #[test]
    fn test_parse_int16_negative() {
        // 0xFFFF = -1 as i16
        let words = [0xFFFFu16];
        let val = parse_register(&words, &RegisterType::Int16, &Endianness::Big);
        assert_eq!(val, -1.0);
    }

    #[test]
    fn test_parse_int32_big_endian() {
        // 0x0001 0x0002 = 65538
        let words = [0x0001, 0x0002];
        let val = parse_register(&words, &RegisterType::Int32, &Endianness::Big);
        assert_eq!(val, 65538.0);
    }

    #[test]
    fn test_parse_float32_big_endian() {
        // IEEE 754 for 220.5: 0x435C 0x8000
        let words = [0x435C, 0x8000];
        let val = parse_register(&words, &RegisterType::Float32, &Endianness::Big);
        assert!((val - 220.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_float32_little_endian() {
        // Same 220.5 but words swapped for LE
        let words = [0x8000, 0x435C];
        let val = parse_register(&words, &RegisterType::Float32, &Endianness::Little);
        assert!((val - 220.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_with_scale() {
        let words = [2205u16];
        let val =
            parse_register(&words, &RegisterType::Uint16, &Endianness::Big) * 0.1;
        assert!((val - 220.5).abs() < 0.01);
    }

    // ── combine_u16 tests ──

    #[test]
    fn test_combine_big_endian() {
        let result = combine_u16(0x1234, 0x5678, &Endianness::Big);
        assert_eq!(result, 0x12345678);
    }

    #[test]
    fn test_combine_little_endian() {
        let result = combine_u16(0x5678, 0x1234, &Endianness::Little);
        assert_eq!(result, 0x12345678);
    }

    // ── Backoff tests ──

    #[test]
    fn test_backoff_sequence() {
        let mut b = Backoff::new();
        assert_eq!(b.next(), 1000);
        assert_eq!(b.next(), 2000);
        assert_eq!(b.next(), 4000);
        assert_eq!(b.next(), 8000);
    }

    #[test]
    fn test_backoff_resets() {
        let mut b = Backoff::new();
        b.next();
        b.next();
        b.reset();
        assert_eq!(b.next(), 1000);
    }

    #[test]
    fn test_backoff_max() {
        let mut b = Backoff::new();
        for _ in 0..20 {
            b.next();
        }
        assert_eq!(b.next(), 30_000); // capped at max
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib modbus`
Expected: 11 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/modbus.rs
git commit -m "feat: Modbus register parsing and async polling engine"
```

---

## Task 6: SQLite Storage

**Files:**
- Modify: `src/storage.rs`

- [ ] **Step 1: Write storage.rs with tests**

```rust
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::models::DataPoint;

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database: {path}"))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS telemetry (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                metric TEXT NOT NULL,
                value REAL NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_device_time_metric
                ON telemetry(device_id, metric, timestamp);",
        )
        .context("Failed to create schema")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory DB")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS telemetry (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                metric TEXT NOT NULL,
                value REAL NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_device_time_metric
                ON telemetry(device_id, metric, timestamp);",
        )
        .context("Failed to create schema")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn insert_batch(&self, records: &[(String, i64, String, f64)]) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        let conn = self.conn.clone();
        let records = records.to_vec();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            for (device_id, timestamp, metric, value) in &records {
                tx.execute(
                    "INSERT INTO telemetry (device_id, timestamp, metric, value) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![device_id, timestamp, metric, value],
                )
                .context("Insert failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn query_history(
        &self,
        device_id: &str,
        metric: &str,
        from: i64,
        to: i64,
    ) -> Result<Vec<DataPoint>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let metric = metric.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT timestamp, value FROM telemetry
                     WHERE device_id = ?1 AND metric = ?2
                     AND timestamp >= ?3 AND timestamp <= ?4
                     ORDER BY timestamp",
                )
                .context("Prepare failed")?;

            let rows = stmt
                .query_map(
                    rusqlite::params![device_id, metric, from, to],
                    |row| {
                        Ok(DataPoint {
                            timestamp: row.get(0)?,
                            value: row.get(1)?,
                        })
                    },
                )
                .context("Query failed")?;

            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_query() {
        let storage = Storage::new_in_memory().unwrap();

        let records = vec![
            ("dev1".into(), 1000i64, "voltage".into(), 220.5f64),
            ("dev1".into(), 1060i64, "voltage".into(), 221.0f64),
            ("dev1".into(), 1120i64, "voltage".into(), 219.8f64),
        ];
        storage.insert_batch(&records).await.unwrap();

        let points = storage
            .query_history("dev1", "voltage", 0, 2000)
            .await
            .unwrap();

        assert_eq!(points.len(), 3);
        assert_eq!(points[0], DataPoint { timestamp: 1000, value: 220.5 });
        assert_eq!(points[1], DataPoint { timestamp: 1060, value: 221.0 });
        assert_eq!(points[2], DataPoint { timestamp: 1120, value: 219.8 });
    }

    #[tokio::test]
    async fn test_query_filters_by_device_and_metric() {
        let storage = Storage::new_in_memory().unwrap();

        let records = vec![
            ("dev1".into(), 1000, "voltage".into(), 220.0),
            ("dev1".into(), 1000, "current".into(), 5.0),
            ("dev2".into(), 1000, "voltage".into(), 380.0),
        ];
        storage.insert_batch(&records).await.unwrap();

        let points = storage
            .query_history("dev1", "voltage", 0, 2000)
            .await
            .unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, 220.0);
    }

    #[tokio::test]
    async fn test_query_time_range() {
        let storage = Storage::new_in_memory().unwrap();

        let records = vec![
            ("dev1".into(), 1000, "voltage".into(), 220.0),
            ("dev1".into(), 2000, "voltage".into(), 221.0),
            ("dev1".into(), 3000, "voltage".into(), 222.0),
        ];
        storage.insert_batch(&records).await.unwrap();

        let points = storage
            .query_history("dev1", "voltage", 1500, 2500)
            .await
            .unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, 221.0);
    }

    #[tokio::test]
    async fn test_empty_batch() {
        let storage = Storage::new_in_memory().unwrap();
        let records: Vec<(String, i64, String, f64)> = vec![];
        storage.insert_batch(&records).await.unwrap();
    }

    #[tokio::test]
    async fn test_no_results() {
        let storage = Storage::new_in_memory().unwrap();
        let points = storage
            .query_history("ghost", "voltage", 0, 9999)
            .await
            .unwrap();
        assert!(points.is_empty());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib storage`
Expected: 5 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/storage.rs
git commit -m "feat: SQLite storage with batch insert and history queries"
```

---

## Task 7: HTTP API

**Files:**
- Modify: `src/api.rs`

- [ ] **Step 1: Write api.rs with routes, error handling, and tests**

```rust
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use crate::cache::DeviceCache;
use crate::models::*;
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub cache: DeviceCache,
    pub storage: Storage,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/live", get(get_live))
        .route("/api/health", get(get_health))
        .route("/api/history", get(get_history))
        .with_state(state)
        .layer(cors)
}

async fn get_live(State(state): State<AppState>) -> Json<LiveResponse> {
    let devices = state.cache.get_all().await;
    Json(LiveResponse { devices })
}

async fn get_health(State(state): State<AppState>) -> Json<HealthResponse> {
    let statuses = state.cache.get_statuses().await;
    Json(HealthResponse {
        status: "ok".to_string(),
        devices: statuses,
    })
}

async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Vec<DataPoint>>, AppError> {
    let now = chrono::Utc::now().timestamp();
    let from = params.from.unwrap_or(now - 3600);
    let to = params.to.unwrap_or(now);

    let data = state
        .storage
        .query_history(&params.device_id, &params.metric, from, to)
        .await?;

    Ok(Json(data))
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": self.0.to_string()})),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        AppError(err.into())
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_app() -> Router {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState { cache, storage };
        create_router(state)
    }

    async fn seed_cache(cache: &DeviceCache) {
        let metrics = HashMap::from([
            ("voltage".to_string(), 220.5),
            ("current".to_string(), 5.2),
        ]);
        cache.update("plts_1", metrics).await;
    }

    #[tokio::test]
    async fn test_live_empty() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["devices"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_live_with_data() {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState {
            cache: cache.clone(),
            storage,
        };
        let app = create_router(state);

        seed_cache(&cache).await;

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let devices = json["devices"].as_array().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0]["device_id"], "plts_1");
        assert_eq!(devices[0]["status"], "online");
        assert_eq!(devices[0]["metrics"]["voltage"], 220.5);
    }

    #[tokio::test]
    async fn test_health() {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState {
            cache: cache.clone(),
            storage,
        };
        let app = create_router(state);

        seed_cache(&cache).await;

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["devices"]["plts_1"], "online");
    }

    #[tokio::test]
    async fn test_history_empty() {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();

        // Seed some data
        storage
            .insert_batch(&[
                ("plts_1".into(), 1000, "voltage".into(), 220.0),
                ("plts_1".into(), 1060, "voltage".into(), 221.0),
            ])
            .await
            .unwrap();

        let state = AppState { cache, storage };
        let app = create_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/history?device_id=plts_1&metric=voltage&from=0&to=2000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let points = json.as_array().unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0]["value"], 220.0);
        assert_eq!(points[1]["value"], 221.0);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib api`
Expected: 4 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/api.rs
git commit -m "feat: axum HTTP API with live, health, and history endpoints"
```

---

## Task 8: Main Wiring

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write main.rs wiring all modules together**

```rust
mod api;
mod cache;
mod config;
mod modbus;
mod models;
mod storage;

use std::sync::Arc;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

use api::AppState;
use cache::DeviceCache;
use storage::Storage;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.yaml".to_string());
    let app_config = config::load_config(&config_path)?;

    tracing::info!(
        "Loaded config: {} devices, port {}",
        app_config.devices.len(),
        app_config.server_port
    );

    let cache = DeviceCache::new();
    let storage = Storage::new(&app_config.db_path)?;

    // Spawn one poller task per device
    for device_config in &app_config.devices {
        let device = device_config.clone();
        let cache = cache.clone();
        tokio::spawn(async move {
            modbus::poll_device(device, cache).await;
        });
        tracing::info!("Started poller for device {}", device_config.id);
    }

    // Spawn persistence task — batch-write cache snapshots to SQLite
    {
        let cache = cache.clone();
        let storage = storage.clone();
        let interval = app_config.persist_interval_secs;
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_secs(interval));
            loop {
                ticker.tick().await;
                let snapshots = cache.get_all().await;
                if snapshots.is_empty() {
                    continue;
                }
                let now = chrono::Utc::now().timestamp();
                let mut records = Vec::new();
                for snap in &snapshots {
                    for (metric, value) in &snap.metrics {
                        records.push((
                            snap.device_id.clone(),
                            now,
                            metric.clone(),
                            *value,
                        ));
                    }
                }
                if let Err(e) = storage.insert_batch(&records).await {
                    tracing::error!("Persistence error: {e}");
                }
            }
        });
    }

    let state = AppState { cache, storage };
    let app = api::create_router(state);

    let addr = format!("0.0.0.0:{}", app_config.server_port);
    tracing::info!("Server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

- [ ] **Step 2: Build the project**

Run: `cargo build`
Expected: compiles with no errors

- [ ] **Step 3: Run all tests**

Run: `cargo test`
Expected: all tests pass across all modules

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: main entry point wiring pollers, persistence, and API server"
```

---

## Self-Review

### 1. Spec Coverage
| PRD Requirement | Task |
|---|---|
| Config-driven register mapping | Task 2 (models) + Task 3 (config loader) |
| Multi-device polling | Task 5 (modbus) + Task 8 (main spawns per-device) |
| Register types: int16, uint16, int32, float32 | Task 5 (parse_register) |
| Endianness: big, little, swapped | Task 5 (combine_u16) |
| Exponential backoff on failure | Task 5 (Backoff struct) |
| Mark device offline | Task 4 (set_status) + Task 5 (poller) |
| In-memory cache | Task 4 (DeviceCache) |
| GET /api/live | Task 7 |
| GET /api/health | Task 7 |
| GET /api/history with query params | Task 7 |
| SQLite persistence with batch insert | Task 6 |
| Async (Tokio) | Task 8 |
| CORS support | Task 7 |
| Scale factor on registers | Task 5 (parse_register * scale) |

### 2. Placeholder Scan
No TBD, TODO, or placeholder patterns found.

### 3. Type Consistency
All types flow consistently: `DeviceCache` methods return `Vec<DeviceSnapshot>`, `Storage` methods return `Vec<DataPoint>`, API handlers wrap in `LiveResponse` / `HealthResponse` / `Json<Vec<DataPoint>>`.
