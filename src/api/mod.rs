pub(crate) mod auth;
mod push;
mod screens;
mod data_sources;
mod layouts;
mod shortcuts;
mod settings;
mod alarms;
mod modbus_devices;
mod data;
mod system;
mod generators;
pub(crate) mod stream;
mod flows;

use axum::{
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header::{self, CONTENT_TYPE}},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, patch, post, put},
    Json, Router,
};
use axum::extract::{ConnectInfo, Extension, DefaultBodyLimit};
use axum_extra::extract::CookieJar;
use serde_json::json;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::auth::oidc::{JwksManager, OidcConfig};
use crate::cache::DeviceCache;
use crate::poller_manager::PollerManager;
use crate::storage::Storage;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub cache: DeviceCache,
    pub storage: Storage,
    pub poller_manager: PollerManager,
    pub started_at: std::time::Instant,
    pub auth_secret: String,
    pub cors_origin: Option<String>,
    pub cookie_secure: bool,
    pub oidc_config: OidcConfig,
    pub jwks_manager: JwksManager,
    pub retention_config: crate::models::RetentionConfig,
    pub flow_engine: Option<Arc<crate::flow_engine::engine::FlowEngine>>,
    pub login_rate_limiter: crate::security::LoginRateLimiter,
    pub audit_logger: crate::security::AuditLogger,
    pub intrusion_detector: crate::security::IntrusionDetector,
}

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{:#}", self.0)})),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        AppError(err.into())
    }
}

pub fn create_router(state: AppState) -> Router {
    // Build CORS layer only for API routes — static assets should not have CORS headers
    let cors = match &state.cors_origin {
        Some(origin) => {
            let origins: Vec<HeaderValue> = origin
                .split(',')
                .map(|o| o.trim().parse().expect("Invalid cors_origin value"))
                .collect();
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
                .allow_headers([CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION])
                .allow_credentials(true)
        }
        None => {
            // In production, cors_origin MUST be configured. This default only allows
            // local development origins and does NOT allow credentials.
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:5173".parse().unwrap(),
                    "http://localhost:5174".parse().unwrap(),
                    "http://127.0.0.1:5173".parse().unwrap(),
                    "http://127.0.0.1:5174".parse().unwrap(),
                ])
                .allow_methods([
                    Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE,
                ])
                .allow_headers([CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION])
                // NOTE: allow_credentials is intentionally FALSE when no explicit origin is set
                // to prevent credential leakage to arbitrary origins.
        }
    };

    // Public routes (no auth required)
    let public = Router::new()
        .route("/api/health", get(system::get_health))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::auth_me))
        .route("/api/auth/csrf-token", get(auth::get_csrf_token))
        .route("/api/settings/branding", get(settings::get_branding))
        .route("/api/settings/alarm-cooldown", get(settings::get_alarm_cooldown));

    // Protected routes (any authenticated user — read-only for non-admins)
    let protected = Router::new()
        .route("/api/live", get(system::get_live))
        .route("/api/stream", get(stream::stream))
        .route("/api/system", get(system::get_system))
        .route("/api/version", get(system::get_version))
        .route("/api/auth/change-password", post(auth::change_password))
        .route("/api/data", get(data::get_data))
        .route("/api/history", get(data::get_history))
        .route("/api/export/csv", get(data::export_csv))
        .route("/api/screens", get(screens::list_screens))
        .route("/api/screens/{id}/data-sources", get(screens::list_screen_sensors))
        .route("/api/screens/{id}/layouts", get(layouts::get_layouts))
        .route("/api/screens/{id}/twin-layouts", get(layouts::get_twin_layouts))
        .route("/api/screens/{id}/twin-connections", get(layouts::get_twin_connections))
        .route("/api/screens/{id}/canvas-layouts", get(layouts::get_canvas_layouts))
        .route("/api/screens/{id}/scada-equipment", get(layouts::get_scada_equipment))
        .route("/api/screens/{id}/scada-connections", get(layouts::get_scada_connections))
        .route("/api/data-sources", get(data_sources::list_data_sources))
        .route(
            "/api/shortcuts",
            get(shortcuts::get_shortcuts).put(shortcuts::save_shortcuts),
        )
        .route(
            "/api/nav-shortcuts",
            get(shortcuts::get_nav_shortcuts).put(shortcuts::save_nav_shortcuts),
        )
        .route(
            "/api/alarms",
            get(alarms::list_alarms).post(alarms::acknowledge_all),
        )
        .route("/api/alarms/active/count", get(alarms::active_alarm_count))
        .route(
            "/api/alarms/{id}/acknowledge",
            post(alarms::acknowledge_alarm),
        )
        .route(
            "/api/push/subscribe",
            post(push::subscribe).delete(push::unsubscribe),
        )
        .route("/api/modbus-devices", get(modbus_devices::list_modbus_devices))
        .route("/api/modbus-devices/status", get(modbus_devices::get_poller_statuses))
        .route(
            "/api/modbus-devices/{id}",
            get(modbus_devices::get_modbus_device),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            cookie_auth_middleware,
        ));

    // Admin-only routes (authenticated + admin role required)
    let admin_protected = Router::new()
        .route("/api/seed", post(auth::seed))
        .route("/api/settings/branding", put(settings::update_branding))
        .route("/api/settings/alarm-cooldown", put(settings::update_alarm_cooldown))
        .route("/api/screens", post(screens::create_screen))
        .route("/api/screens/{id}", put(screens::update_screen).delete(screens::delete_screen))
        .route("/api/screens/{id}/data-sources", post(screens::add_screen_sensor))
        .route(
            "/api/screens/{id}/data-sources/{data_source_id}",
            put(screens::update_screen_sensor).delete(screens::remove_screen_sensor),
        )
        .route("/api/screens/{id}/layouts", put(layouts::save_layouts))
        .route("/api/screens/{id}/twin-layouts", put(layouts::save_twin_layouts))
        .route("/api/screens/{id}/twin-connections", put(layouts::save_twin_connections))
        .route("/api/screens/{id}/twin-camera", put(layouts::save_twin_camera))
        .route("/api/screens/{id}/canvas-layouts", put(layouts::save_canvas_layouts))
        .route("/api/screens/{id}/scada-equipment", put(layouts::save_scada_equipment))
        .route("/api/screens/{id}/scada-connections", put(layouts::save_scada_connections))
        .route("/api/data-sources", post(data_sources::create_data_source))
        .route("/api/data-sources/reorder", patch(data_sources::reorder_data_sources))
        .route(
            "/api/data-sources/{id}",
            put(data_sources::update_data_source).delete(data_sources::delete_data_source),
        )
        .route("/api/modbus-devices", post(modbus_devices::create_modbus_device))
        .route("/api/modbus-devices/scan", post(modbus_devices::scan_modbus_registers))
        .route(
            "/api/modbus-devices/{id}",
            put(modbus_devices::update_modbus_device).delete(modbus_devices::delete_modbus_device),
        )
        .route("/api/modbus-devices/{id}/registers", put(modbus_devices::replace_registers))
        .route("/api/modbus-devices/{id}/restart", post(modbus_devices::restart_poller))
        .route("/api/users", get(auth::list_users).post(auth::create_user_handler))
        .route("/api/users/{id}", put(auth::update_user_handler).delete(auth::delete_user_handler))
        .route("/api/flows", get(flows::list_flows).post(flows::create_flow))
        .route("/api/flows/{id}", get(flows::get_flow).put(flows::update_flow).delete(flows::delete_flow))
        .route("/api/flows/{id}/start", post(flows::start_flow))
        .route("/api/flows/{id}/stop", post(flows::stop_flow))
        .route("/api/flows/{id}/test", post(flows::test_flow))
        .route("/api/flows/{id}/runs", get(flows::list_flow_runs))
        .layer(middleware::from_fn(admin_only_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            cookie_auth_middleware,
        ));

    async fn api_not_found() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not found"})),
        )
    }

    // API router with CORS, rate limiting, security headers, body limit
    let api_router = Router::new()
        .merge(public)
        .merge(protected)
        .merge(admin_protected)
        .route("/api/{*path}", get(api_not_found))
        .layer(cors)
        .layer(DefaultBodyLimit::max(1024 * 1024)) // 1MB body limit
        .layer(middleware::from_fn(crate::security::request_id_middleware))
        .layer(middleware::from_fn(crate::security::security_headers_middleware))
        .layer(middleware::from_fn(crate::security::strip_forwarded_host_middleware));

    // Static file serving WITHOUT CORS headers
    let static_router = Router::new()
        .nest_service(
            "/lit",
            ServeDir::new("client-lit").fallback(ServeFile::new("client-lit/index.html")),
        )
        .fallback_service(
            ServeDir::new("static").fallback(ServeFile::new("static/index.html")),
        )
        .layer(middleware::from_fn(crate::security::block_source_maps_middleware))
        .layer(middleware::from_fn(crate::security::limit_range_requests_middleware))
        .layer(middleware::from_fn(crate::security::security_headers_middleware))
        .layer(middleware::from_fn(crate::security::strip_forwarded_host_middleware));

    Router::new()
        .merge(api_router)
        .merge(static_router)
        .with_state(state)
}

async fn admin_only_middleware(
    Extension(auth): Extension<AuthUser>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    if auth.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "admin access required"})),
        )
            .into_response();
    }
    next.run(req).await
}

#[derive(Clone)]
pub struct AuthUser {
    pub username: String,
    pub role: String,
}

async fn cookie_auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    // Extract client IP if available (may not be present in test oneshot)
    let client_ip = req.extensions().get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut req = req;

    // Try Bearer token (OIDC) first
    if state.oidc_config.enabled {
        if let Some(auth_header) = req.headers().get("authorization") {
            if let Some(token) = auth_header.to_str().ok().and_then(|v| v.strip_prefix("Bearer ")) {
                if let Some(claims) = state.jwks_manager.validate_token(token, &state.oidc_config).await {
                    let role = crate::auth::oidc::extract_role(&claims, &state.oidc_config);
                    req.extensions_mut().insert(AuthUser {
                        username: claims.sub,
                        role,
                    });
                    return next.run(req).await;
                }
            }
        }
    }

    // Fall back to local cookie auth
    let jar = CookieJar::from_headers(req.headers());
    let token = jar
        .get("voltrus_token")
        .map(|c| c.value())
        .unwrap_or("");
    if token.is_empty() {
        state.intrusion_detector.record_401(&client_ip);
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        )
            .into_response();
    }
    match auth::verify_token(token, &state.auth_secret) {
        Some(claims) => {
            req.extensions_mut().insert(AuthUser {
                username: claims.username,
                role: claims.role,
            });
            next.run(req).await
        }
        None => {
            state.intrusion_detector.record_401(&client_ip);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "unauthorized"})),
            )
                .into_response()
        }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::{Request, StatusCode};
    use std::collections::HashMap;
    use tower::ServiceExt;

    fn test_oidc_config() -> OidcConfig {
        OidcConfig {
            enabled: false,
            issuer: String::new(),
            audience: String::new(),
            jwks_url: String::new(),
            role_claim: "realm_access.roles".to_string(),
            admin_role: "voltrus-admin".to_string(),
        }
    }

    fn test_jwks_manager() -> JwksManager {
        JwksManager::new("")
    }

    pub fn test_app() -> Router {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();
        let poller_manager = PollerManager::new(cache.clone());
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
            audit_logger: crate::security::AuditLogger::new().0,
            intrusion_detector: crate::security::IntrusionDetector::new(),
        };
        create_router(state)
    }

    pub async fn seed_cache(cache: &DeviceCache) {
        let metrics = HashMap::from([
            ("voltage".to_string(), 220.5),
            ("current".to_string(), 5.2),
        ]);
        cache.update("plts_1", metrics).await;
    }

    #[tokio::test]
    async fn test_live_empty() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_health_open() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_auth_me_unauthenticated() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["authenticated"], false);
    }

    #[tokio::test]
    async fn test_seed_requires_auth() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/seed")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    async fn seed_and_get_cookie(app: &Router) -> String {
        // Seed directly via storage (bypass HTTP)
        let _state = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .ok();

        // We need to seed via storage - use the login approach with direct seed
        // For tests, we create a test-specific approach using the seed endpoint
        // after obtaining a token through direct storage seeding
        // Simplified: use a separate test helper
        let storage = Storage::new_in_memory().unwrap();
        let password_hash = crate::api::auth::hash_password("admin");
        storage.create_user("admin", &password_hash, "", "admin").await.unwrap();

        // Login
        let login_body = serde_json::json!({"username": "admin", "password": "admin"});
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        set_cookie
            .split(';')
            .next()
            .unwrap()
            .trim()
            .trim_start_matches("voltrus_token=")
            .to_string()
    }

    #[tokio::test]
    async fn test_login_with_seeded_user() {
        let storage = Storage::new_in_memory().unwrap();
        let password_hash = crate::api::auth::hash_password("admin");
        storage.create_user("admin", &password_hash, "", "admin").await.unwrap();

        let cache = DeviceCache::new();
        let poller_manager = PollerManager::new(cache.clone());
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
            audit_logger: crate::security::AuditLogger::new().0,
            intrusion_detector: crate::security::IntrusionDetector::new(),
        };
        let app = create_router(state);

        // Login with admin/admin
        let login_body = serde_json::json!({"username": "admin", "password": "admin"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .expect("should have set-cookie")
            .to_str()
            .unwrap()
            .to_string();
        assert!(set_cookie.contains("voltrus_token="));

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["username"], "admin");
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let storage = Storage::new_in_memory().unwrap();
        let password_hash = crate::api::auth::hash_password("admin");
        storage.create_user("admin", &password_hash, "", "admin").await.unwrap();

        let cache = DeviceCache::new();
        let poller_manager = PollerManager::new(cache.clone());
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
            audit_logger: crate::security::AuditLogger::new().0,
            intrusion_detector: crate::security::IntrusionDetector::new(),
        };
        let app = create_router(state);

        let login_body = serde_json::json!({"username": "admin", "password": "wrong"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_branding_default() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/settings/branding")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["title"], "Voltrus Lite");
        assert_eq!(json["subtitle"], "Monitoring Dashboard");
    }

    #[tokio::test]
    async fn test_protected_route_requires_cookie() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/system")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_full_auth_flow() {
        let storage = Storage::new_in_memory().unwrap();
        let password_hash = crate::api::auth::hash_password("admin");
        storage.create_user("admin", &password_hash, "", "admin").await.unwrap();

        let cache = DeviceCache::new();
        let poller_manager = PollerManager::new(cache.clone());
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
            audit_logger: crate::security::AuditLogger::new().0,
            intrusion_detector: crate::security::IntrusionDetector::new(),
        };
        let app = create_router(state);

        // Login
        let login_body = serde_json::json!({"username": "admin", "password": "admin"});
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Extract cookie value
        let cookie_val = set_cookie
            .split(';')
            .next()
            .unwrap()
            .trim()
            .trim_start_matches("voltrus_token=")
            .to_string();

        // Access protected route with cookie
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/system")
                    .header("cookie", format!("voltrus_token={}", cookie_val))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_flow_crud() {
        let storage = Storage::new_in_memory().unwrap();
        let password_hash = crate::api::auth::hash_password("admin");
        storage.create_user("admin", &password_hash, "", "admin").await.unwrap();

        let cache = DeviceCache::new();
        let poller_manager = PollerManager::new(cache.clone());
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
            audit_logger: crate::security::AuditLogger::new().0,
            intrusion_detector: crate::security::IntrusionDetector::new(),
        };
        let app = create_router(state);

        // Login to get auth cookie
        let login_body = serde_json::json!({"username": "admin", "password": "admin"});
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .expect("should have set-cookie")
            .to_str()
            .unwrap()
            .to_string();

        let cookie_val = set_cookie
            .split(';')
            .next()
            .unwrap()
            .trim()
            .trim_start_matches("voltrus_token=")
            .to_string();

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
                    .uri("/api/flows")
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
                    .uri("/api/flows/test_flow")
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
                    .uri("/api/flows/test_flow/test")
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
                    .uri("/api/flows/test_flow")
                    .header("cookie", format!("voltrus_token={}", cookie_val))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }
}
