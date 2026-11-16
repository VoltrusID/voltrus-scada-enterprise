//! 21 CFR Part 11: electronic signatures, record integrity and password policy.

pub mod handlers;

pub use voltrus::compliance::{ComplianceConfig, ElectronicSignature};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateSignatureRequest {
    pub resource_type: String,
    pub resource_id: String,
    pub reason: String,
    pub meaning: String,
    /// The signer's own password, checked again at signing (21 CFR 11.200(a)):
    /// a signature is never made from the session alone.
    #[serde(default)]
    pub password: String,
}

/// Failed signature re-authentications that lock the account, like failed
/// sign-ins do in the core (15 minutes).
pub const SIGN_LOCK_AFTER_FAILURES: i64 = 10;
pub const SIGN_LOCK_SECS: i64 = 900;

#[derive(Debug, Deserialize)]
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

/// Recompute a stored signature's hash. Rows written before the handler
/// stored its own timestamp may carry a `created_at` up to two seconds after
/// the one that was hashed, so those are accepted too.
pub fn signature_matches(sig: &ElectronicSignature) -> bool {
    (0..=2).any(|skew| {
        let expected = generate_signature_hash(
            &sig.user_id,
            &sig.action,
            &sig.resource_type,
            &sig.resource_id,
            &sig.reason,
            &sig.meaning,
            sig.created_at - skew,
        );
        expected == sig.signature_hash
    })
}

fn csv_field(s: &str) -> String {
    let s = s.replace(['\r', '\n'], " ");
    // A leading formula character would run as a formula in a spreadsheet.
    let s = if s.starts_with(['=', '+', '-', '@']) {
        format!("'{s}")
    } else {
        s
    };
    if s.contains([',', '"']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

/// The signatures as CSV, with the time in UTC and whether each hash verifies.
pub fn signatures_csv(signatures: &[ElectronicSignature]) -> String {
    let mut out = String::from(
        "id,signed_at_utc,signer,meaning,reason,action,resource_type,resource_id,signature_hash,hash_valid\n",
    );
    for s in signatures {
        let when = time::OffsetDateTime::from_unix_timestamp(s.created_at)
            .ok()
            .and_then(|t| {
                t.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            })
            .unwrap_or_else(|| s.created_at.to_string());
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            s.id,
            when,
            csv_field(&s.user_id),
            csv_field(&s.meaning),
            csv_field(&s.reason),
            csv_field(&s.action),
            csv_field(&s.resource_type),
            csv_field(&s.resource_id),
            s.signature_hash,
            signature_matches(s),
        ));
    }
    out
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

    fn signed(created_at: i64, hashed_at: i64) -> ElectronicSignature {
        ElectronicSignature {
            id: 1,
            user_id: "op1".into(),
            action: "sign".into(),
            resource_type: "batch".into(),
            resource_id: "B-17".into(),
            reason: "Release, after QA".into(),
            meaning: "Approved".into(),
            signature_hash: generate_signature_hash(
                "op1",
                "sign",
                "batch",
                "B-17",
                "Release, after QA",
                "Approved",
                hashed_at,
            ),
            created_at,
        }
    }

    #[test]
    fn signature_hash_verifies_and_detects_tampering() {
        let sig = signed(1000, 1000);
        assert!(signature_matches(&sig));
        // A legacy row stored a second after it was hashed.
        assert!(signature_matches(&signed(1001, 1000)));
        let mut tampered = sig.clone();
        tampered.meaning = "Reviewed".into();
        assert!(!signature_matches(&tampered));
        let mut moved = sig;
        moved.created_at = 5000;
        assert!(!signature_matches(&moved));
    }

    #[test]
    fn signatures_csv_quotes_and_flags_validity() {
        let mut bad = signed(1000, 1000);
        bad.id = 2;
        bad.user_id = "=cmd".into();
        let csv = signatures_csv(&[signed(1000, 1000), bad]);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("id,signed_at_utc,signer,meaning,reason"));
        assert!(lines[1].starts_with("1,1970-01-01T00:16:40Z,op1,Approved,\"Release, after QA\""));
        assert!(lines[1].ends_with(",true"));
        assert!(lines[2].contains(",'=cmd,"));
        assert!(lines[2].ends_with(",false"));
    }

    #[test]
    fn sign_request_takes_the_password() {
        let req: CreateSignatureRequest = serde_json::from_str(
            r#"{"resource_type":"batch","resource_id":"1","reason":"r","meaning":"Approved","password":"pw"}"#,
        )
        .unwrap();
        assert_eq!(req.password, "pw");
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
