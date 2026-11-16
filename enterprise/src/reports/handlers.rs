use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode as HttpStatus,
    response::IntoResponse,
};
use serde_json::json;

use voltrus::api::AppState;

fn pdf_reporting_required(state: &AppState) -> Option<(HttpStatus, Json<serde_json::Value>)> {
    if !state.tier().pdf_reporting() {
        return Some((
            HttpStatus::FORBIDDEN,
            Json(json!({
                "error": "PDF reporting requires an Integrator or Enterprise licence. See https://voltrus.id/voltrus-scada"
            })),
        ));
    }
    None
}

/// GET /api/v1/reports — list all reports
/// A list field (recipients, devices) as stored: JSON text. Accepts a JSON
/// array (what the web client sends), JSON text of an array (the SDKs) or the
/// string `"all"` for devices.
fn list_as_text<'de, D>(de: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    Ok(match Option::<serde_json::Value>::deserialize(de)? {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => Some(s),
        Some(v @ serde_json::Value::Array(_)) => Some(
            // Device ids are strings in the core; numbers from a client become strings.
            serde_json::to_string(
                &v.as_array()
                    .into_iter()
                    .flatten()
                    .map(|x| match x {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect::<Vec<_>>(),
            )
            .map_err(serde::de::Error::custom)?,
        ),
        Some(other) => {
            return Err(serde::de::Error::custom(format!(
                "expected a list, got {other}"
            )));
        }
    })
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateReportRequest {
    pub name: String,
    pub schedule: String,
    #[serde(default, deserialize_with = "list_as_text")]
    pub device_ids: Option<String>,
    #[serde(default, deserialize_with = "list_as_text")]
    pub email_recipients: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateReportRequest {
    pub name: Option<String>,
    pub schedule: Option<String>,
    #[serde(default, deserialize_with = "list_as_text")]
    pub device_ids: Option<String>,
    #[serde(default, deserialize_with = "list_as_text")]
    pub email_recipients: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct GenerateQuery {
    /// `html` (default) or `pdf`.
    pub format: Option<String>,
}

pub async fn list_reports(State(state): State<AppState>) -> impl IntoResponse {
    if let Some((status, body)) = pdf_reporting_required(&state) {
        return (status, body).into_response();
    }

    match state.storage.list_reports().await {
        Ok(reports) => (HttpStatus::OK, Json(json!(reports))).into_response(),
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to list reports: {}", e)})),
        )
            .into_response(),
    }
}

/// POST /api/v1/reports — create report
pub async fn create_report(
    State(state): State<AppState>,
    Json(req): Json<CreateReportRequest>,
) -> impl IntoResponse {
    if let Some((status, body)) = pdf_reporting_required(&state) {
        return (status, body).into_response();
    }

    // Validate name
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "Report name is required"})),
        )
            .into_response();
    }

    // Validate schedule
    if !matches!(req.schedule.as_str(), "daily" | "weekly" | "monthly") {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "Schedule must be 'daily', 'weekly', or 'monthly'"})),
        )
            .into_response();
    }

    // Validate recipients
    let email_recipients = req.email_recipients.clone().unwrap_or_default();
    let recipients: Vec<String> = match serde_json::from_str(&email_recipients) {
        Ok(r) => r,
        Err(_) => {
            return (
                HttpStatus::BAD_REQUEST,
                Json(json!({"error": "email_recipients must be a valid JSON array of email addresses"})),
            )
                .into_response();
        }
    };
    if recipients.is_empty() {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "At least one email recipient is required"})),
        )
            .into_response();
    }
    // Basic email validation
    for addr in &recipients {
        if !addr.contains('@') || !addr.contains('.') {
            return (
                HttpStatus::BAD_REQUEST,
                Json(json!({"error": format!("Invalid email address: {}", addr)})),
            )
                .into_response();
        }
    }

    let enabled = req.enabled.unwrap_or(true);
    let device_ids = req.device_ids.unwrap_or_else(|| "all".to_string());

    // Validate device_ids if it's not "all"
    if device_ids != "all" && serde_json::from_str::<Vec<String>>(&device_ids).is_err() {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "device_ids must be a valid JSON array or the string 'all'"})),
        )
            .into_response();
    }

    match state
        .storage
        .create_report(
            &name,
            &req.schedule,
            enabled,
            &device_ids,
            &email_recipients,
        )
        .await
    {
        Ok(id) => match state.storage.get_report(id).await {
            Ok(Some(report)) => (HttpStatus::CREATED, Json(json!(report))).into_response(),
            Ok(None) => (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Report created but could not be retrieved"})),
            )
                .into_response(),
            Err(e) => (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to retrieve created report: {}", e)})),
            )
                .into_response(),
        },
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to create report: {}", e)})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/reports/{id} — update report
pub async fn update_report(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateReportRequest>,
) -> impl IntoResponse {
    if let Some((status, body)) = pdf_reporting_required(&state) {
        return (status, body).into_response();
    }

    // Validate schedule if provided
    if let Some(ref schedule) = req.schedule
        && !matches!(schedule.as_str(), "daily" | "weekly" | "monthly")
    {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "Schedule must be 'daily', 'weekly', or 'monthly'"})),
        )
            .into_response();
    }

    // Validate recipients if provided
    if let Some(ref recipients_str) = req.email_recipients {
        let recipients: Vec<String> = match serde_json::from_str(recipients_str) {
            Ok(r) => r,
            Err(_) => {
                return (
                    HttpStatus::BAD_REQUEST,
                    Json(json!({"error": "email_recipients must be a valid JSON array of email addresses"})),
                )
                    .into_response();
            }
        };
        if recipients.is_empty() {
            return (
                HttpStatus::BAD_REQUEST,
                Json(json!({"error": "At least one email recipient is required"})),
            )
                .into_response();
        }
        for addr in &recipients {
            if !addr.contains('@') || !addr.contains('.') {
                return (
                    HttpStatus::BAD_REQUEST,
                    Json(json!({"error": format!("Invalid email address: {}", addr)})),
                )
                    .into_response();
            }
        }
    }

    // Validate device_ids if provided
    if let Some(ref device_ids) = req.device_ids
        && device_ids != "all"
        && serde_json::from_str::<Vec<String>>(device_ids).is_err()
    {
        return (
            HttpStatus::BAD_REQUEST,
            Json(json!({"error": "device_ids must be a valid JSON array or the string 'all'"})),
        )
            .into_response();
    }

    // Check report exists
    match state.storage.get_report(id).await {
        Ok(None) => {
            return (
                HttpStatus::NOT_FOUND,
                Json(json!({"error": "Report not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to find report: {}", e)})),
            )
                .into_response();
        }
        _ => {}
    }

    match state
        .storage
        .update_report(
            id,
            req.name
                .as_deref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty()),
            req.schedule.as_deref(),
            req.device_ids.as_deref(),
            req.email_recipients.as_deref(),
            req.enabled,
        )
        .await
    {
        Ok(()) => match state.storage.get_report(id).await {
            Ok(Some(report)) => (HttpStatus::OK, Json(json!(report))).into_response(),
            Ok(None) => (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Report updated but could not be retrieved"})),
            )
                .into_response(),
            Err(e) => (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to retrieve updated report: {}", e)})),
            )
                .into_response(),
        },
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to update report: {}", e)})),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/reports/{id} — delete report
pub async fn delete_report(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Some((status, body)) = pdf_reporting_required(&state) {
        return (status, body).into_response();
    }

    // Check report exists
    match state.storage.get_report(id).await {
        Ok(None) => {
            return (
                HttpStatus::NOT_FOUND,
                Json(json!({"error": "Report not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to find report: {}", e)})),
            )
                .into_response();
        }
        _ => {}
    }

    match state.storage.delete_report(id).await {
        Ok(()) => (HttpStatus::NO_CONTENT, "").into_response(),
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to delete report: {}", e)})),
        )
            .into_response(),
    }
}

/// POST /api/v1/reports/{id}/generate — manual generate
pub async fn generate_report(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<GenerateQuery>,
) -> impl IntoResponse {
    if let Some((status, body)) = pdf_reporting_required(&state) {
        return (status, body).into_response();
    }

    let report = match state.storage.get_report(id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                HttpStatus::NOT_FOUND,
                Json(json!({"error": "Report not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to find report: {}", e)})),
            )
                .into_response();
        }
    };

    // A preview: it does not move the schedule (`last_generated`) and sends no mail.
    let data = match crate::reports::render::collect_report_data(&state.storage, &report).await {
        Ok(d) => d,
        Err(e) => {
            return (
                HttpStatus::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to generate report: {}", e)})),
            )
                .into_response();
        }
    };
    if query.format.as_deref() == Some("pdf") {
        let name = crate::reports::render::report_filename_with(&report, "pdf");
        (
            HttpStatus::OK,
            [
                ("content-type", "application/pdf".to_string()),
                (
                    "content-disposition",
                    format!("attachment; filename=\"{name}\""),
                ),
            ],
            crate::reports::render::render_pdf(&data),
        )
            .into_response()
    } else {
        (
            HttpStatus::OK,
            [("content-type", "text/html; charset=utf-8")],
            crate::reports::render::render_html(&data),
        )
            .into_response()
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;

    #[test]
    fn lists_arrive_as_arrays_or_text() {
        let from_ui: CreateReportRequest = serde_json::from_str(
            r#"{"name":"D","schedule":"daily","email_recipients":["a@x.io","b@x.io"],"device_ids":[1,"plc2"]}"#,
        )
        .unwrap();
        assert_eq!(
            from_ui.email_recipients.as_deref(),
            Some(r#"["a@x.io","b@x.io"]"#)
        );
        assert_eq!(from_ui.device_ids.as_deref(), Some(r#"["1","plc2"]"#));

        let from_sdk: CreateReportRequest = serde_json::from_str(
            r#"{"name":"D","schedule":"daily","email_recipients":"[\"a@x.io\"]","device_ids":"all"}"#,
        )
        .unwrap();
        assert_eq!(from_sdk.email_recipients.as_deref(), Some(r#"["a@x.io"]"#));
        assert_eq!(from_sdk.device_ids.as_deref(), Some("all"));

        let update: UpdateReportRequest = serde_json::from_str(r#"{"enabled":false}"#).unwrap();
        assert!(update.email_recipients.is_none() && update.device_ids.is_none());

        assert!(
            serde_json::from_str::<CreateReportRequest>(
                r#"{"name":"D","schedule":"daily","email_recipients":5}"#
            )
            .is_err()
        );
    }
}
