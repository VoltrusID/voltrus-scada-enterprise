use super::*;
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_api_key_crud() {
    let storage = Storage::new_in_memory().unwrap();
    let password_hash = crate::api::auth::hash_password("admin");
    storage
        .create_user("admin", &password_hash, "", "admin")
        .await
        .unwrap();

    let cache = DeviceCache::new();
    let poller_manager = PollerManager::new(
        cache.clone(),
        storage.clone(),
        crate::network::NetworkManager::new(vec![]),
    );
    let state = AppState {
        cache,
        storage,
        poller_manager,
        started_at: std::time::Instant::now(),
        auth_secret: "test-secret".to_string(),
        cors_origin: None,
        cookie_secure: false,
        oidc_config: test_oidc_config(),
        jwks_manager: test_jwks_manager(),
        retention_config: crate::models::RetentionConfig::default(),
        flow_engine: None,
        login_rate_limiter: crate::security::LoginRateLimiter::new(),
        global_rate_limiter: crate::security::GlobalRateLimiter::new(),
        audit_logger: crate::security::AuditLogger::new().0,
        intrusion_detector: crate::security::IntrusionDetector::new(),
        license_status: crate::license::LicenseStatus::Valid {
            license_key: "polar_test_key".to_string(),
            expires_at: None,
            days_remaining: None,
            tier: Some("professional".to_string()),
            device_limit: None,
        },
        license_tier: crate::license::LicenseTier::Professional,
        feature_flags: crate::license::LicenseTier::Professional.to_feature_flags(),
        smtp_config: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
        config_path: "config.yaml".to_string(),
        hub_manager: None,
        stream_proxy: None,
        ws_manager: Arc::new(crate::ws_manager::WsManager::new()),
        mqtt_broker_state: None,
        network_manager: crate::network::NetworkManager::new(vec![]),
        health_monitor: Arc::new(crate::ha::watchdog::HealthMonitor::new(
            crate::models::WatchdogConfig::default(),
        )),
        ha_state: crate::ha::HaState::disabled(),
        widget_registry: crate::widget_sdk::WidgetRegistry::new(std::path::PathBuf::from(
            "voltrus/widgets",
        )),
        modbus_pool: std::sync::Arc::new(crate::modbus::ModbusPool::new()),
    };
    let app = create_router(state);

    // Login to get admin cookie
    let login_body = serde_json::json!({"username": "admin", "password": "admin"});
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let cookie_val = extract_cookie_value(resp.headers(), "voltrus_token")
        .expect("voltrus_token cookie not found");

    // Create API key
    let create_body = serde_json::json!({
        "name": "test-export-key",
        "permissions": "[\"data:read\", \"data:export\"]"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/api-keys")
                .header("content-type", "application/json")
                .header("cookie", format!("voltrus_token={}", cookie_val))
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["name"], "test-export-key");
    assert!(json["key"].as_str().unwrap().len() == 64);
    let plain_key = json["key"].as_str().unwrap().to_string();
    let key_id = json["id"].as_i64().unwrap();

    // List API keys
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/api-keys")
                .header("cookie", format!("voltrus_token={}", cookie_val))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Use API key to access protected route (X-API-Key header)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/system")
                .header("X-API-Key", &plain_key)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Use API key via query parameter
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/system?api_key={}", plain_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Invalid API key should fail
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/system")
                .header("X-API-Key", "invalid-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Revoke API key
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/api-keys/{}", key_id))
                .header("cookie", format!("voltrus_token={}", cookie_val))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Revoked key should no longer work
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/system")
                .header("X-API-Key", &plain_key)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
