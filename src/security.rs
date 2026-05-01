use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request};
use axum::http::header;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use serde_json::json;

// ── Rate Limiter ──

const LOGIN_MAX_ATTEMPTS: u32 = 5;
const LOGIN_WINDOW_SECS: u64 = 60;

#[derive(Clone)]
pub struct LoginRateLimiter {
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl LoginRateLimiter {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn is_allowed(&self, key: &str) -> bool {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();
        let window = Duration::from_secs(LOGIN_WINDOW_SECS);

        let entries = attempts.entry(key.to_string()).or_default();
        entries.retain(|t| now.duration_since(*t) < window);

        entries.len() < LOGIN_MAX_ATTEMPTS as usize
    }

    pub fn record_attempt(&self, key: &str) {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();
        let entries = attempts.entry(key.to_string()).or_default();
        entries.retain(|t| now.duration_since(*t) < Duration::from_secs(LOGIN_WINDOW_SECS));
        entries.push(now);
    }

    pub fn cleanup(&self) {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();
        let window = Duration::from_secs(LOGIN_WINDOW_SECS * 2);
        attempts.retain(|_, entries| {
            entries.retain(|t| now.duration_since(*t) < window);
            !entries.is_empty()
        });
    }
}

impl Default for LoginRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware that rate-limits login attempts by IP address.
pub async fn login_rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let limiter = req
        .extensions()
        .get::<LoginRateLimiter>()
        .cloned()
        .unwrap_or_default();

    let key = addr.ip().to_string();

    if !limiter.is_allowed(&key) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(json!({"error": "Too many login attempts. Please try again later."})),
        )
            .into_response();
    }

    limiter.record_attempt(&key);
    next.run(req).await
}

// ── Security Headers ──

pub async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Prevent clickjacking
    headers.insert(
        "X-Frame-Options",
        header::HeaderValue::from_static("DENY"),
    );

    // Prevent MIME type sniffing
    headers.insert(
        "X-Content-Type-Options",
        header::HeaderValue::from_static("nosniff"),
    );

    // XSS protection for legacy browsers
    headers.insert(
        "X-XSS-Protection",
        header::HeaderValue::from_static("1; mode=block"),
    );

    // Referrer policy
    headers.insert(
        "Referrer-Policy",
        header::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        header::HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: blob:; \
             font-src 'self'; \
             connect-src 'self'; \
             media-src 'self'; \
             object-src 'none'; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self';",
        ),
    );

    // Permissions Policy
    headers.insert(
        "Permissions-Policy",
        header::HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()",
        ),
    );

    // HSTS - only in production (when not localhost)
    // Note: The caller should conditionally apply this based on config
    headers.insert(
        "Strict-Transport-Security",
        header::HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    response
}

// ── Range Request Limiting ──

const MAX_RANGE_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

/// Middleware that limits HTTP Range request sizes to prevent DoS via
/// large range requests on static assets.
pub async fn limit_range_requests_middleware(req: Request, next: Next) -> Response {
    let mut req = req;
    if let Some(range) = req.headers().get("range") {
        if let Ok(range_str) = range.to_str() {
            // Parse simple byte ranges like "bytes=0-10485759"
            // Reject if the requested range exceeds the limit
            if let Some(size) = parse_range_size(range_str) {
                if size > MAX_RANGE_SIZE_BYTES {
                    return (
                        StatusCode::RANGE_NOT_SATISFIABLE,
                        axum::Json(json!({"error": "Requested range too large"})),
                    )
                        .into_response();
                }
            } else {
                // If we can't parse the range, strip it to be safe
                req.headers_mut().remove("range");
            }
        } else {
            req.headers_mut().remove("range");
        }
    }
    next.run(req).await
}

fn parse_range_size(range: &str) -> Option<u64> {
    // Handle "bytes=start-end" format
    let range = range.strip_prefix("bytes=")?;
    for part in range.split(',') {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            let start: u64 = start.parse().ok()?;
            if end.is_empty() {
                // Range like "bytes=start-" means from start to end of file
                // We can't know the file size, so treat as potentially large
                return None;
            }
            let end: u64 = end.parse().ok()?;
            if end >= start {
                return Some(end - start + 1);
            }
        }
    }
    None
}

// ── Source Map Blocking ──

/// Middleware that blocks requests for source map files (.map) to prevent
/// information disclosure in production.
pub async fn block_source_maps_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path();
    if path.ends_with(".map") {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }
    next.run(req).await
}

// ── X-Forwarded-Host Stripping ──

pub async fn strip_forwarded_host_middleware(req: Request, next: Next) -> Response {
    let mut req = req;
    // Remove X-Forwarded-Host to prevent host header injection attacks
    // If you have a known reverse proxy, configure it explicitly instead
    req.headers_mut().remove("X-Forwarded-Host");
    req.headers_mut().remove("X-Forwarded-Port");
    next.run(req).await
}

// ── Input Sanitization ──

const MAX_BRAND_LENGTH: usize = 200;
const MAX_LOGO_LENGTH: usize = 50000;

/// Sanitize a branding string: strip HTML tags, trim, and limit length.
pub fn sanitize_brand_input(input: &str) -> String {
    let trimmed = input.trim();
    let without_html = strip_html_tags(trimmed);
    let limited = if without_html.len() > MAX_BRAND_LENGTH {
        &without_html[..MAX_BRAND_LENGTH]
    } else {
        &without_html
    };
    limited.to_string()
}

/// Sanitize logo input (base64 or URL): trim and limit length.
pub fn sanitize_logo_input(input: &str) -> String {
    let trimmed = input.trim();
    let limited = if trimmed.len() > MAX_LOGO_LENGTH {
        &trimmed[..MAX_LOGO_LENGTH]
    } else {
        trimmed
    };
    limited.to_string()
}

fn strip_html_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

// ── CSRF Token ──

use rand::Rng;

const CSRF_TOKEN_LENGTH: usize = 32;

pub fn generate_csrf_token() -> String {
    let token: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(CSRF_TOKEN_LENGTH)
        .map(char::from)
        .collect();
    token
}

/// Validate that the X-CSRF-Token header matches the expected token.
pub fn validate_csrf_token(req: &Request, expected: &str) -> bool {
    req.headers()
        .get("X-CSRF-Token")
        .and_then(|v| v.to_str().ok())
        .map(|t| t == expected)
        .unwrap_or(false)
}

/// Middleware that validates CSRF tokens for state-changing operations.
/// The expected token should be stored in the request extensions.
pub async fn csrf_validation_middleware(req: Request, next: Next) -> Response {
    let expected = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_default();

    if !validate_csrf_token(&req, &expected) {
        return (
            StatusCode::FORBIDDEN,
            axum::Json(json!({"error": "CSRF token missing or invalid"})),
        )
            .into_response();
    }

    next.run(req).await
}

// ── API Request Signing ──

/// Validate an HMAC-SHA256 signature on a request.
/// Expected header: `X-Signature: <hex_hmac>`
/// The signature is computed over the request body using a shared secret.
pub fn validate_request_signature(req: &Request, body: &[u8], secret: &str) -> bool {
    let expected = req
        .headers()
        .get("X-Signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    let result = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison to prevent timing attacks
    if expected.len() != result.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in expected.bytes().zip(result.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

// ── Audit Logging ──

#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub action: String,
    pub username: String,
    pub resource: String,
    pub details: Option<String>,
}

#[derive(Clone)]
pub struct AuditLogger {
    sender: tokio::sync::mpsc::UnboundedSender<AuditEvent>,
}

impl AuditLogger {
    pub fn new() -> (Self, tokio::sync::mpsc::UnboundedReceiver<AuditEvent>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (Self { sender: tx }, rx)
    }

    pub fn log(&self, event: AuditEvent) {
        let _ = self.sender.send(event);
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        Self { sender: tx }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let limiter = LoginRateLimiter::new();
        assert!(limiter.is_allowed("user1"));
        limiter.record_attempt("user1");
        assert!(limiter.is_allowed("user1"));
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = LoginRateLimiter::new();
        for _ in 0..LOGIN_MAX_ATTEMPTS {
            limiter.record_attempt("user2");
        }
        assert!(!limiter.is_allowed("user2"));
    }

    #[test]
    fn test_sanitize_brand_strips_html() {
        assert_eq!(sanitize_brand_input("Hello <script>alert(1)</script>"), "Hello alert(1)");
        assert_eq!(sanitize_brand_input("  Normal Title  "), "Normal Title");
    }

    #[test]
    fn test_sanitize_brand_limits_length() {
        let long = "a".repeat(MAX_BRAND_LENGTH + 100);
        let result = sanitize_brand_input(&long);
        assert_eq!(result.len(), MAX_BRAND_LENGTH);
    }

    #[test]
    fn test_generate_csrf_token() {
        let t1 = generate_csrf_token();
        let t2 = generate_csrf_token();
        assert_eq!(t1.len(), CSRF_TOKEN_LENGTH);
        assert_ne!(t1, t2);
    }
}
