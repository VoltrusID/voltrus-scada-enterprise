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
        daily_budget_secs: i64,
        daily_used_secs: i64,
        polling_active: bool,
        tier: LicenseTier,
        feature_flags: serde_json::Value,
    },
    EngineeringPollingStopped {
        started_at: i64,
        daily_budget_secs: i64,
        daily_used_secs: i64,
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
            LicenseStatus::Valid { .. }
                | LicenseStatus::Engineering { .. }
                | LicenseStatus::EngineeringPollingStopped { .. }
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
