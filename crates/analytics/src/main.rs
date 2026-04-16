mod models;
mod storage;

use crate::models::{events_to_record_batch, UnifiedEvent};
use crate::storage::LakehouseStorage;
use deespee_proto::deespee;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    println!("❄️ Rust Lakehouse Engine (Arrow + Parquet) starting...");

    // 1. Setup Lakehouse Storage
    let storage = LakehouseStorage::new("data/lakehouse/events");

    // 2. Mock some events (Simulating real-time arrival)
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    let mock_events = vec![
        UnifiedEvent::from_notification(deespee::EventNotification {
            event_id: uuid::Uuid::new_v4().to_string(),
            r#type: deespee::event_notification::EventType::Win as i32,
            user_id: "user-1".to_string(),
            bid_id: "bid-1".to_string(),
            ad_id: "ad-1".to_string(),
            cost: 2.50,
            timestamp: now as u64,
            campaign_id: "camp-1".to_string(),
        }),
        UnifiedEvent::from_notification(deespee::EventNotification {
            event_id: uuid::Uuid::new_v4().to_string(),
            r#type: deespee::event_notification::EventType::Impression as i32,
            user_id: "user-1".to_string(),
            bid_id: "bid-1".to_string(),
            ad_id: "ad-1".to_string(),
            cost: 0.0,
            timestamp: (now + 1) as u64,
            campaign_id: "camp-1".to_string(),
        }),
    ];

    // 3. TRANSFORM: Convert to Arrow RecordBatch (Zero-copy ready)
    let batch = events_to_record_batch(mock_events);
    println!(
        "⚡ Real-time: Processed Arrow batch with {} rows",
        batch.num_rows()
    );

    // 4. LOAD: Sink to Parquet (Lakehouse file)
    storage.write_batch(&batch, "fct_events")?;

    println!("✅ Phase 4 Step 2 Complete: Real-time Arrow Pipeline verified.");
    Ok(())
}
