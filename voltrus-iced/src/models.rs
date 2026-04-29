use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub ok: bool,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthMeResponse {
    pub username: String,
    pub role: String,
}

// ---------------------------------------------------------------------------
// Sensor / Data Source
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SensorConfig {
    pub id: i64,
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub min_val: f64,
    #[serde(default)]
    pub max_val: f64,
    #[serde(default)]
    pub warn_val: f64,
    #[serde(default)]
    pub crit_val: f64,
    #[serde(default)]
    pub base_val: f64,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub decimals: i64,
    #[serde(default)]
    pub source_type: String,
    #[serde(default)]
    pub source_config: String,
    #[serde(default)]
    pub device_id: String,
    #[serde(default)]
    pub equipment_id: Option<String>,
    #[serde(default)]
    pub chart_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SensorData {
    pub id: i64,
    pub key: String,
    pub value: f64,
    pub warn: f64,
    pub crit: f64,
    pub source_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataPoint {
    pub ts: i64,
    pub value: f64,
}

// ---------------------------------------------------------------------------
// Screens
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct Screen {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub screen_type: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateScreenRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenameScreenRequest {
    pub name: String,
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

/// Simplified sensor info returned in history responses.
#[derive(Debug, Clone, Deserialize)]
pub struct HistorySensor {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub unit: String,
    pub color: String,
    #[serde(default)]
    pub warn_val: f64,
    #[serde(default)]
    pub crit_val: f64,
    #[serde(default)]
    pub decimals: i64,
    #[serde(default)]
    pub source_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryResponse {
    pub sensors: Vec<HistorySensor>,
    pub data: serde_json::Value,
    pub range: String,
    pub interval: u64,
}

// ---------------------------------------------------------------------------
// Alarms
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct Alarm {
    pub id: i64,
    pub source_key: String,
    pub source_name: String,
    pub severity: String,
    pub value: f64,
    pub threshold: f64,
    pub message: String,
    pub triggered_at: i64,
    pub resolved_at: Option<i64>,
    pub acknowledged_at: Option<i64>,
    pub acknowledged_by: Option<String>,
    pub cooldown_until: Option<i64>,
}

// ---------------------------------------------------------------------------
// Branding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct BrandingSettings {
    pub title: String,
    pub subtitle: String,
    pub logo: String,
}

// ---------------------------------------------------------------------------
// SSE live data event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct SseLiveData {
    #[serde(flatten)]
    pub sensors: std::collections::HashMap<String, SensorData>,
}
