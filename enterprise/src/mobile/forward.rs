//! Phone requests the relay forwards, run through the site's own router.
//!
//! A request goes through the same layers as one on the HTTP port: the device
//! token is checked by the auth middleware, roles apply, and rate limits count
//! it against [`RELAY_PEER`] (forwarded requests carry no client address).

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode, header};
use serde::Deserialize;
use serde_json::{Value, json};
use tower::ServiceExt;

/// Request headers passed on to the router; the rest are dropped.
const FORWARDED_HEADERS: [&str; 3] = ["authorization", "content-type", "accept-language"];

pub const MAX_REQUEST_BODY: usize = 64 * 1024;
pub const MAX_RESPONSE_BODY: usize = 1024 * 1024;

/// The peer address of every forwarded request: 192.0.2.1 (TEST-NET-1, never a
/// real host), so the requests are not taken for local ones (loopback is exempt
/// from rate limits and trusted for proxy headers) and show up as the relay in
/// the security log.
pub const RELAY_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 0);

/// A `request` frame.
#[derive(Debug, Deserialize)]
pub struct RelayRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
}

/// The routes a phone may reach through the relay (the relay checks the same list).
pub fn allowed(method: &str, path: &str) -> bool {
    let Some(rest) = path.strip_prefix("/api/v1/") else {
        return false;
    };
    let segments: Vec<&str> = rest.split('/').collect();
    let id = |s: &str| !s.is_empty() && s.len() <= 18 && s.bytes().all(|b| b.is_ascii_digit());
    match (method, segments.as_slice()) {
        ("POST", ["mobile", "pair"]) => true,
        ("GET" | "PATCH" | "DELETE", ["mobile", "device"]) => true,
        ("GET", ["mobile", "summary"]) => true,
        ("GET", ["alarms"]) => true,
        ("GET", ["alarms", n, "journal"]) => id(n),
        ("POST", ["alarms", n, "acknowledge"]) => id(n),
        _ => false,
    }
}

/// The `response` frame for `request`.
pub async fn answer(router: Option<Router>, request: RelayRequest) -> Value {
    let id = request.id.clone();
    let (status, content_type, body) = match run(router, request).await {
        Ok(answer) => answer,
        Err((status, code)) => (
            status,
            Some("application/json".to_string()),
            json!({"error": code, "code": code}).to_string(),
        ),
    };
    let mut headers = serde_json::Map::new();
    if let Some(ct) = content_type {
        headers.insert("content-type".into(), Value::String(ct));
    }
    json!({
        "type": "response",
        "id": id,
        "status": status.as_u16(),
        "headers": headers,
        "body": body,
    })
}

type Answer = (StatusCode, Option<String>, String);

async fn run(
    router: Option<Router>,
    request: RelayRequest,
) -> Result<Answer, (StatusCode, &'static str)> {
    if !allowed(&request.method, &request.path) {
        return Err((StatusCode::FORBIDDEN, "path_not_allowed"));
    }
    let body = request.body.unwrap_or_default();
    if body.len() > MAX_REQUEST_BODY {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"));
    }
    let Some(router) = router else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "site_starting"));
    };
    let uri = match request.query.as_deref().filter(|q| !q.is_empty()) {
        Some(q) => format!("{}?{q}", request.path),
        None => request.path.clone(),
    };
    let method = Method::from_bytes(request.method.as_bytes())
        .map_err(|_| (StatusCode::BAD_REQUEST, "bad_request"))?;
    let mut builder = Request::builder().method(method).uri(uri);
    for (name, value) in &request.headers {
        if let Some(allowed) = FORWARDED_HEADERS
            .iter()
            .find(|h| h.eq_ignore_ascii_case(name))
        {
            builder = builder.header(*allowed, value);
        }
    }
    let mut req = builder
        .body(Body::from(body))
        .map_err(|_| (StatusCode::BAD_REQUEST, "bad_request"))?;
    req.extensions_mut().insert(ConnectInfo(RELAY_PEER));

    let response = router
        .oneshot(req)
        .await
        .unwrap_or_else(|never| match never {});
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = axum::body::to_bytes(response.into_body(), MAX_RESPONSE_BODY)
        .await
        .map_err(|_| (StatusCode::BAD_GATEWAY, "response_too_large"))?;
    Ok((
        status,
        content_type,
        String::from_utf8_lossy(&bytes).into_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::allowed;

    #[test]
    fn only_the_listed_routes_are_forwarded() {
        for (m, p) in [
            ("POST", "/api/v1/mobile/pair"),
            ("GET", "/api/v1/mobile/device"),
            ("PATCH", "/api/v1/mobile/device"),
            ("DELETE", "/api/v1/mobile/device"),
            ("GET", "/api/v1/mobile/summary"),
            ("GET", "/api/v1/alarms"),
            ("GET", "/api/v1/alarms/12/journal"),
            ("POST", "/api/v1/alarms/12/acknowledge"),
        ] {
            assert!(allowed(m, p), "{m} {p}");
        }
        for (m, p) in [
            ("GET", "/api/v1/mobile/pair"),
            ("POST", "/api/v1/alarms"),
            ("GET", "/api/v1/alarms/"),
            ("GET", "/api/v1/alarms/x/journal"),
            ("GET", "/api/v1/alarms//journal"),
            ("POST", "/api/v1/alarms/12/acknowledge/"),
            ("GET", "/api/v1/alarms/12/../../users"),
            ("GET", "/api/v1/users"),
            ("GET", "/api/v1/mobile/devices"),
            ("get", "/api/v1/alarms"),
            ("GET", "/api/v1/mobile/summary?x=1"),
            ("GET", "api/v1/alarms"),
        ] {
            assert!(!allowed(m, p), "{m} {p}");
        }
    }
}
