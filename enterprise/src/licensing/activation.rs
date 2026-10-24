use anyhow::Context;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Ed25519 public keys trusted to sign activations, newest first.
///
/// The key at index 0 signs every new activation (the private half is the
/// `ED25519_PRIVATE_KEY` secret of the activation worker). The older keys stay
/// trusted only so activation files issued before a rotation keep verifying; drop
/// one once no binding signed with it remains (see docs/official-build.md).
const TRUSTED_PUBLIC_KEYS: &[[u8; 32]] = &[
    // 2026-10 rotation.
    [
        0x6b, 0xa3, 0x3a, 0x35, 0xe4, 0xb5, 0x16, 0x24, 0x40, 0xdb, 0x72, 0x50, 0x1e, 0x8c, 0xac,
        0xb3, 0x4c, 0x8d, 0xb6, 0x10, 0x30, 0xb3, 0xbf, 0xcc, 0xf1, 0xeb, 0x7a, 0xee, 0xca, 0xe8,
        0x28, 0x9b,
    ],
    // Original key (v0.17.5 to v0.47.0). Its private half was committed once to the
    // former private repository history (removed before this repository went public),
    // so it is treated as compromised: retire it as soon as the old-signed
    // activations have been re-issued.
    [
        0xc8, 0x94, 0xd0, 0x18, 0xb5, 0x47, 0xc2, 0x22, 0x5d, 0x75, 0x06, 0x01, 0x96, 0x9b, 0x10,
        0x25, 0x7b, 0xf9, 0x64, 0x75, 0x07, 0xb6, 0xb7, 0xb7, 0xa9, 0x0d, 0xe9, 0xd4, 0x2f, 0x92,
        0xe5, 0xca,
    ],
];

const ACTIVATION_MAGIC: &[u8] = b"VOLACT\x01";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LicenseActivation {
    pub version: u8,
    pub license_key: String,
    pub fingerprint_hash: String,
    pub issued_at: String,
    pub expires_at: Option<String>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub device_limit: Option<u32>,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActivationPayload {
    version: u8,
    license_key: String,
    fingerprint_hash: String,
    issued_at: String,
    expires_at: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    device_limit: Option<u32>,
}

/// Check the magic header and the Ed25519 signature. Expiry is judged separately
/// (see `verification`) because a lapsed monthly key has a grace period.
pub fn verify_activation_signature(data: &[u8]) -> anyhow::Result<LicenseActivation> {
    verify_with_keys(data, TRUSTED_PUBLIC_KEYS).map(|(activation, _)| activation)
}

/// True when a stored activation verifies under a retired key only, so it should
/// be re-issued by the activation server (which signs with the newest key).
pub fn needs_resigning(data: &[u8]) -> bool {
    matches!(verify_with_keys(data, TRUSTED_PUBLIC_KEYS), Ok((_, index)) if index > 0)
}

/// Verify against an explicit key set, trying each key in order; returns the
/// activation and the index of the key that signed it. Production code always
/// passes [`TRUSTED_PUBLIC_KEYS`]; tests pass their own.
fn verify_with_keys(
    data: &[u8],
    keys: &[[u8; 32]],
) -> anyhow::Result<(LicenseActivation, usize)> {
    if !data.starts_with(ACTIVATION_MAGIC) {
        anyhow::bail!("Invalid activation file: bad magic header");
    }
    let json = &data[ACTIVATION_MAGIC.len()..];
    let activation: LicenseActivation = serde_json::from_slice(json)?;

    let payload = ActivationPayload {
        version: activation.version,
        license_key: activation.license_key.clone(),
        fingerprint_hash: activation.fingerprint_hash.clone(),
        issued_at: activation.issued_at.clone(),
        expires_at: activation.expires_at.clone(),
        tier: activation.tier.clone(),
        device_limit: activation.device_limit,
    };
    let payload_json = serde_json::to_vec(&payload)?;

    let sig_bytes = hex::decode(&activation.signature)?;
    let signature = Signature::from_slice(&sig_bytes)?;

    let signer = keys.iter().position(|key| {
        VerifyingKey::from_bytes(key)
            .is_ok_and(|key| key.verify(&payload_json, &signature).is_ok())
    });
    match signer {
        Some(index) => Ok((activation, index)),
        None => anyhow::bail!("Invalid activation signature"),
    }
}

pub fn import_activation_file(path: &str) -> anyhow::Result<LicenseActivation> {
    let data =
        std::fs::read(path).with_context(|| format!("Failed to read activation file: {}", path))?;
    verify_activation_signature(&data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn keypair(seed: u8) -> (SigningKey, [u8; 32]) {
        let signing = SigningKey::from_bytes(&[seed; 32]);
        let public = signing.verifying_key().to_bytes();
        (signing, public)
    }

    fn signed(signing: &SigningKey, tier: &str) -> Vec<u8> {
        let payload = ActivationPayload {
            version: 2,
            license_key: "VOLTRUS-TEST".into(),
            fingerprint_hash: "sha256:x".into(),
            issued_at: "2026-01-01T00:00:00Z".into(),
            expires_at: None,
            tier: Some(tier.into()),
            device_limit: None,
        };
        let signature = signing.sign(&serde_json::to_vec(&payload).unwrap());
        let activation = LicenseActivation {
            version: payload.version,
            license_key: payload.license_key,
            fingerprint_hash: payload.fingerprint_hash,
            issued_at: payload.issued_at,
            expires_at: payload.expires_at,
            tier: payload.tier,
            device_limit: payload.device_limit,
            signature: hex::encode(signature.to_bytes()),
        };
        let mut blob = ACTIVATION_MAGIC.to_vec();
        blob.extend_from_slice(&serde_json::to_vec(&activation).unwrap());
        blob
    }

    #[test]
    fn test_verify_invalid_magic() {
        let result = verify_activation_signature(b"INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_rejects_unsigned_payload() {
        let mut blob = ACTIVATION_MAGIC.to_vec();
        blob.extend_from_slice(
            br#"{"version":1,"license_key":"k","fingerprint_hash":"h","issued_at":"2026-01-01T00:00:00Z","expires_at":null,"signature":"00"}"#,
        );
        assert!(verify_activation_signature(&blob).is_err());
    }

    #[test]
    fn dual_key_accepts_new_and_old_signatures() {
        let (new_sk, new_pk) = keypair(1);
        let (old_sk, old_pk) = keypair(2);
        let keys = [new_pk, old_pk];

        let (_, index) = verify_with_keys(&signed(&new_sk, "enterprise"), &keys).unwrap();
        assert_eq!(index, 0);
        let (activation, index) = verify_with_keys(&signed(&old_sk, "professional"), &keys).unwrap();
        assert_eq!(index, 1);
        assert_eq!(activation.tier.as_deref(), Some("professional"));
    }

    #[test]
    fn retired_or_unknown_key_is_rejected() {
        let (new_sk, new_pk) = keypair(1);
        let (old_sk, _) = keypair(2);
        let (_, other_pk) = keypair(3);
        // Old key dropped from the trusted set: its activations no longer verify.
        assert!(verify_with_keys(&signed(&old_sk, "enterprise"), &[new_pk]).is_err());
        // A key that is not in the set never verifies.
        assert!(verify_with_keys(&signed(&new_sk, "enterprise"), &[other_pk]).is_err());
    }

    #[test]
    fn tampered_payload_is_rejected_under_every_key() {
        let (new_sk, new_pk) = keypair(1);
        let (_, old_pk) = keypair(2);
        let blob = signed(&new_sk, "integrator");
        let tampered = String::from_utf8(blob[ACTIVATION_MAGIC.len()..].to_vec())
            .unwrap()
            .replace("integrator", "enterprise");
        let mut forged = ACTIVATION_MAGIC.to_vec();
        forged.extend_from_slice(tampered.as_bytes());
        assert!(verify_with_keys(&forged, &[new_pk, old_pk]).is_err());
    }

    #[test]
    fn production_key_set_has_the_new_key_first_and_valid_keys() {
        assert_eq!(TRUSTED_PUBLIC_KEYS.len(), 2);
        for key in TRUSTED_PUBLIC_KEYS {
            assert!(VerifyingKey::from_bytes(key).is_ok());
        }
        assert_ne!(TRUSTED_PUBLIC_KEYS[0], TRUSTED_PUBLIC_KEYS[1]);
        // Random bytes are not signed by either production key.
        assert!(!needs_resigning(b"VOLACT\x01{}"));
    }
}
