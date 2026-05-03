use axum::{
    extract::{Path, State},
    http::StatusCode as HttpStatus,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::api::AppState;

fn pdf_reporting_required(state: &AppState) -> Option<(HttpStatus, Json<serde_json::Value>)> {
    if !state.license_tier.pdf_reporting() {
        return Some((
            HttpStatus::FORBIDDEN,
            Json(json!({
                "error": "PDF reporting requires Professional tier or higher. Upgrade at https://voltrus.id"
            })),
        ));
    }
    None
}

/// GET /api/v1/reports -- list all reports
#[utoipa::path(
    get,
    path = "/api/v1/reports",
    responses(
        (status = 200, body = Vec<crate::models::Report>),
        (status = 403, description = "License tier too low")
    ),
    tag = "Reports",
    security(("api_key" = []), ("cookie_auth" = []))
)]
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

/// POST /api/v1/reports -- create report
#[utoipa::path(
    post,
    path = "/api/v1/reports",
    request_body = crate::models::CreateReportRequest,
    responses(
        (status = 201, body = crate::models::Report),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "License tier too low")
    ),
    tag = "Reports",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn create_report(
    State(state): State<AppState>,
    Json(req): Json<crate::models::CreateReportRequest>,
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
    let recipients: Vec<String> = match serde_json::from_str(&req.email_recipients) {
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
    if device_ids != "all" {
        if serde_json::from_str::<Vec<String>>(&device_ids).is_err() {
            return (
                HttpStatus::BAD_REQUEST,
                Json(json!({"error": "device_ids must be a valid JSON array or the string 'all'"})),
            )
                .into_response();
        }
    }

    match state
        .storage
        .create_report(&name, &req.schedule, enabled, &device_ids, &req.email_recipients)
        .await
    {
        Ok(id) => {
            match state.storage.get_report(id).await {
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
            }
        }
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to create report: {}", e)})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/reports/{id} -- update report
#[utoipa::path(
    put,
    path = "/api/v1/reports/{id}",
    params(("id" = i64, Path, description = "Report ID")),
    request_body = crate::models::UpdateReportRequest,
    responses(
        (status = 200, body = crate::models::Report),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "License tier too low"),
        (status = 404, description = "Report not found")
    ),
    tag = "Reports",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn update_report(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<crate::models::UpdateReportRequest>,
) -> impl IntoResponse {
    if let Some((status, body)) = pdf_reporting_required(&state) {
        return (status, body).into_response();
    }

    // Validate schedule if provided
    if let Some(ref schedule) = req.schedule {
        if !matches!(schedule.as_str(), "daily" | "weekly" | "monthly") {
            return (
                HttpStatus::BAD_REQUEST,
                Json(json!({"error": "Schedule must be 'daily', 'weekly', or 'monthly'"})),
            )
                .into_response();
        }
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
    if let Some(ref device_ids) = req.device_ids {
        if device_ids != "all" {
            if serde_json::from_str::<Vec<String>>(device_ids).is_err() {
                return (
                    HttpStatus::BAD_REQUEST,
                    Json(json!({"error": "device_ids must be a valid JSON array or the string 'all'"})),
                )
                    .into_response();
            }
        }
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
            req.name.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()),
            req.schedule.as_deref(),
            req.device_ids.as_deref(),
            req.email_recipients.as_deref(),
            req.enabled,
        )
        .await
    {
        Ok(()) => {
            match state.storage.get_report(id).await {
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
            }
        }
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to update report: {}", e)})),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/reports/{id} -- delete report
#[utoipa::path(
    delete,
    path = "/api/v1/reports/{id}",
    params(("id" = i64, Path, description = "Report ID")),
    responses(
        (status = 204, description = "Report deleted"),
        (status = 403, description = "License tier too low"),
        (status = 404, description = "Report not found")
    ),
    tag = "Reports",
    security(("api_key" = []), ("cookie_auth" = []))
)]
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

/// POST /api/v1/reports/{id}/generate -- manual generate
#[utoipa::path(
    post,
    path = "/api/v1/reports/{id}/generate",
    params(("id" = i64, Path, description = "Report ID")),
    responses(
        (status = 200, description = "Generated HTML report", content_type = "text/html"),
        (status = 403, description = "License tier too low"),
        (status = 404, description = "Report not found")
    ),
    tag = "Reports",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn generate_report(
    State(state): State<AppState>,
    Path(id): Path<i64>,
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

    match crate::report::generate_report_html(&state.storage, &state.cache, &report).await {
        Ok(html) => {
            // Update last_generated
            let now = crate::timeutil::now_secs();
            if let Err(e) = state.storage.update_report_last_generated(id, now).await {
                tracing::warn!("Failed to update last_generated for report {}: {}", id, e);
            }

            // Return HTML content
            (
                HttpStatus::OK,
                [
                    ("content-type", "text/html; charset=utf-8"),
                ],
                html,
            )
                .into_response()
        }
        Err(e) => (
            HttpStatus::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to generate report: {}", e)})),
        )
            .into_response(),
    }
}
