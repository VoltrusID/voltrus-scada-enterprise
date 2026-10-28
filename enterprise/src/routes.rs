use axum::Router;
use axum::routing::{delete, get, post, put};
use voltrus::api::AppState;
use voltrus::edition::ExtensionRoutes;

use crate::{audit, branding, compliance, ha, hub, licensing, reports, sso, video};

pub(crate) fn routes() -> ExtensionRoutes {
    ExtensionRoutes {
        public: public(),
        protected: protected(),
        operator: operator(),
        admin: admin(),
    }
}

fn public() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/auth/saml/metadata",
            get(sso::handlers::saml_metadata),
        )
        .route("/api/v1/auth/saml/login", get(sso::handlers::saml_login))
        .route("/api/v1/auth/saml/acs", post(sso::handlers::saml_acs))
        .route("/api/v1/auth/oidc/login", get(sso::handlers::oidc_login))
        .route(
            "/api/v1/auth/oidc/callback",
            get(sso::handlers::oidc_callback),
        )
        .route("/api/v1/auth/ldap/login", post(sso::handlers::ldap_login))
        .route(
            "/api/v1/settings/branding",
            get(branding::handlers::get_branding),
        )
}

fn protected() -> Router<AppState> {
    Router::new().route("/api/v1/ha/status", get(ha::ha_api::ha_status))
}

fn operator() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/settings/branding",
            put(branding::handlers::update_branding),
        )
        .route(
            "/api/v1/cameras",
            get(video::handlers::list_cameras).post(video::handlers::create_camera),
        )
        .route(
            "/api/v1/cameras/{id}",
            put(video::handlers::update_camera).delete(video::handlers::delete_camera),
        )
        .route(
            "/api/v1/cameras/{id}/snapshot",
            post(video::handlers::trigger_snapshot),
        )
        .route(
            "/api/v1/cameras/{id}/snapshots",
            get(video::handlers::list_snapshots),
        )
        .route(
            "/api/v1/cameras/{id}/snapshots/{snapshot_id}",
            delete(video::handlers::delete_snapshot),
        )
        .route(
            "/api/v1/cameras/{id}/stream",
            get(video::handlers::get_stream_info),
        )
        .route(
            "/api/v1/hub/sites",
            get(hub::handlers::list_sites).post(hub::handlers::create_site),
        )
        .route(
            "/api/v1/hub/sites/{id}",
            put(hub::handlers::update_site).delete(hub::handlers::delete_site),
        )
        .route(
            "/api/v1/hub/sites/{id}/start",
            post(hub::handlers::start_site_connection),
        )
        .route(
            "/api/v1/hub/sites/{id}/stop",
            post(hub::handlers::stop_site_connection),
        )
        .route("/api/v1/hub/status", get(hub::handlers::get_hub_status))
        .route(
            "/api/v1/reports",
            get(reports::handlers::list_reports).post(reports::handlers::create_report),
        )
        .route(
            "/api/v1/reports/{id}",
            put(reports::handlers::update_report).delete(reports::handlers::delete_report),
        )
        .route(
            "/api/v1/reports/{id}/generate",
            post(reports::handlers::generate_report),
        )
}

fn admin() -> Router<AppState> {
    Router::new()
        .route("/api/v1/ha/force-sync", post(ha::ha_api::force_sync))
        .route(
            "/api/v1/ha/force-failover",
            post(ha::ha_api::force_failover),
        )
        .route(
            "/api/v1/compliance/config",
            get(compliance::handlers::get_compliance_config)
                .put(compliance::handlers::update_compliance_config),
        )
        .route(
            "/api/v1/compliance/sign",
            post(compliance::handlers::create_electronic_signature),
        )
        .route(
            "/api/v1/compliance/signatures",
            get(compliance::handlers::list_electronic_signatures),
        )
        .route(
            "/api/v1/security/events",
            get(audit::handlers::list_security_events),
        )
        .route(
            "/api/v1/security/events/export",
            get(audit::handlers::export_security_csv),
        )
        .route(
            "/api/v1/security/thresholds",
            get(audit::handlers::get_security_thresholds)
                .put(audit::handlers::update_security_thresholds),
        )
        .route("/api/license/activate", post(licensing::routes::activate))
        .route("/api/license/request", post(licensing::routes::request))
        .route("/api/license/refresh", post(licensing::routes::refresh))
}
