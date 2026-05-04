pub mod activation;
pub mod fingerprint;
pub mod request;

use crate::storage::Storage;
use serde::Serialize;

// ── License Tier ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LicenseTier {
    Trial,
    Starter,
    Professional,
    Enterprise,
}

impl LicenseTier {
    /// Maps an activation tier string to a LicenseTier enum.
    /// Returns Trial when the input is None or unrecognized.
    pub fn from_tier_str(s: Option<&str>) -> Self {
        match s.map(|t| t.to_lowercase()) {
            Some(ref t) if t == "starter" => LicenseTier::Starter,
            Some(ref t) if t == "professional" || t == "pro" => LicenseTier::Professional,
            Some(ref t) if t == "enterprise" => LicenseTier::Enterprise,
            _ => LicenseTier::Trial,
        }
    }

    /// Maximum number of devices allowed. None means unlimited.
    pub fn max_devices(&self) -> Option<u32> {
        match self {
            LicenseTier::Trial => Some(5),
            LicenseTier::Starter => Some(50),
            LicenseTier::Professional => None,
            LicenseTier::Enterprise => None,
        }
    }

    /// Number of days telemetry data is retained.
    pub fn data_retention_days(&self) -> u32 {
        match self {
            LicenseTier::Trial => 7,
            LicenseTier::Starter | LicenseTier::Professional | LicenseTier::Enterprise => 365,
        }
    }

    pub fn opcua_enabled(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn email_notifications(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn webhook_notifications(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn push_notifications(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn sms_notifications(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn rest_api_export(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn csv_export(&self) -> bool {
        true
    }

    pub fn analytics(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn oee(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn pdf_reporting(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn backup_restore(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn auto_discovery(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn white_label(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    /// Full theme editor (colors, fonts, layouts).
    /// Starter gets basic (logo swap), Professional+ gets full theme control.
    pub fn theme_full(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    /// Per-user screen layouts (each user gets their own layout).
    /// Professional tier and above.
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

    /// SSO with SAML/OIDC enhanced features and LDAP fallback.
    /// Only available on Enterprise tier.
    pub fn sso_saml(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    pub fn cfr_part_11(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// EtherNet/IP (Allen-Bradley) protocol support.
    /// Only available on Enterprise tier.
    pub fn ethernet_ip(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// Siemens S7 protocol support.
    /// Only available on Enterprise tier.
    pub fn s7_protocol(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// DNP3 (IEEE 1815) protocol support — Enterprise only.
    pub fn dnp3(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// RTSP video integration.
    /// Only available on Enterprise tier.
    pub fn rtsp_video(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// Returns all feature flags as a JSON object.
    pub fn to_feature_flags(&self) -> serde_json::Value {
        serde_json::json!({
            "max_devices": self.max_devices(),
            "data_retention_days": self.data_retention_days(),
            "opcua_enabled": self.opcua_enabled(),
            "email_notifications": self.email_notifications(),
            "webhook_notifications": self.webhook_notifications(),
            "push_notifications": self.push_notifications(),
            "sms_notifications": self.sms_notifications(),
            "rest_api_export": self.rest_api_export(),
            "csv_export": self.csv_export(),
            "analytics": self.analytics(),
            "oee": self.oee(),
            "pdf_reporting": self.pdf_reporting(),
            "backup_restore": self.backup_restore(),
            "auto_discovery": self.auto_discovery(),
            "white_label": self.white_label(),
            "theme_full": self.theme_full(),
            "per_user_layouts": self.per_user_layouts(),
            "rbac": self.rbac(),
            "multi_site": self.multi_site(),
            "recipes": self.recipes(),
            "sso": self.sso(),
            "sso_saml": self.sso_saml(),
            "cfr_part_11": self.cfr_part_11(),
            "ethernet_ip": self.ethernet_ip(),
            "s7_protocol": self.s7_protocol(),
            "dnp3": self.dnp3(),
            "rtsp_video": self.rtsp_video(),
        })
    }
}

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
    Trial {
        started_at: i64,
        expires_at: i64,
        days_remaining: i64,
        tier: LicenseTier,
        feature_flags: serde_json::Value,
    },
    TrialExpired {
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
        matches!(self, LicenseStatus::Valid { .. } | LicenseStatus::Trial { .. })
    }

    /// Extracts the LicenseTier from Valid or Trial status.
    /// Defaults to Trial for all other statuses.
    pub fn tier(&self) -> LicenseTier {
        match self {
            LicenseStatus::Valid { tier, .. } => {
                LicenseTier::from_tier_str(tier.as_deref())
            }
            LicenseStatus::Trial { tier, .. } => tier.clone(),
            _ => LicenseTier::Trial,
        }
    }

    /// Returns feature flags for Valid or Trial status, empty object otherwise.
    pub fn feature_flags(&self) -> serde_json::Value {
        match self {
            LicenseStatus::Valid { tier, .. } => {
                LicenseTier::from_tier_str(tier.as_deref()).to_feature_flags()
            }
            LicenseStatus::Trial { feature_flags, .. } => feature_flags.clone(),
            _ => serde_json::json!({}),
        }
    }
}

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

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_tier_max_devices() {
        assert_eq!(LicenseTier::Trial.max_devices(), Some(5));
        assert_eq!(LicenseTier::Starter.max_devices(), Some(50));
        assert_eq!(LicenseTier::Professional.max_devices(), None);
        assert_eq!(LicenseTier::Enterprise.max_devices(), None);
    }

    #[test]
    fn test_license_tier_feature_flags() {
        // Trial tier: limited features
        let trial = LicenseTier::Trial;
        assert_eq!(trial.max_devices(), Some(5));
        assert_eq!(trial.data_retention_days(), 7);
        assert!(!trial.opcua_enabled());
        assert!(!trial.email_notifications());
        assert!(!trial.webhook_notifications());
        assert!(!trial.push_notifications());
        assert!(!trial.sms_notifications());
        assert!(!trial.rest_api_export());
        assert!(trial.csv_export()); // always true
        assert!(!trial.analytics());
        assert!(!trial.oee());
        assert!(!trial.pdf_reporting());
        assert!(!trial.backup_restore());
        assert!(!trial.white_label());
        assert!(!trial.rbac());
        assert!(!trial.multi_site());
        assert!(!trial.recipes());
        assert!(!trial.sso());
        assert!(!trial.sso_saml());
        assert!(!trial.cfr_part_11());
        assert!(!trial.ethernet_ip());
        assert!(!trial.s7_protocol());

        // Starter tier: many features enabled
        let starter = LicenseTier::Starter;
        assert_eq!(starter.max_devices(), Some(50));
        assert_eq!(starter.data_retention_days(), 365);
        assert!(!starter.opcua_enabled());
        assert!(starter.email_notifications());
        assert!(starter.webhook_notifications());
        assert!(starter.push_notifications());
        assert!(!starter.sms_notifications());
        assert!(starter.rest_api_export());
        assert!(starter.csv_export());
        assert!(starter.analytics());
        assert!(starter.oee());
        assert!(!starter.pdf_reporting());
        assert!(starter.backup_restore());
        assert!(starter.white_label());
        assert!(!starter.rbac());
        assert!(!starter.multi_site());
        assert!(!starter.recipes());
        assert!(!starter.sso());
        assert!(!starter.sso_saml());
        assert!(!starter.cfr_part_11());

        // Professional tier: most features enabled
        let pro = LicenseTier::Professional;
        assert_eq!(pro.max_devices(), None);
        assert_eq!(pro.data_retention_days(), 365);
        assert!(pro.opcua_enabled());
        assert!(pro.email_notifications());
        assert!(pro.webhook_notifications());
        assert!(pro.push_notifications());
        assert!(pro.sms_notifications());
        assert!(pro.rest_api_export());
        assert!(pro.csv_export());
        assert!(pro.analytics());
        assert!(pro.oee());
        assert!(pro.pdf_reporting());
        assert!(pro.backup_restore());
        assert!(pro.white_label());
        assert!(pro.rbac());
        assert!(pro.multi_site());
        assert!(pro.recipes());
        assert!(!pro.sso());
        assert!(!pro.sso_saml());
        assert!(!pro.cfr_part_11());

        // Enterprise tier: all features enabled
        let ent = LicenseTier::Enterprise;
        assert_eq!(ent.max_devices(), None);
        assert_eq!(ent.data_retention_days(), 365);
        assert!(ent.opcua_enabled());
        assert!(ent.email_notifications());
        assert!(ent.webhook_notifications());
        assert!(ent.push_notifications());
        assert!(ent.sms_notifications());
        assert!(ent.rest_api_export());
        assert!(ent.csv_export());
        assert!(ent.analytics());
        assert!(ent.oee());
        assert!(ent.pdf_reporting());
        assert!(ent.backup_restore());
        assert!(ent.white_label());
        assert!(ent.rbac());
        assert!(ent.multi_site());
        assert!(ent.recipes());
        assert!(ent.sso());
        assert!(ent.sso_saml());
        assert!(ent.cfr_part_11());
        assert!(ent.ethernet_ip());
        assert!(ent.s7_protocol());
        assert!(ent.rtsp_video());
    }

    #[test]
    fn test_trial_status_is_valid() {
        let tier = LicenseTier::Trial;
        let status = LicenseStatus::Trial {
            started_at: 1000,
            expires_at: 1000 + 30 * 86400,
            days_remaining: 30,
            tier: tier.clone(),
            feature_flags: tier.to_feature_flags(),
        };
        assert!(status.is_valid());
    }

    #[test]
    fn test_trial_expired_is_not_valid() {
        let status = LicenseStatus::TrialExpired {
            started_at: 1000,
            message: "Trial expired".to_string(),
        };
        assert!(!status.is_valid());
    }

    #[test]
    fn test_from_tier_str() {
        assert_eq!(LicenseTier::from_tier_str(None), LicenseTier::Trial);
        assert_eq!(LicenseTier::from_tier_str(Some("")), LicenseTier::Trial);
        assert_eq!(LicenseTier::from_tier_str(Some("unknown")), LicenseTier::Trial);
        assert_eq!(LicenseTier::from_tier_str(Some("starter")), LicenseTier::Starter);
        assert_eq!(LicenseTier::from_tier_str(Some("Starter")), LicenseTier::Starter);
        assert_eq!(LicenseTier::from_tier_str(Some("STARTER")), LicenseTier::Starter);
        assert_eq!(LicenseTier::from_tier_str(Some("professional")), LicenseTier::Professional);
        assert_eq!(LicenseTier::from_tier_str(Some("pro")), LicenseTier::Professional);
        assert_eq!(LicenseTier::from_tier_str(Some("Professional")), LicenseTier::Professional);
        assert_eq!(LicenseTier::from_tier_str(Some("enterprise")), LicenseTier::Enterprise);
        assert_eq!(LicenseTier::from_tier_str(Some("Enterprise")), LicenseTier::Enterprise);
    }

    #[test]
    fn test_to_feature_flags_json_structure() {
        let flags = LicenseTier::Trial.to_feature_flags();
        assert_eq!(flags["max_devices"], 5);
        assert_eq!(flags["data_retention_days"], 7);
        assert_eq!(flags["csv_export"], true);
        assert_eq!(flags["opcua_enabled"], false);

        let flags = LicenseTier::Starter.to_feature_flags();
        assert_eq!(flags["max_devices"], 50);
        assert_eq!(flags["email_notifications"], true);

        let flags = LicenseTier::Enterprise.to_feature_flags();
        assert_eq!(flags["max_devices"], serde_json::Value::Null);
        assert_eq!(flags["sso"], true);
        assert_eq!(flags["cfr_part_11"], true);
        assert_eq!(flags["ethernet_ip"], true);
    }

    #[test]
    fn test_ethernet_ip_enterprise_only() {
        assert!(!LicenseTier::Trial.ethernet_ip());
        assert!(!LicenseTier::Starter.ethernet_ip());
        assert!(!LicenseTier::Professional.ethernet_ip());
        assert!(LicenseTier::Enterprise.ethernet_ip());
    }

    #[test]
    fn test_s7_protocol_enterprise_only() {
        assert!(!LicenseTier::Trial.s7_protocol());
        assert!(!LicenseTier::Starter.s7_protocol());
        assert!(!LicenseTier::Professional.s7_protocol());
        assert!(LicenseTier::Enterprise.s7_protocol());
    }

    #[test]
    fn test_rtsp_video_enterprise_only() {
        assert!(!LicenseTier::Trial.rtsp_video());
        assert!(!LicenseTier::Starter.rtsp_video());
        assert!(!LicenseTier::Professional.rtsp_video());
        assert!(LicenseTier::Enterprise.rtsp_video());
    }

    #[test]
    fn test_theme_full_professional_and_above() {
        assert!(!LicenseTier::Trial.theme_full());
        assert!(!LicenseTier::Starter.theme_full());
        assert!(LicenseTier::Professional.theme_full());
        assert!(LicenseTier::Enterprise.theme_full());
    }

    #[test]
    fn test_per_user_layouts_professional_and_above() {
        assert!(!LicenseTier::Trial.per_user_layouts());
        assert!(!LicenseTier::Starter.per_user_layouts());
        assert!(LicenseTier::Professional.per_user_layouts());
        assert!(LicenseTier::Enterprise.per_user_layouts());
    }

    #[test]
    fn test_white_label_starter_and_above() {
        assert!(!LicenseTier::Trial.white_label());
        assert!(LicenseTier::Starter.white_label());
        assert!(LicenseTier::Professional.white_label());
        assert!(LicenseTier::Enterprise.white_label());
    }

    #[test]
    fn test_trial_status_tier_returns_trial() {
        let tier = LicenseTier::Trial;
        let status = LicenseStatus::Trial {
            started_at: 1000,
            expires_at: 1000 + 30 * 86400,
            days_remaining: 30,
            tier: tier.clone(),
            feature_flags: tier.to_feature_flags(),
        };
        assert_eq!(status.tier(), LicenseTier::Trial);
    }

    #[test]
    fn test_trial_expired_status_tier_returns_trial() {
        let status = LicenseStatus::TrialExpired {
            started_at: 1000,
            message: "Trial expired".to_string(),
        };
        assert_eq!(status.tier(), LicenseTier::Trial);
    }

    #[test]
    fn test_valid_status_tier_maps_correctly() {
        let valid = LicenseStatus::Valid {
            license_key: "key".to_string(),
            expires_at: None,
            days_remaining: None,
            tier: Some("starter".to_string()),
            device_limit: None,
        };
        assert_eq!(valid.tier(), LicenseTier::Starter);
    }

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

        // Valid status without tier defaults to Trial
        let valid_no_tier = LicenseStatus::Valid {
            license_key: "key".to_string(),
            expires_at: None,
            days_remaining: None,
            tier: None,
            device_limit: None,
        };
        assert_eq!(valid_no_tier.tier(), LicenseTier::Trial);

        // Trial status
        let trial = LicenseStatus::Trial {
            started_at: 1000,
            expires_at: 1000 + 30 * 86400,
            days_remaining: 30,
            tier: LicenseTier::Trial,
            feature_flags: LicenseTier::Trial.to_feature_flags(),
        };
        assert_eq!(trial.tier(), LicenseTier::Trial);
        assert_eq!(trial.feature_flags()["csv_export"], true);

        // Other statuses default to Trial
        let unactivated = LicenseStatus::Unactivated {
            message: "msg".to_string(),
        };
        assert_eq!(unactivated.tier(), LicenseTier::Trial);
        assert_eq!(unactivated.feature_flags(), serde_json::json!({}));
    }

    #[test]
    fn test_sso_saml_enterprise_only() {
        assert!(!LicenseTier::Trial.sso_saml());
        assert!(!LicenseTier::Starter.sso_saml());
        assert!(!LicenseTier::Professional.sso_saml());
        assert!(LicenseTier::Enterprise.sso_saml());
    }

    #[test]
    fn test_sso_saml_in_feature_flags() {
        let flags = LicenseTier::Enterprise.to_feature_flags();
        assert_eq!(flags["sso_saml"], true);

        let flags = LicenseTier::Trial.to_feature_flags();
        assert_eq!(flags["sso_saml"], false);

        let flags = LicenseTier::Professional.to_feature_flags();
        assert_eq!(flags["sso_saml"], false);
    }
}
