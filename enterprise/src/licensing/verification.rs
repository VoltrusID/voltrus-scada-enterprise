use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use voltrus::license::LicenseStatus;
use voltrus::models::LicenseActivationRow;
use voltrus::storage::Storage;

use super::activation::{self, LicenseActivation};
use super::fingerprint::{self, Fingerprint};

/// Days a lapsed monthly key keeps its add-on features, so a late renewal or an
/// offline site does not lose features the moment the period ends.
pub const GRACE_DAYS: i64 = 7;

/// Resolve the licence status from the stored activation. No activation is
/// `Unactivated`; nothing here ever blocks the core.
pub async fn resolve_license_status(storage: &Storage) -> LicenseStatus {
    match storage.get_license_activation().await {
        Ok(Some(row)) => evaluate_row(&row, &fingerprint::collect_fingerprint(), OffsetDateTime::now_utc()),
        Ok(None) => LicenseStatus::Unactivated {
            message: "No licence activated. The core runs without one; activate a key to unlock Integrator or Enterprise features.".to_string(),
        },
        Err(e) => {
            tracing::error!("Failed to read licence activation: {}", e);
            LicenseStatus::Corrupted {
                message: "Licence data is corrupted. Please reactivate.".to_string(),
            }
        }
    }
}

/// Judge a stored activation at `now` against the current machine fingerprint.
pub fn evaluate_row(
    row: &LicenseActivationRow,
    current_fp: &Fingerprint,
    now: OffsetDateTime,
) -> LicenseStatus {
    let activation = match activation::verify_activation_signature(&row.activation_blob) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("Licence signature verification failed: {}", e);
            return LicenseStatus::InvalidSignature {
                message: "Invalid activation file. Please reactivate.".to_string(),
            };
        }
    };

    let stored_fp = Fingerprint {
        hostname: row.fingerprint_hostname.clone(),
        cpu: row.fingerprint_cpu.clone(),
        disk_serial: row.fingerprint_disk_serial.clone(),
        mac: row.fingerprint_mac.clone(),
        machine_id: row.fingerprint_machine_id.clone(),
    };
    let score = current_fp.match_score(&stored_fp);
    if score < Fingerprint::MIN_MATCH {
        return LicenseStatus::InvalidFingerprint {
            score,
            required: Fingerprint::MIN_MATCH,
            message: format!(
                "Hardware fingerprint mismatch ({}/{}). This licence may have been moved. Please reactivate.",
                score,
                Fingerprint::MIN_MATCH
            ),
        };
    }

    evaluate_activation(&activation, now)
}

/// Apply expiry and grace to a signature-checked activation.
pub fn evaluate_activation(activation: &LicenseActivation, now: OffsetDateTime) -> LicenseStatus {
    let mut days_remaining = None;
    let mut grace = false;

    if let Some(expires_at) = activation.expires_at.as_deref() {
        let expiry = match OffsetDateTime::parse(expires_at, &Rfc3339) {
            Ok(t) => t,
            Err(_) => {
                return LicenseStatus::InvalidSignature {
                    message: "Activation has an invalid expiry date. Please reactivate."
                        .to_string(),
                };
            }
        };
        if now > expiry + time::Duration::days(GRACE_DAYS) {
            return LicenseStatus::Expired {
                expires_at: expires_at.to_string(),
                message: "This licence has expired. Renew it to unlock Integrator or Enterprise features again; the core keeps running.".to_string(),
            };
        }
        grace = now > expiry;
        days_remaining = Some((expiry - now).whole_days().max(0));
    }

    LicenseStatus::Valid {
        license_key: activation.license_key.clone(),
        expires_at: activation.expires_at.clone(),
        days_remaining,
        tier: activation.tier.clone(),
        grace,
    }
}
