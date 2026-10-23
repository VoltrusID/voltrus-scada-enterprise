use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

use voltrus::api::{AppError, AppState};
use voltrus::models::*;

fn require_enterprise(state: &AppState) -> Result<(), AppError> {
    if !state.tier().iec_62443() {
        return Err(AppError::forbidden(
            "Security audit requires an Enterprise licence. See https://voltrus.id/voltrus-scada",
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct ExportCsvQuery {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

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

pub async fn get_security_thresholds(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    require_enterprise(&state)?;
    let thresholds = state.storage.get_security_thresholds().await?;
    Ok(Json(json!(thresholds)))
}

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
