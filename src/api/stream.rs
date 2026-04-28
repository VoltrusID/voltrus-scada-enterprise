use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::api::AppState;

pub async fn stream(State(state): State<AppState>) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let stream = state.cache.subscribe().filter_map(|result| match result {
        Ok(snapshots) => {
            let data = serde_json::to_string(&snapshots).unwrap_or_default();
            Some(Ok(Event::default().data(data)))
        }
        Err(_) => None,
    });

    Sse::new(Box::pin(stream)).keep_alive(KeepAlive::default())
}
