use super::*;
use super::tier::LicenseTier;

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
