# Voltrus Lite Security Review — v0.17.1
**Target:** https://demo.voltrus.id  
**Date:** 2026-05-01  
**Scope:** Application, Network, OS (VPS with modbus-sim, no real hardware)  
**Methodology:** Black-box penetration testing + code-path analysis

---

## Executive Summary

The v0.17.1 security hardening resolved all P0–P2 concerns from the original audit. **No critical remote-exploitable vulnerabilities were found.** However, several medium-severity issues remain that an attacker can weaponize for user enumeration, information disclosure, and limited DoS. This report assumes the attacker knows the codebase is Rust/Axum + SQLite + React.

---

## Findings

### 🔴 CRITICAL — None Found
No unauthenticated RCE, SQLi, auth bypass, or path traversal to sensitive files.

---

### 🟠 HIGH

#### H1: User Enumeration via Timing Analysis on `/api/auth/login`
**Attack:** An attacker measures response times to determine if a username exists.
- Valid user (`admin`): ~287ms average (Argon2 hash verification executes)
- Invalid user (`voltrus`): ~175ms average (returns immediately after DB lookup)
- **Impact:** Attacker can build a valid username list, then brute-force only existing accounts.
- **Repro:**
  ```bash
  for user in admin nonexistent; do
    time curl -X POST ... -d "{\"username\":\"$user\",\"password\":\"x\"}"
  done
  ```

#### H2: API 404s Return SPA `index.html` with HTTP 200
**Attack:** Any unmatched path (including `/api/nonexistent`, `/.env`, `/config.yaml`) returns the React SPA with `Content-Type: text/html` and HTTP 200.
- **Impact:**
  - API consumers can't detect 404s (breaks automation, monitoring)
  - Directory/file brute-forcing tools get false positives for every path
  - Potential cache pollution if a CDN caches `/data.db` → `index.html`
- **Evidence:**
  ```
  /api/nonexistent → 200 text/html (index.html)
  /data.db         → 200 text/html (index.html fallback, 542 bytes)
  /config.yaml     → 200 text/html (index.html fallback)
  ```

#### H3: Error Messages Leak Framework (Axum / Serde)
**Attack:** Malformed JSON requests return detailed parse errors.
- `{"username":"test",}` → `"Failed to parse the request body as JSON: trailing comma at line 1 column 38"`
- `not-json` → `"Failed to parse the request body as JSON: expected ident at line 1 column 2"`
- **Impact:** Confirms the backend is Axum + serde_json, helping attackers choose targeted exploits.

---

### 🟡 MEDIUM

#### M1: Rate Limiting is Per-Username, Not Per-IP
**Attack:** An attacker with a list of 1000 usernames can attempt 5 passwords each (5,000 attempts/minute) without hitting any rate limit.
- **Current:** Key = `format!("login:{}", username)`
- **Impact:** Distributed brute-force across username space is unthrottled.

#### M2: No Account Lockout After Failed Attempts
**Attack:** Even with per-IP rate limiting, an attacker can retry indefinitely once the window resets. There's no progressive delay or account lockout.
- **Impact:** Persistent attacker will eventually guess weak passwords given enough time.

#### M3: JWT Tokens Valid for 24 Hours with No Refresh / Revocation
**Attack:** A stolen token (via XSS, MITM, or log leak) is valid for a full day. There's no way to revoke it early.
- **Impact:** Long-lived tokens increase blast radius of credential theft.

#### M4: CSP Allows `unsafe-inline` for Scripts and Styles
**Attack:** If an attacker finds an XSS vector (e.g., via a third-party dependency or DOM-based XSS), `script-src 'self' 'unsafe-inline'` allows execution of injected inline scripts.
- **Impact:** CSP is present but weakened; a successful XSS payload can execute.

#### M5: No Request Body Size Limits on Non-Auth Endpoints
**Attack:** An authenticated attacker can send very large POST bodies to admin endpoints (e.g., `/api/flows`, `/api/screens`).
- **Evidence:** 1MB payload to `/api/auth/login` returned 422 in 739ms, but other endpoints weren't tested.
- **Impact:** Memory exhaustion / DoS on the Rust backend.

#### M6: `Accept-Ranges: bytes` on Static Assets Without Enforcement
**Attack:** `alarm.wav` returns `Accept-Ranges: bytes`. While we added a 10MB limit middleware, the header is still advertised.
- **Impact:** An attacker can attempt multi-range requests or abuse range semantics.

#### M7: Missing `Cache-Control: no-store` on Authenticated Responses
**Attack:** Authenticated API responses (e.g., `/api/system`, `/api/users`) don't explicitly tell browsers/proxies not to cache them.
- **Impact:** Browsers may cache sensitive JSON in history/back-forward cache.

#### M8: No Rate Limiting on Non-Auth Endpoints
**Attack:** `/api/health`, `/api/settings/branding`, and other public endpoints have no rate limits.
- **Impact:** Amplification vector for DDoS, or enumeration of device states.

---

### 🟢 LOW / INFORMATIONAL

#### L1: Server Banner Hidden Behind Cloudflare
**Good:** `Server: cloudflare` only. No Axum/Rust version leak.

#### L2: TLS Configuration is Strong
**Good:** TLS 1.2 + 1.3, AES-256-GCM, Let's Encrypt certificate. No weak ciphers.

#### L3: Default Credentials Eliminated
**Good:** `admin/admin` no longer works. `seed_database()` generates a random 16-char password.

#### L4: Security Headers Present
**Good:** HSTS, CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy all active.

#### L5: Origin IP Well Hidden
**Good:** Only Cloudflare edge IPs (104.21.x.x, 172.67.x.x) exposed. No direct origin access.

---

## Attacker Kill Chain (What a Bad Actor Would Do)

1. **Recon:** Probe `/api/health`, `/api/settings/branding` → learns it's Voltrus Lite, sees device names.
2. **User Enumeration:** Time-based analysis on `/api/auth/login` → discovers `admin` exists.
3. **Framework Fingerprinting:** Send malformed JSON → confirms Axum + serde backend.
4. **Distributed Brute Force:** Rotate through username list, 5 attempts each per minute.
5. **Credential Stuffing:** Once password is guessed (or found in breach DB), 24h token is issued.
6. **Privilege Escalation:** If token is stolen, attacker has admin access for 24 hours with no revocation.
7. **DoS:** Send 1MB+ payloads to admin endpoints to exhaust memory.
8. **Information Disclosure:** Any 404 returns HTML 200, making it hard to detect probing.

---

## Remediation Plan

### Phase 1 — High Priority (This Week)

| Fix | File | Details |
|-----|------|---------|
| **Fix timing attack** | `src/api/auth.rs` | Use `tokio::time::sleep` to pad response time so valid and invalid users take ~300ms consistently. Or verify password on a separate spawn_blocking task and always wait for fixed duration. |
| **Fix API 404 fallback** | `src/api/mod.rs` | Add a `404` fallback handler that returns JSON `{"error":"not found"}` for paths starting with `/api/`. Only use SPA fallback for non-API paths. |
| **Sanitize error messages** | `src/api/mod.rs` | Replace Axum's default JSON parse error with generic `"Invalid request"`. Don't expose serde/Axum internals. |

### Phase 2 — Medium Priority (This Sprint)

| Fix | File | Details |
|-----|------|---------|
| **Add IP-based rate limiting** | `src/security.rs` | Extend `LoginRateLimiter` with dual keys: `login:ip:{ip}` AND `login:user:{username}`. Block if either exceeds threshold. |
| **Add account lockout** | `src/api/auth.rs` + `src/storage.rs` | After 10 failed attempts, lock account for 15 minutes. Store `failed_attempts` and `locked_until` in `users` table. |
| **Add request body size limit** | `src/api/mod.rs` | Use `axum::extract::DefaultBodyLimit::max(1024 * 1024)` (1MB) on all routes. |
| **Harden CSP** | `src/security.rs` | Remove `'unsafe-inline'` from `script-src` by using nonce-based CSP. Generate a per-request nonce and inject it into `index.html`. |
| **Add short-lived refresh tokens** | `src/api/auth.rs` | Reduce access token to 15 minutes. Add refresh token (7 days, HTTP-only cookie, rotated on use). Add `/api/auth/refresh` endpoint. |
| **Add `Cache-Control: no-store`** | `src/security.rs` | Add `cache-control: no-store, private, max-age=0` to all API responses. |
| **Add global rate limiting** | `src/api/mod.rs` | Add `tower::limit::RateLimitLayer` or ` Governor` middleware for all public endpoints (e.g., 100 req/min per IP). |

### Phase 3 — Low Priority / Defense in Depth (Next Sprint)

| Fix | File | Details |
|-----|------|---------|
| **Add Content-Length validation** | `src/api/mod.rs` | Reject requests with `Content-Length > 1MB` before body parsing. |
| **Add API versioning** | `src/api/mod.rs` | Prefix all routes with `/api/v1/` to allow future breaking changes without breaking clients. |
| **Add intrusion detection** | `src/main.rs` | Log repeated 401s from same IP to `tracing::warn!` for SIEM ingestion. |
| **Add request ID header** | `src/api/mod.rs` | Inject `X-Request-Id` (UUID) into all responses for forensic tracing. |
| **Add DB encryption at rest** | `src/storage.rs` | Encrypt SQLite with SQLCipher or use `PRAGMA key` for sensitive tables. |
| **Add automated security scanning** | `.github/workflows/` | Add `cargo audit`, `cargo deny`, and `trivy` scans to CI pipeline. |

---

## Quick Wins (Can Implement Today)

```rust
// 1. Fix API 404 fallback in src/api/mod.rs
let api_router = Router::new()
    .merge(public)
    .merge(protected)
    .merge(admin_protected)
    .layer(cors)
    .layer(...)
    .fallback(|| async { (StatusCode::NOT_FOUND, Json(json!({"error":"not found"}))) });

// 2. Fix timing attack in src/api/auth.rs
let start = Instant::now();
let result = do_login_logic().await;
let elapsed = start.elapsed();
if elapsed < Duration::from_millis(300) {
    tokio::time::sleep(Duration::from_millis(300) - elapsed).await;
}
result

// 3. Sanitize parse errors
.layer(HandleErrorLayer::new(|e: JsonRejection| async move {
    (StatusCode::BAD_REQUEST, Json(json!({"error":"Invalid request"})))
}))
```

---

## Verification Checklist

After implementing fixes, verify:
- [ ] `curl -X POST /api/auth/login -d '{"username":"admin","password":"x"}'` and `-d '{"username":"fake","password":"x"}'` both take ~300ms
- [ ] `curl /api/nonexistent` returns `404 {"error":"not found"}`
- [ ] `curl -X POST /api/auth/login -d 'bad'` returns `400 {"error":"Invalid request"}`
- [ ] 6 failed logins from same IP returns `429`
- [ ] 6 failed logins across 6 different usernames from same IP returns `429`
- [ ] Large POST (>1MB) returns `413 Payload Too Large`
- [ ] API responses include `cache-control: no-store`

---

*Reviewed by: Automated offensive security assessment*  
*Risk Rating: MEDIUM — No immediate remote exploit, but several attack paths exist for determined adversaries.*
