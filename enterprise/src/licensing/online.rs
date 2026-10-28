//! Activation against activate.voltrus.id, offline import and periodic renewal.
//!
//! These calls only happen when an operator activates or when an installed
//! Integrator/Enterprise licence is close to its expiry. The open core never
//! contacts Voltrus servers.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{Context, bail};
use base64::Engine;
use time::OffsetDateTime;
use voltrus::license::LicenseStatus;
use voltrus::models::LicenseActivationRow;
use voltrus::storage::Storage;

use super::activation::{self, LicenseActivation};
use super::fingerprint::{Fingerprint, collect_fingerprint};
use super::request::build_request;
use super::verification::{evaluate_activation, resolve_license_status};

const DEFAULT_ACTIVATION_BASE: &str = "https://activate.voltrus.id";

/// Renewal is attempted when the key expires within this many days (or already has).
const RENEW_WINDOW_DAYS: i64 = 10;

/// How often the background task re-checks whether a renewal is due.
const RENEW_CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// A support contract that ends within this many days (or already ended) is
/// re-read from the activation server, so a renewal paid on the console reaches an
/// installed key without reactivating it.
const SUPPORT_REFRESH_WINDOW_DAYS: i64 = 10;

/// While a support contract is near its end or lapsed, ask at most this often.
const SUPPORT_REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

fn endpoint(path: &str) -> String {
    let base = std::env::var("VOLTRUS_ACTIVATION_URL")
        .unwrap_or_else(|_| DEFAULT_ACTIVATION_BASE.to_string());
    format!("{}/{}", base.trim_end_matches('/'), path)
}

/// Ask the activation server to bind a key to this machine and return the signed
/// activation file. `site_name` names the site of a key that has none yet.
pub async fn fetch_activation(
    license_key: &str,
    site_name: Option<&str>,
) -> anyhow::Result<Vec<u8>> {
    post_request("activate", license_key, site_name).await
}

/// Ask the activation server for a fresh activation (new expiry) for a key that is
/// already bound to this machine. Never binds and never uses up an activation.
pub async fn fetch_renewal(license_key: &str) -> anyhow::Result<Vec<u8>> {
    post_request("renew", license_key, None).await
}

async fn post_request(
    path: &str,
    license_key: &str,
    site_name: Option<&str>,
) -> anyhow::Result<Vec<u8>> {
    let request_data = build_request(license_key, site_name)?;
    let file_part = reqwest::multipart::Part::bytes(request_data)
        .file_name("license-request.bin")
        .mime_str("application/octet-stream")?;
    let form = reqwest::multipart::Form::new()
        .text("license_key", license_key.to_string())
        .part("file", file_part);

    let resp = reqwest::Client::new()
        .post(endpoint(path))
        .multipart(form)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .context("Failed to reach the activation server. Check the internet connection, or activate offline.")?;

    let status = resp.status();
    if !status.is_success() {
        // Refusals (revoked, unknown key, not activated) carry {"error": "..."}.
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        match body["error"].as_str() {
            Some(err) => bail!("Activation failed: {err}"),
            None => bail!("Activation server returned HTTP {status}"),
        }
    }

    let result: serde_json::Value = resp.json().await.context("Invalid activation response")?;
    if !result["success"].as_bool().unwrap_or(false) {
        bail!(
            "Activation failed: {}",
            result["error"].as_str().unwrap_or("Unknown error")
        );
    }
    let b64 = result["activation_file"]
        .as_str()
        .context("Activation server did not return an activation file")?;
    Ok(base64::engine::general_purpose::STANDARD.decode(b64)?)
}

/// Verify an activation file and store it. Returns the verified activation.
///
/// Refuses files signed for another machine and files whose licence lapsed
/// beyond the grace period.
pub async fn store_activation(
    storage: &Storage,
    blob: Vec<u8>,
) -> anyhow::Result<LicenseActivation> {
    let activation = activation::verify_activation_signature(&blob)?;
    store_verified(storage, blob, activation, collect_fingerprint()).await
}

/// Judge a signature-checked activation against this machine and the clock, then
/// persist it with the site and support columns the status endpoints report.
async fn store_verified(
    storage: &Storage,
    blob: Vec<u8>,
    activation: LicenseActivation,
    fingerprint: Fingerprint,
) -> anyhow::Result<LicenseActivation> {
    if activation.fingerprint_hash != fingerprint.hash() {
        bail!("This activation file was issued for a different machine.");
    }
    if let LicenseStatus::Expired { message, .. } =
        evaluate_activation(&activation, OffsetDateTime::now_utc())
    {
        bail!(message);
    }
    let row = LicenseActivationRow {
        license_key: activation.license_key.clone(),
        fingerprint_hash: fingerprint.hash(),
        activation_blob: blob,
        fingerprint_hostname: fingerprint.hostname,
        fingerprint_cpu: fingerprint.cpu,
        fingerprint_disk_serial: fingerprint.disk_serial,
        fingerprint_mac: fingerprint.mac,
        fingerprint_machine_id: fingerprint.machine_id,
        activated_at: activation.issued_at.clone(),
        expires_at: activation.expires_at.clone(),
        tier: activation.tier.clone(),
        device_limit: activation.device_limit,
        site_id: activation.site_id.clone(),
        site_name: activation.site_name.clone(),
        support_until: activation.support_until,
    };
    storage.store_license_activation(&row).await?;
    Ok(activation)
}

/// Store an activation file and publish the new status to the running server,
/// so features unlock without a restart.
pub async fn apply_activation(
    storage: &Storage,
    status: &Arc<RwLock<LicenseStatus>>,
    blob: Vec<u8>,
) -> anyhow::Result<LicenseStatus> {
    store_activation(storage, blob).await?;
    refresh_status(storage, status).await
}

/// Re-read the stored activation into the shared status.
pub async fn refresh_status(
    storage: &Storage,
    status: &Arc<RwLock<LicenseStatus>>,
) -> anyhow::Result<LicenseStatus> {
    let fresh = resolve_license_status(storage).await;
    *status.write().unwrap_or_else(|e| e.into_inner()) = fresh.clone();
    Ok(fresh)
}

/// True when the stored activation was signed with a retired key. The activation
/// server re-signs it with the newest key (same machine, no usage spent), which is
/// what lets a retired key be dropped from the binary.
async fn needs_resigning(storage: &Storage) -> bool {
    match storage.get_license_activation().await {
        Ok(Some(row)) => activation::needs_resigning(&row.activation_blob),
        _ => false,
    }
}

/// True when a renewal should be asked for: a monthly key close to its end, or a stored
/// activation this build cannot verify. The latter is what an install holding a file
/// signed with a since-retired key sees after an update: the server answers a renewal
/// for the same machine with a file signed by the current key, without spending an
/// activation.
fn renewal_due(status: &LicenseStatus) -> bool {
    match status {
        LicenseStatus::Valid {
            expires_at: Some(_),
            days_remaining: Some(days),
            ..
        } => *days <= RENEW_WINDOW_DAYS,
        LicenseStatus::Expired { .. } | LicenseStatus::InvalidSignature { .. } => true,
        _ => false,
    }
}

/// True when a support contract is within [`SUPPORT_REFRESH_WINDOW_DAYS`] of its end
/// or past it, at `now` (Unix seconds). A new `support_until` only reaches the install
/// by re-reading the signed activation. Never affects the licence: this only decides
/// whether to ask.
fn support_refresh_due(status: &LicenseStatus, now: i64) -> bool {
    match status {
        LicenseStatus::Valid {
            support_until: Some(until),
            ..
        } => now >= *until - SUPPORT_REFRESH_WINDOW_DAYS * 86_400,
        _ => false,
    }
}

/// Renew from the activation server using the stored licence key.
pub async fn renew_now(
    storage: &Storage,
    status: &Arc<RwLock<LicenseStatus>>,
) -> anyhow::Result<LicenseStatus> {
    let row = storage
        .get_license_activation()
        .await?
        .context("No licence is activated")?;
    let blob = fetch_renewal(&row.license_key).await?;
    apply_activation(storage, status, blob).await
}

/// Background task: refresh monthly keys shortly before they lapse, re-issue
/// activations signed with a retiring key, replace an activation this build
/// cannot verify (signed with a retired key), and re-read the signed activation of a
/// key whose support contract is near its end or lapsed (so a renewal bought on the
/// console arrives without reactivating; at most once a day). It runs once at
/// start-up and then every six hours. Offline sites keep working through the 7 day
/// grace period and can import a new activation file by hand. A lapsed support
/// contract never limits a licence: a failed refresh changes nothing.
pub fn spawn_renewal_task(storage: Storage, status: Arc<RwLock<LicenseStatus>>) {
    tokio::spawn(async move {
        let mut last_support_refresh: Option<std::time::Instant> = None;
        loop {
            let current = status.read().unwrap_or_else(|e| e.into_inner()).clone();
            let support_due =
                support_refresh_due(&current, OffsetDateTime::now_utc().unix_timestamp())
                    && last_support_refresh
                        .is_none_or(|at| at.elapsed() >= SUPPORT_REFRESH_INTERVAL);
            let due = renewal_due(&current) || needs_resigning(&storage).await || support_due;
            if due {
                if support_due {
                    last_support_refresh = Some(std::time::Instant::now());
                }
                match renew_now(&storage, &status).await {
                    Ok(_) => tracing::info!("Licence renewed from the activation server"),
                    Err(e) => tracing::warn!("Licence renewal failed (will retry): {e:#}"),
                }
            }
            tokio::time::sleep(RENEW_CHECK_INTERVAL).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(expires_at: Option<&str>, days_remaining: Option<i64>) -> LicenseStatus {
        valid_with_support(expires_at, days_remaining, None)
    }

    fn valid_with_support(
        expires_at: Option<&str>,
        days_remaining: Option<i64>,
        support_until: Option<i64>,
    ) -> LicenseStatus {
        LicenseStatus::Valid {
            license_key: "VOLTRUS-TEST".to_string(),
            expires_at: expires_at.map(str::to_string),
            days_remaining,
            tier: Some("enterprise".to_string()),
            grace: false,
            legacy_white_label: false,
            site: None,
            support_until,
        }
    }

    #[test]
    fn unverifiable_activation_asks_the_server_for_a_fresh_one() {
        let status = LicenseStatus::InvalidSignature {
            message: "Invalid activation file. Please reactivate.".to_string(),
        };
        assert!(renewal_due(&status));
    }

    #[test]
    fn lapsed_and_nearly_lapsed_monthly_keys_renew() {
        let expired = LicenseStatus::Expired {
            expires_at: "2026-11-01T00:00:00Z".to_string(),
            message: "expired".to_string(),
        };
        assert!(renewal_due(&expired));
        assert!(renewal_due(&valid(
            Some("2026-11-01T00:00:00Z"),
            Some(RENEW_WINDOW_DAYS)
        )));
        assert!(!renewal_due(&valid(
            Some("2027-01-01T00:00:00Z"),
            Some(RENEW_WINDOW_DAYS + 1)
        )));
    }

    #[test]
    fn healthy_or_unrelated_states_do_not_call_the_server() {
        assert!(!renewal_due(&valid(None, None)));
        assert!(!renewal_due(&LicenseStatus::Community));
        assert!(!renewal_due(&LicenseStatus::Unactivated {
            message: "none".to_string(),
        }));
        assert!(!renewal_due(&LicenseStatus::InvalidFingerprint {
            score: 1,
            required: 3,
            message: "moved".to_string(),
        }));
    }

    #[test]
    fn support_contract_near_its_end_or_lapsed_is_re_read() {
        let day = 86_400;
        let until = 1_800_000_000;
        let near = |now| support_refresh_due(&valid_with_support(None, None, Some(until)), now);
        assert!(!near(until - (SUPPORT_REFRESH_WINDOW_DAYS + 1) * day));
        assert!(near(until - SUPPORT_REFRESH_WINDOW_DAYS * day));
        assert!(near(until - 1));
        assert!(near(until));
        assert!(near(until + 400 * day));
    }

    #[test]
    fn keys_without_a_support_contract_never_ask_for_support() {
        assert!(!support_refresh_due(&valid(None, None), i64::MAX));
        assert!(!support_refresh_due(&LicenseStatus::Community, i64::MAX));
        assert!(!support_refresh_due(
            &LicenseStatus::Unactivated {
                message: "none".to_string(),
            },
            i64::MAX
        ));
    }

    #[test]
    fn support_state_alone_never_triggers_a_licence_renewal() {
        // A lapsed contract on a lifetime key is not an expiry.
        assert!(!renewal_due(&valid_with_support(None, None, Some(1))));
    }

    fn fingerprint() -> Fingerprint {
        Fingerprint {
            hostname: "plant-1".into(),
            cpu: "cpu".into(),
            disk_serial: "disk".into(),
            mac: "mac".into(),
            machine_id: "machine".into(),
        }
    }

    fn activation_for(fp: &Fingerprint) -> LicenseActivation {
        LicenseActivation {
            version: 2,
            license_key: "VOLTRUS-TEST".into(),
            fingerprint_hash: fp.hash(),
            issued_at: "2026-10-01T00:00:00Z".into(),
            expires_at: None,
            tier: Some("integrator".into()),
            device_limit: None,
            site_id: Some("site-1".into()),
            site_name: Some("Plant 1".into()),
            support_until: Some(1_900_000_000),
            signature: String::new(),
        }
    }

    #[tokio::test]
    async fn site_and_support_are_persisted_with_the_activation() {
        let storage = Storage::new_in_memory().unwrap();
        let fp = fingerprint();
        store_verified(&storage, b"blob".to_vec(), activation_for(&fp), fp)
            .await
            .unwrap();
        let row = storage.get_license_activation().await.unwrap().unwrap();
        assert_eq!(row.site_id.as_deref(), Some("site-1"));
        assert_eq!(row.site_name.as_deref(), Some("Plant 1"));
        assert_eq!(row.support_until, Some(1_900_000_000));
    }

    #[tokio::test]
    async fn a_key_without_site_or_support_stores_nulls() {
        let storage = Storage::new_in_memory().unwrap();
        let fp = fingerprint();
        let mut activation = activation_for(&fp);
        activation.site_id = None;
        activation.site_name = None;
        activation.support_until = None;
        store_verified(&storage, b"blob".to_vec(), activation, fp)
            .await
            .unwrap();
        let row = storage.get_license_activation().await.unwrap().unwrap();
        assert_eq!(row.site_id, None);
        assert_eq!(row.site_name, None);
        assert_eq!(row.support_until, None);
    }

    #[tokio::test]
    async fn a_renewal_replaces_the_stored_support_end() {
        let storage = Storage::new_in_memory().unwrap();
        let fp = fingerprint();
        store_verified(
            &storage,
            b"first".to_vec(),
            activation_for(&fp),
            fingerprint(),
        )
        .await
        .unwrap();
        let mut renewed = activation_for(&fp);
        renewed.support_until = Some(2_000_000_000);
        store_verified(&storage, b"second".to_vec(), renewed, fp)
            .await
            .unwrap();
        let row = storage.get_license_activation().await.unwrap().unwrap();
        assert_eq!(row.support_until, Some(2_000_000_000));
        assert_eq!(row.activation_blob, b"second");
    }

    #[tokio::test]
    async fn an_activation_for_another_machine_is_refused() {
        let storage = Storage::new_in_memory().unwrap();
        let mut other = fingerprint();
        other.machine_id = "someone-else".into();
        let err = store_verified(
            &storage,
            b"blob".to_vec(),
            activation_for(&other),
            fingerprint(),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("different machine"));
        assert!(storage.get_license_activation().await.unwrap().is_none());
    }
}
