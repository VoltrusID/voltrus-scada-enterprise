use serde::{Deserialize, Serialize};

// ── Electronic Signature (FDA 21 CFR Part 11) ──

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ElectronicSignature {
    pub id: i64,
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub reason: String,
    pub meaning: String,
    pub signature_hash: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSignatureRequest {
    pub resource_type: String,
    pub resource_id: String,
    pub reason: String,
    pub meaning: String,
}

// ── Compliance Configuration ──

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComplianceConfig {
    pub auto_logout_minutes: u32,
    pub password_min_length: u8,
    pub password_require_uppercase: bool,
    pub password_require_number: bool,
    pub password_require_special: bool,
    pub password_rotation_days: u32,
    pub authority_to_change: bool,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            auto_logout_minutes: 15,
            password_min_length: 8,
            password_require_uppercase: true,
            password_require_number: true,
            password_require_special: true,
            password_rotation_days: 90,
            authority_to_change: false,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateComplianceConfigRequest {
    pub auto_logout_minutes: Option<u32>,
    pub password_min_length: Option<u8>,
    pub password_require_uppercase: Option<bool>,
    pub password_require_number: Option<bool>,
    pub password_require_special: Option<bool>,
    pub password_rotation_days: Option<u32>,
    pub authority_to_change: Option<bool>,
}

/// Verify password meets complexity requirements per compliance config.
pub fn verify_password_complexity(password: &str, config: &ComplianceConfig) -> Result<(), String> {
    if password.len() < config.password_min_length as usize {
        return Err(format!(
            "Password must be at least {} characters",
            config.password_min_length
        ));
    }

    if config.password_require_uppercase && !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if config.password_require_number && !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Password must contain at least one number".to_string());
    }

    if config.password_require_special
        && !password
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
    {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}

/// Compute a SHA-256 hash of a record for integrity verification.
pub fn compute_record_checksum(record: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(record.as_bytes());
    hex::encode(hasher.finalize())
}

/// Check if a record's checksum is valid.
pub fn verify_record_integrity(record: &str, checksum: &str) -> bool {
    let computed = compute_record_checksum(record);
    // Constant-time comparison to prevent timing attacks
    if computed.len() != checksum.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in computed.bytes().zip(checksum.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Generate a signature hash for an electronic signature event.
pub fn generate_signature_hash(
    user_id: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    reason: &str,
    meaning: &str,
    timestamp: i64,
) -> String {
    let payload = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        user_id, action, resource_type, resource_id, reason, meaning, timestamp
    );
    compute_record_checksum(&payload)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_compliance_config() {
        let config = ComplianceConfig::default();
        assert_eq!(config.auto_logout_minutes, 15);
        assert_eq!(config.password_min_length, 8);
        assert!(config.password_require_uppercase);
        assert!(config.password_require_number);
        assert!(config.password_require_special);
        assert_eq!(config.password_rotation_days, 90);
        assert!(!config.authority_to_change);
    }

    #[test]
    fn test_password_complexity_default_config() {
        let config = ComplianceConfig::default();
        // Too short
        assert!(verify_password_complexity("Ab1!", &config).is_err());
        // Missing uppercase
        assert!(verify_password_complexity("abcdef1!", &config).is_err());
        // Missing number
        assert!(verify_password_complexity("Abcdefg!", &config).is_err());
        // Missing special
        assert!(verify_password_complexity("Abcdefg1", &config).is_err());
        // Valid
        assert!(verify_password_complexity("Abcdefg1!", &config).is_ok());
    }

    #[test]
    fn test_password_complexity_relaxed_config() {
        let config = ComplianceConfig {
            password_min_length: 6,
            password_require_uppercase: false,
            password_require_number: false,
            password_require_special: false,
            ..Default::default()
        };
        assert!(verify_password_complexity("abc123", &config).is_ok());
    }

    #[test]
    fn test_compute_and_verify_record_integrity() {
        let record = r#"{"id": 1, "name": "test"}"#;
        let checksum = compute_record_checksum(record);
        assert!(verify_record_integrity(record, &checksum));
        // Tampered record
        assert!(!verify_record_integrity(
            r#"{"id": 1, "name": "tampered"}"#,
            &checksum
        ));
    }

    #[test]
    fn test_verify_record_integrity_wrong_length() {
        let record = "some data";
        assert!(!verify_record_integrity(record, "short"));
    }

    #[test]
    fn test_generate_signature_hash_deterministic() {
        let hash1 = generate_signature_hash(
            "user1", "create", "recipe", "42", "approved", "Created", 1000,
        );
        let hash2 = generate_signature_hash(
            "user1", "create", "recipe", "42", "approved", "Created", 1000,
        );
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_generate_signature_hash_unique_for_different_inputs() {
        let hash1 = generate_signature_hash(
            "user1", "create", "recipe", "42", "approved", "Created", 1000,
        );
        let hash2 = generate_signature_hash(
            "user2", "create", "recipe", "42", "approved", "Created", 1000,
        );
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compliance_config_serde_roundtrip() {
        let config = ComplianceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: ComplianceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.auto_logout_minutes, config.auto_logout_minutes);
        assert_eq!(back.password_min_length, config.password_min_length);
    }
}
