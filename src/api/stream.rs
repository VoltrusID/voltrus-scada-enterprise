use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::api::data::evaluate_data_sources;
use crate::api::AppState;

pub async fn stream(State(state): State<AppState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = state
        .cache
        .subscribe()
        .then(move |result| {
            let storage = state.storage.clone();
            async move {
                let snapshots = match result {
                    Ok(s) => s,
                    Err(_) => return None,
                };

                let data_sources = match storage.list_data_sources(None).await {
                    Ok(ds) => ds,
                    Err(_) => return None,
                };

                let now = crate::timeutil::now_secs() as f64;
                let evaluated = evaluate_data_sources(&data_sources, &snapshots, now);
                let data = serde_json::to_string(&evaluated).unwrap_or_default();
                Some(Ok(Event::default().data(data)))
            }
        })
        .filter_map(|x| x);

    Sse::new(Box::pin(stream)).keep_alive(KeepAlive::default())
}
