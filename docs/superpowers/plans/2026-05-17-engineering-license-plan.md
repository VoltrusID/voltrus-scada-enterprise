# Engineering-Only License System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace 30-day trial with unlimited Engineering mode (2h polling per restart), add .voltrus-scada project export/import.

**Architecture:** Replace `Trial`/`TrialExpired` with `Engineering`/`EngineeringPollingStopped` in the license tier/status enums. Enforce 2-hour polling limit via process-start-time tracking in main.rs. Add CLI export/import commands that use SQLite backup + ZIP packaging.

**Tech Stack:** Rust (Axum, rusqlite, serde, zip), React/TypeScript

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src/license/tier.rs` | Modify | Replace Trial with Engineering enum variant |
| `src/license/status.rs` | Modify | Replace Trial/TrialExpired with Engineering/EngineeringPollingStopped |
| `src/license/verification.rs` | Modify | Remove trial logic, add engineering mode resolution |
| `src/license/tests.rs` | Modify | Update all unit tests for Engineering |
| `src/main.rs` | Modify | Add process start time, 2h polling enforcement, background checker |
| `src/api/mod.rs` | Modify | Update license_check_middleware tier string fallbacks |
| `src/api/tests/license_tier.rs` | Modify | Update integration tests |
| `src/api/tests/helpers.rs` | Modify | Update test helper defaults |
| `src/api/tests/flows.rs` | Modify | Update test license status |
| `src/api/tests/api_keys.rs` | Modify | Update test license status |
| `src/api/tests/auth.rs` | Modify | Update test license status |
| `src/api/tests/analytics.rs` | Modify | Update test license tier |
| `src/project_export.rs` | Create | Export/import .voltrus-scada project files |
| `src/cli.rs` | Modify | Add `voltrus export` and `voltrus import` subcommands |
| `client/src/EngineeringBanner.tsx` | Create | Replace TrialBanner with Engineering mode banner |
| `client/src/App.tsx` | Modify | Update license status checks, import EngineeringBanner |
| `client/src/LicenseContext.tsx` | Modify | Add engineering-specific fields |
| `client/src/LicenseActivationScreen.tsx` | Modify | Update messaging |
| `client/src/NotificationSettingsPanel.tsx` | Modify | Replace "Trial" references with "Engineering" |
| `client/src/HardwareCertPanel.tsx` | Modify | Replace "free_trial" tier reference |

---

### Task 1: LicenseTier — Replace Trial with Engineering

**Files:**
- Modify: `src/license/tier.rs`
- Modify: `src/license/tests.rs`

- [ ] **Step 1: Update LicenseTier enum in tier.rs**

Replace `Trial` with `Engineering` in the enum definition:

```rust
// ── License Tier ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LicenseTier {
    Engineering,
    Starter,
    Professional,
    Enterprise,
}
```

- [ ] **Step 2: Update from_tier_str()**

Change the fallback from `LicenseTier::Trial` to `LicenseTier::Engineering`:

```rust
pub fn from_tier_str(s: Option<&str>) -> Self {
    match s.map(|t| t.to_lowercase()) {
        Some(ref t) if t == "starter" => LicenseTier::Starter,
        Some(ref t) if t == "professional" || t == "pro" => LicenseTier::Professional,
        Some(ref t) if t == "enterprise" => LicenseTier::Enterprise,
        _ => LicenseTier::Engineering,
    }
}
```

- [ ] **Step 3: Update all feature gate methods**

Replace every `LicenseTier::Trial` with `LicenseTier::Engineering` in all match expressions. The methods to update:

```rust
pub fn max_devices(&self) -> Option<u32> {
    match self {
        LicenseTier::Engineering => Some(5),
        LicenseTier::Starter => Some(50),
        LicenseTier::Professional => None,
        LicenseTier::Enterprise => None,
    }
}

pub fn data_retention_days(&self) -> u32 {
    match self {
        LicenseTier::Engineering => 7,
        LicenseTier::Starter | LicenseTier::Professional | LicenseTier::Enterprise => 365,
    }
}

pub fn opcua_enabled(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn email_notifications(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn webhook_notifications(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn push_notifications(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn chat_notifications(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn rest_api_export(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn csv_export(&self) -> bool {
    true
}

pub fn analytics(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn oee(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn pdf_reporting(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn backup_restore(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn auto_discovery(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn white_label(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn theme_full(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn per_user_layouts(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn rbac(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn multi_site(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn recipes(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn sso(&self) -> bool {
    matches!(self, LicenseTier::Enterprise)
}

pub fn sso_saml(&self) -> bool {
    matches!(self, LicenseTier::Enterprise)
}

pub fn cfr_part_11(&self) -> bool {
    matches!(self, LicenseTier::Enterprise)
}

pub fn ethernet_ip(&self) -> bool {
    matches!(self, LicenseTier::Enterprise)
}

pub fn s7_protocol(&self) -> bool {
    matches!(self, LicenseTier::Enterprise)
}

pub fn dnp3(&self) -> bool {
    matches!(self, LicenseTier::Enterprise)
}

pub fn bacnet(&self) -> bool {
    matches!(self, LicenseTier::Enterprise)
}

pub fn rtsp_video(&self) -> bool {
    matches!(self, LicenseTier::Enterprise)
}

pub fn ai_copilot(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn mqtt_broker(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn prometheus_export(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn bi_connectors(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn data_egress(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}

pub fn outbound_webhooks(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn flow_engine(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn data_compression(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn health_monitor(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn command_queue(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn templates(&self) -> bool {
    !matches!(self, LicenseTier::Engineering)
}

pub fn templates_write(&self) -> bool {
    matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
}
```

- [ ] **Step 4: Update the log message in main.rs that references "Trial tier"**

In `src/main.rs`, find the line:
```rust
tracing::info!("FlowEngine skipped — Trial tier");
```
Change to:
```rust
tracing::info!("FlowEngine skipped — Engineering tier");
```

- [ ] **Step 5: Update tests in tests.rs**

Replace all `LicenseTier::Trial` with `LicenseTier::Engineering` and update test names:

```rust
#[test]
fn test_license_tier_max_devices() {
    assert_eq!(LicenseTier::Engineering.max_devices(), Some(5));
    assert_eq!(LicenseTier::Starter.max_devices(), Some(50));
    assert_eq!(LicenseTier::Professional.max_devices(), None);
    assert_eq!(LicenseTier::Enterprise.max_devices(), None);
}

#[test]
fn test_license_tier_feature_flags() {
    // Engineering tier: limited features
    let engineering = LicenseTier::Engineering;
    assert_eq!(engineering.max_devices(), Some(5));
    assert_eq!(engineering.data_retention_days(), 7);
    assert!(!engineering.opcua_enabled());
    assert!(!engineering.email_notifications());
    assert!(!engineering.webhook_notifications());
    assert!(!engineering.push_notifications());
    assert!(!engineering.chat_notifications());
    assert!(!engineering.rest_api_export());
    assert!(engineering.csv_export());
    assert!(!engineering.analytics());
    assert!(!engineering.oee());
    assert!(!engineering.pdf_reporting());
    assert!(!engineering.backup_restore());
    assert!(!engineering.white_label());
    assert!(!engineering.rbac());
    assert!(!engineering.multi_site());
    assert!(!engineering.recipes());
    assert!(!engineering.sso());
    assert!(!engineering.sso_saml());
    assert!(!engineering.cfr_part_11());
    assert!(!engineering.ethernet_ip());
    assert!(!engineering.s7_protocol());
    // ... (Starter, Professional, Enterprise tests unchanged)
}

#[test]
fn test_from_tier_str() {
    assert_eq!(LicenseTier::from_tier_str(None), LicenseTier::Engineering);
    assert_eq!(LicenseTier::from_tier_str(Some("")), LicenseTier::Engineering);
    assert_eq!(
        LicenseTier::from_tier_str(Some("unknown")),
        LicenseTier::Engineering
    );
    // ... (starter, professional, enterprise tests unchanged)
}

#[test]
fn test_to_feature_flags_json_structure() {
    let flags = LicenseTier::Engineering.to_feature_flags();
    assert_eq!(flags["max_devices"], 5);
    assert_eq!(flags["data_retention_days"], 7);
    assert_eq!(flags["csv_export"], true);
    assert_eq!(flags["opcua_enabled"], false);
    // ... (Starter, Enterprise tests unchanged)
}

#[test]
fn test_ethernet_ip_enterprise_only() {
    assert!(!LicenseTier::Engineering.ethernet_ip());
    assert!(!LicenseTier::Starter.ethernet_ip());
    assert!(!LicenseTier::Professional.ethernet_ip());
    assert!(LicenseTier::Enterprise.ethernet_ip());
}

#[test]
fn test_s7_protocol_enterprise_only() {
    assert!(!LicenseTier::Engineering.s7_protocol());
    assert!(!LicenseTier::Starter.s7_protocol());
    assert!(!LicenseTier::Professional.s7_protocol());
    assert!(LicenseTier::Enterprise.s7_protocol());
}

#[test]
fn test_rtsp_video_enterprise_only() {
    assert!(!LicenseTier::Engineering.rtsp_video());
    assert!(!LicenseTier::Starter.rtsp_video());
    assert!(!LicenseTier::Professional.rtsp_video());
    assert!(LicenseTier::Enterprise.rtsp_video());
}

#[test]
fn test_theme_full_professional_and_above() {
    assert!(!LicenseTier::Engineering.theme_full());
    assert!(!LicenseTier::Starter.theme_full());
    assert!(LicenseTier::Professional.theme_full());
    assert!(LicenseTier::Enterprise.theme_full());
}

#[test]
fn test_per_user_layouts_professional_and_above() {
    assert!(!LicenseTier::Engineering.per_user_layouts());
    assert!(!LicenseTier::Starter.per_user_layouts());
    assert!(LicenseTier::Professional.per_user_layouts());
    assert!(LicenseTier::Enterprise.per_user_layouts());
}

#[test]
fn test_white_label_starter_and_above() {
    assert!(!LicenseTier::Engineering.white_label());
    assert!(LicenseTier::Starter.white_label());
    assert!(LicenseTier::Professional.white_label());
    assert!(LicenseTier::Enterprise.white_label());
}

#[test]
fn test_sso_saml_enterprise_only() {
    assert!(!LicenseTier::Engineering.sso_saml());
    assert!(!LicenseTier::Starter.sso_saml());
    assert!(!LicenseTier::Professional.sso_saml());
    assert!(LicenseTier::Enterprise.sso_saml());
}

#[test]
fn test_sso_saml_in_feature_flags() {
    let flags = LicenseTier::Enterprise.to_feature_flags();
    assert_eq!(flags["sso_saml"], true);

    let flags = LicenseTier::Engineering.to_feature_flags();
    assert_eq!(flags["sso_saml"], false);

    let flags = LicenseTier::Professional.to_feature_flags();
    assert_eq!(flags["sso_saml"], false);
}
```

- [ ] **Step 6: Run tests to verify**

Run: `cargo test --lib license::tests -- --nocapture`
Expected: All tests pass

---

### Task 2: LicenseStatus — Replace Trial/TrialExpired with Engineering/EngineeringPollingStopped

**Files:**
- Modify: `src/license/status.rs`
- Modify: `src/license/tests.rs`

- [ ] **Step 1: Update LicenseStatus enum in status.rs**

```rust
use super::tier::LicenseTier;
use super::*;

// ── License Status ──

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum LicenseStatus {
    Valid {
        license_key: String,
        expires_at: Option<String>,
        days_remaining: Option<i64>,
        tier: Option<String>,
        device_limit: Option<u32>,
    },
    Engineering {
        started_at: i64,
        polling_elapsed_secs: i64,
        polling_remaining_secs: i64,
        tier: LicenseTier,
        feature_flags: serde_json::Value,
    },
    EngineeringPollingStopped {
        started_at: i64,
        message: String,
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
        matches!(
            self,
            LicenseStatus::Valid { .. } | LicenseStatus::Engineering { .. } | LicenseStatus::EngineeringPollingStopped { .. }
        )
    }

    /// Extracts the LicenseTier from Valid or Engineering status.
    /// Defaults to Engineering for all other statuses.
    pub fn tier(&self) -> LicenseTier {
        match self {
            LicenseStatus::Valid { tier, .. } => LicenseTier::from_tier_str(tier.as_deref()),
            LicenseStatus::Engineering { tier, .. } => tier.clone(),
            LicenseStatus::EngineeringPollingStopped { .. } => LicenseTier::Engineering,
            _ => LicenseTier::Engineering,
        }
    }

    /// Returns feature flags for Valid or Engineering status, empty object otherwise.
    pub fn feature_flags(&self) -> serde_json::Value {
        match self {
            LicenseStatus::Valid { tier, .. } => {
                LicenseTier::from_tier_str(tier.as_deref()).to_feature_flags()
            }
            LicenseStatus::Engineering { feature_flags, .. } => feature_flags.clone(),
            _ => serde_json::json!({}),
        }
    }
}
```

- [ ] **Step 2: Update tests in tests.rs — replace Trial/TrialExpired tests**

Remove old `test_trial_status_is_valid`, `test_trial_expired_is_not_valid`, `test_trial_status_tier_returns_trial`, `test_trial_expired_status_tier_returns_trial` and add:

```rust
#[test]
fn test_engineering_status_is_valid() {
    let tier = LicenseTier::Engineering;
    let status = LicenseStatus::Engineering {
        started_at: 1000,
        polling_elapsed_secs: 3600,
        polling_remaining_secs: 3600,
        tier: tier.clone(),
        feature_flags: tier.to_feature_flags(),
    };
    assert!(status.is_valid());
}

#[test]
fn test_engineering_polling_stopped_is_valid() {
    let status = LicenseStatus::EngineeringPollingStopped {
        started_at: 1000,
        message: "Polling stopped".to_string(),
    };
    assert!(status.is_valid());
}

#[test]
fn test_engineering_status_tier_returns_engineering() {
    let tier = LicenseTier::Engineering;
    let status = LicenseStatus::Engineering {
        started_at: 1000,
        polling_elapsed_secs: 3600,
        polling_remaining_secs: 3600,
        tier: tier.clone(),
        feature_flags: tier.to_feature_flags(),
    };
    assert_eq!(status.tier(), LicenseTier::Engineering);
}

#[test]
fn test_engineering_polling_stopped_tier_returns_engineering() {
    let status = LicenseStatus::EngineeringPollingStopped {
        started_at: 1000,
        message: "Polling stopped".to_string(),
    };
    assert_eq!(status.tier(), LicenseTier::Engineering);
}
```

- [ ] **Step 3: Update test_license_status_tier_and_feature_flags**

Replace the Trial section with Engineering:

```rust
#[test]
fn test_license_status_tier_and_feature_flags() {
    // Valid status with tier
    let valid = LicenseStatus::Valid {
        license_key: "key".to_string(),
        expires_at: None,
        days_remaining: None,
        tier: Some("professional".to_string()),
        device_limit: None,
    };
    assert_eq!(valid.tier(), LicenseTier::Professional);
    assert_eq!(valid.feature_flags()["opcua_enabled"], true);

    // Valid status without tier defaults to Engineering
    let valid_no_tier = LicenseStatus::Valid {
        license_key: "key".to_string(),
        expires_at: None,
        days_remaining: None,
        tier: None,
        device_limit: None,
    };
    assert_eq!(valid_no_tier.tier(), LicenseTier::Engineering);

    // Engineering status
    let tier = LicenseTier::Engineering;
    let engineering = LicenseStatus::Engineering {
        started_at: 1000,
        polling_elapsed_secs: 3600,
        polling_remaining_secs: 3600,
        tier: tier.clone(),
        feature_flags: tier.to_feature_flags(),
    };
    assert_eq!(engineering.tier(), LicenseTier::Engineering);
    assert_eq!(engineering.feature_flags()["csv_export"], true);

    // Other statuses default to Engineering
    let unactivated = LicenseStatus::Unactivated {
        message: "msg".to_string(),
    };
    assert_eq!(unactivated.tier(), LicenseTier::Engineering);
    assert_eq!(unactivated.feature_flags(), serde_json::json!({}));
}
```

- [ ] **Step 4: Run tests to verify**

Run: `cargo test --lib license::tests -- --nocapture`
Expected: All tests pass

---

### Task 3: LicenseVerification — Remove trial logic, add engineering mode

**Files:**
- Modify: `src/license/verification.rs`

- [ ] **Step 1: Replace resolve_license_status()**

```rust
use super::status::LicenseStatus;
use super::tier::LicenseTier;
use super::*;

// ── License Verification (existing CLI flow) ──

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
                message: "License data is corrupted. Please reactivate your installation."
                    .to_string(),
            };
        }
    };

    let activation = match activation::verify_activation_signature(&activation_row.activation_blob)
    {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("License signature verification failed: {}", e);
            return LicenseStatus::InvalidSignature {
                message: "Invalid activation file. Please reactivate your installation."
                    .to_string(),
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

    let (expires_at, days_remaining) = match activation.expires_at {
        Some(ref expires_at_str) => {
            let days = time::OffsetDateTime::parse(
                expires_at_str,
                &time::format_description::well_known::Rfc3339,
            )
            .ok()
            .map(|exp| {
                let now = time::OffsetDateTime::now_utc();
                let duration = exp - now;
                duration.whole_days().max(0)
            });
            (Some(expires_at_str.clone()), days)
        }
        None => (None, None),
    };

    LicenseStatus::Valid {
        license_key: activation.license_key,
        expires_at,
        days_remaining,
        tier: activation.tier,
        device_limit: activation.device_limit,
    }
}

// ── Resolve License Status (server startup entry point) ──

const ENGINEERING_POLLING_LIMIT_SECS: i64 = 7200; // 2 hours

/// Resolves the current license status. If a paid license activation exists,
/// it runs the full verification flow. Otherwise, returns Engineering mode.
pub async fn resolve_license_status(storage: &Storage) -> LicenseStatus {
    // Try paid license first
    match storage.get_license_activation().await {
        Ok(Some(_)) => {
            return verify_license(storage).await;
        }
        Ok(None) => {
            // No paid license — proceed to engineering mode
        }
        Err(e) => {
            tracing::error!("Failed to read license activation: {}", e);
            return LicenseStatus::Corrupted {
                message: "License data is corrupted. Please reactivate your installation."
                    .to_string(),
            };
        }
    }

    // Engineering mode — unlimited time, 2h polling per restart
    let now = time::OffsetDateTime::now_utc();
    let now_ts = now.unix_timestamp();
    let tier = LicenseTier::Engineering;
    let feature_flags = tier.to_feature_flags();

    LicenseStatus::Engineering {
        started_at: now_ts,
        polling_elapsed_secs: 0,
        polling_remaining_secs: ENGINEERING_POLLING_LIMIT_SECS,
        tier,
        feature_flags,
    }
}

/// Calculates polling elapsed/remaining for an Engineering status.
/// Called periodically during server runtime to update the status.
pub fn update_engineering_polling(status: &LicenseStatus) -> LicenseStatus {
    match status {
        LicenseStatus::Engineering {
            started_at,
            tier,
            feature_flags,
            ..
        } => {
            let now = time::OffsetDateTime::now_utc();
            let now_ts = now.unix_timestamp();
            let elapsed = (now_ts - started_at).max(0);
            let remaining = (ENGINEERING_POLLING_LIMIT_SECS - elapsed).max(0);

            if remaining > 0 {
                LicenseStatus::Engineering {
                    started_at: *started_at,
                    polling_elapsed_secs: elapsed,
                    polling_remaining_secs: remaining,
                    tier: tier.clone(),
                    feature_flags: feature_flags.clone(),
                }
            } else {
                LicenseStatus::EngineeringPollingStopped {
                    started_at: *started_at,
                    message: "Data polling stopped (2h limit reached). Restart Voltrus to continue polling. Config remains fully editable.".to_string(),
                }
            }
        }
        other => other.clone(),
    }
}
```

- [ ] **Step 2: Remove TRIAL_DURATION_DAYS, TRIAL_START_KEY, get_trial_start, set_trial_start**

Delete these constants and functions from the file — they are no longer needed.

---

### Task 4: Polling Enforcement in main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add process start time tracking**

After the config load, capture the process start time:

```rust
let process_start = std::time::Instant::now();
```

Place this right after `let app_config = config::load_config(config_path)?;`

- [ ] **Step 2: Update license status resolution to use Arc<RwLock>**

Replace the existing license resolution:

```rust
let license_status = license::resolve_license_status(&storage).await;
let license_status = Arc::new(std::sync::RwLock::new(license_status));
let license_tier = license_status.read().unwrap().tier();
let feature_flags = license_status.read().unwrap().feature_flags();

if !license_status.read().unwrap().is_valid() {
    tracing::warn!(
        "License status: {:?} — skipping Modbus polling",
        license_status.read().unwrap()
    );
}
```

- [ ] **Step 3: Add polling enforcement check before starting pollers**

Replace the Modbus polling block:

```rust
let poller_manager = PollerManager::new(
    cache.clone(),
    storage.clone(),
    network::NetworkManager::new(app_config.networks.clone()),
);

// Check if engineering mode polling is allowed
let polling_allowed = {
    let elapsed = process_start.elapsed().as_secs() as i64;
    elapsed < license::ENGINEERING_POLLING_LIMIT_SECS
};

#[cfg(feature = "modbus")]
if license_status.read().unwrap().is_valid() && polling_allowed {
    let device_configs = storage.get_all_device_configs().await?;
    let device_limit = license_tier.max_devices();
    let mut started_count: u32 = 0;
    for config in &device_configs {
        if let Some(limit) = device_limit
            && started_count >= limit
        {
            tracing::warn!(
                "Device '{}' skipped — {} tier limit of {} devices reached. Upgrade at https://voltrus.id",
                config.id,
                serde_json::to_string(&license_tier).unwrap_or_else(|_| "unknown".to_string()),
                limit
            );
            continue;
        }
        poller_manager.start(config.clone()).await;
        tracing::info!("Started poller for device {}", config.id);
        started_count += 1;
    }
} else if license_status.read().unwrap().is_valid() && !polling_allowed {
    tracing::warn!("Engineering mode: data polling stopped (2h limit reached). Restart to resume.");
} else {
    tracing::warn!("No valid license — Modbus polling disabled");
}
```

- [ ] **Step 4: Add background polling checker task**

After the poller manager setup, spawn a background task:

```rust
// Background checker: stop pollers when engineering mode 2h limit is reached
{
    let pm = poller_manager.clone();
    let ls = license_status.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let elapsed = process_start.elapsed().as_secs() as i64;
            if elapsed >= license::ENGINEERING_POLLING_LIMIT_SECS {
                // Update license status
                let mut status = ls.write().unwrap();
                *status = license::update_engineering_polling(&*status);
                // Stop all pollers if not already stopped
                if matches!(*status, LicenseStatus::EngineeringPollingStopped { .. }) {
                    pm.stop_all().await;
                    tracing::warn!("Engineering mode: all pollers stopped (2h limit reached)");
                }
            } else {
                // Update polling remaining for API responses
                let mut status = ls.write().unwrap();
                *status = license::update_engineering_polling(&*status);
            }
        }
    });
}
```

- [ ] **Step 5: Update AppState struct in api/mod.rs**

Change the `license_status` field type:

```rust
pub license_status: Arc<std::sync::RwLock<crate::license::LicenseStatus>>,
```

Also update the `license_check_middleware` to use `state.license_status.read().unwrap()` wherever `state.license_status` is accessed.

- [ ] **Step 6: Update AppState construction in main.rs**

Update the `license_status` field and fix the `LicenseTier::Trial` reference:

```rust
        license_status: license_status.clone(),
```

And change:
```rust
            if license_tier == license::LicenseTier::Trial {
```
To:
```rust
            if license_tier == license::LicenseTier::Engineering {
```

- [ ] **Step 7: Update license_check_middleware**

In `src/api/mod.rs`, update the middleware to use the RwLock:

```rust
async fn license_check_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let path = req.uri().path();
    if path == "/api/v1/health"
        || path == "/api/license/status"
        || path == "/api/v1/settings/branding"
        || path.starts_with("/api/v1/auth/")
    {
        let mut resp = next.run(req).await;
        let tier_str = serde_json::to_string(&state.license_tier).unwrap_or_default();
        resp.headers_mut().insert(
            HeaderName::from_static("x-license-tier"),
            HeaderValue::from_str(&tier_str).unwrap_or_else(|_| HeaderValue::from_static("engineering")),
        );
        return resp;
    }

    let status = state.license_status.read().unwrap();
    if !status.is_valid() {
        let body = match serde_json::to_value(&*status) {
            Ok(v) => v,
            Err(_) => serde_json::json!({"status": "error", "message": "license invalid"}),
        };
        return (StatusCode::FORBIDDEN, Json(body)).into_response();
    }

    let mut resp = next.run(req).await;
    let tier_str = serde_json::to_string(&state.license_tier).unwrap_or_default();
    resp.headers_mut().insert(
        HeaderName::from_static("x-license-tier"),
        HeaderValue::from_str(&tier_str).unwrap_or_else(|_| HeaderValue::from_static("engineering")),
    );
    resp
}
```

- [ ] **Step 8: Update the license status endpoint**

In `src/api/license.rs`:

```rust
use axum::{Json, extract::State};

use crate::api::AppState;

pub async fn get_license_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let status = state.license_status.read().unwrap();
    let mut status = serde_json::to_value(&*status).unwrap();
    status["tier"] = serde_json::to_value(&state.license_tier).unwrap();
    status["feature_flags"] = state.feature_flags.clone();
    Json(status)
}
```

---

### Task 5: Update all test files to use Engineering instead of Trial

**Files:**
- Modify: `src/api/tests/license_tier.rs`
- Modify: `src/api/tests/helpers.rs`
- Modify: `src/api/tests/flows.rs`
- Modify: `src/api/tests/api_keys.rs`
- Modify: `src/api/tests/auth.rs`
- Modify: `src/api/tests/analytics.rs`
- Modify: `src/video/tests.rs`

- [ ] **Step 1: Update license_tier.rs tests**

Replace all `LicenseTier::Trial` with `LicenseTier::Engineering` and `LicenseStatus::Trial` with `LicenseStatus::Engineering`:

```rust
#[tokio::test]
async fn test_engineering_middleware_allows_access() {
    let tier = crate::license::LicenseTier::Engineering;
    let app = test_app_with_license_status(
        crate::license::LicenseStatus::Engineering {
            started_at: 0,
            polling_elapsed_secs: 3600,
            polling_remaining_secs: 3600,
            tier: tier.clone(),
            feature_flags: tier.to_feature_flags(),
        },
        tier,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_engineering_polling_stopped_middleware_allows_access() {
    let status = crate::license::LicenseStatus::EngineeringPollingStopped {
        started_at: 0,
        message: "stopped".to_string(),
    };
    assert!(status.is_valid());

    let tier = crate::license::LicenseTier::Engineering;
    let app = test_app_with_license_status(
        crate::license::LicenseStatus::EngineeringPollingStopped {
            started_at: 0,
            message: "stopped".to_string(),
        },
        tier,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}
```

Update `test_tier_trial_feature_gates` → `test_tier_engineering_feature_gates`:

```rust
#[tokio::test]
async fn test_tier_engineering_feature_gates() {
    let tier = crate::license::LicenseTier::Engineering;
    let app = test_app_with_license_status(
        crate::license::LicenseStatus::Engineering {
            started_at: 0,
            polling_elapsed_secs: 3600,
            polling_remaining_secs: 3600,
            tier: tier.clone(),
            feature_flags: tier.to_feature_flags(),
        },
        tier.clone(),
    );

    // SHOULD ALLOW (Engineering) — health and branding are open
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Branding returns defaults for engineering (theme_level=none)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/settings/branding")
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
    assert_eq!(json["theme_level"], "none");

    // Feature flags should reflect Engineering
    let flags = tier.to_feature_flags();
    assert_eq!(flags["opcua_enabled"], false);
    assert_eq!(flags["email_notifications"], false);
    assert_eq!(flags["chat_notifications"], false);
    assert_eq!(flags["s7_protocol"], false);
    assert_eq!(flags["ethernet_ip"], false);
    assert_eq!(flags["rtsp_video"], false);
    assert_eq!(flags["theme_full"], false);
    assert_eq!(flags["per_user_layouts"], false);
    assert_eq!(flags["white_label"], false);
    assert_eq!(flags["rbac"], false);
    assert_eq!(flags["multi_site"], false);
    assert_eq!(flags["max_devices"], 5);
    assert_eq!(flags["data_retention_days"], 7);
}
```

Update the branding theme levels test to use Engineering instead of Trial.

- [ ] **Step 2: Update helpers.rs**

Replace all `LicenseStatus::Valid { license_key: "polar_test_key" ... }` references — these are fine as-is since they test paid tiers. No changes needed for the Professional default.

- [ ] **Step 3: Update flows.rs, api_keys.rs, auth.rs, analytics.rs**

These files use `LicenseStatus::Valid` with `polar_test_key` — no changes needed since they test paid tier behavior. Only change `LicenseTier::Trial` references if any exist.

- [ ] **Step 4: Update video/tests.rs**

Replace:
```rust
assert!(!LicenseTier::Trial.rtsp_video());
```
With:
```rust
assert!(!LicenseTier::Engineering.rtsp_video());
```

- [ ] **Step 5: Run all tests to verify**

Run: `cargo test --lib -- --nocapture 2>&1 | head -100`
Expected: All tests pass

---

### Task 6: Create project export/import module

**Files:**
- Create: `src/project_export.rs`

- [ ] **Step 1: Create the module file**

```rust
use anyhow::Result;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::backup::Backup;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::Path;
use tar::{Archive, Builder};

use crate::license::LicenseTier;

/// Metadata stored inside a .voltrus-scada project file.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectManifest {
    pub version: String,
    pub exported_at: String,
    pub voltrus_version: String,
    pub tier_at_export: String,
    pub device_count: u32,
    pub tag_count: u32,
    pub screen_count: u32,
}

/// Export the current project to a .voltrus-scada file.
pub fn export_project(
    db_path: &str,
    output_path: &str,
    tier: &str,
) -> Result<ProjectManifest> {
    if !Path::new(db_path).exists() {
        anyhow::bail!("Database file not found: {}", db_path);
    }

    // Create a consistent SQLite backup
    let temp_db = format!("/tmp/voltrus-export-{}.db", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    {
        let conn = rusqlite::Connection::open(db_path)?;
        let backup_conn = rusqlite::Connection::open(&temp_db)?;
        let backup = Backup::new(&conn, &backup_conn)?;
        backup.run_to_completion(100, std::time::Duration::from_millis(100), None)?;
    }

    // Count entities for manifest
    let conn = rusqlite::Connection::open(&temp_db)?;
    let device_count: u32 = conn.query_row("SELECT COUNT(*) FROM devices", [], |r| r.get(0)).unwrap_or(0);
    let tag_count: u32 = conn.query_row("SELECT COUNT(*) FROM data_sources", [], |r| r.get(0)).unwrap_or(0);
    let screen_count: u32 = conn.query_row("SELECT COUNT(*) FROM screens", [], |r| r.get(0)).unwrap_or(0);
    drop(conn);

    let now = time::OffsetDateTime::now_utc();
    let manifest = ProjectManifest {
        version: "1".to_string(),
        exported_at: now.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
        voltrus_version: env!("CARGO_PKG_VERSION").to_string(),
        tier_at_export: tier.to_string(),
        device_count,
        tag_count,
        screen_count,
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)?;

    // Create tar.gz archive
    let tar_gz = File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    // Add config.sqlite
    tar.append_path_with_name(&temp_db, "config.sqlite")?;

    // Add manifest.json
    let mut header = tar::Header::new_gnu();
    header.set_size(manifest_json.len() as u64);
    header.set_cksum();
    tar.append_data(&mut header, "manifest.json", manifest_json.as_bytes())?;

    tar.finish()?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_db);

    println!("Exported {} devices, {} tags, {} screens to {}", device_count, tag_count, screen_count, output_path);

    Ok(manifest)
}

/// Import a .voltrus-scada project file into the current database.
pub fn import_project(
    db_path: &str,
    input_path: &str,
    dry_run: bool,
    force: bool,
    target_tier: &str,
) -> Result<ProjectManifest> {
    // Validate file exists
    if !Path::new(input_path).exists() {
        anyhow::bail!("Project file not found: {}", input_path);
    }

    // Extract archive
    let tar_gz = File::open(input_path)?;
    let dec = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(dec);

    let temp_dir = format!("/tmp/voltrus-import-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    fs::create_dir_all(&temp_dir)?;
    archive.unpack(&temp_dir)?;

    // Read manifest
    let manifest_path = format!("{}/manifest.json", temp_dir);
    let manifest_json = fs::read_to_string(&manifest_path)?;
    let manifest: ProjectManifest = serde_json::from_str(&manifest_json)?;

    if dry_run {
        println!("Dry run — would import:");
        println!("  {} devices", manifest.device_count);
        println!("  {} tags", manifest.tag_count);
        println!("  {} screens", manifest.screen_count);
        println!("  Exported from tier: {}", manifest.tier_at_export);
        println!("  Target tier: {}", target_tier);

        // Check tier compatibility
        check_tier_compatibility(&manifest.tier_at_export, target_tier)?;

        let _ = fs::remove_dir_all(&temp_dir);
        return Ok(manifest);
    }

    // Check tier compatibility
    check_tier_compatibility(&manifest.tier_at_export, target_tier)?;

    // Import database
    let imported_db = format!("{}/config.sqlite", temp_dir);
    if !Path::new(&imported_db).exists() {
        anyhow::bail!("Invalid project file: missing config.sqlite");
    }

    // Attach imported DB and copy data
    let conn = rusqlite::Connection::open(db_path)?;
    conn.execute(&format!("ATTACH DATABASE '{}' AS imported", imported_db), [])?;

    // List of tables to import (config only, no runtime data)
    let tables = [
        "devices", "data_sources", "screens", "alarms", "flows",
        "settings", "annotations", "shift_handover", "user_notifications",
        "roles", "permissions", "role_permissions", "user_roles",
        "equipment_templates", "egress_jobs", "webhooks",
        "api_keys", "audit_log", "notification_rules",
        "report_schedules", "sim_profiles", "hub_sites",
        "cameras", "recipes", "recipe_parameters",
        "script_proposals", "ai_settings", "naming_conventions",
        "branding_settings", "compression_settings", "retention_settings",
    ];

    for table in &tables {
        let sql = if force {
            format!("DELETE FROM {}; INSERT INTO {} SELECT * FROM imported.{}", table, table, table)
        } else {
            format!("INSERT OR IGNORE INTO {} SELECT * FROM imported.{}", table, table)
        };
        let _ = conn.execute(&sql, []);
    }

    conn.execute("DETACH DATABASE imported", [])?;
    drop(conn);

    // Reset user passwords on import
    let conn = rusqlite::Connection::open(db_path)?;
    let _ = conn.execute_batch("UPDATE users SET must_change_password = 1");

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);

    println!("Imported {} devices, {} tags, {} screens", manifest.device_count, manifest.tag_count, manifest.screen_count);

    Ok(manifest)
}

/// Check if the target tier supports features from the exported project.
fn check_tier_compatibility(exported_tier: &str, target_tier: &str) -> Result<()> {
    let exported = LicenseTier::from_tier_str(Some(exported_tier));
    let target = LicenseTier::from_tier_str(Some(target_tier));

    // If target is Engineering, everything imports but features are gated at runtime
    if matches!(target, LicenseTier::Engineering) {
        println!("Note: Target is Engineering mode. Enterprise features will be imported but gated at runtime.");
        return Ok(());
    }

    // Check if target tier supports exported features
    if exported.opcua_enabled() && !target.opcua_enabled() {
        println!("Warning: OPC-UA devices imported but {} tier does not support OPC-UA", target_tier);
    }
    if exported.s7_protocol() && !target.s7_protocol() {
        println!("Warning: S7 devices imported but {} tier does not support S7", target_tier);
    }
    if exported.ethernet_ip() && !target.ethernet_ip() {
        println!("Warning: EtherNet/IP devices imported but {} tier does not support EtherNet/IP", target_tier);
    }
    if exported.dnp3() && !target.dnp3() {
        println!("Warning: DNP3 devices imported but {} tier does not support DNP3", target_tier);
    }
    if exported.bacnet() && !target.bacnet() {
        println!("Warning: BACnet devices imported but {} tier does not support BACnet", target_tier);
    }
    if exported.rtsp_video() && !target.rtsp_video() {
        println!("Warning: Camera configs imported but {} tier does not support RTSP video", target_tier);
    }
    if exported.cfr_part_11() && !target.cfr_part_11() {
        println!("Warning: CFR Part 11 settings imported but {} tier does not support compliance mode", target_tier);
    }
    if exported.sso() && !target.sso() {
        println!("Warning: SSO settings imported but {} tier does not support SSO", target_tier);
    }

    Ok(())
}
```

- [ ] **Step 2: Add module declaration in main.rs**

Add `mod project_export;` to the module list in `src/main.rs`.

---

### Task 7: Add CLI export/import commands

**Files:**
- Modify: `src/cli.rs`

- [ ] **Step 1: Update HELP_TEXT**

Add to the help text:

```
    voltrus export [OPTIONS]           Export project to .voltrus-scada file
    voltrus import <FILE> [OPTIONS]    Import project from .voltrus-scada file

EXPORT OPTIONS:
    --output <PATH>    Output file path (default: voltrus-project.voltrus-scada)
    --config <PATH>    Config file path for db_path lookup (default: config.yaml)

IMPORT OPTIONS:
    <FILE>             Path to .voltrus-scada file
    --config <PATH>    Config file path (default: config.yaml)
    --dry-run          Show what would be imported without making changes
    --force            Overwrite existing config with imported data
```

- [ ] **Step 2: Add CLI handlers**

In the `handle_cli` match block, add:

```rust
"export" => handle_export(args),
"import" => handle_import(args),
```

- [ ] **Step 3: Implement handle_export**

```rust
fn handle_export(args: &[String]) -> bool {
    let config_path = args.iter().position(|a| a == "--config")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("config.yaml");

    let output_path = args.iter().position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("./voltrus-project.voltrus-scada");

    let app_config = match config::load_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config error: {e}");
            std::process::exit(1);
        }
    };

    let storage = match Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        }
    };

    // Determine current tier
    let license_status = match crate::license::resolve_license_status(&storage).await {
        crate::license::LicenseStatus::Valid { tier, .. } => tier.unwrap_or_else(|| "engineering".to_string()),
        _ => "engineering".to_string(),
    };

    match crate::project_export::export_project(&app_config.db_path, output_path, &license_status) {
        Ok(manifest) => {
            println!("Project exported successfully!");
            println!("  Devices: {}", manifest.device_count);
            println!("  Tags: {}", manifest.tag_count);
            println!("  Screens: {}", manifest.screen_count);
            true
        }
        Err(e) => {
            eprintln!("Export failed: {e}");
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 4: Implement handle_import**

```rust
fn handle_import(args: &[String]) -> bool {
    if args.len() < 3 || args[2].starts_with('-') {
        eprintln!("Usage: {} import <FILE> [--config <PATH>] [--dry-run] [--force]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[2];
    let config_path = args.iter().position(|a| a == "--config")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("config.yaml");

    let dry_run = args.contains(&"--dry-run".to_string());
    let force = args.contains(&"--force".to_string());

    let app_config = match config::load_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config error: {e}");
            std::process::exit(1);
        }
    };

    let storage = match Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        }
    };

    // Determine target tier
    let license_status = match crate::license::resolve_license_status(&storage).await {
        crate::license::LicenseStatus::Valid { tier, .. } => tier.unwrap_or_else(|| "engineering".to_string()),
        _ => "engineering".to_string(),
    };

    match crate::project_export::import_project(&app_config.db_path, input_path, dry_run, force, &license_status) {
        Ok(manifest) => {
            if dry_run {
                println!("\nDry run complete. No changes were made.");
            } else {
                println!("\nImport complete!");
            }
            true
        }
        Err(e) => {
            eprintln!("Import failed: {e}");
            std::process::exit(1);
        }
    }
}
```

---

### Task 8: Frontend — Create EngineeringBanner

**Files:**
- Create: `client/src/EngineeringBanner.tsx`

- [ ] **Step 1: Create the EngineeringBanner component**

```tsx
import { type LicenseInfo } from './LicenseContext';

interface EngineeringBannerProps {
  licenseInfo: LicenseInfo;
  onActivate: () => void;
}

export function EngineeringBanner({ licenseInfo, onActivate }: EngineeringBannerProps) {
  if (licenseInfo.status === 'Engineering') {
    const elapsed = licenseInfo.polling_elapsed_secs ?? 0;
    const remaining = licenseInfo.polling_remaining_secs ?? 0;
    const hours = Math.floor(remaining / 3600);
    const minutes = Math.floor((remaining % 3600) / 60);

    const isLow = remaining < 1800; // Less than 30 minutes

    return (
      <div style={{
        background: isLow ? '#b45309' : '#1e3a5f',
        color: '#ffffff',
        padding: '10px 20px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 12,
        fontFamily: 'Inter, sans-serif',
        fontSize: 13,
        fontWeight: 500,
        flexShrink: 0,
        zIndex: 40,
      }}>
        <span>Engineering Mode &mdash; {hours}h {minutes}m polling remaining</span>
        <button
          onClick={onActivate}
          style={{
            background: 'rgba(255,255,255,0.2)',
            border: '1px solid rgba(255,255,255,0.4)',
            borderRadius: 6,
            padding: '4px 14px',
            color: '#ffffff',
            fontSize: 12,
            fontWeight: 600,
            cursor: 'pointer',
            fontFamily: 'Inter, sans-serif',
          }}
        >
          Activate License
        </button>
      </div>
    );
  }

  if (licenseInfo.status === 'EngineeringPollingStopped') {
    return (
      <div style={{
        background: '#b45309',
        color: '#ffffff',
        padding: '10px 20px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 12,
        fontFamily: 'Inter, sans-serif',
        fontSize: 13,
        fontWeight: 500,
        flexShrink: 0,
        zIndex: 40,
      }}>
        <span>Data polling stopped (2h limit). Restart Voltrus to continue polling. Config remains fully editable.</span>
        <button
          onClick={onActivate}
          style={{
            background: 'rgba(255,255,255,0.2)',
            border: '1px solid rgba(255,255,255,0.4)',
            borderRadius: 6,
            padding: '4px 14px',
            color: '#ffffff',
            fontSize: 12,
            fontWeight: 600,
            cursor: 'pointer',
            fontFamily: 'Inter, sans-serif',
          }}
        >
          Activate License
        </button>
      </div>
    );
  }

  return null;
}
```

---

### Task 9: Frontend — Update App.tsx

**Files:**
- Modify: `client/src/App.tsx`

- [ ] **Step 1: Update imports**

Replace:
```tsx
import { TrialBanner } from './TrialBanner'
```
With:
```tsx
import { EngineeringBanner } from './EngineeringBanner'
```

- [ ] **Step 2: Update license status check**

Replace:
```tsx
const isLicensed = licenseStatus === 'Valid' || licenseStatus === 'Trial' || licenseStatus === 'TrialExpired'
```
With:
```tsx
const isLicensed = licenseStatus === 'Valid' || licenseStatus === 'Engineering' || licenseStatus === 'EngineeringPollingStopped'
```

- [ ] **Step 3: Update banner rendering**

Replace:
```tsx
{(licenseStatus === 'Trial' || licenseStatus === 'TrialExpired') && (
  <TrialBanner licenseInfo={licenseInfo} onActivate={() => setShowLicenseActivation(true)} />
)}
```
With:
```tsx
{(licenseStatus === 'Engineering' || licenseStatus === 'EngineeringPollingStopped') && (
  <EngineeringBanner licenseInfo={licenseInfo} onActivate={() => setShowLicenseActivation(true)} />
)}
```

---

### Task 10: Frontend — Update LicenseContext

**Files:**
- Modify: `client/src/LicenseContext.tsx`

- [ ] **Step 1: Add engineering-specific fields**

```tsx
import { createContext, useContext } from 'react';

export interface LicenseInfo {
  status: string;
  tier: string;
  feature_flags: Record<string, unknown>;
  days_remaining?: number | null;
  started_at?: number | null;
  expires_at?: number | null;
  license_key?: string;
  polling_elapsed_secs?: number | null;
  polling_remaining_secs?: number | null;
}

export const LicenseContext = createContext<LicenseInfo | null>(null);

export function useLicense(): LicenseInfo | null {
  return useContext(LicenseContext);
}
```

---

### Task 11: Frontend — Update remaining "Trial" references

**Files:**
- Modify: `client/src/NotificationSettingsPanel.tsx`
- Modify: `client/src/HardwareCertPanel.tsx`
- Modify: `client/src/LicenseActivationScreen.tsx`

- [ ] **Step 1: Update NotificationSettingsPanel.tsx**

Replace:
```tsx
const isTrialNotificationsDisabled = license?.tier === 'trial' && license.feature_flags?.email_notifications === false
```
With:
```tsx
const isEngineeringNotificationsDisabled = license?.tier === 'engineering' && license.feature_flags?.email_notifications === false
```

And update the UI reference from `FeatureBadge tier="Starter"` condition.

- [ ] **Step 2: Update HardwareCertPanel.tsx**

Replace `'free_trial'` tier reference with `'engineering'`:

```tsx
case 'engineering': return 'Engineering'
```

- [ ] **Step 3: Update LicenseActivationScreen.tsx**

Update any "trial" messaging text to reference "Engineering mode" instead.

---

### Task 12: Remove trial_start from settings

**Files:**
- Modify: `src/storage/settings_kv.rs` (if trial_start handling exists)
- Modify: `src/license/verification.rs` (already done in Task 3)

- [ ] **Step 1: Search for and remove any trial_start references**

Search: `grep -r "trial_start" src/`
Remove any remaining references. The settings_kv store may have a `trial_start` key that is no longer read — it's harmless to leave but should be cleaned up.

---

### Task 13: Build and test

- [ ] **Step 1: Run cargo build**

Run: `cargo build 2>&1 | tail -30`
Expected: Clean build with no errors

- [ ] **Step 2: Run all tests**

Run: `cargo test 2>&1 | tail -50`
Expected: All tests pass

- [ ] **Step 3: Run frontend build**

Run: `cd client && npm run build 2>&1 | tail -20`
Expected: Clean build with no errors

---

### Task 14: Commit

- [ ] **Step 1: Commit all changes**

```bash
git add -A
git commit -m "feat: replace trial with engineering mode, add project export/import

- Replace Trial/TrialExpired with Engineering/EngineeringPollingStopped
- Engineering mode: unlimited time, 2h polling per restart
- Add voltrus export/import CLI commands (.voltrus-scada format)
- Update frontend: EngineeringBanner replaces TrialBanner
- Update all tests and references"
```
