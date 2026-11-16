use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;

use crate::compliance::*;
use voltrus::api::{AppError, AppState};

// ── Query params for listing signatures ──

#[derive(Debug, Deserialize)]
pub struct SignatureQuery {
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
}

fn license_forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"error": "CFR Part 11 requires an Enterprise licence"})),
    )
        .into_response()
}

// ── Get compliance config ──

/// The settings as the Compliance screen edits them: the core's
/// `ComplianceConfig` fields, the add-on's extras, and whether they are
/// enforced (an administrator saved them at least once).
fn config_body(p: &super::policy::Policy) -> serde_json::Value {
    let mut body = serde_json::to_value(&p.config).unwrap_or_else(|_| json!({}));
    if let (Some(obj), Ok(serde_json::Value::Object(extras))) =
        (body.as_object_mut(), serde_json::to_value(&p.extras))
    {
        obj.extend(extras);
        obj.insert("enforced".into(), json!(p.saved));
    }
    body
}

pub async fn get_compliance_config(State(state): State<AppState>) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
        return Ok(license_forbidden());
    }
    let p = super::policy::load(&state.storage).await?;
    Ok(Json(config_body(&p)).into_response())
}

// ── Update compliance config (admin-only) ──

pub async fn update_compliance_config(
    State(state): State<AppState>,
    Extension(auth): Extension<voltrus::api::AuthUser>,
    Json(body): Json<UpdateComplianceConfigRequest>,
) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
        return Ok(license_forbidden());
    }

    let current_policy = super::policy::load(&state.storage).await?;
    let current = current_policy.config.clone();

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

    let bad =
        |msg: &str| Ok((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response());
    if updated.auto_logout_minutes == 0 {
        return bad("auto_logout_minutes must be at least 1");
    }
    if updated.auto_logout_minutes > 24 * 60 {
        return bad("auto_logout_minutes is at most 1440 (one day)");
    }
    // The core refuses passwords shorter than 8 characters whatever the policy says.
    if updated.password_min_length < 8 {
        return bad("password_min_length must be at least 8");
    }
    if updated.password_min_length > 128 {
        return bad("password_min_length is at most 128");
    }
    if updated.password_rotation_days > 3650 {
        return bad("password_rotation_days is at most 3650 (0 turns expiry off)");
    }

    let mut extras = current_policy.extras.clone();
    if let Some(n) = body.password_history {
        extras.password_history = n;
    }
    if let Some(h) = body.session_max_hours {
        extras.session_max_hours = h;
    }
    if let Some(sig) = body.signatures {
        extras.signatures = sig;
    }
    let extras = match super::policy::validate_extras(extras) {
        Ok(e) => e,
        Err(msg) => return bad(&msg),
    };

    state.storage.upsert_compliance_config(&updated).await?;
    super::policy::save_extras(&state.storage, &extras, &auth.username).await?;

    let signed: Vec<String> = [
        ("tag_write", &extras.signatures.actions.tag_write),
        ("alarm_ack", &extras.signatures.actions.alarm_ack),
        ("alarm_shelve", &extras.signatures.actions.alarm_shelve),
        ("recipe_load", &extras.signatures.actions.recipe_load),
        ("mode_switch", &extras.signatures.actions.mode_switch),
    ]
    .into_iter()
    .filter(|(_, r)| r.enabled)
    .map(|(a, r)| format!("{a}=\"{}\"", r.meaning))
    .collect();
    let exceptions: Vec<String> = extras
        .signatures
        .unattended_exceptions
        .iter()
        .map(|e| format!("{}:{}", e.origin, e.principal))
        .collect();
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "update_compliance_config".to_string(),
        username: auth.username.clone(),
        resource: "compliance_config".to_string(),
        details: Some(format!(
            "auto_logout={}min, session_max={}h, password_min_len={}, rotation={}days, history={}, signed=[{}], tag_scope={:?}, unattended_exceptions=[{}]",
            updated.auto_logout_minutes,
            extras.session_max_hours,
            updated.password_min_length,
            updated.password_rotation_days,
            extras.password_history,
            signed.join(", "),
            extras.signatures.tag_scope,
            exceptions.join(", "),
        )),
    });

    let saved = super::policy::Policy {
        saved: true,
        config: updated,
        extras,
    };
    Ok(Json(json!({"ok": true, "config": config_body(&saved)})).into_response())
}

// ── Create electronic signature ──

pub async fn create_electronic_signature(
    State(state): State<AppState>,
    Extension(auth): Extension<voltrus::api::AuthUser>,
    Json(body): Json<CreateSignatureRequest>,
) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
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

    if body.resource_type.trim().is_empty() || body.resource_id.trim().is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "resource_type and resource_id are required"})),
        )
            .into_response());
    }

    if let Some(refused) = reauthenticate(&state, &auth.username, &body.password).await {
        return Ok(refused);
    }

    let now = voltrus::timeutil::now_secs();
    let signature_hash = generate_signature_hash(
        &auth.username,
        "sign",
        &body.resource_type,
        &body.resource_id,
        &body.reason,
        &body.meaning,
        now,
    );

    // Stored with the same timestamp that was hashed, so the hash verifies. The
    // signer's full name on file is recorded with it, so a later rename does not
    // change what the signature manifests.
    let (id, full_name) = {
        let (user, rtype, rid, reason, meaning, hash) = (
            auth.username.clone(),
            body.resource_type.clone(),
            body.resource_id.clone(),
            body.reason.clone(),
            body.meaning.clone(),
            signature_hash.clone(),
        );
        state
            .storage
            .with_conn(move |conn| {
                let full_name = super::export::full_name_of(conn, &user)?;
                let tx = conn.unchecked_transaction()?;
                tx.execute(
                    "INSERT INTO electronic_signatures (user_id, action, resource_type, resource_id, reason, meaning, signature_hash, created_at) VALUES (?1, 'sign', ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![user, rtype, rid, reason, meaning, hash, now],
                )?;
                let id = tx.last_insert_rowid();
                if let Some(name) = &full_name {
                    tx.execute(
                        "INSERT OR REPLACE INTO compliance_signature_names (signature_id, full_name) VALUES (?1, ?2)",
                        rusqlite::params![id, name],
                    )?;
                }
                tx.commit()?;
                Ok((id, full_name))
            })
            .await?
    };

    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "electronic_signature".to_string(),
        username: auth.username.clone(),
        resource: format!("{}:{}", body.resource_type, body.resource_id),
        details: Some(format!(
            "meaning={}, reason={}, id={}, hash={}",
            body.meaning, body.reason, id, signature_hash
        )),
    });

    Ok(Json(json!({
        "ok": true,
        "id": id,
        "signer": auth.username,
        "signer_full_name": full_name,
        "meaning": body.meaning,
        "signature_hash": signature_hash,
        "created_at": now,
    }))
    .into_response())
}

fn refused(status: StatusCode, error: &str) -> Response {
    (status, Json(json!({ "error": error }))).into_response()
}

/// Check the signer's password again. `None` when it is right; otherwise the
/// response to send. Failures count towards the same lockout as sign-in.
async fn reauthenticate(state: &AppState, username: &str, password: &str) -> Option<Response> {
    if password.is_empty() {
        return Some(refused(
            StatusCode::BAD_REQUEST,
            "password is required: enter your own password to sign",
        ));
    }
    let user = match state.storage.get_user_by_username(username).await {
        Ok(Some(u)) if !u.password_hash.is_empty() => u,
        Ok(_) => {
            // API keys and accounts without a local password (single sign-on).
            return Some(refused(
                StatusCode::FORBIDDEN,
                "electronic signatures need a local user account with a password",
            ));
        }
        Err(e) => {
            tracing::error!("signature: looking up {username}: {e:#}");
            return Some(refused(
                StatusCode::INTERNAL_SERVER_ERROR,
                "user lookup failed",
            ));
        }
    };
    let now = voltrus::timeutil::now_secs();
    if user.disabled || user.locked_until.is_some_and(|t| t > now) {
        return Some(refused(
            StatusCode::FORBIDDEN,
            "this account is locked or disabled and cannot sign",
        ));
    }
    let hash = user.password_hash.clone();
    let candidate = password.to_string();
    let ok =
        tokio::task::spawn_blocking(move || voltrus::api::auth::verify_password(&candidate, &hash))
            .await
            .unwrap_or(false);
    if ok {
        let _ = state.storage.reset_failed_attempts(username).await;
        return None;
    }

    let attempt = user.failed_attempts + 1;
    let _ = state.storage.increment_failed_attempts(username).await;
    if attempt >= SIGN_LOCK_AFTER_FAILURES {
        let _ = state
            .storage
            .lock_account(username, now + SIGN_LOCK_SECS)
            .await;
    }
    let _ = state
        .storage
        .insert_security_event(&voltrus::models::CreateSecurityEvent {
            event_type: "signature_auth_failed".into(),
            severity: "warning".into(),
            source_ip: None,
            user_agent: None,
            username: Some(username.to_string()),
            details: Some(format!(
                "Wrong password at electronic signature (attempt {attempt})"
            )),
        })
        .await;
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "electronic_signature_refused".to_string(),
        username: username.to_string(),
        resource: "electronic_signature".to_string(),
        details: Some(format!("wrong password, attempt {attempt}")),
    });
    Some(refused(StatusCode::UNAUTHORIZED, "password is incorrect"))
}

// ── Verify and export signatures ──

pub async fn verify_electronic_signatures(
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
        return Ok(license_forbidden());
    }
    let signatures = state.storage.list_electronic_signatures(None, None).await?;
    let extras = super::export::signature_extras(&state.storage, &signatures).await?;
    let invalid: Vec<i64> = signatures
        .iter()
        .filter(|s| {
            !extras
                .get(&s.id)
                .map_or_else(|| signature_matches(s), |e| e.verifies(s))
        })
        .map(|s| s.id)
        .collect();
    Ok(Json(json!({
        "ok": invalid.is_empty(),
        "checked": signatures.len(),
        "invalid_ids": invalid,
    }))
    .into_response())
}

pub async fn export_electronic_signatures(
    State(state): State<AppState>,
    Query(query): Query<super::export::SignatureFilter>,
) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
        return Ok(license_forbidden());
    }
    let signatures = state
        .storage
        .list_electronic_signatures(
            query.resource_type.as_deref().filter(|s| !s.is_empty()),
            query.resource_id.as_deref().filter(|s| !s.is_empty()),
        )
        .await?;
    // The same range and signer filter as the PDF export.
    let signatures = super::export::filter_signatures(signatures, &query);
    let extras = super::export::signature_extras(&state.storage, &signatures).await?;
    let filename = format!(
        "electronic-signatures-{}.csv",
        time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Iso8601::DATE)
            .unwrap_or_else(|_| "export".to_string())
    );
    Ok((
        StatusCode::OK,
        [
            (
                axum::http::header::CONTENT_TYPE,
                "text/csv; charset=utf-8".to_string(),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        signatures_csv_with(&signatures, &extras),
    )
        .into_response())
}

// ── List electronic signatures ──

pub async fn list_electronic_signatures(
    State(state): State<AppState>,
    Query(query): Query<SignatureQuery>,
) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
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
