use crate::models::UnifiedEvent;
use crate::AppState;
use axum::{body::Bytes, extract::State, http::StatusCode, response::IntoResponse};
use deespee_proto::deespee;
use prost::Message;
use std::sync::Arc;

pub async fn handle_event_notification(
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

pub async fn handle_tracking_event(
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
