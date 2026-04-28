use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::expression;
use crate::models::*;
use super::{generators, AppState, AppError};

/// GET /api/data?screen=X — fetch data sources, generate values using generators
/// For source_type == "modbus": read from cache, evaluate expression if present
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
            // Find the device's cached metrics
            let mut metrics: Option<&std::collections::HashMap<String, f64>> = None;
            for snapshot in &cache_data {
                if snapshot.device_id == data_source.device_id
                    && snapshot.status == DeviceStatus::Online
                {
                    metrics = Some(&snapshot.metrics);
                    break;
                }
            }

            match metrics {
                None => f64::NAN,
                Some(m) => {
                    // Check if source_config has an expression
                    if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&data_source.source_config) {
                        if let Some(expr_str) = cfg.get("expression").and_then(|v| v.as_str()) {
                            match expression::evaluate(expr_str, m) {
                                Ok(val) => val,
                                Err(e) => {
                                    tracing::warn!("Expression eval error for {}: {e}", data_source.key);
                                    f64::NAN
                                }
                            }
                        } else {
                            // Fallback: single register lookup by key
                            m.get(&data_source.key).copied().unwrap_or(f64::NAN)
                        }
                    } else {
                        m.get(&data_source.key).copied().unwrap_or(f64::NAN)
                    }
                }
            }
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

/// GET /api/history?range=30m&screen=X&agg=sampling — generate history data points
/// `agg`: "sampling" = pick evenly-spaced points, "avg" = average per bucket, "minmax" = min/max/avg per bucket
/// Returns compact format: each sensor has {values: [f64], start: unix_ms, interval: ms, count: usize}
/// For minmax: additionally includes {mins: [f64], maxs: [f64]}
pub async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<DataQuery>,
) -> Result<impl IntoResponse, AppError> {
    let range_str = params.range.as_deref().unwrap_or("30m");
    let detail = params.detail.as_deref();
    let agg = params.agg.as_deref().unwrap_or("sampling");
    let screen_id = params.screen.as_deref();
    let data_sources = state.storage.list_data_sources(screen_id).await?;

    let (start, end) = generators::parse_range(range_str);
    let interval_ms = generators::interval_for_range(range_str, detail);
    let duration_secs = end - start;
    let num_points = ((duration_secs * 1000) / interval_ms as i64) as usize;

    let mut data_source_summaries: Vec<DataSourceSummary> = Vec::new();
    let mut data_map = serde_json::Map::new();

    // For avg/minmax: number of sub-samples within each bucket interval
    let sub_samples = if agg == "sampling" { 1 } else { (interval_ms / 1000).max(1) as usize };

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

        let mut values = Vec::with_capacity(num_points);
        let mut mins: Vec<f64> = Vec::new();
        let mut maxs: Vec<f64> = Vec::new();

        for i in 0..num_points {
            let bucket_start = start + ((end - start) * i as i64 / num_points.max(1) as i64);

            match agg {
                "avg" | "minmax" => {
                    let (avg, mn, mx) = generators::aggregate_bucket(
                        data_source, bucket_start as f64, sub_samples,
                    );
                    values.push(avg);
                    if agg == "minmax" {
                        mins.push(mn);
                        maxs.push(mx);
                    }
                }
                _ => {
                    // sampling: just pick the value at the bucket start
                    let value = generators::generate_value(data_source, bucket_start as f64);
                    values.push((value * 100.0).round() / 100.0);
                }
            }
        }

        let mut sensor_data = serde_json::Map::new();
        sensor_data.insert("values".into(), json!(values));
        sensor_data.insert("start".into(), json!(start * 1000));
        sensor_data.insert("interval".into(), json!(interval_ms));
        sensor_data.insert("count".into(), json!(num_points));
        if agg == "minmax" {
            sensor_data.insert("mins".into(), json!(mins));
            sensor_data.insert("maxs".into(), json!(maxs));
        }
        data_map.insert(data_source.key.clone(), json!(sensor_data));
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
    let interval_ms = generators::interval_for_range(range_str, params.detail.as_deref());
    let duration_secs = end - start;
    let num_points = ((duration_secs * 1000) / interval_ms as i64) as usize;

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
