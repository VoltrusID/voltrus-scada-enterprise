use anyhow::Context;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Ed25519 public keys trusted to sign activations, newest first.
///
/// The key at index 0 signs every new activation (the private half is the
/// `ED25519_PRIVATE_KEY` secret of the activation worker). During a rotation the
/// retiring key stays listed after it so activation files issued before the rotation
/// keep verifying; drop it once no live binding is signed with it (see
/// docs/official-build.md).
///
/// The original key (v0.17.5 to v0.47.0) was retired in v0.47.1. Its private half
/// was committed once to the former private repository history, so it is treated as
/// compromised and must never be listed again.
const TRUSTED_PUBLIC_KEYS: &[[u8; 32]] = &[
    // 2026-10 rotation.
    [
        0x6b, 0xa3, 0x3a, 0x35, 0xe4, 0xb5, 0x16, 0x24, 0x40, 0xdb, 0x72, 0x50, 0x1e, 0x8c, 0xac,
        0xb3, 0x4c, 0x8d, 0xb6, 0x10, 0x30, 0xb3, 0xbf, 0xcc, 0xf1, 0xeb, 0x7a, 0xee, 0xca, 0xe8,
        0x28, 0x9b,
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
    /// Per-site licence: issuer-side id and display name of the licensed site.
    /// Absent on keys issued before per-site licensing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    /// End of the annual support contract, Unix seconds. Absent = no contract
    /// (lifetime or grandfathered keys).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_until: Option<i64>,
    pub signature: String,
}

/// The exact bytes the issuer signs: this struct serialised by `serde_json`, fields
/// in declaration order. The three per-site fields are omitted when absent, so an
/// activation issued without them re-serialises to the bytes signed before they
/// existed and keeps verifying. The activation worker builds the same JSON
/// (`generateActivation` in worker/license-activate/src/index.ts); keep both in
/// step and in this order.
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    site_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    site_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    support_until: Option<i64>,
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
fn verify_with_keys(data: &[u8], keys: &[[u8; 32]]) -> anyhow::Result<(LicenseActivation, usize)> {
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
        site_id: activation.site_id.clone(),
        site_name: activation.site_name.clone(),
        support_until: activation.support_until,
    };
    let payload_json = serde_json::to_vec(&payload)?;

    let sig_bytes = hex::decode(&activation.signature)?;
    let signature = Signature::from_slice(&sig_bytes)?;

    let signer = keys.iter().position(|key| {
        VerifyingKey::from_bytes(key).is_ok_and(|key| key.verify(&payload_json, &signature).is_ok())
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
        signed_for_site(signing, tier, None, None)
    }

    /// An activation as the worker issues it for a per-site key.
    fn signed_for_site(
        signing: &SigningKey,
        tier: &str,
        site: Option<(&str, &str)>,
        support_until: Option<i64>,
    ) -> Vec<u8> {
        let payload = ActivationPayload {
            version: 2,
            license_key: "VOLTRUS-TEST".into(),
            fingerprint_hash: "sha256:x".into(),
            issued_at: "2026-01-01T00:00:00Z".into(),
            expires_at: None,
            tier: Some(tier.into()),
            device_limit: None,
            site_id: site.map(|(id, _)| id.into()),
            site_name: site.map(|(_, name)| name.into()),
            support_until,
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
            site_id: payload.site_id,
            site_name: payload.site_name,
            support_until: payload.support_until,
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
        let (activation, index) =
            verify_with_keys(&signed(&old_sk, "professional"), &keys).unwrap();
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

    /// Activations bound to a fake fingerprint and lapsed in January 2020, signed with
    /// the real production keys: the first by the 2026-10 key (what the worker signs
    /// today), the second by the retired original key. They test the shipped key set,
    /// not only the verification code. Neither unlocks anything: the fingerprint is not
    /// a machine and the expiry is years past the grace period.
    const FIXTURE_SIGNED_BY_2026_10_KEY: &str = "Vk9MQUNUAXsidmVyc2lvbiI6MiwibGljZW5zZV9rZXkiOiJWT0xUUlVTLUZJWFRVUkUiLCJmaW5nZXJwcmludF9oYXNoIjoic2hhMjU2OmZpeHR1cmUiLCJpc3N1ZWRfYXQiOiIyMDE5LTEyLTAxVDAwOjAwOjAwLjAwMFoiLCJleHBpcmVzX2F0IjoiMjAyMC0wMS0wMVQwMDowMDowMC4wMDBaIiwidGllciI6ImludGVncmF0b3IiLCJkZXZpY2VfbGltaXQiOm51bGwsInNpZ25hdHVyZSI6IjlhNmRjMGVlMjk4YjQ2N2Q2YWY2YzRmNWIyODJiNjhjOWFlMGI5Y2RlMTg0ZDIwOGQ4MWNjMjNlOWRiYjQ0NjBmZjgwN2QwMjA3NjBhYjEyY2YyYThkNWIxZDhkYjQzZjAwNWU2Y2RmNDk1NzhhMTE1YmIyYjQ2NzcxZjVlMjA1In0=";
    const FIXTURE_SIGNED_BY_RETIRED_KEY: &str = "Vk9MQUNUAXsidmVyc2lvbiI6MiwibGljZW5zZV9rZXkiOiJWT0xUUlVTLUZJWFRVUkUiLCJmaW5nZXJwcmludF9oYXNoIjoic2hhMjU2OmZpeHR1cmUiLCJpc3N1ZWRfYXQiOiIyMDE5LTEyLTAxVDAwOjAwOjAwLjAwMFoiLCJleHBpcmVzX2F0IjoiMjAyMC0wMS0wMVQwMDowMDowMC4wMDBaIiwidGllciI6ImludGVncmF0b3IiLCJkZXZpY2VfbGltaXQiOm51bGwsInNpZ25hdHVyZSI6ImI4NzEzNmI1ZmQ5ZTAyZWM1ZjRmMzdkOTU5MWUzZDE4OTBiZDA5OTg0OTBjN2U1YjUzYjEwYTZhOTFmNGVkYTY4MjBjNGQ5Y2Y0YzdkMTlhZmQ2ODRhMmEwNjRkZDYxNjE1ODhhOWNmYzU1NDM3ZDNhMzJjZjI3NWQ0M2M0ZjBjIn0=";

    fn fixture(b64: &str) -> Vec<u8> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap()
    }

    #[test]
    fn production_key_set_is_the_2026_10_key_only() {
        assert_eq!(TRUSTED_PUBLIC_KEYS.len(), 1);
        assert!(VerifyingKey::from_bytes(&TRUSTED_PUBLIC_KEYS[0]).is_ok());
        let retired: [u8; 32] =
            hex::decode("c894d018b547c2225d750601969b10257bf9647507b6b7b7a90de9d42f92e5ca")
                .unwrap()
                .try_into()
                .unwrap();
        assert!(!TRUSTED_PUBLIC_KEYS.contains(&retired));
    }

    #[test]
    fn production_keys_accept_the_2026_10_signature_and_reject_the_retired_one() {
        let blob = fixture(FIXTURE_SIGNED_BY_2026_10_KEY);
        let (activation, index) = verify_with_keys(&blob, TRUSTED_PUBLIC_KEYS).unwrap();
        assert_eq!(index, 0);
        assert_eq!(activation.tier.as_deref(), Some("integrator"));
        assert!(!needs_resigning(&blob));

        let retired = fixture(FIXTURE_SIGNED_BY_RETIRED_KEY);
        let err = verify_activation_signature(&retired).unwrap_err();
        assert_eq!(err.to_string(), "Invalid activation signature");
        assert!(!needs_resigning(&retired));
        // Random bytes are not signed by the production key either.
        assert!(!needs_resigning(b"VOLACT\x01{}"));
    }

    #[test]
    fn per_site_activation_verifies_and_carries_its_fields() {
        let (sk, pk) = keypair(1);
        let blob = signed_for_site(
            &sk,
            "integrator",
            Some(("site-1", "Plant 1")),
            Some(1_798_761_600),
        );
        let (activation, index) = verify_with_keys(&blob, &[pk]).unwrap();
        assert_eq!(index, 0);
        assert_eq!(activation.site_id.as_deref(), Some("site-1"));
        assert_eq!(activation.site_name.as_deref(), Some("Plant 1"));
        assert_eq!(activation.support_until, Some(1_798_761_600));
    }

    #[test]
    fn per_site_activation_verifies_under_a_retired_key_too() {
        let (new_sk, new_pk) = keypair(1);
        let (old_sk, old_pk) = keypair(2);
        let keys = [new_pk, old_pk];
        let blob = signed_for_site(&old_sk, "enterprise", Some(("site-1", "Plant 1")), Some(10));
        assert_eq!(verify_with_keys(&blob, &keys).unwrap().1, 1);
        let blob = signed_for_site(&new_sk, "enterprise", None, Some(10));
        assert_eq!(verify_with_keys(&blob, &keys).unwrap().1, 0);
    }

    #[test]
    fn the_support_end_and_site_are_covered_by_the_signature() {
        let (sk, pk) = keypair(1);
        let blob = signed_for_site(
            &sk,
            "integrator",
            Some(("site-1", "Plant 1")),
            Some(1_798_761_600),
        );
        let text = String::from_utf8(blob[ACTIVATION_MAGIC.len()..].to_vec()).unwrap();
        for (from, to) in [
            ("1798761600", "1900000000"),
            ("\"site-1\"", "\"site-2\""),
            ("Plant 1", "Plant 2"),
        ] {
            assert!(text.contains(from), "{from} in {text}");
            let mut forged = ACTIVATION_MAGIC.to_vec();
            forged.extend_from_slice(text.replace(from, to).as_bytes());
            assert!(verify_with_keys(&forged, &[pk]).is_err(), "{from} -> {to}");
        }
        // Dropping the fields from a signed per-site activation is a forgery too.
        let mut stripped = ACTIVATION_MAGIC.to_vec();
        let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
        for field in ["site_id", "site_name", "support_until"] {
            value.as_object_mut().unwrap().remove(field);
        }
        stripped.extend_from_slice(value.to_string().as_bytes());
        assert!(verify_with_keys(&stripped, &[pk]).is_err());
    }

    #[test]
    fn adding_support_to_an_old_activation_is_a_forgery() {
        let (sk, pk) = keypair(1);
        let blob = signed(&sk, "integrator");
        let mut value: serde_json::Value =
            serde_json::from_slice(&blob[ACTIVATION_MAGIC.len()..]).unwrap();
        value["support_until"] = serde_json::json!(4_000_000_000_i64);
        let mut forged = ACTIVATION_MAGIC.to_vec();
        forged.extend_from_slice(value.to_string().as_bytes());
        assert!(verify_with_keys(&forged, &[pk]).is_err());
    }

    #[test]
    fn a_payload_without_the_per_site_fields_serialises_to_the_pre_per_site_bytes() {
        // The exact JSON signed before site_id / site_name / support_until existed.
        let payload = ActivationPayload {
            version: 2,
            license_key: "VOLTRUS-TEST".into(),
            fingerprint_hash: "sha256:x".into(),
            issued_at: "2026-01-01T00:00:00Z".into(),
            expires_at: None,
            tier: Some("integrator".into()),
            device_limit: None,
            site_id: None,
            site_name: None,
            support_until: None,
        };
        assert_eq!(
            String::from_utf8(serde_json::to_vec(&payload).unwrap()).unwrap(),
            r#"{"version":2,"license_key":"VOLTRUS-TEST","fingerprint_hash":"sha256:x","issued_at":"2026-01-01T00:00:00Z","expires_at":null,"tier":"integrator","device_limit":null}"#
        );
    }

    #[test]
    fn a_payload_with_the_per_site_fields_appends_them_in_the_documented_order() {
        let payload = ActivationPayload {
            version: 2,
            license_key: "VOLTRUS-TEST".into(),
            fingerprint_hash: "sha256:x".into(),
            issued_at: "2026-01-01T00:00:00Z".into(),
            expires_at: None,
            tier: Some("integrator".into()),
            device_limit: None,
            site_id: Some("site-1".into()),
            site_name: Some("Plant \"1\"".into()),
            support_until: Some(1_798_761_600),
        };
        assert_eq!(
            String::from_utf8(serde_json::to_vec(&payload).unwrap()).unwrap(),
            r#"{"version":2,"license_key":"VOLTRUS-TEST","fingerprint_hash":"sha256:x","issued_at":"2026-01-01T00:00:00Z","expires_at":null,"tier":"integrator","device_limit":null,"site_id":"site-1","site_name":"Plant \"1\"","support_until":1798761600}"#
        );
    }

    #[test]
    fn support_only_and_site_only_activations_verify() {
        let (sk, pk) = keypair(1);
        // A lifetime key with a support contract but no site yet, and the reverse.
        let support_only = signed_for_site(&sk, "integrator", None, Some(1_798_761_600));
        let (a, _) = verify_with_keys(&support_only, &[pk]).unwrap();
        assert_eq!((a.site_id, a.support_until), (None, Some(1_798_761_600)));
        let site_only = signed_for_site(&sk, "integrator", Some(("site-1", "Plant 1")), None);
        let (a, _) = verify_with_keys(&site_only, &[pk]).unwrap();
        assert_eq!(
            (a.site_id.as_deref(), a.support_until),
            (Some("site-1"), None)
        );
    }

    /// Activations issued by the real worker code (`generateActivation`, test key = seed 0x11 x 32,
    /// clock fixed at 2026-10-28T00:00:00Z) for a fixed request. The worker's own suite
    /// (worker/license-activate/test/site-support.test.ts) asserts the same strings, so a change to
    /// either side's serialisation fails one of the two suites. The first has no per-site fields: the
    /// format every activation had before per-site licensing. The second carries a site (with a quote and
    /// a non-ASCII character in its name, the cases where two JSON writers could differ) and a
    /// support end.
    const VECTOR_WORKER_OLD_FORMAT: &str = "Vk9MQUNUAXsidmVyc2lvbiI6MiwibGljZW5zZV9rZXkiOiJWT0xUUlVTLVZFQ1RPUiIsImZpbmdlcnByaW50X2hhc2giOiJzaGEyNTY6dmVjdG9yIiwiaXNzdWVkX2F0IjoiMjAyNi0xMC0yOFQwMDowMDowMC4wMDBaIiwiZXhwaXJlc19hdCI6bnVsbCwidGllciI6ImludGVncmF0b3IiLCJkZXZpY2VfbGltaXQiOm51bGwsInNpZ25hdHVyZSI6IjA3ZjUwMWNlYzk2MjU1ZTM5ZmZlOThmZWE1Y2ViOTNhZDczNGI0OWVlZTVmNDZhMGY3ZWM2Mjc3NWNlMDJlN2ZkODIwNWIwMzYxYmU1NjA2NTg2ZTExMDU3MTI0ZTdjNDE2MzdjYjQyMTNlOTBmZmIxMjZiMGY0YmIyNzZlNDA2In0=";
    const VECTOR_WORKER_PER_SITE: &str = "Vk9MQUNUAXsidmVyc2lvbiI6MiwibGljZW5zZV9rZXkiOiJWT0xUUlVTLVZFQ1RPUiIsImZpbmdlcnByaW50X2hhc2giOiJzaGEyNTY6dmVjdG9yIiwiaXNzdWVkX2F0IjoiMjAyNi0xMC0yOFQwMDowMDowMC4wMDBaIiwiZXhwaXJlc19hdCI6bnVsbCwidGllciI6ImludGVncmF0b3IiLCJkZXZpY2VfbGltaXQiOm51bGwsInNpdGVfaWQiOiIwYjBlOGE1Yy0xZjNhLTRmNmUtOWQwYS01ZDFkOGY0YzdhMTEiLCJzaXRlX25hbWUiOiJQbGFudCBcIjFcIiDDqSIsInN1cHBvcnRfdW50aWwiOjE3OTg3NjE2MDAsInNpZ25hdHVyZSI6IjExZmQ2MDA3ZGExYzk3ZDQ0YWVmZjRlZjcxNDAwN2I5ZjdmZDI3ZmUyMzQzYmMwNTMzNzk1YTM3NGM3ZTFjMDg4OGM3ZTAxMjA2YTZlNGIzZjhlZmJkMTJmZGUxMGQ5OWU4NzBlNjNhMWQyMDViNDUxYjk5YTZmZTBmNDY5MTA4In0=";

    #[test]
    fn worker_issued_activation_without_per_site_fields_still_verifies() {
        let (_, pk) = keypair(0x11);
        let (activation, index) =
            verify_with_keys(&fixture(VECTOR_WORKER_OLD_FORMAT), &[pk]).unwrap();
        assert_eq!(index, 0);
        assert_eq!(activation.license_key, "VOLTRUS-VECTOR");
        assert_eq!(activation.tier.as_deref(), Some("integrator"));
        assert_eq!(activation.site_id, None);
        assert_eq!(activation.site_name, None);
        assert_eq!(activation.support_until, None);
    }

    #[test]
    fn worker_issued_per_site_activation_verifies_with_its_fields() {
        let (_, pk) = keypair(0x11);
        let (activation, index) =
            verify_with_keys(&fixture(VECTOR_WORKER_PER_SITE), &[pk]).unwrap();
        assert_eq!(index, 0);
        assert_eq!(
            activation.site_id.as_deref(),
            Some("0b0e8a5c-1f3a-4f6e-9d0a-5d1d8f4c7a11")
        );
        assert_eq!(activation.site_name.as_deref(), Some("Plant \"1\" \u{e9}"));
        assert_eq!(activation.support_until, Some(1_798_761_600));
    }

    #[test]
    fn worker_vectors_do_not_verify_under_another_key() {
        let (_, other) = keypair(3);
        assert!(verify_with_keys(&fixture(VECTOR_WORKER_OLD_FORMAT), &[other]).is_err());
        assert!(verify_with_keys(&fixture(VECTOR_WORKER_PER_SITE), &[other]).is_err());
    }

    #[test]
    fn a_build_without_the_per_site_fields_cannot_verify_a_per_site_file() {
        // Why the worker only signs the fields for a client that lists "site_support": a build that
        // predates them re-serialises the payload without them and rejects the signature. Simulated
        // here by dropping the fields the way such a build's struct does.
        #[derive(Serialize, Deserialize)]
        struct OldPayload {
            version: u8,
            license_key: String,
            fingerprint_hash: String,
            issued_at: String,
            expires_at: Option<String>,
            tier: Option<String>,
            device_limit: Option<u32>,
        }
        let (_, pk) = keypair(0x11);
        let blob = fixture(VECTOR_WORKER_PER_SITE);
        let old: OldPayload = serde_json::from_slice(&blob[ACTIVATION_MAGIC.len()..]).unwrap();
        let signature: String = serde_json::from_slice::<serde_json::Value>(
            &blob[ACTIVATION_MAGIC.len()..],
        )
        .unwrap()["signature"]
            .as_str()
            .unwrap()
            .to_string();
        let sig = Signature::from_slice(&hex::decode(signature).unwrap()).unwrap();
        let verifies = VerifyingKey::from_bytes(&pk)
            .unwrap()
            .verify(&serde_json::to_vec(&old).unwrap(), &sig)
            .is_ok();
        assert!(!verifies);
        // And the same old struct verifies the old-format vector, so only the new fields matter.
        let old_blob = fixture(VECTOR_WORKER_OLD_FORMAT);
        let old: OldPayload = serde_json::from_slice(&old_blob[ACTIVATION_MAGIC.len()..]).unwrap();
        let signature: String =
            serde_json::from_slice::<serde_json::Value>(&old_blob[ACTIVATION_MAGIC.len()..])
                .unwrap()["signature"]
                .as_str()
                .unwrap()
                .to_string();
        let sig = Signature::from_slice(&hex::decode(signature).unwrap()).unwrap();
        assert!(
            VerifyingKey::from_bytes(&pk)
                .unwrap()
                .verify(&serde_json::to_vec(&old).unwrap(), &sig)
                .is_ok()
        );
    }
}
