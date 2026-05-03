use axum::{extract::State, Json};

use crate::api::AppState;

#[utoipa::path(
    get,
    path = "/api/license/status",
    responses((status = 200)),
    tag = "License"
)]
pub async fn get_license_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut status = serde_json::to_value(&state.license_status).unwrap();
    status["tier"] = serde_json::to_value(&state.license_tier).unwrap();
    status["feature_flags"] = state.feature_flags.clone();
    Json(status)
}
