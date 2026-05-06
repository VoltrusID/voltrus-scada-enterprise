# License Activation System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement an offline-capable license activation system for Voltrus that binds 1 license to 1 server via hardware fingerprinting and Ed25519-signed activation files, with an air-gap bridge flow through a Cloudflare Worker at `activate.voltrus.id`.

**Architecture:** A new `src/license/` module handles hardware fingerprinting, request/activation file generation, signature verification, and fuzzy fingerprint matching. A license middleware blocks API routes (but not static files or the license status endpoint) when unlicensed. The frontend checks license status on load and shows an activation guide. A Cloudflare Worker validates Polar.sh keys and signs activation tokens, storing bindings in D1.

**Tech Stack:** Rust, SQLite (rusqlite), Ed25519 (ed25519-dalek), Cloudflare Worker (D1 + TypeScript), Polar.sh API

---

## File Structure

```
src/
  license/
    mod.rs          # LicenseStatus enum, verify_license()
    fingerprint.rs  # Hardware fingerprint collection and fuzzy matching
    request.rs      # Generate license-request.bin
    activation.rs   # Verify Ed25519 signatures on license-activation.bin
  api/
    license.rs      # GET /api/license/status endpoint
  models.rs         # Add LicenseActivationRow struct
  storage.rs        # Add license table schema and storage methods
  main.rs           # CLI args, startup license check
  api/mod.rs        # Add license route and middleware
  Cargo.toml        # Add ed25519-dalek dependency

(separate repo: voltrus-lite-cloudflare-worker)
  package.json
  wrangler.toml
  schema.sql
  src/index.ts      # Cloudflare Worker with D1
```

---

## Task 1: Database Schema and Storage Methods

**Files:**
- Modify: `src/models.rs` (end of file)
- Modify: `src/storage.rs:2248-2253` (init_schema)
- Modify: `src/storage.rs:2556` (before tests)

### Step 1.1: Add LicenseActivationRow model

Add at the end of `src/models.rs`:

```rust
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
}
```

### Step 1.2: Add license table to schema

In `src/storage.rs` `init_schema`, after the audit_log index and before `Ok(())`:

```rust
    // Migration: add license_activation table
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS license_activation (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            license_key TEXT NOT NULL UNIQUE,
            fingerprint_hash TEXT NOT NULL,
            fingerprint_hostname TEXT NOT NULL,
            fingerprint_cpu TEXT NOT NULL,
            fingerprint_disk_serial TEXT NOT NULL,
            fingerprint_mac TEXT NOT NULL,
            fingerprint_machine_id TEXT NOT NULL,
            activation_blob BLOB NOT NULL,
            activated_at TEXT NOT NULL,
            expires_at TEXT
        );",
    );
```

### Step 1.3: Add storage methods

Add before `#[cfg(test)]` in `src/storage.rs`:

```rust
// ── License Activation ──

impl Storage {
    pub async fn get_license_activation(&self) -> Result<Option<crate::models::LicenseActivationRow>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT license_key, fingerprint_hash, activation_blob, fingerprint_hostname, fingerprint_cpu, fingerprint_disk_serial, fingerprint_mac, fingerprint_machine_id, activated_at, expires_at FROM license_activation LIMIT 1")
                .context("Prepare failed")?;
            let result = stmt.query_row([], |row| {
                Ok(crate::models::LicenseActivationRow {
                    license_key: row.get(0)?,
                    fingerprint_hash: row.get(1)?,
                    activation_blob: row.get(2)?,
                    fingerprint_hostname: row.get(3)?,
                    fingerprint_cpu: row.get(4)?,
                    fingerprint_disk_serial: row.get(5)?,
                    fingerprint_mac: row.get(6)?,
                    fingerprint_machine_id: row.get(7)?,
                    activated_at: row.get(8)?,
                    expires_at: row.get(9)?,
                })
            }).ok();
            Ok(result)
        }).await.context("Task join failed")?
    }

    pub async fn store_license_activation(&self, row: &crate::models::LicenseActivationRow) -> Result<()> {
        let conn = self.conn.clone();
        let row = row.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO license_activation (license_key, fingerprint_hash, fingerprint_hostname, fingerprint_cpu, fingerprint_disk_serial, fingerprint_mac, fingerprint_machine_id, activation_blob, activated_at, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    row.license_key,
                    row.fingerprint_hash,
                    row.fingerprint_hostname,
                    row.fingerprint_cpu,
                    row.fingerprint_disk_serial,
                    row.fingerprint_mac,
                    row.fingerprint_machine_id,
                    row.activation_blob,
                    row.activated_at,
                    row.expires_at,
                ],
            ).context("Store license activation failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn clear_license_activation(&self) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM license_activation", [])
                .context("Clear license activation failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }
}
```

### Step 1.4: Verify compilation

Run: `cargo check`
Expected: No errors.

### Step 1.5: Commit

```bash
git add src/models.rs src/storage.rs
git commit -m "feat(license): add license_activation table, model, and storage methods"
```

---

## Task 2: Hardware Fingerprint Module

**File:** Create `src/license/fingerprint.rs`

### Step 2.1: Create fingerprint module

```rust
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Fingerprint {
    pub hostname: String,
    pub cpu: String,
    pub disk_serial: String,
    pub mac: String,
    pub machine_id: String,
}

impl Fingerprint {
    pub const MIN_MATCH: usize = 3;
    pub const TOTAL_COMPONENTS: usize = 5;

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.hostname.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.cpu.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.disk_serial.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.mac.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.machine_id.as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }

    pub fn match_score(&self, other: &Fingerprint) -> usize {
        let mut score = 0;
        if self.hostname == other.hostname { score += 1; }
        if self.cpu == other.cpu { score += 1; }
        if self.disk_serial == other.disk_serial { score += 1; }
        if self.mac == other.mac { score += 1; }
        if self.machine_id == other.machine_id { score += 1; }
        score
    }

    pub fn to_components_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("hostname".to_string(), self.hostname.clone());
        map.insert("cpu".to_string(), self.cpu.clone());
        map.insert("disk_serial".to_string(), self.disk_serial.clone());
        map.insert("mac".to_string(), self.mac.clone());
        map.insert("machine_id".to_string(), self.machine_id.clone());
        map
    }
}

pub fn collect_fingerprint() -> Fingerprint {
    Fingerprint {
        hostname: get_hostname(),
        cpu: get_cpu_info(),
        disk_serial: get_disk_serial(),
        mac: get_mac_address(),
        machine_id: get_machine_id(),
    }
}

fn get_hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .ok_or(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

fn get_cpu_info() -> String {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();
    sys.cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_disk_serial() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("diskutil")
            .args(["info", "-all"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Device Identifier") {
                    if let Some(id) = line.split(':').nth(1) {
                        return id.trim().to_string();
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/block") {
            for entry in entries.flatten() {
                let serial_path = entry.path().join("serial");
                if let Ok(serial) = std::fs::read_to_string(&serial_path) {
                    let serial = serial.trim();
                    if !serial.is_empty() {
                        return serial.to_string();
                    }
                }
            }
        }
    }

    "unknown".to_string()
}

fn get_mac_address() -> String {
    use sysinfo::Networks;
    let networks = Networks::new_with_refreshed_list();
    for (name, network) in &networks {
        if name == "lo" || name.starts_with("loopback") {
            continue;
        }
        let mac = network.mac_address();
        let mac_str = mac.to_string();
        if mac_str != "00:00:00:00:00:00" && !mac_str.is_empty() {
            return mac_str;
        }
    }
    "unknown".to_string()
}

fn get_machine_id() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            return id.trim().to_string();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("IOPlatformUUID") {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start + 1..].find('"') {
                            return line[start + 1..start + 1 + end].to_string();
                        }
                    }
                }
            }
        }
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_hash_consistency() {
        let fp = Fingerprint {
            hostname: "test".to_string(),
            cpu: "intel".to_string(),
            disk_serial: "abc".to_string(),
            mac: "aa:bb:cc:dd:ee:ff".to_string(),
            machine_id: "123".to_string(),
        };
        let hash1 = fp.hash();
        let hash2 = fp.hash();
        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("sha256:"));
    }

    #[test]
    fn test_fingerprint_match_score() {
        let fp1 = Fingerprint {
            hostname: "a".to_string(),
            cpu: "b".to_string(),
            disk_serial: "c".to_string(),
            mac: "d".to_string(),
            machine_id: "e".to_string(),
        };
        let fp2 = Fingerprint {
            hostname: "a".to_string(),
            cpu: "b".to_string(),
            disk_serial: "x".to_string(),
            mac: "y".to_string(),
            machine_id: "e".to_string(),
        };
        assert_eq!(fp1.match_score(&fp2), 3);
    }
}
```

### Step 2.2: Verify compilation

Run: `cargo check`
Expected: No errors.

### Step 2.3: Commit

```bash
git add src/license/fingerprint.rs
git commit -m "feat(license): add hardware fingerprint collection and matching"
```

---

## Task 3: Request and Activation File Modules

**Files:**
- Create: `src/license/request.rs`
- Create: `src/license/activation.rs`
- Modify: `Cargo.toml`

### Step 3.1: Create request module

```rust
use crate::license::fingerprint::{Fingerprint, collect_fingerprint};
use serde::{Deserialize, Serialize};

const REQUEST_MAGIC: &[u8] = b"VOLREQ\x01";

#[derive(Serialize, Deserialize, Debug)]
struct LicenseRequest {
    version: u8,
    license_key: String,
    fingerprint: FingerprintData,
    timestamp: String,
    nonce: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FingerprintData {
    hash: String,
    components: std::collections::HashMap<String, String>,
}

pub fn generate_request(license_key: &str) -> anyhow::Result<String> {
    let fingerprint = collect_fingerprint();
    let hash = fingerprint.hash();
    let components = fingerprint.to_components_map();

    let nonce: String = (0..32)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect();

    let timestamp = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());

    let request = LicenseRequest {
        version: 1,
        license_key: license_key.to_string(),
        fingerprint: FingerprintData { hash, components },
        timestamp,
        nonce,
    };

    let json = serde_json::to_vec(&request)?;
    let mut file_data = Vec::new();
    file_data.extend_from_slice(REQUEST_MAGIC);
    file_data.extend_from_slice(&json);

    let filename = "license-request.bin".to_string();
    std::fs::write(&filename, &file_data)?;

    Ok(filename)
}

pub fn parse_request(data: &[u8]) -> anyhow::Result<LicenseRequest> {
    if !data.starts_with(REQUEST_MAGIC) {
        anyhow::bail!("Invalid request file: bad magic header");
    }
    let json = &data[REQUEST_MAGIC.len()..];
    let request: LicenseRequest = serde_json::from_slice(json)?;
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_parse_request() {
        let path = generate_request("test-key-123").unwrap();
        let data = std::fs::read(&path).unwrap();
        let request = parse_request(&data).unwrap();
        assert_eq!(request.version, 1);
        assert_eq!(request.license_key, "test-key-123");
        assert!(request.fingerprint.hash.starts_with("sha256:"));
        assert!(!request.nonce.is_empty());
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_parse_invalid_magic() {
        let result = parse_request(b"INVALID");
        assert!(result.is_err());
    }
}
```

### Step 3.2: Create activation module

```rust
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use serde::{Deserialize, Serialize};

/// Embedded Ed25519 public key. Replace with your actual public key.
const EMBEDDED_PUBLIC_KEY: &[u8] = &[
    // TODO: Replace with your 32-byte Ed25519 public key
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const ACTIVATION_MAGIC: &[u8] = b"VOLACT\x01";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LicenseActivation {
    pub version: u8,
    pub license_key: String,
    pub fingerprint_hash: String,
    pub issued_at: String,
    pub expires_at: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActivationPayload {
    version: u8,
    license_key: String,
    fingerprint_hash: String,
    issued_at: String,
    expires_at: String,
}

pub fn verify_activation_signature(data: &[u8]) -> anyhow::Result<LicenseActivation> {
    if !data.starts_with(ACTIVATION_MAGIC) {
        anyhow::bail!("Invalid activation file: bad magic header");
    }
    let json = &data[ACTIVATION_MAGIC.len()..];
    let activation: LicenseActivation = serde_json::from_slice(json)?;

    let verifying_key = VerifyingKey::from_bytes(
        EMBEDDED_PUBLIC_KEY.try_into().map_err(|_| anyhow::anyhow!("Invalid public key length"))?
    )?;

    let payload = ActivationPayload {
        version: activation.version,
        license_key: activation.license_key.clone(),
        fingerprint_hash: activation.fingerprint_hash.clone(),
        issued_at: activation.issued_at.clone(),
        expires_at: activation.expires_at.clone(),
    };
    let payload_json = serde_json::to_vec(&payload)?;

    let sig_bytes = hex::decode(&activation.signature)?;
    let signature = Signature::from_slice(&sig_bytes)?;

    verifying_key.verify(&payload_json, &signature)
        .map_err(|_| anyhow::anyhow!("Invalid activation signature"))?;

    let expires = time::OffsetDateTime::parse(
        &activation.expires_at,
        &time::format_description::well_known::Rfc3339
    ).map_err(|e| anyhow::anyhow!("Invalid expiry date: {}", e))?;
    if time::OffsetDateTime::now_utc() > expires {
        anyhow::bail!("License expired on {}", activation.expires_at);
    }

    Ok(activation)
}

pub fn import_activation_file(path: &str) -> anyhow::Result<LicenseActivation> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read activation file: {}", path))?;
    verify_activation_signature(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_invalid_magic() {
        let result = verify_activation_signature(b"INVALID");
        assert!(result.is_err());
    }
}
```

### Step 3.3: Add dependency

Add to `Cargo.toml` after `reqwest`:

```toml
ed25519-dalek = "=2.1.1"
```

### Step 3.4: Verify compilation

Run: `cargo check`
Expected: No errors (may download ed25519-dalek).

### Step 3.5: Commit

```bash
git add Cargo.toml Cargo.lock src/license/request.rs src/license/activation.rs
git commit -m "feat(license): add request/activation file handling and ed25519-dalek"
```

---

## Task 4: License Module Core

**File:** Create `src/license/mod.rs`

### Step 4.1: Create license mod.rs

```rust
pub mod activation;
pub mod fingerprint;
pub mod request;

use crate::storage::Storage;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum LicenseStatus {
    Valid {
        license_key: String,
        expires_at: Option<String>,
        days_remaining: Option<i64>,
    },
    Unactivated {
        message: String,
    },
    InvalidFingerprint {
        score: usize,
        required: usize,
        message: String,
    },
    Expired {
        expires_at: String,
        message: String,
    },
    InvalidSignature {
        message: String,
    },
    Corrupted {
        message: String,
    },
}

impl LicenseStatus {
    pub fn is_valid(&self) -> bool {
        matches!(self, LicenseStatus::Valid { .. })
    }
}

pub async fn verify_license(storage: &Storage) -> LicenseStatus {
    let activation_row = match storage.get_license_activation().await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return LicenseStatus::Unactivated {
                message: "License not found. Please activate your installation.".to_string(),
            };
        }
        Err(e) => {
            tracing::error!("Failed to read license activation: {}", e);
            return LicenseStatus::Corrupted {
                message: "License data is corrupted. Please reactivate your installation.".to_string(),
            };
        }
    };

    let activation = match activation::verify_activation_signature(&activation_row.activation_blob) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("License signature verification failed: {}", e);
            return LicenseStatus::InvalidSignature {
                message: "Invalid activation file. Please reactivate your installation.".to_string(),
            };
        }
    };

    let current_fp = fingerprint::collect_fingerprint();
    let stored_fp = fingerprint::Fingerprint {
        hostname: activation_row.fingerprint_hostname,
        cpu: activation_row.fingerprint_cpu,
        disk_serial: activation_row.fingerprint_disk_serial,
        mac: activation_row.fingerprint_mac,
        machine_id: activation_row.fingerprint_machine_id,
    };

    let score = current_fp.match_score(&stored_fp);
    if score < fingerprint::Fingerprint::MIN_MATCH {
        return LicenseStatus::InvalidFingerprint {
            score,
            required: fingerprint::Fingerprint::MIN_MATCH,
            message: format!(
                "Hardware fingerprint mismatch ({}/{}). This license may have been moved. Please reactivate.",
                score,
                fingerprint::Fingerprint::MIN_MATCH
            ),
        };
    }

    let days_remaining = time::OffsetDateTime::parse(
        &activation.expires_at,
        &time::format_description::well_known::Rfc3339
    ).ok().map(|exp| {
        let now = time::OffsetDateTime::now_utc();
        let duration = exp - now;
        duration.whole_days().max(0)
    });

    LicenseStatus::Valid {
        license_key: activation.license_key,
        expires_at: Some(activation.expires_at),
        days_remaining,
    }
}
```

### Step 4.2: Register module in main.rs

Add to `src/main.rs` after `mod timeutil;`:

```rust
mod license;
```

### Step 4.3: Verify compilation

Run: `cargo check`
Expected: No errors.

### Step 4.4: Commit

```bash
git add src/license/mod.rs src/main.rs
git commit -m "feat(license): add license status verification and fuzzy fingerprint matching"
```

---

## Task 5: CLI Integration and Startup

**Files:**
- Modify: `src/main.rs` (arg parsing, AppState)
- Modify: `src/api/mod.rs` (AppState struct, test helpers)

### Step 5.1: Add CLI argument handling

Replace the existing arg parsing in `src/main.rs`:

```rust
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--generate-request" => {
                if args.len() < 3 {
                    eprintln!("Usage: {} --generate-request <LICENSE_KEY>", args[0]);
                    std::process::exit(1);
                }
                let path = license::request::generate_request(&args[2])?;
                println!("License request written to: {}", path);
                return Ok(());
            }
            "--import-activation" => {
                if args.len() < 3 {
                    eprintln!("Usage: {} --import-activation <PATH> [CONFIG_PATH]", args[0]);
                    std::process::exit(1);
                }
                let activation_path = &args[2];
                let config_path = args.get(3).map(|s| s.as_str()).unwrap_or("config.yaml");
                let app_config = config::load_config(config_path)?;
                let storage = Storage::new(&app_config.db_path)?;
                let activation = license::activation::import_activation_file(activation_path)?;
                let fingerprint = license::fingerprint::collect_fingerprint();
                let row = crate::models::LicenseActivationRow {
                    license_key: activation.license_key.clone(),
                    fingerprint_hash: fingerprint.hash(),
                    activation_blob: std::fs::read(activation_path)?,
                    fingerprint_hostname: fingerprint.hostname,
                    fingerprint_cpu: fingerprint.cpu,
                    fingerprint_disk_serial: fingerprint.disk_serial,
                    fingerprint_mac: fingerprint.mac,
                    fingerprint_machine_id: fingerprint.machine_id,
                    activated_at: activation.issued_at,
                    expires_at: Some(activation.expires_at),
                };
                storage.store_license_activation(&row).await?;
                println!("License activated successfully: {}", activation.license_key);
                return Ok(());
            }
            "--license-status" => {
                let config_path = args.get(2).map(|s| s.as_str()).unwrap_or("config.yaml");
                let app_config = config::load_config(config_path)?;
                let storage = Storage::new(&app_config.db_path)?;
                let status = license::verify_license(&storage).await;
                println!("{}", serde_json::to_string_pretty(&status)?);
                std::process::exit(if status.is_valid() { 0 } else { 1 });
            }
            _ => {}
        }
    }

    let config_path = args.get(1).map(|s| s.as_str()).unwrap_or("config.yaml");
    let app_config = config::load_config(config_path)?;
```

### Step 5.2: Add license status to AppState

In `src/main.rs`, after `let storage = Storage::new(&app_config.db_path)?;`:

```rust
    let license_status = license::verify_license(&storage).await;
    if !license_status.is_valid() {
        tracing::warn!("License status: {:?}", license_status);
    }
```

Update `AppState` creation:

```rust
    let state = AppState {
        cache: cache.clone(),
        storage: storage.clone(),
        poller_manager,
        started_at: std::time::Instant::now(),
        auth_secret,
        cors_origin: app_config.cors_origin.clone(),
        cookie_secure: app_config.cookie_secure,
        oidc_config,
        jwks_manager,
        retention_config: app_config.retention.clone(),
        flow_engine: Some(Arc::new(flow_engine)),
        login_rate_limiter,
        audit_logger,
        intrusion_detector,
        license_status: license_status.clone(),
    };
```

### Step 5.3: Add license_status to AppState struct

In `src/api/mod.rs`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub cache: DeviceCache,
    pub storage: Storage,
    pub poller_manager: PollerManager,
    pub started_at: std::time::Instant,
    pub auth_secret: String,
    pub cors_origin: Option<String>,
    pub cookie_secure: bool,
    pub oidc_config: OidcConfig,
    pub jwks_manager: JwksManager,
    pub retention_config: crate::models::RetentionConfig,
    pub flow_engine: Option<Arc<crate::flow_engine::engine::FlowEngine>>,
    pub login_rate_limiter: crate::security::LoginRateLimiter,
    pub audit_logger: crate::security::AuditLogger,
    pub intrusion_detector: crate::security::IntrusionDetector,
    pub license_status: crate::license::LicenseStatus,
}
```

### Step 5.4: Update test helpers

In `src/api/mod.rs` `test_app()`:

```rust
    let state = AppState {
        cache,
        storage,
        poller_manager,
        started_at: std::time::Instant::now(),
        auth_secret: "test-secret".to_string(),
        cors_origin: None,
        cookie_secure: false,
        oidc_config: test_oidc_config(),
        jwks_manager: test_jwks_manager(),
        retention_config: crate::models::RetentionConfig::default(),
        flow_engine: None,
        login_rate_limiter: crate::security::LoginRateLimiter::new(),
        audit_logger: crate::security::AuditLogger::new().0,
        intrusion_detector: crate::security::IntrusionDetector::new(),
        license_status: crate::license::LicenseStatus::Unactivated {
            message: "test".to_string(),
        },
    };
```

Update all other inline `AppState` creations in tests similarly.

### Step 5.5: Verify compilation

Run: `cargo check`
Expected: No errors.

### Step 5.6: Commit

```bash
git add src/main.rs src/api/mod.rs
git commit -m "feat(license): integrate CLI commands and startup license check"
```

---

## Task 6: API Endpoint and Middleware

**Files:**
- Create: `src/api/license.rs`
- Modify: `src/api/mod.rs`

### Step 6.1: Create license API handler

```rust
use axum::{extract::State, Json};

use crate::api::AppState;

pub async fn get_license_status(State(state): State<AppState>) -> Json<crate::license::LicenseStatus> {
    Json(state.license_status.clone())
}
```

### Step 6.2: Register license module and route

Add `mod license;` to `src/api/mod.rs` after `mod flows;`.

Add route to public router:

```rust
    let public = Router::new()
        .route("/api/health", get(system::get_health))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::auth_me))
        .route("/api/auth/csrf-token", get(auth::get_csrf_token))
        .route("/api/settings/branding", get(settings::get_branding))
        .route("/api/settings/alarm-cooldown", get(settings::get_alarm_cooldown))
        .route("/api/license/status", get(license::get_license_status));
```

### Step 6.3: Add license check middleware

Add middleware function before `create_router`:

```rust
async fn license_check_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let path = req.uri().path();
    if path == "/api/health" || path == "/api/license/status" {
        return next.run(req).await;
    }

    if !state.license_status.is_valid() {
        let body = match serde_json::to_value(&state.license_status) {
            Ok(v) => v,
            Err(_) => serde_json::json!({"status": "error", "message": "license invalid"}),
        };
        return (
            StatusCode::FORBIDDEN,
            Json(body),
        ).into_response();
    }

    next.run(req).await
}
```

Add to `api_router`:

```rust
    let api_router = Router::new()
        .merge(public)
        .merge(protected)
        .merge(admin_protected)
        .route("/api/{*path}", get(api_not_found))
        .layer(middleware::from_fn_with_state(state.clone(), license_check_middleware))
        .layer(cors)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn(crate::security::request_id_middleware))
        .layer(middleware::from_fn(crate::security::security_headers_middleware))
        .layer(middleware::from_fn(crate::security::strip_forwarded_host_middleware));
```

### Step 6.4: Verify compilation

Run: `cargo check`
Expected: No errors.

### Step 6.5: Commit

```bash
git add src/api/license.rs src/api/mod.rs
git commit -m "feat(license): add license status API and middleware"
```

---

## Task 7: Cloudflare Worker

**Files:**
- Create: `worker/license-activate/package.json`
- Create: `worker/license-activate/wrangler.toml`
- Create: `worker/license-activate/schema.sql`
- Create: `worker/license-activate/src/index.ts`

### Step 7.1: Create package.json

```json
{
  "name": "voltrus-license-activate",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "deploy": "wrangler deploy",
    "dev": "wrangler dev"
  },
  "dependencies": {
    "@noble/ed25519": "^2.1.0"
  },
  "devDependencies": {
    "wrangler": "^3.0.0"
  }
}
```

### Step 7.2: Create wrangler.toml

```toml
name = "voltrus-license-activate"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[[d1_databases]]
binding = "DB"
database_name = "voltrus-licenses"
database_id = "YOUR_D1_DATABASE_ID"
```

### Step 7.3: Create D1 schema

```sql
CREATE TABLE IF NOT EXISTS license_bindings (
    license_key TEXT PRIMARY KEY,
    fingerprint_hash TEXT NOT NULL,
    activated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    customer_email TEXT,
    revoked INTEGER DEFAULT 0
);
```

### Step 7.4: Create worker source

```typescript
import { sign } from "@noble/ed25519";

export interface Env {
  DB: D1Database;
  POLAR_API_KEY: string;
  ED25519_PRIVATE_KEY: string;
  ADMIN_TOKEN: string;
}

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "POST, GET, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization",
};

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "OPTIONS") {
      return new Response(null, { headers: CORS_HEADERS });
    }

    const url = new URL(request.url);
    const path = url.pathname;

    try {
      if (path === "/activate" && request.method === "POST") {
        return await handleActivate(request, env);
      }
      if (path === "/admin" && request.method === "GET") {
        return await handleAdminPanel(request, env);
      }
      if (path === "/admin/transfer" && request.method === "POST") {
        return await handleTransfer(request, env);
      }
      return jsonResponse({ error: "Not found" }, 404);
    } catch (e: any) {
      return jsonResponse({ error: e.message || "Internal error" }, 500);
    }
  },
};

async function handleActivate(request: Request, env: Env): Promise<Response> {
  const formData = await request.formData();
  const file = formData.get("file") as File | null;
  if (!file) {
    return jsonResponse({ success: false, error: "No file uploaded" }, 400);
  }

  const bytes = new Uint8Array(await file.arrayBuffer());
  const request_data = parseRequestFile(bytes);
  if (!request_data) {
    return jsonResponse({ success: false, error: "Invalid request file" }, 400);
  }

  const polarValid = await validatePolarLicense(request_data.license_key, env.POLAR_API_KEY);
  if (!polarValid.valid) {
    return jsonResponse({ success: false, error: polarValid.error || "Invalid license key" }, 403);
  }

  const existing = await env.DB.prepare(
    "SELECT fingerprint_hash, revoked FROM license_bindings WHERE license_key = ?"
  ).bind(request_data.license_key).first<{ fingerprint_hash: string; revoked: number }>();

  if (existing) {
    if (existing.revoked) {
      return jsonResponse({ success: false, error: "License has been revoked" }, 403);
    }
    if (existing.fingerprint_hash === request_data.fingerprint.hash) {
      const activation = await generateActivation(request_data, env);
      return jsonResponse({ success: true, activation_file: activation }, 200);
    }
    return jsonResponse(
      { success: false, error: "License already bound to a different device. Contact support to transfer." },
      403
    );
  }

  await env.DB.prepare(
    "INSERT INTO license_bindings (license_key, fingerprint_hash) VALUES (?, ?)"
  ).bind(request_data.license_key, request_data.fingerprint.hash).run();

  const activation = await generateActivation(request_data, env);
  return jsonResponse({ success: true, activation_file: activation }, 200);
}

async function handleAdminPanel(request: Request, env: Env): Promise<Response> {
  const auth = request.headers.get("Authorization");
  if (!auth || auth !== `Bearer ${env.ADMIN_TOKEN}`) {
    return new Response("Unauthorized", { status: 401 });
  }

  const bindings = await env.DB.prepare(
    "SELECT license_key, fingerprint_hash, activated_at, revoked FROM license_bindings ORDER BY activated_at DESC"
  ).all<{ license_key: string; fingerprint_hash: string; activated_at: string; revoked: number }>();

  const rows = (bindings.results || [])
    .map((b) => `
    <tr>
      <td>${escapeHtml(b.license_key)}</td>
      <td>${escapeHtml(b.fingerprint_hash)}</td>
      <td>${b.activated_at}</td>
      <td>${b.revoked ? "REVOKED" : "Active"}</td>
      <td>
        <form method="POST" action="/admin/transfer" style="display:inline">
          <input type="hidden" name="license_key" value="${escapeHtml(b.license_key)}" />
          <button type="submit">Transfer</button>
        </form>
      </td>
    </tr>
  `).join("");

  const html = `<!DOCTYPE html>
<html>
<head><title>Voltrus License Admin</title></head>
<body>
<h1>License Bindings</h1>
<table border="1">
<tr><th>License Key</th><th>Fingerprint</th><th>Activated</th><th>Status</th><th>Action</th></tr>
${rows}
</table>
</body>
</html>`;

  return new Response(html, { headers: { "Content-Type": "text/html" } });
}

async function handleTransfer(request: Request, env: Env): Promise<Response> {
  const auth = request.headers.get("Authorization");
  if (!auth || auth !== `Bearer ${env.ADMIN_TOKEN}`) {
    return new Response("Unauthorized", { status: 401 });
  }

  const formData = await request.formData();
  const license_key = formData.get("license_key") as string;
  if (!license_key) {
    return jsonResponse({ error: "Missing license_key" }, 400);
  }

  await env.DB.prepare("DELETE FROM license_bindings WHERE license_key = ?")
    .bind(license_key).run();

  return jsonResponse({ success: true, message: "License transferred" }, 200);
}

function parseRequestFile(bytes: Uint8Array): any {
  const magic = new TextDecoder().decode(bytes.slice(0, 7));
  if (magic !== "VOLREQ\x01") return null;
  try {
    const json = new TextDecoder().decode(bytes.slice(7));
    return JSON.parse(json);
  } catch {
    return null;
  }
}

async function validatePolarLicense(key: string, apiKey: string): Promise<{ valid: boolean; error?: string }> {
  // TODO: Replace with actual Polar.sh API validation
  if (!key.startsWith("polar_")) {
    return { valid: false, error: "Invalid license key format" };
  }
  return { valid: true };
}

async function generateActivation(request_data: any, env: Env): Promise<string> {
  const payload = {
    version: 1,
    license_key: request_data.license_key,
    fingerprint_hash: request_data.fingerprint.hash,
    issued_at: new Date().toISOString(),
    expires_at: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString(),
  };

  const payload_json = new TextEncoder().encode(JSON.stringify(payload));
  const private_key = hexToBytes(env.ED25519_PRIVATE_KEY);
  const signature = await sign(payload_json, private_key);

  const activation = {
    ...payload,
    signature: bytesToHex(signature),
  };

  const json = JSON.stringify(activation);
  const magic = new TextEncoder().encode("VOLACT\x01");
  const result = new Uint8Array(magic.length + json.length);
  result.set(magic);
  result.set(new TextEncoder().encode(json), magic.length);
  return btoa(String.fromCharCode(...result));
}

function jsonResponse(data: any, status: number = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json", ...CORS_HEADERS },
  });
}

function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  return bytes;
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
```

### Step 7.5: Commit worker files

```bash
git add worker/license-activate/
git commit -m "feat(license): add Cloudflare Worker for license activation"
```

---

## Task 8: Frontend Integration

**Goal:** When the React app loads, it should call `GET /api/license/status`. If the status is not `Valid`, show an activation screen instead of the login/dashboard.

**Implementation:**
1. In the React app's entry point (or a top-level layout component), add a fetch to `/api/license/status` on mount.
2. If `status !== "Valid"`, render a full-screen `<LicenseActivationGuide />` component.
3. The guide shows the 3-step process with copy-pasteable commands.
4. Add a "Check Again" button that re-fetches the status.

**Note:** The exact frontend implementation depends on the React app structure (in `client-lit/` or `static/`). Since these are built artifacts, modify the source React/TypeScript files and rebuild.

### Example React component:

```tsx
function LicenseActivationGuide() {
  return (
    <div style={{ padding: 40, maxWidth: 600, margin: '0 auto' }}>
      <h1>Welcome to Voltrus</h1>
      <p>Your license is not activated.</p>
      <ol>
        <li>Run: <code>./voltrus-lite --generate-request YOUR_LICENSE_KEY</code></li>
        <li>Copy <code>license-request.bin</code> to a PC with internet</li>
        <li>Upload at <a href="https://activate.voltrus.id">activate.voltrus.id</a></li>
        <li>Copy the returned <code>license-activation.bin</code> back</li>
        <li>Run: <code>./voltrus-lite --import-activation license-activation.bin</code></li>
      </ol>
    </div>
  );
}
```

---

## Self-Review

**Spec coverage check:**
- Hardware fingerprint (5 components, 3/5 matching) → Task 2 ✅
- Request/activation file formats with magic headers → Tasks 3 ✅
- Ed25519 signature verification → Task 3 ✅
- Polar.sh integration placeholder → Task 7 ✅
- Cloudflare Worker with D1 → Task 7 ✅
- Admin panel → Task 7 ✅
- CLI args (`--generate-request`, `--import-activation`, `--license-status`) → Task 5 ✅
- Startup license check → Task 5 ✅
- API endpoint `/api/license/status` → Task 6 ✅
- License middleware → Task 6 ✅
- Fuzzy fingerprint matching → Task 4 ✅
- Frontend activation screen → Task 8 ✅

**Placeholder scan:**
- `EMBEDDED_PUBLIC_KEY` in activation.rs is a TODO but documented ✅
- `validatePolarLicense` in worker is a TODO but documented ✅
- No other TBDs or vague requirements ✅

**Type consistency:**
- `LicenseStatus` variants match across `mod.rs` and API usage ✅
- `LicenseActivationRow` fields match storage methods ✅
- `Fingerprint::MIN_MATCH` used consistently ✅

**Missing items:**
- Polar.sh webhook handler → Marked as future enhancement in spec, not in MVP
- Grace period for fingerprint mismatch → Future enhancement
- Feature flags in activation blob → Future enhancement
