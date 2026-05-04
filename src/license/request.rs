#![allow(dead_code)]
use crate::license::fingerprint::collect_fingerprint;
use serde::{Deserialize, Serialize};

const REQUEST_MAGIC: &[u8] = b"VOLREQ\x01";

#[derive(Serialize, Deserialize, Debug)]
pub struct LicenseRequest {
    version: u8,
    license_key: String,
    fingerprint: FingerprintData,
    timestamp: String,
    nonce: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FingerprintData {
    hash: String,
    components: std::collections::HashMap<String, String>,
}

pub fn generate_request(license_key: &str) -> anyhow::Result<String> {
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
    };

    let json = serde_json::to_vec(&request)?;
    let mut file_data = Vec::new();
    file_data.extend_from_slice(REQUEST_MAGIC);
    file_data.extend_from_slice(&json);

    let filename = "license-request.bin".to_string();
    std::fs::write(&filename, &file_data)?;

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
        let path = generate_request("test-key-123").unwrap();
        let data = std::fs::read(&path).unwrap();
        let request = parse_request(&data).unwrap();
        assert_eq!(request.version, 1);
        assert_eq!(request.license_key, "test-key-123");
        assert!(request.fingerprint.hash.starts_with("sha256:"));
        assert!(!request.nonce.is_empty());
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_parse_invalid_magic() {
        let result = parse_request(b"INVALID");
        assert!(result.is_err());
    }
}
