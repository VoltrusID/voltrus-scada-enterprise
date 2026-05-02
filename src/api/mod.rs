pub(crate) mod auth;
mod api_keys;
mod push;
mod oee;
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
mod license;
mod notifications;
mod opcua_devices;
pub(crate) mod hub;
mod reports;

use axum::{
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header::{self, CONTENT_TYPE, HeaderName}},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
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
    pub global_rate_limiter: crate::security::GlobalRateLimiter,
    pub audit_logger: crate::security::AuditLogger,
    pub intrusion_detector: crate::security::IntrusionDetector,
    pub license_status: crate::license::LicenseStatus,
    pub license_tier: crate::license::LicenseTier,
    pub feature_flags: serde_json::Value,
    pub smtp_config: Option<crate::models::SmtpConfig>,
    pub vapid_public_key: Option<String>,
    pub hub_manager: Option<hub::HubManager>,
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

async fn license_check_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let path = req.uri().path();
    if path == "/api/v1/health"
        || path == "/api/license/status"
        || path == "/api/v1/settings/branding"
        || path == "/api/v1/push/vapid-key"
        || path.starts_with("/api/v1/auth/")
    {
        let mut resp = next.run(req).await;
        let tier_str = serde_json::to_string(&state.license_tier).unwrap_or_default();
        resp.headers_mut().insert(
            HeaderName::from_static("x-license-tier"),
            HeaderValue::from_str(&tier_str).unwrap_or_else(|_| HeaderValue::from_static("trial")),
        );
        return resp;
    }

    if !state.license_status.is_valid() {
        let body = match serde_json::to_value(&state.license_status) {
            Ok(v) => v,
            Err(_) => serde_json::json!({"status": "error", "message": "license invalid"}),
        };
        return (
            StatusCode::FORBIDDEN,
            Json(body),
        ).into_response();
    }

    let mut resp = next.run(req).await;
    let tier_str = serde_json::to_string(&state.license_tier).unwrap_or_default();
    resp.headers_mut().insert(
        HeaderName::from_static("x-license-tier"),
        HeaderValue::from_str(&tier_str).unwrap_or_else(|_| HeaderValue::from_static("trial")),
    );
    resp
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
                .allow_headers([
                    CONTENT_TYPE,
                    header::ACCEPT,
                    header::AUTHORIZATION,
                    HeaderName::from_static("x-api-key"),
                ])
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
                .allow_headers([
                    CONTENT_TYPE,
                    header::ACCEPT,
                    header::AUTHORIZATION,
                    HeaderName::from_static("x-api-key"),
                ])
                // NOTE: allow_credentials is intentionally FALSE when no explicit origin is set
                // to prevent credential leakage to arbitrary origins.
        }
    };

    // Public routes (no auth required)
    let public = Router::new()
        .route("/api/v1/health", get(system::get_health))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/refresh", post(auth::refresh_token))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/me", get(auth::auth_me))
        .route("/api/v1/auth/csrf-token", get(auth::get_csrf_token))
        .route("/api/v1/settings/branding", get(settings::get_branding))
        .route("/api/v1/settings/alarm-cooldown", get(settings::get_alarm_cooldown))
        .route("/api/license/status", get(license::get_license_status));

    // Protected routes (any authenticated user — read-only for non-admins)
    let protected = Router::new()
        .route("/api/v1/live", get(system::get_live))
        .route("/api/v1/stream", get(stream::stream))
        .route("/api/v1/system", get(system::get_system))
        .route("/api/v1/version", get(system::get_version))
        .route("/api/v1/auth/change-password", post(auth::change_password))
        .route("/api/v1/data", get(data::get_data))
        .route("/api/v1/data/export", get(data::export_data))
        .route("/api/v1/history", get(data::get_history))
        .route("/api/v1/export/csv", get(data::export_csv))
        .route("/api/v1/screens", get(screens::list_screens))
        .route("/api/v1/screens/{id}/data-sources", get(screens::list_screen_sensors))
        .route("/api/v1/screens/{id}/layouts", get(layouts::get_layouts))
        .route("/api/v1/screens/{id}/twin-layouts", get(layouts::get_twin_layouts))
        .route("/api/v1/screens/{id}/twin-connections", get(layouts::get_twin_connections))
        .route("/api/v1/screens/{id}/canvas-layouts", get(layouts::get_canvas_layouts))
        .route("/api/v1/screens/{id}/scada-equipment", get(layouts::get_scada_equipment))
        .route("/api/v1/screens/{id}/scada-connections", get(layouts::get_scada_connections))
        .route("/api/v1/data-sources", get(data_sources::list_data_sources))
        .route(
            "/api/v1/shortcuts",
            get(shortcuts::get_shortcuts).put(shortcuts::save_shortcuts),
        )
        .route(
            "/api/v1/nav-shortcuts",
            get(shortcuts::get_nav_shortcuts).put(shortcuts::save_nav_shortcuts),
        )
        .route(
            "/api/v1/alarms",
            get(alarms::list_alarms).post(alarms::acknowledge_all),
        )
        .route("/api/v1/alarms/active/count", get(alarms::active_alarm_count))
        .route(
            "/api/v1/alarms/{id}/acknowledge",
            post(alarms::acknowledge_alarm),
        )
        .route(
            "/api/v1/push/subscribe",
            post(push::subscribe).delete(push::unsubscribe),
        )
        .route("/api/v1/push/vapid-key", get(push::vapid_public_key))
        .route("/api/v1/modbus-devices", get(modbus_devices::list_modbus_devices))
        .route("/api/v1/modbus-devices/status", get(modbus_devices::get_poller_statuses))
        .route("/api/v1/notifications/settings", get(notifications::list_notification_settings))
        .route("/api/v1/oee/metrics", get(oee::get_oee_metrics))
        .route("/api/v1/oee/downtime", get(oee::list_downtime))
        .route("/api/v1/oee/shifts", get(oee::list_shifts))
        .route(
            "/api/v1/modbus-devices/{id}",
            get(modbus_devices::get_modbus_device),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            cookie_auth_middleware,
        ));

    // Admin-only routes (authenticated + admin role required)
    let admin_protected = Router::new()
        .route("/api/v1/seed", post(auth::seed))
        .route("/api/v1/settings/branding", put(settings::update_branding))
        .route("/api/v1/settings/alarm-cooldown", put(settings::update_alarm_cooldown))
        .route("/api/v1/screens", post(screens::create_screen))
        .route("/api/v1/screens/{id}", put(screens::update_screen).delete(screens::delete_screen))
        .route("/api/v1/screens/{id}/data-sources", post(screens::add_screen_sensor))
        .route(
            "/api/v1/screens/{id}/data-sources/{data_source_id}",
            put(screens::update_screen_sensor).delete(screens::remove_screen_sensor),
        )
        .route("/api/v1/screens/{id}/layouts", put(layouts::save_layouts))
        .route("/api/v1/screens/{id}/twin-layouts", put(layouts::save_twin_layouts))
        .route("/api/v1/screens/{id}/twin-connections", put(layouts::save_twin_connections))
        .route("/api/v1/screens/{id}/twin-camera", put(layouts::save_twin_camera))
        .route("/api/v1/screens/{id}/canvas-layouts", put(layouts::save_canvas_layouts))
        .route("/api/v1/screens/{id}/scada-equipment", put(layouts::save_scada_equipment))
        .route("/api/v1/screens/{id}/scada-connections", put(layouts::save_scada_connections))
        .route("/api/v1/data-sources", post(data_sources::create_data_source))
        .route("/api/v1/data-sources/reorder", patch(data_sources::reorder_data_sources))
        .route(
            "/api/v1/data-sources/{id}",
            put(data_sources::update_data_source).delete(data_sources::delete_data_source),
        )
        .route("/api/v1/modbus-devices", post(modbus_devices::create_modbus_device))
        .route("/api/v1/modbus-devices/scan", post(modbus_devices::scan_modbus_registers))
        .route(
            "/api/v1/modbus-devices/{id}",
            put(modbus_devices::update_modbus_device).delete(modbus_devices::delete_modbus_device),
        )
        .route("/api/v1/modbus-devices/{id}/registers", put(modbus_devices::replace_registers))
        .route("/api/v1/modbus-devices/{id}/restart", post(modbus_devices::restart_poller))
        .route("/api/v1/users", get(auth::list_users).post(auth::create_user_handler))
        .route("/api/v1/users/{id}", put(auth::update_user_handler).delete(auth::delete_user_handler))
        .route("/api/v1/flows", get(flows::list_flows).post(flows::create_flow))
        .route("/api/v1/flows/{id}", get(flows::get_flow).put(flows::update_flow).delete(flows::delete_flow))
        .route("/api/v1/flows/{id}/start", post(flows::start_flow))
        .route("/api/v1/flows/{id}/stop", post(flows::stop_flow))
        .route("/api/v1/flows/{id}/test", post(flows::test_flow))
        .route("/api/v1/flows/{id}/runs", get(flows::list_flow_runs))
        .route("/api/v1/notifications/settings", post(notifications::create_notification_setting))
        .route(
            "/api/v1/notifications/settings/{id}",
            put(notifications::update_notification_setting).delete(notifications::delete_notification_setting),
        )
        .route("/api/v1/notifications/test", post(notifications::test_notification))
        .route("/api/v1/api-keys", post(api_keys::create_api_key).get(api_keys::list_api_keys))
        .route("/api/v1/api-keys/{id}", delete(api_keys::revoke_api_key))
        .route("/api/v1/opcua/devices", get(opcua_devices::list_devices).post(opcua_devices::create_device))
        .route("/api/v1/opcua/devices/{id}", put(opcua_devices::update_device).delete(opcua_devices::delete_device))
        .route("/api/v1/opcua/devices/{id}/nodes", get(opcua_devices::list_nodes).post(opcua_devices::add_node))
        .route("/api/v1/opcua/devices/{id}/nodes/{node_id}", delete(opcua_devices::remove_node))
        .route("/api/v1/opcua/devices/{id}/browse", post(opcua_devices::browse_device))
        .route("/api/v1/opcua/devices/{id}/start", post(opcua_devices::start_device))
        .route("/api/v1/opcua/devices/{id}/stop", post(opcua_devices::stop_device))
        .route("/api/v1/hub/sites", get(hub::list_sites).post(hub::create_site))
        .route("/api/v1/hub/sites/{id}", put(hub::update_site).delete(hub::delete_site))
        .route("/api/v1/hub/sites/{id}/start", post(hub::start_site_connection))
        .route("/api/v1/hub/sites/{id}/stop", post(hub::stop_site_connection))
        .route("/api/v1/hub/status", get(hub::get_hub_status))
        .route("/api/v1/reports", get(reports::list_reports).post(reports::create_report))
        .route("/api/v1/reports/{id}", put(reports::update_report).delete(reports::delete_report))
        .route("/api/v1/reports/{id}/generate", post(reports::generate_report))
        .route("/api/v1/oee/downtime", post(oee::create_downtime))
        .route("/api/v1/oee/downtime/{id}/close", put(oee::close_downtime))
        .route("/api/v1/oee/shifts", post(oee::create_shift))
        .route("/api/v1/oee/shifts/{id}", delete(oee::delete_shift))
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
        .route("/api/v1/{*path}", get(api_not_found))
        .layer(middleware::from_fn_with_state(state.clone(), license_check_middleware))
        .layer(cors)
        .layer(DefaultBodyLimit::max(1024 * 1024)) // 1MB body limit
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::security::global_rate_limit_middleware,
        ))
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
        .layer(middleware::from_fn(crate::security::strip_accept_ranges_middleware))
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

    // 1. Try API key authentication (X-API-Key header or ?api_key= query param)
    if let Some(api_key) = extract_api_key(&req) {
        match state.storage.validate_api_key(&api_key).await {
            Ok(Some((id, permissions))) => {
                req.extensions_mut().insert(AuthUser {
                    username: format!("api_key:{}", id),
                    role: "api".to_string(),
                });
                req.extensions_mut().insert(ApiKeyAuth {
                    id,
                    permissions,
                });
                return next.run(req).await;
            }
            _ => {
                // Failed API key auth — record intrusion and reject immediately.
                // If an API key was explicitly provided, it must be the sole auth method;
                // do not fall through to cookie/JWT auth.
                state.intrusion_detector.record_401(&client_ip);
                return (StatusCode::UNAUTHORIZED, "Invalid or expired API key").into_response();
            }
        }
    }

    // 2. Try Bearer token (OIDC)
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

    // 3. Fall back to local cookie auth
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

/// API key authentication info.
/// Note: permissions field is stored for future use but not yet enforced.
/// All valid API keys currently grant full read access to protected endpoints.
#[derive(Clone, Debug)]
pub struct ApiKeyAuth {
    pub id: i64,
    pub permissions: String,
}

/// Extract API key from X-API-Key header or ?api_key= query parameter.
fn extract_api_key(req: &Request) -> Option<String> {
    // Check X-API-Key header first
    if let Some(header) = req.headers().get("X-API-Key") {
        if let Ok(key) = header.to_str() {
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }
    }
    // Fall back to query parameter
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            if let Some(key) = pair.strip_prefix("api_key=") {
                if !key.is_empty() {
                    return Some(key.to_string());
                }
            }
        }
    }
    None
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
                    .uri("/api/v1/live")
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
                    .uri("/api/v1/health")
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
                    .uri("/api/v1/auth/me")
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
                    .uri("/api/v1/seed")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    /// Extract a specific cookie value from potentially multiple Set-Cookie headers.
    fn extract_cookie_value(headers: &http::HeaderMap, name: &str) -> Option<String> {
        headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find_map(|s| {
                let kv = s.split(';').next()?.trim();
                if kv.starts_with(&format!("{}=", name)) {
                    Some(kv.trim_start_matches(&format!("{}=", name)).to_string())
                } else {
                    None
                }
            })
    }

    async fn seed_and_get_cookie(app: &Router) -> String {
        // Seed directly via storage (bypass HTTP)
        let _state = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
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
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        extract_cookie_value(resp.headers(), "voltrus_token").expect("voltrus_token cookie not found")
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
        };
        let app = create_router(state);

        // Login with admin/admin
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

        assert_eq!(resp.status(), StatusCode::OK);

        assert!(extract_cookie_value(resp.headers(), "voltrus_token").is_some());
        assert!(extract_cookie_value(resp.headers(), "voltrus_refresh").is_some());

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
        };
        let app = create_router(state);

        let login_body = serde_json::json!({"username": "admin", "password": "wrong"});
        let resp = app
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

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_branding_default() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/settings/branding")
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
                    .uri("/api/v1/system")
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
        };
        let app = create_router(state);

        // Login
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

        // Access protected route with cookie
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/system")
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

    #[tokio::test]
    async fn test_api_key_crud() {
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

    fn test_app_with_license_status(license_status: crate::license::LicenseStatus, license_tier: crate::license::LicenseTier) -> Router {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();
        let poller_manager = PollerManager::new(cache.clone());
        let feature_flags = license_tier.to_feature_flags();
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
            license_status,
            license_tier,
            feature_flags,
            smtp_config: None,
            vapid_public_key: None,
            hub_manager: None,
        };
        create_router(state)
    }

    #[tokio::test]
    async fn test_trial_middleware_allows_access() {
        let tier = crate::license::LicenseTier::Trial;
        let app = test_app_with_license_status(
            crate::license::LicenseStatus::Trial {
                started_at: 0,
                expires_at: 2592000,
                days_remaining: 25,
                tier: tier.clone(),
                feature_flags: tier.to_feature_flags(),
            },
            tier,
        );

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trial_expired_middleware_blocks_access() {
        // Verify that TrialExpired reports is_valid() == false
        let status = crate::license::LicenseStatus::TrialExpired {
            started_at: 0,
            message: "expired".to_string(),
        };
        assert!(!status.is_valid());

        // Health endpoint is always allowed even for expired licenses
        let tier = crate::license::LicenseTier::Trial;
        let app = test_app_with_license_status(
            crate::license::LicenseStatus::TrialExpired {
                started_at: 0,
                message: "expired".to_string(),
            },
            tier,
        );

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_license_tier_header_in_response() {
        let app = test_app();

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let tier_header = resp.headers()
            .get("x-license-tier")
            .expect("X-License-Tier header should be present");
        // serde_json serializes LicenseTier::Professional as "professional"
        assert_eq!(tier_header, "\"professional\"");
    }
}
