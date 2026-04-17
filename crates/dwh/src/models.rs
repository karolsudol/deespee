use arrow::array::{Float32Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use deespee_proto::deespee;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedEvent {
    pub event_id: String,
    pub event_type: String,
    pub user_id: String,
    pub campaign_id: String,
    pub bid_id: String,
    pub cost: f32,
    pub timestamp: i64,
}

impl UnifiedEvent {
    pub fn from_notification(event: deespee::EventNotification) -> Self {
        let event_type = match deespee::event_notification::EventType::try_from(event.r#type) {
            Ok(deespee::event_notification::EventType::Win) => "win",
            Ok(deespee::event_notification::EventType::Impression) => "impression",
            Ok(deespee::event_notification::EventType::Click) => "click",
            Ok(deespee::event_notification::EventType::Conversion) => "conversion",
            Err(_) => "unknown",
        };

        Self {
            event_id: event.event_id,
            event_type: event_type.to_string(),
            user_id: event.user_id,
            campaign_id: event.campaign_id,
            bid_id: event.bid_id,
            cost: event.cost,
            timestamp: event.timestamp as i64,
        }
    }

    pub fn from_tracking(event: deespee::TrackingEvent) -> Self {
        let event_type = match deespee::tracking_event::InteractionType::try_from(event.r#type) {
            Ok(deespee::tracking_event::InteractionType::Impression) => "impression",
            Ok(deespee::tracking_event::InteractionType::Click) => "click",
            Ok(deespee::tracking_event::InteractionType::Conversion) => "conversion",
            Ok(deespee::tracking_event::InteractionType::Viewability) => "viewability",
            Err(_) => "unknown",
        };

        Self {
            event_id: event.event_id,
            event_type: event_type.to_string(),
            user_id: event.user_id,
            campaign_id: event.campaign_id,
            bid_id: event.bid_id,
            cost: 0.0,
            timestamp: event.timestamp as i64,
        }
    }
}

pub fn events_to_record_batch(events: Vec<UnifiedEvent>) -> RecordBatch {
    let schema = Arc::new(arrow::datatypes::Schema::new(vec![
        arrow::datatypes::Field::new("event_id", arrow::datatypes::DataType::Utf8, false),
        arrow::datatypes::Field::new("event_type", arrow::datatypes::DataType::Utf8, false),
        arrow::datatypes::Field::new("user_id", arrow::datatypes::DataType::Utf8, false),
        arrow::datatypes::Field::new("campaign_id", arrow::datatypes::DataType::Utf8, false),
        arrow::datatypes::Field::new("bid_id", arrow::datatypes::DataType::Utf8, false),
        arrow::datatypes::Field::new("cost", arrow::datatypes::DataType::Float32, false),
        arrow::datatypes::Field::new("timestamp", arrow::datatypes::DataType::Int64, false),
    ]));

    let event_ids = StringArray::from(
        events
            .iter()
            .map(|e| e.event_id.as_str())
            .collect::<Vec<_>>(),
    );
    let event_types = StringArray::from(
        events
            .iter()
            .map(|e| e.event_type.as_str())
            .collect::<Vec<_>>(),
    );
    let user_ids = StringArray::from(
        events
            .iter()
            .map(|e| e.user_id.as_str())
            .collect::<Vec<_>>(),
    );
    let campaign_ids = StringArray::from(
        events
            .iter()
            .map(|e| e.campaign_id.as_str())
            .collect::<Vec<_>>(),
    );
    let bid_ids = StringArray::from(events.iter().map(|e| e.bid_id.as_str()).collect::<Vec<_>>());
    let costs = Float32Array::from(events.iter().map(|e| e.cost).collect::<Vec<_>>());
    let timestamps = Int64Array::from(events.iter().map(|e| e.timestamp).collect::<Vec<_>>());

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(event_ids),
            Arc::new(event_types),
            Arc::new(user_ids),
            Arc::new(campaign_ids),
            Arc::new(bid_ids),
            Arc::new(costs),
            Arc::new(timestamps),
        ],
    )
    .expect("Failed to create RecordBatch")
}
