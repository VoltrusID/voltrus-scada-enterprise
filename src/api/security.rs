use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

use super::{AppError, AppState};
use crate::models::*;

fn require_enterprise(state: &AppState) -> Result<(), AppError> {
    if !matches!(state.license_tier, crate::license::LicenseTier::Enterprise) {
        return Err(AppError(anyhow::anyhow!(
            "Security audit requires Enterprise tier. Upgrade at https://voltrus.id"
        )));
    }
    Ok(())
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListEventsQuery {
    pub event_type: Option<String>,
    pub severity: Option<String>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    100
}

#[utoipa::path(
    get,
    path = "/api/v1/security/events",
    params(ListEventsQuery),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 403),
    ),
    tag = "Security",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn list_security_events(
    State(state): State<AppState>,
    Query(params): Query<ListEventsQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_enterprise(&state)?;
    let events = state
        .storage
        .list_security_events(
            params.event_type.as_deref(),
            params.severity.as_deref(),
            params.from,
            params.to,
            params.limit.min(1000),
            params.offset,
        )
        .await?;

    Ok(Json(json!({
        "events": events,
        "count": events.len(),
    })))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ExportCsvQuery {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/security/events/export",
    params(ExportCsvQuery),
    responses(
        (status = 200, description = "CSV export"),
        (status = 403),
    ),
    tag = "Security",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn export_security_csv(
    State(state): State<AppState>,
    Query(params): Query<ExportCsvQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_enterprise(&state)?;
    let csv = state
        .storage
        .export_security_events_csv(params.from, params.to)
        .await?;

    let filename = format!(
        "security-log-{}.csv",
        time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Iso8601::DATE)
            .unwrap_or_else(|_| "unknown".to_string())
    );

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("text/csv")),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
            ),
        ],
        csv,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/security/thresholds",
    responses(
        (status = 200, body = crate::models::SecurityThresholds),
        (status = 403),
    ),
    tag = "Security",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn get_security_thresholds(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    require_enterprise(&state)?;
    let thresholds = state.storage.get_security_thresholds().await?;
    Ok(Json(json!(thresholds)))
}

#[utoipa::path(
    put,
    path = "/api/v1/security/thresholds",
    request_body = crate::models::SecurityThresholds,
    responses(
        (status = 200, body = serde_json::Value),
        (status = 400),
        (status = 403),
    ),
    tag = "Security",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_security_thresholds(
    State(state): State<AppState>,
    Json(body): Json<SecurityThresholds>,
) -> Result<impl IntoResponse, AppError> {
    // Validate ranges
    require_enterprise(&state)?;
    if body.failed_login_threshold == 0
        || body.suspicious_401_threshold == 0
        || body.rate_limit_threshold == 0
    {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Thresholds must be greater than 0"})),
        )
            .into_response());
    }

    state.storage.update_security_thresholds(&body).await?;

    let _ = state
        .storage
        .insert_security_event(&CreateSecurityEvent {
            event_type: "config_change".into(),
            severity: "info".into(),
            source_ip: None,
            user_agent: None,
            username: None,
            details: Some("Security thresholds updated".into()),
        })
        .await;

    Ok((StatusCode::OK, Json(json!({"ok": true}))).into_response())
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_thresholds_default() {
        let t = SecurityThresholds::default();
        assert_eq!(t.failed_login_threshold, 5);
        assert_eq!(t.failed_login_window_secs, 60);
        assert_eq!(t.suspicious_401_threshold, 10);
        assert_eq!(t.suspicious_401_window_secs, 60);
        assert_eq!(t.rate_limit_threshold, 100);
        assert_eq!(t.rate_limit_window_secs, 60);
    }

    #[test]
    fn test_security_thresholds_serde_roundtrip() {
        let t = SecurityThresholds::default();
        let json = serde_json::to_string(&t).unwrap();
        let back: SecurityThresholds = serde_json::from_str(&json).unwrap();
        assert_eq!(back.failed_login_threshold, t.failed_login_threshold);
        assert_eq!(back.suspicious_401_threshold, t.suspicious_401_threshold);
    }

    #[test]
    fn test_create_security_event_default_severity() {
        let e = CreateSecurityEvent {
            event_type: "test".into(),
            severity: "info".into(),
            source_ip: None,
            user_agent: None,
            username: None,
            details: None,
        };
        assert_eq!(e.severity, "info");
    }

    #[test]
    fn test_list_events_query_default_limit() {
        let q = ListEventsQuery {
            event_type: None,
            severity: None,
            from: None,
            to: None,
            limit: default_limit(),
            offset: 0,
        };
        assert_eq!(q.limit, 100);
    }

    // Integration tests using the test router

    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_security_events_endpoint_requires_auth() {
        let app = super::super::tests::test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/security/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_security_thresholds_requires_admin() {
        let app = super::super::tests::test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/security/thresholds")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_security_csv_export_requires_auth() {
        let app = super::super::tests::test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/security/events/export")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
