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
        None => (None, None), // Perpetual license
    };

    LicenseStatus::Valid {
        license_key: activation.license_key,
        expires_at,
        days_remaining,
        tier: activation.tier,
        device_limit: activation.device_limit,
    }
}

// ── Engineering Mode Polling ──

/// Daily polling budget: 4 hours (14400 seconds).
pub const ENGINEERING_DAILY_BUDGET_SECS: i64 = 14400;

/// Tampered budget penalty: 1 hour (3600 seconds).
const ENGINEERING_TAMPERED_BUDGET_SECS: i64 = 3600;

/// Polling tick interval in seconds (background checker runs every 60s).
pub const ENGINEERING_POLLING_TICK_SECS: i64 = 60;

// ── HMAC Tamper Protection ──

fn derive_hmac_key() -> Vec<u8> {
    let fp = fingerprint::collect_fingerprint();
    format!("voltrus-engineering-{}", fp.machine_id).into_bytes()
}

/// Sign the polling state with HMAC-SHA256.
pub fn sign_polling_state(date: &str, used_secs: i64) -> String {
    use hmac::Mac;
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&derive_hmac_key())
        .expect("HMAC key length is valid");
    mac.update(format!("{}:{}", date, used_secs).as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Verify the HMAC of a stored polling state.
pub fn verify_polling_hmac(date: &str, used_secs: i64, expected_hmac: &str) -> bool {
    // Constant-time comparison via hmac crate
    use hmac::Mac;
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&derive_hmac_key())
        .expect("HMAC key length is valid");
    mac.update(format!("{}:{}", date, used_secs).as_bytes());
    match hex::decode(expected_hmac) {
        Ok(expected_bytes) => mac.verify_slice(&expected_bytes).is_ok(),
        Err(_) => false,
    }
}

/// Stored polling state (persisted in settings_kv).
#[derive(serde::Serialize, serde::Deserialize)]
struct EngineeringPollingState {
    date: String,
    used_secs: i64,
    hmac: String,
}

/// Load the daily polling state from storage. Returns (used_secs, budget_secs).
/// If date has changed since last write, the counter resets.
/// If HMAC verification fails (tampering), budget is reduced to 1h.
pub async fn load_daily_polling_state(storage: &Storage) -> (i64, i64) {
    let today = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Iso8601::DATE)
        .unwrap_or_else(|_| "unknown".to_string());

    match storage.get_settings(&["engineering_polling_state"]).await {
        Ok(settings) => {
            if let Some(setting) = settings.into_iter().next() {
                match serde_json::from_str::<EngineeringPollingState>(&setting.value) {
                    Ok(state) => {
                        if state.date != today {
                            // New day — reset counter
                            tracing::info!(
                                "Engineering polling: new day ({}) detected, resetting daily counter",
                                today
                            );
                            return (0, ENGINEERING_DAILY_BUDGET_SECS);
                        }
                        // Verify HMAC
                        if verify_polling_hmac(&state.date, state.used_secs, &state.hmac) {
                            (state.used_secs, ENGINEERING_DAILY_BUDGET_SECS)
                        } else {
                            tracing::warn!(
                                "Engineering polling: HMAC mismatch detected — treating as tampered, budget reduced to 1h"
                            );
                            (state.used_secs, ENGINEERING_TAMPERED_BUDGET_SECS)
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Engineering polling: failed to parse stored state: {}", e);
                        (0, ENGINEERING_DAILY_BUDGET_SECS)
                    }
                }
            } else {
                (0, ENGINEERING_DAILY_BUDGET_SECS)
            }
        }
        Err(e) => {
            tracing::warn!(
                "Engineering polling: failed to load state from storage: {}",
                e
            );
            (0, ENGINEERING_DAILY_BUDGET_SECS)
        }
    }
}

/// Save the daily polling state to storage with a fresh HMAC.
pub async fn save_daily_polling_state(storage: &Storage, used_secs: i64) {
    let today = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Iso8601::DATE)
        .unwrap_or_else(|_| "unknown".to_string());

    let hmac = sign_polling_state(&today, used_secs);
    let state = EngineeringPollingState {
        date: today,
        used_secs,
        hmac,
    };
    match serde_json::to_string(&state) {
        Ok(json) => {
            if let Err(e) = storage
                .upsert_setting("engineering_polling_state", &json)
                .await
            {
                tracing::warn!("Engineering polling: failed to save state: {}", e);
            }
        }
        Err(e) => {
            tracing::warn!("Engineering polling: failed to serialize state: {}", e);
        }
    }
}

// ── Resolve License Status (server startup entry point) ──

/// Resolves the current license status. If a paid license activation exists,
/// it runs the full verification flow. Otherwise, returns Engineering mode
/// with the daily cumulative polling budget.
pub async fn resolve_license_status(storage: &Storage) -> LicenseStatus {
    // Try paid license first
    match storage.get_license_activation().await {
        Ok(Some(_)) => {
            // A license activation exists; run the full verify flow
            return verify_license(storage).await;
        }
        Ok(None) => {
            // No paid license — fall through to engineering mode
        }
        Err(e) => {
            tracing::error!("Failed to read license activation: {}", e);
            return LicenseStatus::Corrupted {
                message: "License data is corrupted. Please reactivate your installation."
                    .to_string(),
            };
        }
    }

    // No activation found — return Engineering mode with daily budget
    let now_ts = time::OffsetDateTime::now_utc().unix_timestamp();
    let tier = LicenseTier::Engineering;
    let feature_flags = tier.to_feature_flags();

    let (used_secs, budget_secs) = load_daily_polling_state(storage).await;
    let remaining_secs = (budget_secs - used_secs).max(0);

    if remaining_secs <= 0 {
        LicenseStatus::EngineeringPollingStopped {
            started_at: now_ts,
            daily_budget_secs: budget_secs,
            daily_used_secs: used_secs,
            message: "Daily polling budget exhausted. Resumes tomorrow at midnight UTC. Activate a license for unlimited polling.".to_string(),
        }
    } else {
        LicenseStatus::Engineering {
            started_at: now_ts,
            polling_elapsed_secs: used_secs,
            polling_remaining_secs: remaining_secs,
            daily_budget_secs: budget_secs,
            daily_used_secs: used_secs,
            polling_active: false, // Starts paused; user must explicitly start
            tier,
            feature_flags,
        }
    }
}

// ── Engineering Polling Enforcement ──

/// Updates the daily polling usage and checks against the budget.
///
/// - If the status is `Engineering`, adds `tick_secs` to the daily used counter,
///   checks against budget, and persists to SQLite with fresh HMAC.
/// - Returns updated `Engineering` if time remains, or transitions to
///   `EngineeringPollingStopped` when the budget is exhausted.
/// - For all other statuses, returns a clone as-is.
pub async fn update_engineering_polling_daily(
    status: &LicenseStatus,
    storage: &Storage,
    tick_secs: i64,
) -> LicenseStatus {
    match status {
        LicenseStatus::Engineering {
            started_at,
            tier,
            feature_flags,
            daily_budget_secs,
            daily_used_secs,
            polling_active,
            ..
        } => {
            let new_used = if *polling_active {
                daily_used_secs + tick_secs
            } else {
                *daily_used_secs
            };
            let budget = *daily_budget_secs;
            let remaining = (budget - new_used).max(0);

            // Persist to SQLite with fresh HMAC
            save_daily_polling_state(storage, new_used).await;

            if remaining > 0 {
                LicenseStatus::Engineering {
                    started_at: *started_at,
                    polling_elapsed_secs: new_used,
                    polling_remaining_secs: remaining,
                    daily_budget_secs: budget,
                    daily_used_secs: new_used,
                    polling_active: *polling_active,
                    tier: tier.clone(),
                    feature_flags: feature_flags.clone(),
                }
            } else {
                LicenseStatus::EngineeringPollingStopped {
                    started_at: *started_at,
                    daily_budget_secs: budget,
                    daily_used_secs: new_used,
                    message: "Daily polling budget exhausted. Resumes tomorrow at midnight UTC. Activate a license for unlimited polling.".to_string(),
                }
            }
        }
        other => other.clone(),
    }
}
