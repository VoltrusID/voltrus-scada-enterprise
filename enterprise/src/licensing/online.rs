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
use super::fingerprint::collect_fingerprint;
use super::request::build_request;
use super::verification::{evaluate_activation, resolve_license_status};

const DEFAULT_ACTIVATION_BASE: &str = "https://activate.voltrus.id";

/// Renewal is attempted when the key expires within this many days (or already has).
const RENEW_WINDOW_DAYS: i64 = 10;

/// How often the background task re-checks whether a renewal is due.
const RENEW_CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

fn endpoint(path: &str) -> String {
    let base = std::env::var("VOLTRUS_ACTIVATION_URL")
        .unwrap_or_else(|_| DEFAULT_ACTIVATION_BASE.to_string());
    format!("{}/{}", base.trim_end_matches('/'), path)
}

/// Ask the activation server to bind a key to this machine and return the signed
/// activation file.
pub async fn fetch_activation(license_key: &str) -> anyhow::Result<Vec<u8>> {
    post_request("activate", license_key).await
}

/// Ask the activation server for a fresh activation (new expiry) for a key that is
/// already bound to this machine. Never binds and never uses up an activation.
pub async fn fetch_renewal(license_key: &str) -> anyhow::Result<Vec<u8>> {
    post_request("renew", license_key).await
}

async fn post_request(path: &str, license_key: &str) -> anyhow::Result<Vec<u8>> {
    let request_data = build_request(license_key)?;
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
    let fingerprint = collect_fingerprint();
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

/// True when the stored key is a monthly key that is due for renewal.
fn renewal_due(status: &LicenseStatus) -> bool {
    match status {
        LicenseStatus::Valid {
            expires_at: Some(_),
            days_remaining: Some(days),
            ..
        } => *days <= RENEW_WINDOW_DAYS,
        LicenseStatus::Expired { .. } => true,
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

/// Background task: refresh monthly keys shortly before they lapse. Offline
/// sites keep working through the 7 day grace period and can import a new
/// activation file by hand.
pub fn spawn_renewal_task(storage: Storage, status: Arc<RwLock<LicenseStatus>>) {
    tokio::spawn(async move {
        loop {
            let due = renewal_due(&status.read().unwrap_or_else(|e| e.into_inner()).clone());
            if due {
                match renew_now(&storage, &status).await {
                    Ok(_) => tracing::info!("Licence renewed from the activation server"),
                    Err(e) => tracing::warn!("Licence renewal failed (will retry): {e:#}"),
                }
            }
            tokio::time::sleep(RENEW_CHECK_INTERVAL).await;
        }
    });
}
