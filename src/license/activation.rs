use anyhow::Context;
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use serde::{Deserialize, Serialize};

/// Embedded Ed25519 public key for verifying license activation signatures.
const EMBEDDED_PUBLIC_KEY: &[u8] = &[
    0xc8, 0x94, 0xd0, 0x18, 0xb5, 0x47, 0xc2, 0x22,
    0x5d, 0x75, 0x06, 0x01, 0x96, 0x9b, 0x10, 0x25,
    0x7b, 0xf9, 0x64, 0x75, 0x07, 0xb6, 0xb7, 0xb7,
    0xa9, 0x0d, 0xe9, 0xd4, 0x2f, 0x92, 0xe5, 0xca,
];

const ACTIVATION_MAGIC: &[u8] = b"VOLACT\x01";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LicenseActivation {
    pub version: u8,
    pub license_key: String,
    pub fingerprint_hash: String,
    pub issued_at: String,
    pub expires_at: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActivationPayload {
    version: u8,
    license_key: String,
    fingerprint_hash: String,
    issued_at: String,
    expires_at: String,
}

pub fn verify_activation_signature(data: &[u8]) -> anyhow::Result<LicenseActivation> {
    if !data.starts_with(ACTIVATION_MAGIC) {
        anyhow::bail!("Invalid activation file: bad magic header");
    }
    let json = &data[ACTIVATION_MAGIC.len()..];
    let activation: LicenseActivation = serde_json::from_slice(json)?;

    let verifying_key = VerifyingKey::from_bytes(
        EMBEDDED_PUBLIC_KEY.try_into().map_err(|_| anyhow::anyhow!("Invalid public key length"))?
    )?;

    let payload = ActivationPayload {
        version: activation.version,
        license_key: activation.license_key.clone(),
        fingerprint_hash: activation.fingerprint_hash.clone(),
        issued_at: activation.issued_at.clone(),
        expires_at: activation.expires_at.clone(),
    };
    let payload_json = serde_json::to_vec(&payload)?;

    let sig_bytes = hex::decode(&activation.signature)?;
    let signature = Signature::from_slice(&sig_bytes)?;

    verifying_key.verify(&payload_json, &signature)
        .map_err(|_| anyhow::anyhow!("Invalid activation signature"))?;

    let expires = time::OffsetDateTime::parse(
        &activation.expires_at,
        &time::format_description::well_known::Rfc3339
    ).map_err(|e| anyhow::anyhow!("Invalid expiry date: {}", e))?;
    if time::OffsetDateTime::now_utc() > expires {
        anyhow::bail!("License expired on {}", activation.expires_at);
    }

    Ok(activation)
}

pub fn import_activation_file(path: &str) -> anyhow::Result<LicenseActivation> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read activation file: {}", path))?;
    verify_activation_signature(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_invalid_magic() {
        let result = verify_activation_signature(b"INVALID");
        assert!(result.is_err());
    }
}
