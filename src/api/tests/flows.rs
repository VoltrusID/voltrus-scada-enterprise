use super::*;
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_flow_crud() {
    let storage = Storage::new_in_memory().unwrap();
    let password_hash = crate::api::auth::hash_password("admin");
    storage.create_user("admin", &password_hash, "", "admin").await.unwrap();

    let cache = DeviceCache::new();
    let poller_manager = PollerManager::new(cache.clone(), storage.clone());
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
        smtp_config: None,
        vapid_public_key: None,
        hub_manager: None,
        stream_proxy: None,
        ws_manager: Arc::new(crate::ws_manager::WsManager::new()),
        mqtt_broker_state: None,
    };
    let app = create_router(state);

    // Login to get auth cookie
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

    // Create flow
    let flow = serde_json::json!({
        "id": "test_flow",
        "name": "Test Flow",
        "enabled": true,
        "trigger_config": serde_json::json!({"type": "modbus", "device_id": "*"}).to_string(),
        "nodes": serde_json::json!([
            {"id": "n1", "type": "filter", "condition": "metrics.temp > 0"}
        ]).to_string(),
        "created_at": 0,
        "updated_at": 0,
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/flows")
                .header("content-type", "application/json")
                .header("cookie", format!("voltrus_token={}", cookie_val))
                .body(Body::from(serde_json::to_string(&flow).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Get flow
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/flows/test_flow")
                .header("cookie", format!("voltrus_token={}", cookie_val))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Test flow
    let test_payload = serde_json::json!({
        "payload": {
            "metrics": {
                "temp": 25.0
            }
        }
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/flows/test_flow/test")
                .header("content-type", "application/json")
                .header("cookie", format!("voltrus_token={}", cookie_val))
                .body(Body::from(serde_json::to_string(&test_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);

    // Delete flow
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/flows/test_flow")
                .header("cookie", format!("voltrus_token={}", cookie_val))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
