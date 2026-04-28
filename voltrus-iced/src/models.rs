use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub ok: bool,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataSourceData {
    pub id: i64,
    pub key: String,
    pub value: f64,
    pub warn: f64,
    pub crit: f64,
    pub source_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataSourceSummary {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub unit: String,
    pub color: String,
    pub warn_val: f64,
    pub crit_val: f64,
    pub decimals: i64,
    pub source_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataPoint {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryResponse {
    pub sensors: Vec<DataSourceSummary>,
    pub data: serde_json::Value,
    pub range: String,
    pub interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrandingSettings {
    pub title: String,
    pub subtitle: String,
    pub logo: String,
}
