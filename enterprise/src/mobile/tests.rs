//! The connector against a fake relay (a tokio-tungstenite server in the test).

// The fake relay's handshake callbacks return tungstenite's `ErrorResponse`.
#![allow(clippy::result_large_err)]

use std::sync::{Arc, RwLock};
use std::time::Duration;

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::http::HeaderMap;
use voltrus::edition::{MobileAlarmPush, MobilePushSender, MobilePushTarget};
use voltrus::license::LicenseStatus;
use voltrus::models::LicenseActivationRow;
use voltrus::storage::Storage;

use super::{RelayConnector, SITE_SECRET_KEY, Timing};

const WAIT: Duration = Duration::from_secs(10);

fn integrator() -> Arc<RwLock<LicenseStatus>> {
    Arc::new(RwLock::new(LicenseStatus::Valid {
        license_key: "VOLTRUS-TEST".into(),
        expires_at: None,
        days_remaining: None,
        tier: Some("integrator".into()),
        grace: false,
        legacy_white_label: false,
        site: None,
        support_until: None,
    }))
}

/// An activation file as the relay sees it. The connector passes it on
/// unchecked; the relay verifies the signature.
fn activation_blob() -> Vec<u8> {
    let mut blob = b"VOLACT\x01".to_vec();
    blob.extend_from_slice(
        json!({
            "version": 2, "license_key": "VOLTRUS-TEST", "fingerprint_hash": "sha256:x",
            "issued_at": "2026-11-01T00:00:00Z", "expires_at": null, "tier": "integrator",
            "device_limit": null, "site_id": "site-test", "signature": "00",
        })
        .to_string()
        .as_bytes(),
    );
    blob
}

async fn store_activation(storage: &Storage) {
    storage
        .store_license_activation(&LicenseActivationRow {
            license_key: "VOLTRUS-TEST".into(),
            fingerprint_hash: "sha256:x".into(),
            activation_blob: activation_blob(),
            fingerprint_hostname: String::new(),
            fingerprint_cpu: String::new(),
            fingerprint_disk_serial: String::new(),
            fingerprint_mac: String::new(),
            fingerprint_machine_id: String::new(),
            activated_at: "2026-11-01T00:00:00Z".into(),
            expires_at: None,
            tier: Some("integrator".into()),
            device_limit: None,
            site_id: Some("site-test".into()),
            site_name: None,
            support_until: None,
        })
        .await
        .unwrap();
}

/// Core's router on `storage`, as the server builds it.
fn site_router(storage: Storage, license: Arc<RwLock<LicenseStatus>>) -> axum::Router {
    use voltrus::cache::DeviceCache;
    use voltrus::poller_manager::PollerManager;
    let cache = DeviceCache::new();
    let poller_manager = PollerManager::new(
        cache.clone(),
        storage.clone(),
        voltrus::network::NetworkManager::new(vec![]),
    );
    let state = voltrus::api::AppState {
        cache,
        storage,
        poller_manager,
        started_at: std::time::Instant::now(),
        auth_secret: "relay-test-secret".to_string(),
        cors_origin: None,
        cookie_secure: voltrus::models::CookieSecure::Never,
        public_demo: Default::default(),
        retention_config: voltrus::models::RetentionConfig::default(),
        flow_engine: None,
        login_rate_limiter: voltrus::security::LoginRateLimiter::new(),
        global_rate_limiter: voltrus::security::GlobalRateLimiter::new(),
        audit_logger: voltrus::security::AuditLogger::new().0,
        mcp_guard: Default::default(),
        intrusion_detector: voltrus::security::IntrusionDetector::new(),
        license_status: license,
        ext: Arc::new(voltrus::edition::Community),
        smtp_config: Arc::new(tokio::sync::RwLock::new(None)),
        config_path: "config.yaml".to_string(),
        ws_manager: Arc::new(voltrus::ws_manager::WsManager::new()),
        mqtt_broker_state: None,
        network_manager: voltrus::network::NetworkManager::new(vec![]),
        health_monitor: Arc::new(voltrus::health::watchdog::HealthMonitor::new(
            voltrus::models::WatchdogConfig::default(),
        )),
        widget_registry: voltrus::widget_sdk::WidgetRegistry::new(std::path::PathBuf::from(
            "voltrus/widgets",
        )),
        modbus_pool: Arc::new(voltrus::modbus::ModbusPool::new()),
    };
    voltrus::api::create_router(state)
}

struct Site {
    storage: Storage,
    connector: Arc<RelayConnector>,
    listener: TcpListener,
}

async fn site() -> Site {
    let storage = Storage::new_in_memory().unwrap();
    store_activation(&storage).await;
    let license = integrator();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let timing = Timing {
        first_backoff: Duration::from_millis(50),
        ..Timing::default()
    };
    let connector = Arc::new(RelayConnector::new(
        &url,
        storage.clone(),
        license.clone(),
        timing,
    ));
    connector.use_router(site_router(storage.clone(), license));
    connector.spawn();
    Site {
        storage,
        connector,
        listener,
    }
}

type Ws = WebSocketStream<TcpStream>;

/// Accept the site's next connection; returns its handshake headers.
async fn accept(listener: &TcpListener) -> (HeaderMap, Ws) {
    let (tcp, _) = tokio::time::timeout(WAIT, listener.accept())
        .await
        .expect("the site connects")
        .unwrap();
    let mut headers = HeaderMap::new();
    let ws = tokio_tungstenite::accept_hdr_async(tcp, |req: &Request, resp: Response| {
        assert_eq!(req.uri().path(), "/v1/sites/connect");
        headers = req.headers().clone();
        Ok(resp)
    })
    .await
    .unwrap();
    (headers, ws)
}

async fn send(ws: &mut Ws, frame: Value) {
    ws.send(Message::text(frame.to_string())).await.unwrap();
}

/// The next frame other than a ping.
async fn recv(ws: &mut Ws) -> Value {
    loop {
        let msg = tokio::time::timeout(WAIT, ws.next())
            .await
            .expect("a frame from the site")
            .expect("the connection is open")
            .unwrap();
        if let Message::Text(text) = msg {
            let frame: Value = serde_json::from_str(&text).unwrap();
            if frame["type"] != "ping" {
                return frame;
            }
        }
    }
}

async fn eventually<F: AsyncFn() -> bool>(what: &str, check: F) {
    let deadline = tokio::time::Instant::now() + WAIT;
    while !check().await {
        assert!(tokio::time::Instant::now() < deadline, "timed out: {what}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn stored_secret(storage: &Storage) -> Option<String> {
    storage
        .get_settings(&[SITE_SECRET_KEY])
        .await
        .unwrap()
        .into_iter()
        .next()
        .map(|s| s.value)
}

/// Accept a connection and greet it.
async fn connected(site: &Site) -> Ws {
    let (_, mut ws) = accept(&site.listener).await;
    send(&mut ws, json!({"type": "hello", "site_id": "site-test"})).await;
    let connector = site.connector.clone();
    eventually("connected", async || connector.connected()).await;
    ws
}

/// Forward a phone request and return the site's `response` frame.
async fn forward(
    ws: &mut Ws,
    id: &str,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Value {
    let mut headers = json!({"content-type": "application/json", "accept-language": "id"});
    if let Some(token) = token {
        headers["authorization"] = json!(format!("Bearer {token}"));
    }
    send(
        ws,
        json!({
            "type": "request", "id": id, "method": method, "path": path, "query": null,
            "headers": headers, "body": body.map(|b| b.to_string()),
        }),
    )
    .await;
    let frame = recv(ws).await;
    assert_eq!(frame["type"], "response", "{frame}");
    assert_eq!(frame["id"], id);
    frame
}

fn body(frame: &Value) -> Value {
    serde_json::from_str(frame["body"].as_str().unwrap()).unwrap()
}

#[tokio::test]
async fn connects_with_the_activation_and_sends_the_issued_secret_next_time() {
    let site = site().await;
    let (headers, mut ws) = accept(&site.listener).await;
    // Known from the activation before the relay says hello, for the QR code.
    let relay = site.connector.relay().unwrap();
    assert_eq!(relay.site_id, "site-test");
    assert!(!relay.connected);
    let expected = base64::engine::general_purpose::STANDARD.encode(activation_blob());
    assert_eq!(headers["x-voltrus-activation"], expected.as_str());
    assert!(!headers.contains_key("x-voltrus-site-secret"));
    assert_eq!(headers["x-voltrus-version"], env!("CARGO_PKG_VERSION"));

    send(
        &mut ws,
        json!({"type": "hello", "site_id": "site-test", "site_secret": "s3cret",
               "limits": {"devices": 50, "pushes_per_hour": 600}}),
    )
    .await;
    let storage = site.storage.clone();
    eventually("secret stored", async || {
        stored_secret(&storage).await.as_deref() == Some("s3cret")
    })
    .await;
    assert!(site.connector.relay().unwrap().connected);

    // The relay drops the connection; the site comes back with its secret.
    ws.close(None).await.unwrap();
    drop(ws);
    let (headers, mut ws) = accept(&site.listener).await;
    assert_eq!(headers["x-voltrus-site-secret"], "s3cret");
    assert_eq!(headers["x-voltrus-activation"], expected.as_str());
    send(&mut ws, json!({"type": "hello", "site_id": "site-test"})).await;
    let connector = site.connector.clone();
    eventually("reconnected", async || connector.connected()).await;
    assert_eq!(
        stored_secret(&site.storage).await.as_deref(),
        Some("s3cret")
    );
}

fn alarm_push(details: bool) -> MobileAlarmPush {
    MobileAlarmPush {
        event: "raised".into(),
        alarm_id: 12,
        site_name: "Pabrik A".into(),
        source_key: "boiler_temp".into(),
        source_name: "Boiler temperature".into(),
        severity: "high".into(),
        priority: "critical".into(),
        message: "Too hot".into(),
        value: 98.5,
        unit: "°C".into(),
        actor: None,
        ts: 1_764_000_000,
        details,
        held_back: 0,
        targets: vec![MobilePushTarget {
            device_id: 7,
            platform: "android".into(),
            push_token: "ExponentPushToken[a]".into(),
            lang: "id".into(),
        }],
    }
}

#[tokio::test]
async fn a_push_without_details_is_blanked_and_answered_by_its_push_result() {
    let site = site().await;
    let push = alarm_push(false);
    assert!(
        site.connector.send(&push).await.is_err(),
        "not connected yet"
    );
    let mut ws = connected(&site).await;

    let connector = site.connector.clone();
    let sending = tokio::spawn(async move { connector.send(&alarm_push(false)).await });
    let frame = recv(&mut ws).await;
    assert_eq!(frame["type"], "push");
    let p = &frame["push"];
    assert_eq!(p["event"], "raised");
    assert_eq!(p["alarm_id"], 12);
    assert_eq!(p["site_name"], "Pabrik A");
    assert_eq!(p["priority"], "critical");
    assert_eq!(p["details"], false);
    for blank in ["source_key", "source_name", "message", "unit"] {
        assert_eq!(p[blank], "", "{blank}");
    }
    assert_eq!(p["value"], 0.0);
    assert_eq!(p["targets"][0]["push_token"], "ExponentPushToken[a]");
    assert_eq!(p["targets"][0]["lang"], "id");

    send(
        &mut ws,
        json!({"type": "push_result", "id": frame["id"],
               "deliveries": [{"device_id": 7, "error": "DeviceNotRegistered", "unregistered": true}]}),
    )
    .await;
    let deliveries = sending.await.unwrap().unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].device_id, 7);
    assert!(deliveries[0].unregistered);
    assert_eq!(deliveries[0].error.as_deref(), Some("DeviceNotRegistered"));

    // With details on, the alarm goes out as it is.
    let connector = site.connector.clone();
    let sending = tokio::spawn(async move { connector.send(&alarm_push(true)).await });
    let frame = recv(&mut ws).await;
    assert_eq!(frame["push"]["source_name"], "Boiler temperature");
    assert_eq!(frame["push"]["value"], 98.5);
    send(
        &mut ws,
        json!({"type": "push_result", "id": frame["id"], "deliveries": []}),
    )
    .await;
    assert!(sending.await.unwrap().unwrap().is_empty());
}

#[tokio::test]
async fn phone_requests_run_through_the_router_and_unregistered_tokens_are_cleared() {
    let site = site().await;
    site.storage
        .create_user("budi", "not-a-hash", "", "operator")
        .await
        .unwrap();
    let code = site
        .storage
        .create_mobile_pairing("budi", 600)
        .await
        .unwrap();
    let mut ws = connected(&site).await;

    // Pairing through the relay.
    let paired = forward(
        &mut ws,
        "r1",
        "POST",
        "/api/v1/mobile/pair",
        None,
        Some(json!({"code": code, "name": "Pixel", "platform": "android",
                    "push_token": "ExponentPushToken[b]", "lang": "id"})),
    )
    .await;
    assert_eq!(paired["status"], 201, "{paired}");
    assert_eq!(paired["headers"]["content-type"], "application/json");
    let paired = body(&paired);
    let token = paired["token"].as_str().unwrap().to_string();
    let device_id = paired["device"]["id"].as_i64().unwrap();

    // The relay reports the token dead: the site forgets it.
    send(
        &mut ws,
        json!({"type": "unregistered",
               "devices": [{"device_id": device_id, "push_token": "ExponentPushToken[b]"}]}),
    )
    .await;
    let storage = site.storage.clone();
    eventually("push token cleared", async || {
        storage
            .get_mobile_device(device_id)
            .await
            .unwrap()
            .is_some_and(|d| d.push_token.is_none())
    })
    .await;

    let summary = forward(
        &mut ws,
        "r2",
        "GET",
        "/api/v1/mobile/summary",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(summary["status"], 200, "{summary}");
    assert!(body(&summary).is_object());

    // Not on the list: refused before the router.
    let users = forward(&mut ws, "r3", "GET", "/api/v1/users", Some(&token), None).await;
    assert_eq!(users["status"], 403);
    assert_eq!(body(&users)["code"], "path_not_allowed");

    // A request body over 64 KB.
    let big = forward(
        &mut ws,
        "r4",
        "PATCH",
        "/api/v1/mobile/device",
        Some(&token),
        Some(json!({"name": "x".repeat(70 * 1024)})),
    )
    .await;
    assert_eq!(big["status"], 413);

    // Revoked: the auth middleware refuses the token.
    assert!(site.storage.delete_mobile_device(device_id).await.unwrap());
    let revoked = forward(
        &mut ws,
        "r5",
        "GET",
        "/api/v1/mobile/summary",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(revoked["status"], 401, "{revoked}");
    assert_eq!(body(&revoked)["code"], "device_revoked");
}

#[tokio::test]
async fn requests_are_answered_concurrently() {
    let site = site().await;
    let mut ws = connected(&site).await;
    for i in 0..5 {
        send(
            &mut ws,
            json!({"type": "request", "id": format!("c{i}"), "method": "GET",
                   "path": "/api/v1/mobile/summary", "headers": {}, "body": null}),
        )
        .await;
    }
    let mut ids = Vec::new();
    for _ in 0..5 {
        let frame = recv(&mut ws).await;
        assert_eq!(frame["status"], 401, "no token");
        ids.push(frame["id"].as_str().unwrap().to_string());
    }
    ids.sort();
    assert_eq!(ids, ["c0", "c1", "c2", "c3", "c4"]);
}

#[tokio::test]
async fn a_refused_site_waits_and_a_site_without_the_tier_does_not_connect() {
    // Refused before the upgrade: no retry within the hour.
    let site = site().await;
    let (tcp, _) = site.listener.accept().await.unwrap();
    let refused = tokio_tungstenite::accept_hdr_async(tcp, |_: &Request, _: Response| {
        let mut resp = tokio_tungstenite::tungstenite::handshake::server::ErrorResponse::new(Some(
            r#"{"error":"revoked","code":"licence_revoked"}"#.to_string(),
        ));
        *resp.status_mut() = tokio_tungstenite::tungstenite::http::StatusCode::FORBIDDEN;
        Err(resp)
    })
    .await;
    assert!(refused.is_err());
    let again = tokio::time::timeout(Duration::from_millis(500), site.listener.accept()).await;
    assert!(again.is_err(), "a refused site does not retry at once");

    // Community: never connects, and has no relay for the QR code.
    let storage = Storage::new_in_memory().unwrap();
    store_activation(&storage).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let connector = Arc::new(RelayConnector::new(
        &format!("http://{}", listener.local_addr().unwrap()),
        storage,
        Arc::new(RwLock::new(LicenseStatus::Community)),
        Timing::default(),
    ));
    connector.spawn();
    let attempt = tokio::time::timeout(Duration::from_millis(500), listener.accept()).await;
    assert!(attempt.is_err(), "no connection without mobile_push");
    assert!(connector.relay().is_none());
}

#[test]
fn the_site_id_comes_from_the_activation_or_the_licence_key() {
    assert_eq!(
        super::site_id_of(&activation_blob()).as_deref(),
        Some("site-test")
    );
    let mut blob = b"VOLACT\x01".to_vec();
    blob.extend_from_slice(br#"{"license_key":"VOLTRUS-TEST"}"#);
    let id = super::site_id_of(&blob).unwrap();
    assert_eq!(id.len(), 32);
    use sha2::Digest;
    assert!(hex::encode(sha2::Sha256::digest(b"VOLTRUS-TEST")).starts_with(&id));
    assert_eq!(super::site_id_of(b"nope"), None);
}
