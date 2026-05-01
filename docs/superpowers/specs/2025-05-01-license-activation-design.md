# License Activation System Design

**Date:** 2025-05-01  
**Project:** Voltrus Lite  
**Status:** Approved

---

## 1. Goals

- Enforce **1 license = 1 server** for Voltrus Lite deployments.
- Support **fully air-gapped installations** via an offline file-exchange activation flow.
- Allow **graceful server moves/upgrades** without requiring manual support intervention.
- Integrate with **Polar.sh** for license key generation, expiry, and revocation.
- Keep infrastructure minimal: a single **Cloudflare Worker**.

---

## 2. Non-Goals

- This system does not attempt to prevent a determined attacker from extracting and sharing a valid license file across multiple air-gapped servers. The threat model assumes B2B contractual enforcement is the primary deterrent, and the technical measures create a meaningful speed bump.
- No real-time "phone home" heartbeat or usage telemetry.
- No floating/concurrent license pooling.

---

## 3. Architecture Overview

```
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│  Air-Gapped     │  (USB)  │  Online PC      │  HTTPS  │  Cloudflare     │
│  Voltrus Server │ ──────► │  (Browser)      │ ──────► │  Worker         │
│                 │         │                 │         │                 │
│  1. Generate    │         │  3. Upload      │         │  4. Validate    │
│     request     │         │     request     │         │     key + fp    │
│     (req.bin)   │         │     to worker   │         │     with Polar  │
│                 │         │                 │         │                 │
│  6. Import      │  (USB)  │  5. Download    │         │                 │
│     activation  │ ◄────── │     activation  │         │                 │
│     (act.bin)   │         │     (act.bin)   │         │                 │
└─────────────────┘         └─────────────────┘         └─────────────────┘
```

### Flow

1. Admin installs Voltrus on an air-gapped server and runs `./voltrus-lite --generate-request <license-key>`.
2. The app computes a hardware fingerprint, bundles it with the license key, and writes `license-request.bin`.
3. Admin copies `license-request.bin` to a laptop with internet access.
4. The admin uploads the file to `https://activate.voltrus.io` (Cloudflare Worker).
5. The Worker validates the Polar.sh license key, checks if it is already bound to a different fingerprint, and signs an activation token with Ed25519.
6. The Worker returns `license-activation.bin`.
7. The admin copies it back to the air-gapped server.
8. The app runs `./voltrus-lite --import-activation license-activation.bin` and stores it in SQLite.
9. On every startup, the app verifies the Ed25519 signature and ensures the local hardware fingerprint matches.

---

## 4. Hardware Fingerprint

To bind a license to a server while allowing for hardware changes, we use a composite fingerprint of 5 identifiers with fuzzy matching.

### Components

| Component | Source | Weight |
|-----------|--------|--------|
| Hostname | `sysinfo` / `uname -n` | 1 |
| CPU ID / Model | `sysinfo` processor name | 1 |
| Root Disk Serial | Platform-specific (`diskutil` on macOS, `/sys/block/*/serial` on Linux) | 1 |
| Main MAC Address | First non-loopback MAC address | 1 |
| Machine ID | `/etc/machine-id` (Linux), `IOPlatformUUID` (macOS) | 1 |

### Matching Rule

Require **at least 3 out of 5** components to match.

**Scenarios:**

| Scenario | Match Count | Result |
|----------|-------------|--------|
| Reboot | 5/5 | ✅ Valid |
| Disk swap | 4/5 | ✅ Valid |
| Full VM migration (new MAC) | 3/5 | ✅ Valid |
| Copied to entirely new server | 0–2/5 | ❌ Requires reactivation |

### Storage

The fingerprint components are stored in plaintext inside the request file. Only the **SHA-256 hash** of the concatenated, sorted components is included in the signed activation blob.

---

## 5. File Formats

All files are JSON, minified, gzip-compressed, and prefixed with a 7-byte magic header.

### Request File (`license-request.bin`)

**Magic Header:** `VOLREQ\x01`

```json
{
  "version": 1,
  "license_key": "polar_sh_key_abc123",
  "fingerprint": {
    "hash": "sha256:deadbeef...",
    "components": {
      "hostname": "plant-server-01",
      "cpu": "Intel Xeon E5-2680",
      "disk_serial": "S123456789",
      "mac": "aa:bb:cc:dd:ee:ff",
      "machine_id": "a1b2c3d4..."
    }
  },
  "timestamp": "2025-05-01T12:00:00Z",
  "nonce": "random-128-bit-hex"
}
```

### Activation File (`license-activation.bin`)

**Magic Header:** `VOLACT\x01`

```json
{
  "version": 1,
  "license_key": "polar_sh_key_abc123",
  "fingerprint_hash": "sha256:deadbeef...",
  "issued_at": "2025-05-01T12:05:00Z",
  "expires_at": "2026-05-01T12:05:00Z",
  "signature": "ed25519:..."
}
```

The `signature` field is an Ed25519 detached signature over the canonical JSON of the payload (excluding the `signature` field itself), using the Worker's private key.

---

## 6. Cloudflare Worker

### Endpoints

#### `POST /activate`

Accepts a `multipart/form-data` upload containing the `license-request.bin` file.

**Success (200):**

```json
{
  "success": true,
  "activation_file": "<base64-encoded activation blob>"
}
```

**Error (400/403):**

```json
{
  "success": false,
  "error": "License already bound to a different device. Contact support to transfer."
}
```

### Logic

1. Parse the uploaded request file (validate magic header, decompress, parse JSON).
2. Extract `license_key` and `fingerprint_hash`.
3. Call the Polar.sh API to validate the license key:
   - Must be active.
   - Must not be expired.
   - Must not be revoked.
4. Query Cloudflare KV for an existing binding: `license_key → fingerprint_hash`.
   - **Same fingerprint:** Return the cached activation (idempotent).
   - **Different fingerprint:** Return a `403` error with a support message.
   - **No binding:** Proceed to activation.
5. Generate an activation blob:
   - Set `issued_at` to now.
   - Set `expires_at` to the Polar.sh license expiry date.
   - Sign with the Worker's Ed25519 private key.
6. Store the new binding in KV: `license_key → fingerprint_hash`.
7. Return the base64-encoded activation blob.

### Environment Variables / Secrets

| Name | Type | Description |
|------|------|-------------|
| `POLAR_API_KEY` | Secret | Polar.sh API key for license validation. |
| `ED25519_PRIVATE_KEY` | Secret | Base64-encoded Ed25519 private key for signing activations. |
| `KV_NAMESPACE` | Binding | Cloudflare KV namespace for storing license-to-fingerprint mappings. |

### Optional: License Transfer

A `POST /deactivate` endpoint can be added for support use:

- Accepts a Polar.sh license key (validated via API).
- Deletes the KV binding for that key.
- Allows the customer to reactivate on a new server.

This endpoint should be protected by a separate support token and is not exposed to end users.

---

## 7. Rust Backend Integration

### New Module: `src/license/`

```
src/license/
├── mod.rs          # Main API: verify_license(), LicenseStatus enum
├── fingerprint.rs  # Hardware fingerprint computation
├── request.rs      # Generate license-request.bin
└── activation.rs   # Verify and import license-activation.bin
```

### Database Schema

```sql
CREATE TABLE license_activation (
    id INTEGER PRIMARY KEY,
    license_key TEXT NOT NULL UNIQUE,
    fingerprint_hash TEXT NOT NULL,
    activation_blob BLOB NOT NULL,
    activated_at TEXT NOT NULL,
    expires_at TEXT
);
```

### Startup Check

In `main.rs`, before starting the Axum server:

```rust
let license_status = license::verify_license(&db).await?;
if !license_status.is_valid() {
    eprintln!("License invalid or not activated.");
    eprintln!("Run: ./voltrus-lite --generate-request <YOUR_KEY>");
    std::process::exit(1);
}
```

### CLI Arguments

| Argument | Description |
|----------|-------------|
| `--generate-request <KEY>` | Computes fingerprint, creates `license-request.bin` in the current working directory. |
| `--import-activation <PATH>` | Validates the activation file signature and fingerprint, then stores it in SQLite. |
| `--license-status` | Prints the current license status (key, expiry, fingerprint match score) and exits. |

### Dependencies

Add to `Cargo.toml`:

```toml
ed25519-dalek = "2.1"
sysinfo = "0.34.2"  # Already present, used for fingerprinting
```

---

## 8. Frontend UI

When the license is missing or invalid, the React app renders a full-screen activation guide instead of the login/dashboard.

### Activation Screen

```
┌─────────────────────────────────────────┐
│  Welcome to Voltrus Lite                │
│                                         │
│  Your license is not activated.         │
│                                         │
│  Step 1: Run this command on your       │
│          server terminal:               │
│  ┌─────────────────────────────────┐    │
│  │ ./voltrus-lite --generate-request│   │
│  │      YOUR_LICENSE_KEY            │   │
│  └─────────────────────────────────┘    │
│                                         │
│  Step 2: Copy license-request.bin to    │
│          a PC with internet and upload  │
│          it at https://activate.voltrus.io│
│                                         │
│  Step 3: Copy the returned              │
│          license-activation.bin back    │
│          to this server and run:        │
│  ┌─────────────────────────────────┐    │
│  │ ./voltrus-lite --import-activation│  │
│  │      license-activation.bin      │   │
│  └─────────────────────────────────┘    │
│                                         │
│  [ Check Again ]  [ View License Status] │
└─────────────────────────────────────────┘
```

### API Endpoint for UI

`GET /api/license/status`

Returns:

```json
{
  "status": "unactivated",
  "message": "License not found. Please activate your installation."
}
```

Or:

```json
{
  "status": "valid",
  "license_key": "polar_sh_key_abc123",
  "expires_at": "2026-05-01T12:05:00Z",
  "days_remaining": 365
}
```

Or:

```json
{
  "status": "invalid_fingerprint",
  "message": "Hardware fingerprint mismatch (2/5). This license may have been moved. Please reactivate."
}
```

---

## 9. Security Model

| Threat | Likelihood | Mitigation |
|--------|------------|------------|
| **Forged activation files** | Medium | Ed25519 signature verification with a public key embedded in the binary. |
| **Tampered request fingerprint** | Low | The fingerprint is not secret; the Worker validates the Polar.sh key independently. |
| **Replay attacks** | Low | Each request file contains a unique nonce. The Worker can optionally enforce nonce uniqueness (not required for MVP). |
| **License sharing across servers** | Medium (air-gap) | Fingerprint binding prevents casual sharing. Determined users on identical VMs could still share, but this is the fundamental limit of offline licensing. |
| **Clock tampering to extend expiry** | Low | Expiry is enforced locally. Polar.sh re-validation on any online contact catches this. For air-gapped servers, contractual terms are the enforcement. |
| **Binary patching to bypass check** | Low | The license check is in the Rust binary. Patching requires reverse engineering and recompilation. Standard risk for self-hosted software. |

---

## 10. Error Handling & Edge Cases

| Scenario | Behavior |
|----------|----------|
| **Server startup with no license** | Exit with code 1, print activation instructions to stderr. |
| **Fingerprint mismatch on startup** | Exit with code 1, print mismatch score and reactivation instructions. |
| **Expired license on startup** | Exit with code 1, print expiry date and renewal instructions. |
| **Corrupted activation blob in DB** | Treat as unactivated; print error and instructions. |
| **Importing activation for wrong fingerprint** | CLI prints error and refuses to import. |
| **Importing activation with invalid signature** | CLI prints "Invalid activation file." and refuses to import. |
| **Worker KV is down** | Worker returns `503`. User retries upload later. |
| **Polar.sh API is down** | Worker returns `503`. User retries upload later. |

---

## 11. Future Enhancements (Post-MVP)

- **Online re-validation:** When the server has internet, periodically re-validate the Polar.sh key to catch revocations or renewals.
- **Grace period:** If a fingerprint mismatch is detected, enter a 7-day grace period before hard-locking, to allow for planned migrations.
- **License transfer UI:** A self-service portal where a logged-in admin can generate a transfer token to move a license without contacting support.
- **Feature flags:** Embed feature entitlements in the activation blob (e.g., `max_screens`, `modbus_devices_limit`).

---

## 12. Open Questions

1. **Worker domain name:** What is the final domain for the activation worker? (`activate.voltrus.io` is a placeholder.)
2. **Support process for transfers:** Who handles manual license transfers? Should there be a simple admin panel or just email support?
3. **Polar.sh webhook:** Should the Worker listen for Polar.sh webhooks (e.g., `subscription.cancelled`) to proactively revoke KV bindings?

---

## 13. Appendix: Ed25519 Key Generation

Generate a new signing key pair for the Worker:

```bash
# Using openssl
openssl genpkey -algorithm ED25519 -out ed25519-private.pem
openssl pkey -in ed25519-private.pem -pubout -out ed25519-public.pem
```

The public key is embedded in the Rust binary (as a `&[u8; 32]` constant). The private key is stored as a Cloudflare Worker secret.
