use axum::{extract::State, Json};

use crate::api::AppState;

pub async fn get_license_status(State(state): State<AppState>) -> Json<crate::license::LicenseStatus> {
    Json(state.license_status.clone())
}
