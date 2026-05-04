use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;

use super::{AppError, AppState};
use crate::compliance::*;

// ── Query params for listing signatures ──

#[derive(Debug, Deserialize)]
pub struct SignatureQuery {
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
}

fn license_forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"error": "CFR Part 11 requires Enterprise license"})),
    )
        .into_response()
}

// ── Get compliance config ──

pub async fn get_compliance_config(State(state): State<AppState>) -> Result<Response, AppError> {
    if !state.license_tier.cfr_part_11() {
        return Ok(license_forbidden());
    }
    match state.storage.get_compliance_config().await? {
        Some(config) => Ok(Json(config).into_response()),
        None => Ok(Json(ComplianceConfig::default()).into_response()),
    }
}

// ── Update compliance config (admin-only) ──

pub async fn update_compliance_config(
    State(state): State<AppState>,
    Extension(auth): Extension<super::AuthUser>,
    Json(body): Json<UpdateComplianceConfigRequest>,
) -> Result<Response, AppError> {
    if !state.license_tier.cfr_part_11() {
        return Ok(license_forbidden());
    }

    let current = state
        .storage
        .get_compliance_config()
        .await?
        .unwrap_or_default();

    let updated = ComplianceConfig {
        auto_logout_minutes: body
            .auto_logout_minutes
            .unwrap_or(current.auto_logout_minutes),
        password_min_length: body
            .password_min_length
            .unwrap_or(current.password_min_length),
        password_require_uppercase: body
            .password_require_uppercase
            .unwrap_or(current.password_require_uppercase),
        password_require_number: body
            .password_require_number
            .unwrap_or(current.password_require_number),
        password_require_special: body
            .password_require_special
            .unwrap_or(current.password_require_special),
        password_rotation_days: body
            .password_rotation_days
            .unwrap_or(current.password_rotation_days),
        authority_to_change: body
            .authority_to_change
            .unwrap_or(current.authority_to_change),
    };

    // Validate ranges
    if updated.auto_logout_minutes == 0 {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "auto_logout_minutes must be at least 1"})),
        )
            .into_response());
    }
    if updated.password_min_length == 0 {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "password_min_length must be at least 1"})),
        )
            .into_response());
    }
    if updated.password_rotation_days == 0 {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "password_rotation_days must be at least 1"})),
        )
            .into_response());
    }

    state.storage.upsert_compliance_config(&updated).await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_compliance_config".to_string(),
        username: auth.username.clone(),
        resource: "compliance_config".to_string(),
        details: Some(format!(
            "auto_logout={}min, password_min_len={}, rotation={}days",
            updated.auto_logout_minutes,
            updated.password_min_length,
            updated.password_rotation_days
        )),
    });

    Ok(Json(json!({"ok": true})).into_response())
}

// ── Create electronic signature ──

pub async fn create_electronic_signature(
    State(state): State<AppState>,
    Extension(auth): Extension<super::AuthUser>,
    Json(body): Json<CreateSignatureRequest>,
) -> Result<Response, AppError> {
    if !state.license_tier.cfr_part_11() {
        return Ok(license_forbidden());
    }

    if body.reason.trim().is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "reason is required"})),
        )
            .into_response());
    }

    if body.meaning.trim().is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "meaning is required"})),
        )
            .into_response());
    }

    let now = crate::timeutil::now_secs();
    let signature_hash = generate_signature_hash(
        &auth.username,
        "sign",
        &body.resource_type,
        &body.resource_id,
        &body.reason,
        &body.meaning,
        now,
    );

    let id = state
        .storage
        .create_electronic_signature(
            &auth.username,
            "sign",
            &body.resource_type,
            &body.resource_id,
            &body.reason,
            &body.meaning,
            &signature_hash,
        )
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "electronic_signature".to_string(),
        username: auth.username.clone(),
        resource: format!("{}:{}", body.resource_type, body.resource_id),
        details: Some(format!("meaning={}, id={}", body.meaning, id)),
    });

    Ok(Json(json!({
        "ok": true,
        "id": id,
        "signature_hash": signature_hash,
        "created_at": now,
    }))
    .into_response())
}

// ── List electronic signatures ──

pub async fn list_electronic_signatures(
    State(state): State<AppState>,
    Query(query): Query<SignatureQuery>,
) -> Result<Response, AppError> {
    if !state.license_tier.cfr_part_11() {
        return Ok(license_forbidden());
    }

    let signatures = state
        .storage
        .list_electronic_signatures(query.resource_type.as_deref(), query.resource_id.as_deref())
        .await?;

    Ok(Json(signatures).into_response())
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_compliance_config_request_deserialize() {
        let json = r#"{"auto_logout_minutes": 10}"#;
        let req: UpdateComplianceConfigRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.auto_logout_minutes, Some(10));
        assert!(req.password_min_length.is_none());
    }

    #[test]
    fn test_create_signature_request_deserialize() {
        let json = r#"{"resource_type":"recipe","resource_id":"42","reason":"QA review","meaning":"Verified"}"#;
        let req: CreateSignatureRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.resource_type, "recipe");
        assert_eq!(req.resource_id, "42");
        assert_eq!(req.reason, "QA review");
        assert_eq!(req.meaning, "Verified");
    }

    #[test]
    fn test_signature_query_deserialize() {
        let req = SignatureQuery {
            resource_type: Some("recipe".to_string()),
            resource_id: Some("42".to_string()),
        };
        assert_eq!(req.resource_type, Some("recipe".to_string()));
        assert_eq!(req.resource_id, Some("42".to_string()));
    }
}
