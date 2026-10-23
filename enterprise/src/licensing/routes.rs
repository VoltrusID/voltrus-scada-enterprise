//! Admin HTTP API for activating and renewing a licence from the dashboard.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use voltrus::api::{AppState, AuthUser};
use voltrus::license::LicenseStatus;

use super::{cli, online, request};

#[derive(Debug, Deserialize)]
pub struct ActivateBody {
    /// Online activation through activate.voltrus.id.
    pub license_key: Option<String>,
    /// Offline: the signed activation file, base64 encoded.
    pub activation_base64: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RequestBody {
    pub license_key: String,
}

fn error(status: StatusCode, message: impl std::fmt::Display) -> Response {
    (status, Json(json!({"error": message.to_string()}))).into_response()
}

/// Same shape as `GET /api/license/status`.
fn status_json(status: &LicenseStatus) -> serde_json::Value {
    let mut value = serde_json::to_value(status).unwrap_or_default();
    value["tier"] = json!(status.tier());
    value["feature_flags"] = status.feature_flags();
    value
}

async fn finish(
    state: &AppState,
    who: &AuthUser,
    action: &str,
    result: anyhow::Result<LicenseStatus>,
) -> Response {
    match result {
        Ok(status) => {
            crate::rt().apply_license_effects().await;
            state.audit_logger.log(voltrus::security::AuditEvent {
                action: action.to_string(),
                username: who.username.clone(),
                resource: "license".to_string(),
                details: None,
            });
            Json(json!({"ok": true, "status": status_json(&status)})).into_response()
        }
        Err(e) => error(StatusCode::BAD_REQUEST, format!("{e:#}")),
    }
}

/// POST /api/license/activate
pub async fn activate(
    State(state): State<AppState>,
    axum::Extension(who): axum::Extension<AuthUser>,
    Json(body): Json<ActivateBody>,
) -> Response {
    let blob = match (&body.activation_base64, &body.license_key) {
        (Some(b64), _) => match cli::decode_activation_b64(b64) {
            Ok(blob) => blob,
            Err(e) => {
                return error(
                    StatusCode::BAD_REQUEST,
                    format!("Invalid activation file: {e}"),
                );
            }
        },
        (None, Some(key)) if !key.trim().is_empty() => {
            match online::fetch_activation(key.trim()).await {
                Ok(blob) => blob,
                Err(e) => return error(StatusCode::BAD_REQUEST, format!("{e:#}")),
            }
        }
        _ => {
            return error(
                StatusCode::BAD_REQUEST,
                "Send license_key (online) or activation_base64 (offline)",
            );
        }
    };
    let result = online::apply_activation(&state.storage, &state.license_status, blob).await;
    finish(&state, &who, "activate_license", result).await
}

/// POST /api/license/request: offline flow, returns the request file to upload at
/// activate.voltrus.id.
pub async fn request(Json(body): Json<RequestBody>) -> Response {
    match request::build_request(body.license_key.trim()) {
        Ok(data) => Json(json!({
            "request_base64": base64::engine::general_purpose::STANDARD.encode(data),
        }))
        .into_response(),
        Err(e) => error(StatusCode::BAD_REQUEST, format!("{e:#}")),
    }
}

/// POST /api/license/refresh: renew a monthly key online.
pub async fn refresh(
    State(state): State<AppState>,
    axum::Extension(who): axum::Extension<AuthUser>,
) -> Response {
    let result = online::renew_now(&state.storage, &state.license_status).await;
    finish(&state, &who, "refresh_license", result).await
}
