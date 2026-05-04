pub(crate) mod auth;
mod api_keys;
mod audit_log;
mod bacnet_devices;
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
mod analytics;
mod recipes;
mod rbac;
mod compliance;
mod enip_devices;
mod s7_devices;
mod dnp3_devices;
mod video;
mod security;
mod annotations;
mod shift_handover;
mod user_notifications;
mod commands;
mod tag_tree;
mod naming_convention;
mod simulation;
mod scripts;
mod config_snapshots;
mod ai_copilot;
mod websocket;
mod auth_middleware;
mod import_nodered;
mod csv_import;

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
    pub stream_proxy: Option<Arc<crate::video::StreamProxy>>,
    pub ws_manager: Arc<crate::ws_manager::WsManager>,
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
        .route("/api/v1/auth/providers", get(auth::list_auth_providers))
        .route("/api/v1/auth/saml/metadata", post(auth::saml_metadata))
        .route("/api/v1/auth/saml/acs", post(auth::saml_acs))
        .route("/api/v1/auth/ldap/login", post(auth::ldap_login))
        .route("/api/v1/settings/branding", get(settings::get_branding))
        .route("/api/v1/settings/alarm-cooldown", get(settings::get_alarm_cooldown))
        .route("/api/v1/settings/naming-convention", get(naming_convention::get_naming_convention))
        .route("/api/license/status", get(license::get_license_status))
        .route("/api/v1/ws", get(websocket::websocket_handler));

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
        .route("/api/v1/tags/tree", get(tag_tree::get_tag_tree))
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
        // User notifications (any authenticated user sees their own)
        .route("/api/v1/user-notifications", get(user_notifications::list_notifications))
        .route("/api/v1/user-notifications/{id}/read", put(user_notifications::mark_read))
        .route("/api/v1/user-notifications/{id}", delete(user_notifications::dismiss_notification))
        // Command read endpoints (authenticated users)
        .route("/api/v1/commands", get(commands::list_commands))
        .route("/api/v1/commands/{command_id}", get(commands::get_command_status))
        .route("/api/v1/commands/{command_id}/audit", get(commands::get_command_audit))
        .route("/api/v1/oee/metrics", get(oee::get_oee_metrics))
        .route("/api/v1/oee/downtime", get(oee::list_downtime))
        .route("/api/v1/oee/shifts", get(oee::list_shifts))
        .route("/api/v1/analytics/aggregate", get(analytics::get_aggregate))
        .route("/api/v1/analytics/compare", get(analytics::get_compare))
        .route("/api/v1/settings/ui-theme", get(settings::get_ui_theme).put(settings::update_ui_theme))
        .route("/api/v1/settings/ui-lang", get(settings::get_ui_lang).put(settings::update_ui_lang))
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
        .route("/api/v1/settings/naming-convention", put(naming_convention::update_naming_convention))
        // Alarm management: shelving, audit trail, analytics, summary
        .route("/api/v1/alarms/{id}/shelf", post(alarms::shelf_alarm))
        .route("/api/v1/alarms/{id}/unshelf", post(alarms::unshelf_alarm))
        .route("/api/v1/alarms/audit", get(alarms::list_alarm_audit))
        .route("/api/v1/alarms/analytics", get(alarms::list_alarm_analytics))
        .route("/api/v1/alarms/summary", get(alarms::get_alarm_summary))
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
        .route("/api/v1/modbus-devices/scan-network", post(modbus_devices::scan_network))
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
        // EtherNet/IP (Allen-Bradley) routes (admin-only, Enterprise-gated)
        .route("/api/v1/enip/devices", get(enip_devices::list_devices).post(enip_devices::create_device))
        .route("/api/v1/enip/devices/{id}", put(enip_devices::update_device).delete(enip_devices::delete_device))
        .route("/api/v1/enip/devices/{id}/tags", get(enip_devices::list_tags).post(enip_devices::add_tag))
        .route("/api/v1/enip/devices/{id}/tags/{tag_id}", delete(enip_devices::remove_tag))
        .route("/api/v1/enip/devices/{id}/browse", post(enip_devices::browse_device))
        .route("/api/v1/enip/devices/{id}/start", post(enip_devices::start_device))
        .route("/api/v1/enip/devices/{id}/stop", post(enip_devices::stop_device))
        // S7 (Siemens) routes (admin-only, Enterprise-gated)
        .route("/api/v1/s7/devices", get(s7_devices::list_devices).post(s7_devices::create_device))
        .route("/api/v1/s7/devices/{id}", put(s7_devices::update_device).delete(s7_devices::delete_device))
        .route("/api/v1/s7/devices/{id}/tags", get(s7_devices::list_tags).post(s7_devices::add_tag))
        .route("/api/v1/s7/devices/{id}/tags/{tag_id}", delete(s7_devices::remove_tag))
        .route("/api/v1/s7/devices/{id}/browse", post(s7_devices::browse_device))
        .route("/api/v1/s7/devices/{id}/start", post(s7_devices::start_device))
        .route("/api/v1/s7/devices/{id}/stop", post(s7_devices::stop_device))
        // DNP3 (IEEE 1815) routes (admin-only, Enterprise-gated)
        .route("/api/v1/dnp3/devices", get(dnp3_devices::list_devices).post(dnp3_devices::create_device))
        .route("/api/v1/dnp3/devices/{id}", get(dnp3_devices::get_device).put(dnp3_devices::update_device).delete(dnp3_devices::delete_device))
        .route("/api/v1/dnp3/devices/{id}/points", post(dnp3_devices::add_point))
        .route("/api/v1/dnp3/devices/{id}/points/{point_id}", delete(dnp3_devices::remove_point))
        .route("/api/v1/dnp3/scan", post(dnp3_devices::scan_network))
        .route("/api/v1/dnp3/devices/{id}/integrity", post(dnp3_devices::trigger_integrity))
        .route("/api/v1/dnp3/devices/{id}/freeze", post(dnp3_devices::trigger_freeze))
        .route("/api/v1/dnp3/group-variations", get(dnp3_devices::list_group_variations))
        // BACnet/IP routes
        .route("/api/v1/bacnet/devices", get(bacnet_devices::list_devices).post(bacnet_devices::create_device))
        .route("/api/v1/bacnet/devices/{id}", put(bacnet_devices::update_device).delete(bacnet_devices::delete_device))
        .route("/api/v1/bacnet/devices/{id}/objects", get(bacnet_devices::list_objects).post(bacnet_devices::add_object))
        .route("/api/v1/bacnet/devices/{id}/objects/{object_id}", delete(bacnet_devices::remove_object))
        .route("/api/v1/bacnet/devices/{id}/browse", post(bacnet_devices::browse_device))
        .route("/api/v1/bacnet/devices/{id}/start", post(bacnet_devices::start_device))
        .route("/api/v1/bacnet/devices/{id}/stop", post(bacnet_devices::stop_device))
        .route("/api/v1/bacnet/discover", post(bacnet_devices::discover_network))
        // RTSP Video / Camera routes (admin-only, Enterprise-gated)
        .route("/api/v1/cameras", get(video::list_cameras).post(video::create_camera))
        .route("/api/v1/cameras/{id}", put(video::update_camera).delete(video::delete_camera))
        .route("/api/v1/cameras/{id}/snapshot", post(video::trigger_snapshot))
        .route("/api/v1/cameras/{id}/snapshots", get(video::list_snapshots))
        .route("/api/v1/cameras/{id}/snapshots/{snapshot_id}", delete(video::delete_snapshot))
        .route("/api/v1/cameras/{id}/stream", get(video::get_stream_info))
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
        .route("/api/v1/recipes", get(recipes::list_recipes).post(recipes::create_recipe))
        .route("/api/v1/recipes/{id}", get(recipes::get_recipe).put(recipes::update_recipe).delete(recipes::delete_recipe))
        .route("/api/v1/recipes/{id}/load", post(recipes::load_recipe))
        // RBAC routes (admin-only, license-gated)
        .route("/api/v1/roles", get(rbac::list_roles).post(rbac::create_role))
        .route("/api/v1/roles/{id}", put(rbac::update_role).delete(rbac::delete_role))
        .route("/api/v1/users/{id}/role", put(rbac::assign_user_role))
        // Compliance routes (admin-only, license-gated)
        .route("/api/v1/compliance/config", get(compliance::get_compliance_config).put(compliance::update_compliance_config))
        .route("/api/v1/compliance/sign", post(compliance::create_electronic_signature))
        .route("/api/v1/compliance/signatures", get(compliance::list_electronic_signatures))
        // Security audit routes (admin-only, Enterprise-gated)
        .route("/api/v1/security/events", get(security::list_security_events))
        .route("/api/v1/security/events/export", get(security::export_security_csv))
        .route("/api/v1/security/thresholds", get(security::get_security_thresholds).put(security::update_security_thresholds))
        // Audit log routes (admin-only)
        .route("/api/v1/audit", get(audit_log::list_audit_logs))
        .route("/api/v1/audit/export", get(audit_log::export_audit_csv))
        // Annotation routes (admin-only, Professional-gated)
        .route("/api/v1/annotations", get(annotations::list_annotations).post(annotations::create_annotation))
        .route("/api/v1/annotations/{id}", get(annotations::get_annotation).put(annotations::update_annotation).delete(annotations::delete_annotation))
        // Shift Handover routes (admin-only, Professional-gated)
        .route("/api/v1/shift-handover", get(shift_handover::list_shift_handovers).post(shift_handover::create_shift_handover))
        .route("/api/v1/shift-handover/auto-report", get(shift_handover::auto_shift_report))
        .route("/api/v1/shift-handover/{id}", get(shift_handover::get_shift_handover))
        // Command write/cancel routes (admin-only)
        .route("/api/v1/tags/{id}/write", post(commands::write_tag))
        .route("/api/v1/tags/{id}/path", put(tag_tree::update_tag_path))
        .route("/api/v1/commands/{command_id}/cancel", post(commands::cancel_command))
        .route("/api/v1/data-sources/{id}/write-policy", patch(commands::update_write_policy))
        // Simulation routes (admin-only)
        .route("/api/v1/sim-profiles", get(simulation::list_sim_profiles).post(simulation::create_sim_profile))
        .route("/api/v1/sim-profiles/{id}", put(simulation::update_sim_profile).delete(simulation::delete_sim_profile))
        .route("/api/v1/devices/{id}/simulate", post(simulation::toggle_simulation))
        .route("/api/v1/devices/{id}/go-live", post(simulation::go_live))
        // Script routes (admin-only)
        .route("/api/v1/scripts", get(scripts::list_scripts).post(scripts::create_script))
        .route("/api/v1/scripts/{id}", put(scripts::update_script).delete(scripts::delete_script))
        .route("/api/v1/scripts/{id}/test", post(scripts::test_script))
        .route("/api/v1/scripts/{id}/executions", get(scripts::list_executions))
        // Node-RED import (admin-only)
        .route("/api/v1/import/node-red", post(import_nodered::import_node_red))
        // Config snapshot routes (admin-only)
        .route("/api/v1/config/snapshots", get(config_snapshots::list_snapshots).post(config_snapshots::create_snapshot))
        .route("/api/v1/config/snapshots/{id}", get(config_snapshots::get_snapshot))
        .route("/api/v1/config/snapshots/{from_id}/diff/{to_id}", get(config_snapshots::diff_snapshots))
        .route("/api/v1/config/snapshots/{id}/rollback", post(config_snapshots::rollback))
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

    // CSV import router with increased body limit (admin-only)
    let import_router = Router::new()
        .route("/api/v1/import/csv", post(csv_import::import_csv))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB for CSV upload
        .layer(middleware::from_fn(admin_only_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            cookie_auth_middleware,
        ));

    // API router with CORS, rate limiting, security headers, body limit
    let api_router = Router::new()
        .merge(public)
        .merge(protected)
        .merge(admin_protected)
        .merge(import_router)
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

// Middleware types re-exported from middleware module
pub(super) use auth_middleware::{admin_only_middleware, cookie_auth_middleware, AuthUser, ApiKeyAuth};

#[cfg(test)]
mod tests;
