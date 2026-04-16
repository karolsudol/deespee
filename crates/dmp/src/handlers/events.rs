use crate::state::AppState;
use axum::{body::Bytes, http::StatusCode, response::IntoResponse};
use deespee_proto::deespee;
use prost::Message;
use std::sync::Arc;

pub async fn handle_pubsub_push(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    body: Bytes,
) -> impl IntoResponse {
    let event = match deespee::EventNotification::decode(body) {
        Ok(e) => e,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    if event.r#type == deespee::event_notification::EventType::Win as i32 {
        // Increment user frequency
        {
            let mut freq_map = state.frequency.lock().unwrap();
            let counter = freq_map.entry(event.user_id.clone()).or_insert(0);
            *counter += 1;
            println!(
                "📈 Incrementing frequency for user {}: new count = {}",
                event.user_id, counter
            );
        }

        // Increment campaign spend
        if !event.campaign_id.is_empty() {
            let mut states_map = state.campaign_states.lock().unwrap();
            let spend = states_map.entry(event.campaign_id.clone()).or_insert(0.0);
            *spend += event.cost;
            println!(
                "💰 Campaign {} spend updated: ${:.2}",
                event.campaign_id, spend
            );
        }
    }

    StatusCode::OK.into_response()
}
