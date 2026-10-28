#![allow(dead_code)]
use super::fingerprint::collect_fingerprint;
use serde::{Deserialize, Serialize};

const REQUEST_MAGIC: &[u8] = b"VOLREQ\x01";

/// Longest site name the issuer accepts.
pub const MAX_SITE_NAME_CHARS: usize = 120;

/// Capability this build advertises in every request. The issuer signs the per-site
/// fields (`site_id`, `site_name`, `support_until`) into an activation only for a
/// client that lists it, because a build without them cannot verify such a file.
const FEATURE_SITE_SUPPORT: &str = "site_support";

#[derive(Serialize, Deserialize, Debug)]
pub struct LicenseRequest {
    version: u8,
    license_key: String,
    fingerprint: FingerprintData,
    timestamp: String,
    nonce: String,
    /// Display name for the site a first activation is for (`--site`). The issuer
    /// only uses it when the key has no site yet; renaming later happens on the console.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    site_name: Option<String>,
    /// What this build can verify, see [`FEATURE_SITE_SUPPORT`].
    #[serde(default)]
    features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FingerprintData {
    hash: String,
    components: std::collections::HashMap<String, String>,
}

/// Trim a site name and reject one the issuer would refuse. Empty means none.
pub fn normalize_site_name(raw: &str) -> anyhow::Result<Option<String>> {
    let name = raw.trim();
    if name.is_empty() {
        return Ok(None);
    }
    if name.chars().count() > MAX_SITE_NAME_CHARS {
        anyhow::bail!("Site name is longer than {MAX_SITE_NAME_CHARS} characters");
    }
    if name.chars().any(char::is_control) {
        anyhow::bail!("Site name contains control characters");
    }
    Ok(Some(name.to_string()))
}

/// Build the signed-request payload (magic header plus JSON) for a licence key and,
/// optionally, the name of the site it is activated for.
pub fn build_request(license_key: &str, site_name: Option<&str>) -> anyhow::Result<Vec<u8>> {
    let site_name = match site_name {
        Some(raw) => normalize_site_name(raw)?,
        None => None,
    };
    let fingerprint = collect_fingerprint();
    let hash = fingerprint.hash();
    let components = fingerprint.to_components_map();

    let nonce: String = (0..32)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect();

    let timestamp = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());

    let request = LicenseRequest {
        version: 1,
        license_key: license_key.to_string(),
        fingerprint: FingerprintData { hash, components },
        timestamp,
        nonce,
        site_name,
        features: vec![FEATURE_SITE_SUPPORT.to_string()],
    };

    let json = serde_json::to_vec(&request)?;
    let mut file_data = Vec::new();
    file_data.extend_from_slice(REQUEST_MAGIC);
    file_data.extend_from_slice(&json);

    Ok(file_data)
}

/// Write `license-request.bin` in the current directory and return its path.
pub fn generate_request(license_key: &str, site_name: Option<&str>) -> anyhow::Result<String> {
    let filename = "license-request.bin".to_string();
    std::fs::write(&filename, build_request(license_key, site_name)?)?;
    Ok(filename)
}

pub fn parse_request(data: &[u8]) -> anyhow::Result<LicenseRequest> {
    if !data.starts_with(REQUEST_MAGIC) {
        anyhow::bail!("Invalid request file: bad magic header");
    }
    let json = &data[REQUEST_MAGIC.len()..];
    let request: LicenseRequest = serde_json::from_slice(json)?;
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_parse_request() {
        let data = build_request("test-key-123", None).unwrap();
        let request = parse_request(&data).unwrap();
        assert_eq!(request.version, 1);
        assert_eq!(request.license_key, "test-key-123");
        assert!(request.fingerprint.hash.starts_with("sha256:"));
        assert!(!request.nonce.is_empty());
        assert_eq!(request.site_name, None);
        assert_eq!(request.features, vec!["site_support".to_string()]);
    }

    #[test]
    fn request_carries_the_trimmed_site_name() {
        let data = build_request("test-key-123", Some("  Plant 1, Line A  ")).unwrap();
        let request = parse_request(&data).unwrap();
        assert_eq!(request.site_name.as_deref(), Some("Plant 1, Line A"));
        // The wire format is the plain JSON the worker reads.
        let json: serde_json::Value = serde_json::from_slice(&data[REQUEST_MAGIC.len()..]).unwrap();
        assert_eq!(json["site_name"], "Plant 1, Line A");
        assert_eq!(json["features"][0], "site_support");
    }

    #[test]
    fn request_without_a_site_omits_the_field() {
        let data = build_request("test-key-123", Some("   ")).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&data[REQUEST_MAGIC.len()..]).unwrap();
        assert!(json.get("site_name").is_none());
    }

    #[test]
    fn site_names_the_issuer_would_refuse_are_rejected_up_front() {
        assert!(build_request("k", Some(&"x".repeat(MAX_SITE_NAME_CHARS + 1))).is_err());
        assert!(build_request("k", Some(&"x".repeat(MAX_SITE_NAME_CHARS))).is_ok());
        assert!(build_request("k", Some("bad\nname")).is_err());
    }

    #[test]
    fn request_files_from_older_builds_still_parse() {
        let mut data = REQUEST_MAGIC.to_vec();
        data.extend_from_slice(
            br#"{"version":1,"license_key":"k","fingerprint":{"hash":"sha256:x","components":{}},"timestamp":"t","nonce":"n"}"#,
        );
        let request = parse_request(&data).unwrap();
        assert_eq!(request.site_name, None);
        assert!(request.features.is_empty());
    }

    #[test]
    fn test_parse_invalid_magic() {
        let result = parse_request(b"INVALID");
        assert!(result.is_err());
    }
}
