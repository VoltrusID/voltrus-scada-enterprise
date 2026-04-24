use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use crate::cache::DeviceCache;
use crate::models::*;
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub cache: DeviceCache,
    pub storage: Storage,
    pub started_at: std::time::Instant,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/live", get(get_live))
        .route("/api/health", get(get_health))
        .route("/api/history", get(get_history))
        .route("/api/system", get(get_system))
        .with_state(state)
        .layer(cors)
}

async fn get_live(State(state): State<AppState>) -> Json<LiveResponse> {
    let devices = state.cache.get_all().await;
    Json(LiveResponse { devices })
}

async fn get_health(State(state): State<AppState>) -> Json<HealthResponse> {
    let statuses = state.cache.get_statuses().await;
    Json(HealthResponse {
        status: "ok".to_string(),
        devices: statuses,
    })
}

async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Vec<DataPoint>>, AppError> {
    let now = chrono::Utc::now().timestamp();
    let from = params.from.unwrap_or(now - 3600);
    let to = params.to.unwrap_or(now);

    let data = state
        .storage
        .query_history(&params.device_id, &params.metric, from, to)
        .await?;

    Ok(Json(data))
}

async fn get_system(State(state): State<AppState>) -> Result<Json<SystemResponse>, AppError> {
    use sysinfo::{Disks, ProcessesToUpdate, ProcessRefreshKind, System};

    let mut sys = System::new_all();
    sys.refresh_all();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu(),
    );

    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let os = System::name().unwrap_or_else(|| "unknown".to_string());
    let uptime_secs = System::uptime();
    let cpu_usage = sys.global_cpu_usage();
    let mem_total = sys.total_memory() as f64 / (1024.0 * 1024.0);
    let mem_used = sys.used_memory() as f64 / (1024.0 * 1024.0);
    let mem_percent = if mem_total > 0.0 { (mem_used / mem_total) * 100.0 } else { 0.0 };

    let disks = Disks::new_with_refreshed_list();
    let disk = disks.list().iter()
        .find(|d| d.mount_point().to_str().unwrap_or("") == "/")
        .or_else(|| disks.list().first());

    let (disk_total_gb, disk_used_gb, disk_percent) = if let Some(d) = disk {
        let total = d.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
        let available = d.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);
        let used = total - available;
        let pct = if total > 0.0 { (used / total) * 100.0 } else { 0.0 };
        (total, used, pct)
    } else {
        (0.0, 0.0, 0.0)
    };

    let my_pid = sysinfo::Pid::from(std::process::id() as usize);
    let process_info = if let Some(proc) = sys.process(my_pid) {
        let proc_start = proc.start_time();
        let now = chrono::Utc::now().timestamp() as u64;
        let proc_uptime = now.saturating_sub(proc_start);
        let binary_size_mb = std::fs::metadata(std::env::current_exe().unwrap_or_default())
            .map(|m| m.len() as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0);
        ProcessInfo {
            pid: std::process::id(),
            memory_mb: proc.memory() as f64 / (1024.0 * 1024.0),
            cpu_percent: proc.cpu_usage(),
            uptime_secs: proc_uptime,
            binary_size_mb,
        }
    } else {
        ProcessInfo {
            pid: std::process::id(),
            memory_mb: 0.0,
            cpu_percent: 0.0,
            uptime_secs: state.started_at.elapsed().as_secs(),
            binary_size_mb: 0.0,
        }
    };

    Ok(Json(SystemResponse {
        hostname,
        os,
        uptime_secs,
        cpu_usage: (cpu_usage * 10.0).round() / 10.0,
        memory_total_mb: (mem_total * 10.0).round() / 10.0,
        memory_used_mb: (mem_used * 10.0).round() / 10.0,
        memory_percent: (mem_percent * 10.0).round() / 10.0,
        disk_total_gb: (disk_total_gb * 10.0).round() / 10.0,
        disk_used_gb: (disk_used_gb * 10.0).round() / 10.0,
        disk_percent: (disk_percent * 10.0).round() / 10.0,
        process: process_info,
    }))
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": self.0.to_string()})),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        AppError(err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_app() -> Router {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState { cache, storage, started_at: std::time::Instant::now() };
        create_router(state)
    }

    async fn seed_cache(cache: &DeviceCache) {
        let metrics = HashMap::from([
            ("voltage".to_string(), 220.5),
            ("current".to_string(), 5.2),
        ]);
        cache.update("plts_1", metrics).await;
    }

    #[tokio::test]
    async fn test_live_empty() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["devices"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_live_with_data() {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState { cache: cache.clone(), storage, started_at: std::time::Instant::now() };
        let app = create_router(state);

        seed_cache(&cache).await;

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let devices = json["devices"].as_array().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0]["device_id"], "plts_1");
        assert_eq!(devices[0]["status"], "online");
        assert_eq!(devices[0]["metrics"]["voltage"], 220.5);
    }

    #[tokio::test]
    async fn test_health() {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState { cache: cache.clone(), storage, started_at: std::time::Instant::now() };
        let app = create_router(state);

        seed_cache(&cache).await;

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["devices"]["plts_1"], "online");
    }

    #[tokio::test]
    async fn test_history_with_data() {
        let cache = DeviceCache::new();
        let storage = Storage::new_in_memory().unwrap();

        storage
            .insert_batch(&[
                ("plts_1".into(), 1000, "voltage".into(), 220.0),
                ("plts_1".into(), 1060, "voltage".into(), 221.0),
            ])
            .await
            .unwrap();

        let state = AppState { cache, storage, started_at: std::time::Instant::now() };
        let app = create_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/history?device_id=plts_1&metric=voltage&from=0&to=2000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let points = json.as_array().unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0]["value"], 220.0);
        assert_eq!(points[1]["value"], 221.0);
    }

    #[tokio::test]
    async fn test_system_endpoint() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/system")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["hostname"].is_string());
        assert!(json["os"].is_string());
        assert!(json["cpu_usage"].is_number());
        assert!(json["memory_total_mb"].is_number());
        assert!(json["process"]["pid"].is_number());
        assert!(json["process"]["memory_mb"].is_number());
    }
}
