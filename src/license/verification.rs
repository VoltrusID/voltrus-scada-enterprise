use super::*;
use super::status::LicenseStatus;
use super::tier::LicenseTier;

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

    let (expires_at, days_remaining) = match activation.expires_at {
        Some(ref expires_at_str) => {
            let days = time::OffsetDateTime::parse(
                expires_at_str,
                &time::format_description::well_known::Rfc3339
            ).ok().map(|exp| {
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

// ── Resolve License Status (server startup entry point) ──

const TRIAL_DURATION_DAYS: i64 = 30;
const TRIAL_START_KEY: &str = "trial_start";

/// Resolves the current license status. If a paid license activation exists,
/// it runs the full verification flow. Otherwise, it checks/initializes trial state.
pub async fn resolve_license_status(storage: &Storage) -> LicenseStatus {
    // Try paid license first
    match storage.get_license_activation().await {
        Ok(Some(_)) => {
            // A license activation exists; run the full verify flow
            return verify_license(storage).await;
        }
        Ok(None) => {
            // No paid license — proceed to trial logic
        }
        Err(e) => {
            tracing::error!("Failed to read license activation: {}", e);
            return LicenseStatus::Corrupted {
                message: "License data is corrupted. Please reactivate your installation.".to_string(),
            };
        }
    }

    // Trial flow
    let now = time::OffsetDateTime::now_utc();
    let now_ts = now.unix_timestamp();

    let started_at = match get_trial_start(storage).await {
        Ok(Some(ts)) => ts,
        Ok(None) => {
            // First launch — initialize trial
            if let Err(e) = set_trial_start(storage, now_ts).await {
                tracing::error!("Failed to set trial start: {}", e);
                return LicenseStatus::Corrupted {
                    message: "Failed to initialize trial. Please check storage.".to_string(),
                };
            }
            now_ts
        }
        Err(e) => {
            tracing::error!("Failed to read trial start: {}", e);
            return LicenseStatus::Corrupted {
                message: "Trial data is corrupted. Please check storage.".to_string(),
            };
        }
    };

    let elapsed_days = (now_ts - started_at) / 86400;
    let days_remaining = (TRIAL_DURATION_DAYS - elapsed_days).max(0);
    let expires_at = started_at + (TRIAL_DURATION_DAYS * 86400);

    if days_remaining > 0 {
        let tier = LicenseTier::Trial;
        let feature_flags = tier.to_feature_flags();
        LicenseStatus::Trial {
            started_at,
            expires_at,
            days_remaining,
            tier,
            feature_flags,
        }
    } else {
        LicenseStatus::TrialExpired {
            started_at,
            message: format!(
                "Your {}-day trial has expired. Please activate a license to continue using Voltrus Lite.",
                TRIAL_DURATION_DAYS
            ),
        }
    }
}

/// Reads the trial start timestamp from the settings KV store.
async fn get_trial_start(storage: &Storage) -> anyhow::Result<Option<i64>> {
    let settings = storage.get_settings(&[TRIAL_START_KEY]).await?;
    if let Some(setting) = settings.into_iter().next() {
        Ok(Some(setting.value.parse::<i64>()?))
    } else {
        Ok(None)
    }
}

/// Persists the trial start timestamp to the settings KV store.
async fn set_trial_start(storage: &Storage, ts: i64) -> anyhow::Result<()> {
    storage.upsert_setting(TRIAL_START_KEY, &ts.to_string()).await
}
