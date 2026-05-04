use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

use super::{AppError, AppState, AuthUser};

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default = "default_permissions")]
    pub permissions: String,
}

fn default_permissions() -> String {
    "[]".to_string()
}

pub async fn create_api_key(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.name.trim().is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "name is required"})),
        )
            .into_response());
    }

    let (id, plain_key) = state
        .storage
        .create_api_key(
            &body.name,
            &auth.username,
            body.expires_at,
            &body.permissions,
        )
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "create_api_key".to_string(),
        username: auth.username.clone(),
        resource: format!("api_key:{}", id),
        details: Some(format!("name={}", body.name)),
    });

    Ok(Json(json!({
        "ok": true,
        "id": id,
        "name": body.name,
        "key": plain_key,
        "warning": "Store this key securely. It will not be shown again.",
        "note": "Permissions are stored for future use. All valid API keys currently grant full read access."
    }))
    .into_response())
}

pub async fn list_api_keys(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let keys = state.storage.list_api_keys().await?;
    Ok(Json(json!({ "keys": keys })))
}

pub async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = state.storage.revoke_api_key(id).await?;
    if !deleted {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "API key not found"})),
        )
            .into_response());
    }

    state.audit_logger.log(crate::security::AuditEvent {
        action: "revoke_api_key".to_string(),
        username: auth.username.clone(),
        resource: format!("api_key:{}", id),
        details: None,
    });

    Ok(Json(json!({"ok": true})).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_permissions() {
        assert_eq!(default_permissions(), "[]");
    }

    #[test]
    fn test_create_api_key_request_deserialize() {
        let json_str = r#"{"name": "test-key"}"#;
        let req: CreateApiKeyRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.name, "test-key");
        assert_eq!(req.expires_at, None);
        assert_eq!(req.permissions, "[]");
    }

    #[test]
    fn test_create_api_key_request_with_permissions() {
        let json_str = r#"{"name": "export-key", "permissions": "[\"data:read\",\"data:export\"]", "expires_at": 9999999999}"#;
        let req: CreateApiKeyRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.name, "export-key");
        assert_eq!(req.expires_at, Some(9999999999));
        assert_eq!(req.permissions, "[\"data:read\",\"data:export\"]");
    }
}
