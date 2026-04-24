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
        let state = AppState { cache, storage };
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
        let state = AppState { cache: cache.clone(), storage };
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
        let state = AppState { cache: cache.clone(), storage };
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

        let state = AppState { cache, storage };
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
}
