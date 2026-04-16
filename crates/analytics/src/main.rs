mod models;
mod storage;

use crate::models::{events_to_record_batch, UnifiedEvent};
use crate::storage::LakehouseStorage;
use axum::{
    body::Bytes, extract::State, http::StatusCode, response::IntoResponse, routing::post, Router,
};
use deespee_proto::deespee;
use prost::Message;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct AppState {
    buffer: Mutex<Vec<UnifiedEvent>>,
    storage: LakehouseStorage,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    println!("❄️ Rust Lakehouse Engine starting on port 8004...");

    let state = Arc::new(AppState {
        buffer: Mutex::new(Vec::new()),
        storage: LakehouseStorage::new("data/lakehouse/events"),
    });

    // Background task: Periodically flush buffer to Parquet
    let flush_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let events_to_flush = {
                let mut buf = flush_state.buffer.lock().unwrap();
                if buf.is_empty() {
                    continue;
                }
                std::mem::take(&mut *buf)
            };

            let count_raw = events_to_flush.len();

            // Deduplication logic: Keep only the first occurrence of each event_id in this batch
            let mut seen = std::collections::HashSet::new();
            let deduplicated_events: Vec<_> = events_to_flush
                .into_iter()
                .filter(|e| seen.insert(e.event_id.clone()))
                .collect();

            let count = deduplicated_events.len();
            if count < count_raw {
                println!(
                    "🧹 Deduplication: Filtered {} duplicate events",
                    count_raw - count
                );
            }

            println!("⚡ Background: Flushing {} events to Lakehouse...", count);
            let batch = events_to_record_batch(deduplicated_events);
            if let Err(e) = flush_state.storage.write_batch(&batch, "fct_events") {
                eprintln!("❌ Failed to flush events: {}", e);
            } else {
                println!("✅ Successfully persisted {} events.", count);
            }
        }
    });

    let app = Router::new()
        .route("/events", post(handle_event_notification))
        .route("/tracking", post(handle_tracking_event))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8004));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handle_event_notification(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> impl IntoResponse {
    if let Ok(event) = deespee::EventNotification::decode(body) {
        let unified = UnifiedEvent::from_notification(event);
        state.buffer.lock().unwrap().push(unified);
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}

async fn handle_tracking_event(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> impl IntoResponse {
    if let Ok(event) = deespee::TrackingEvent::decode(body) {
        let unified = UnifiedEvent::from_tracking(event);
        state.buffer.lock().unwrap().push(unified);
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}
