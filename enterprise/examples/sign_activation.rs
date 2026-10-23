//! Test-only tool: sign a licence activation file the way the activation worker does.
//!
//! Private repository only. It reads a private key from a file the caller names
//! (the real key lives outside git, in the secrets directory) and binds the file
//! to this machine's fingerprint unless `--fingerprint` says otherwise. The
//! official binary has no way to trust any other key than its embedded ones, so
//! activations made here only verify when signed with a real trusted key.
//!
//! cargo run --example sign_activation -- \
//!     --key /path/to/license-signing-key.txt --tier enterprise \
//!     [--license-key K] [--expires 2027-01-01T00:00:00Z] [--fingerprint sha256:..] \
//!     [--issued-at RFC3339] --out activation.bin
//!
//! The key file holds either a bare 64 hex character seed or a `PRIVATE_KEY=<hex>` line.

use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;
use voltrus_enterprise::licensing::fingerprint::collect_fingerprint;

const MAGIC: &[u8] = b"VOLACT\x01";

/// Field order is the signed byte layout: it must match the verifier's payload struct.
#[derive(Serialize)]
struct Payload {
    version: u8,
    license_key: String,
    fingerprint_hash: String,
    issued_at: String,
    expires_at: Option<String>,
    tier: Option<String>,
    device_limit: Option<u32>,
}

#[derive(Serialize)]
struct Activation {
    #[serde(flatten)]
    payload: Payload,
    signature: String,
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn load_seed(path: &str) -> [u8; 32] {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("read {path}: {e}")));
    let hex_seed = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.strip_prefix("PRIVATE_KEY=").unwrap_or(l))
        .next()
        .unwrap_or_else(|| die("key file has no key"));
    hex::decode(hex_seed)
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b).ok())
        .unwrap_or_else(|| die("key must be 64 hex characters"))
}

fn die(message: &str) -> ! {
    eprintln!("sign_activation: {message}");
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let key_path = arg(&args, "--key").unwrap_or_else(|| die("--key FILE is required"));
    let out = arg(&args, "--out").unwrap_or_else(|| die("--out FILE is required"));
    let tier = arg(&args, "--tier").unwrap_or_else(|| die("--tier is required"));
    let issued_at = arg(&args, "--issued-at").unwrap_or_else(|| {
        time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap()
    });

    let signing = SigningKey::from_bytes(&load_seed(&key_path));
    let payload = Payload {
        version: 2,
        license_key: arg(&args, "--license-key").unwrap_or_else(|| "VOLTRUS-TEST-KEY".to_string()),
        fingerprint_hash: arg(&args, "--fingerprint")
            .unwrap_or_else(|| collect_fingerprint().hash()),
        issued_at,
        expires_at: arg(&args, "--expires"),
        tier: Some(tier),
        device_limit: None,
    };
    let signature = signing.sign(&serde_json::to_vec(&payload).unwrap());
    let activation = Activation {
        payload,
        signature: hex::encode(signature.to_bytes()),
    };

    let mut blob = MAGIC.to_vec();
    blob.extend_from_slice(&serde_json::to_vec(&activation).unwrap());
    std::fs::write(&out, blob).unwrap_or_else(|e| die(&format!("write {out}: {e}")));
    eprintln!(
        "wrote {out} (public key {})",
        hex::encode(signing.verifying_key().to_bytes())
    );
}
