use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::api::AppState;

/// GET /api/v1/stream — SSE stream of real-time events
#[utoipa::path(
    get,
    path = "/api/v1/stream",
    responses((status = 200, description = "SSE stream of real-time events")),
    tag = "Stream",
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub async fn stream(State(state): State<AppState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = state
        .cache
        .subscribe_sse()
        .filter_map(|result| match result {
            Ok(payload) => Some(Ok(Event::default().data(payload))),
            Err(_) => None,
        });

    Sse::new(Box::pin(stream)).keep_alive(KeepAlive::default())
}

#[derive(serde::Serialize)]
struct StreamPayload {
    sensors: Vec<crate::models::DataSourceData>,
}

pub fn spawn_sse_composer(cache: crate::cache::DeviceCache, storage: crate::storage::Storage) {
    tokio::spawn(async move {
        let mut rx = cache.trigger_receiver();
        loop {
            if rx.changed().await.is_err() {
                break;
            }

            let data_sources = match storage.list_data_sources(None).await {
                Ok(ds) => ds,
                Err(e) => {
                    tracing::warn!("SSE composer: list_data_sources failed: {e}");
                    continue;
                }
            };

            let snapshots = cache.get_all().await;
            let now = crate::timeutil::now_secs() as f64;
            let sensors = super::data::evaluate_data_sources(&data_sources, &snapshots, now);

            let payload = StreamPayload { sensors };
            let json = serde_json::to_string(&payload).unwrap_or_default();
            cache.publish_sse_payload(json);
        }
    });
}
