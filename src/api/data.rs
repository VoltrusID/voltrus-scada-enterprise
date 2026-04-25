use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{generators, AppState, AppError};

/// GET /api/data?screen=X — fetch data sources, generate values using generators
/// For source_type == "modbus": read from cache
pub async fn get_data(
    State(state): State<AppState>,
    Query(params): Query<DataQuery>,
) -> Result<impl IntoResponse, AppError> {
    let screen_id = params.screen.as_deref();
    let data_sources = state.storage.list_data_sources(screen_id).await?;

    let cache_data = state.cache.get_all().await;
    let now = chrono::Utc::now().timestamp() as f64;

    let mut result = Vec::new();
    for data_source in &data_sources {
        let value = if data_source.source_type == "modbus" {
            // Try to get live data from cache
            let mut val = generators::generate_value(data_source, now);
            for snapshot in &cache_data {
                if snapshot.device_id == data_source.device_id {
                    // Extract metric from data source key
                    if let Some(cached_val) = snapshot.metrics.get(&data_source.key) {
                        val = *cached_val;
                        break;
                    }
                }
            }
            val
        } else {
            generators::generate_value(data_source, now)
        };

        result.push(DataSourceData {
            id: data_source.id,
            key: data_source.key.clone(),
            value,
            warn: data_source.warn_val,
            crit: data_source.crit_val,
            source_type: data_source.source_type.clone(),
        });
    }

    Ok(Json(result))
}

/// GET /api/history?range=30m&screen=X — generate history data points
pub async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<DataQuery>,
) -> Result<impl IntoResponse, AppError> {
    let range_str = params.range.as_deref().unwrap_or("30m");
    let screen_id = params.screen.as_deref();
    let data_sources = state.storage.list_data_sources(screen_id).await?;

    let (start, end) = generators::parse_range(range_str);
    let interval_ms = generators::interval_for_range(range_str);
    let duration_secs = end - start;
    let num_points = ((duration_secs * 1000) / interval_ms as i64).min(500) as usize;

    let mut data_source_summaries: Vec<DataSourceSummary> = Vec::new();
    let mut data_map = serde_json::Map::new();

    for data_source in &data_sources {
        data_source_summaries.push(DataSourceSummary {
            id: data_source.id,
            key: data_source.key.clone(),
            name: data_source.name.clone(),
            unit: data_source.unit.clone(),
            color: data_source.color.clone(),
            warn_val: data_source.warn_val,
            crit_val: data_source.crit_val,
            decimals: data_source.decimals,
        });

        let mut points = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let t = start + ((end - start) * i as i64 / num_points.max(1) as i64);
            let t_f64 = t as f64;
            let value = generators::generate_value(data_source, t_f64);
            points.push(json!({"ts": t * 1000, "value": value}));
        }
        data_map.insert(data_source.key.clone(), json!(points));
    }

    Ok(Json(HistoryResponse {
        sensors: data_source_summaries,
        data: serde_json::Value::Object(data_map),
        range: range_str.to_string(),
        interval: interval_ms,
    }))
}

/// GET /api/export/csv?sensor=X&range=30m — generate CSV from history data
pub async fn export_csv(
    State(state): State<AppState>,
    Query(params): Query<DataQuery>,
) -> Result<impl IntoResponse, AppError> {
    let range_str = params.range.as_deref().unwrap_or("30m");
    let sensor_key = params.sensor.as_deref().unwrap_or("");

    // Find data sources matching the key
    let all_data_sources = state.storage.list_data_sources(None).await?;
    let data_sources: Vec<&DataSource> = if sensor_key.is_empty() {
        all_data_sources.iter().collect()
    } else {
        all_data_sources
            .iter()
            .filter(|s| s.key == sensor_key)
            .collect()
    };

    let (start, end) = generators::parse_range(range_str);
    let interval_ms = generators::interval_for_range(range_str);
    let duration_secs = end - start;
    let num_points = ((duration_secs * 1000) / interval_ms as i64).min(500) as usize;

    // Build CSV
    let mut csv = String::from("timestamp");
    for data_source in &data_sources {
        csv.push_str(&format!(",{}", data_source.key));
    }
    csv.push('\n');

    for i in 0..num_points {
        let t = start + ((end - start) * i as i64 / num_points.max(1) as i64);
        csv.push_str(&format!("{}", t));
        for data_source in &data_sources {
            let value = generators::generate_value(data_source, t as f64);
            csv.push_str(&format!(",{:.2}", value));
        }
        csv.push('\n');
    }

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv".to_string()),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"export.csv\"".to_string(),
            ),
        ],
        csv,
    ))
}
