mod handlers;
mod models;
mod reader;
mod storage;

use crate::handlers::{
    ingest::{handle_event_notification, handle_tracking_event},
    query::handle_query,
};
use crate::models::{events_to_record_batch, UnifiedEvent};
use crate::storage::LakehouseStorage;
use axum::{routing::post, Router};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AppState {
    pub buffer: Mutex<Vec<UnifiedEvent>>,
    pub storage: LakehouseStorage,
    pub base_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    println!("❄️ Rust Lakehouse Engine starting on port 8004...");

    let base_path = "data/lakehouse/events".to_string();

    // Create storage directory if it doesn't exist
    if !std::path::Path::new(&base_path).exists() {
        std::fs::create_dir_all(&base_path).expect("Failed to create storage directory");
    }

    let state = Arc::new(AppState {
        buffer: Mutex::new(Vec::new()),
        storage: LakehouseStorage::new(&base_path),
        base_path,
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

            // Deduplication logic
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
        .route("/query", post(handle_query))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8004));
    println!("🔍 Analytics API active on port 8004 (/events, /tracking, /query)");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
