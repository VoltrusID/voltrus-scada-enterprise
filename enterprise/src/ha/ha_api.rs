use std::time::Duration;

use axum::extract::{Extension, Query, State};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;

use voltrus::api::{AppState, AuthUser};

fn requires_enterprise() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"error": "Hot standby requires an Enterprise licence"})),
    )
}

fn not_enabled() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "HA not enabled"})),
    )
}

#[derive(Deserialize)]
pub struct ActionQuery {
    /// How long to wait for the action to finish, in seconds (default 30, at most 300).
    wait_secs: Option<u64>,
    /// Failover on a standby that has no link to the primary: take over even if
    /// its copy is incomplete or its witness cannot be reached.
    #[serde(default)]
    force: bool,
}

impl ActionQuery {
    fn wait(&self) -> Duration {
        Duration::from_secs(self.wait_secs.unwrap_or(30).clamp(1, 300))
    }
}

/// GET /api/v1/ha/status
pub async fn ha_status() -> impl IntoResponse {
    Json(crate::rt().ha_state.get_status().await)
}

fn audit(state: &AppState, user: &AuthUser, action: &str, details: String) {
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: action.to_string(),
        username: user.username.clone(),
        resource: "ha".to_string(),
        details: Some(details),
    });
}

/// POST /api/v1/ha/force-sync
///
/// Runs a full resynchronisation of the standby from a fresh snapshot of the
/// primary. On the standby the primary is asked to do it. Answers 200 with the
/// report once it is done, 202 if it is still running after `wait_secs`.
pub async fn force_sync(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(q): Query<ActionQuery>,
) -> impl IntoResponse {
    if !state.tier().ha() {
        return requires_enterprise();
    }
    let ha = &crate::rt().ha_state;
    let Some(node) = ha.node() else {
        return not_enabled();
    };
    audit(
        &state,
        &user,
        "ha_force_sync",
        format!("role {:?}", node.role()),
    );
    match node.force_sync(q.wait()).await {
        Ok(report) => {
            let status = node.status().await;
            if status.sync_state == "full_sync" {
                (
                    StatusCode::ACCEPTED,
                    Json(json!({"status": "sync_in_progress", "progress": status.sync_progress})),
                )
            } else {
                (
                    StatusCode::OK,
                    Json(json!({"status": "synced", "report": report})),
                )
            }
        }
        Err(e) => (
            StatusCode::CONFLICT,
            Json(json!({"error": format!("{e:#}")})),
        ),
    }
}

/// POST /api/v1/ha/force-failover
///
/// On the primary: a controlled switchover. The primary stops acting, the
/// standby receives every remaining change, takes over, and the old primary
/// follows it. On a standby that is linked to its primary: asks the primary to
/// do the same. On a standby without a link: takes over (the primary is
/// presumed down) subject to the witness; `force=true` skips the data and
/// reachability checks. Answers once the roles have changed.
pub async fn force_failover(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(q): Query<ActionQuery>,
) -> impl IntoResponse {
    if !state.tier().ha() {
        return requires_enterprise();
    }
    let ha = &crate::rt().ha_state;
    let Some(node) = ha.node() else {
        return not_enabled();
    };
    audit(
        &state,
        &user,
        "ha_force_failover",
        format!("role {:?}, force {}", node.role(), q.force),
    );
    match node.force_failover(q.force, q.wait()).await {
        Ok(t) => (
            StatusCode::OK,
            Json(json!({
                "status": "failover_complete",
                "node_id": node.status().await.node_id,
                "new_role": t.role,
                "epoch": t.epoch,
                "duration_ms": t.duration_ms,
            })),
        ),
        Err(e) => (
            StatusCode::CONFLICT,
            Json(json!({"error": format!("{e:#}")})),
        ),
    }
}
