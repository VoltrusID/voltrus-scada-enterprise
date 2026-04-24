use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{generators, AppState, AppError};

/// GET /api/data?screen=X — fetch sensors, generate values using generators
/// For source_type == "modbus": read from cache
pub async fn get_data(
    State(state): State<AppState>,
    Query(params): Query<DataQuery>,
) -> Result<impl IntoResponse, AppError> {
    let screen_id = params.screen.as_deref();
    let sensors = state.storage.list_sensors(screen_id).await?;

    let cache_data = state.cache.get_all().await;
    let now = chrono::Utc::now().timestamp() as f64;

    let mut result = Vec::new();
    for sensor in &sensors {
        let value = if sensor.source_type == "modbus" {
            // Try to get live data from cache
            let mut val = generators::generate_value(sensor, now);
            for snapshot in &cache_data {
                if snapshot.device_id == sensor.device_id {
                    // Extract metric from sensor key
                    if let Some(cached_val) = snapshot.metrics.get(&sensor.key) {
                        val = *cached_val;
                        break;
                    }
                }
            }
            val
        } else {
            generators::generate_value(sensor, now)
        };

        result.push(SensorData {
            id: sensor.id,
            key: sensor.key.clone(),
            value,
            warn: sensor.warn_val,
            crit: sensor.crit_val,
            source_type: sensor.source_type.clone(),
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
    let sensors = state.storage.list_sensors(screen_id).await?;

    let (start, end) = generators::parse_range(range_str);
    let interval_ms = generators::interval_for_range(range_str);
    let duration_secs = end - start;
    let num_points = ((duration_secs * 1000) / interval_ms as i64).min(500) as usize;

    let mut sensor_summaries: Vec<SensorSummary> = Vec::new();
    let mut data_map = serde_json::Map::new();

    for sensor in &sensors {
        sensor_summaries.push(SensorSummary {
            id: sensor.id,
            key: sensor.key.clone(),
            name: sensor.name.clone(),
            unit: sensor.unit.clone(),
            color: sensor.color.clone(),
            warn_val: sensor.warn_val,
            crit_val: sensor.crit_val,
            decimals: sensor.decimals,
        });

        let mut points = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let t = start + ((end - start) * i as i64 / num_points.max(1) as i64);
            let t_f64 = t as f64;
            let value = generators::generate_value(sensor, t_f64);
            points.push(json!({"ts": t * 1000, "value": value}));
        }
        data_map.insert(sensor.key.clone(), json!(points));
    }

    Ok(Json(HistoryResponse {
        sensors: sensor_summaries,
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

    // Find sensors matching the key
    let all_sensors = state.storage.list_sensors(None).await?;
    let sensors: Vec<&Sensor> = if sensor_key.is_empty() {
        all_sensors.iter().collect()
    } else {
        all_sensors
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
    for sensor in &sensors {
        csv.push_str(&format!(",{}", sensor.key));
    }
    csv.push('\n');

    for i in 0..num_points {
        let t = start + ((end - start) * i as i64 / num_points.max(1) as i64);
        csv.push_str(&format!("{}", t));
        for sensor in &sensors {
            let value = generators::generate_value(sensor, t as f64);
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
