mod devices;
mod objects;
mod discover;

pub use devices::*;
pub use objects::*;
pub use discover::*;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::api::{AppError, AppState, AuthUser};
use crate::models::{
    BacnetDeviceConfig, BacnetObjectConfig, CreateBacnetDeviceRequest, CreateBacnetObjectRequest,
    UpdateBacnetDeviceRequest,
};
