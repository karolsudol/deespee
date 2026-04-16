use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use base64::{engine::general_purpose, Engine as _};
use deespee_proto::deespee;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct TrackParams {
    pub bid_id: String,
    pub user_id: String,
    pub campaign_id: String,
}

pub async fn handle_impression(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<TrackParams>,
) -> impl IntoResponse {
    if !state.verification.is_valid(&headers) {
        state.record_event("fraud_impression");
        return (axum::http::StatusCode::FORBIDDEN, "Invalid Traffic").into_response();
    }

    log_event(
        &params,
        deespee::tracking_event::InteractionType::Impression,
    );
    state.record_event("impression");

    let pixel = general_purpose::STANDARD
        .decode("R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7")
        .unwrap();
    ([(axum::http::header::CONTENT_TYPE, "image/gif")], pixel).into_response()
}

pub async fn handle_click(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<TrackParams>,
) -> impl IntoResponse {
    if !state.verification.is_valid(&headers) {
        state.record_event("fraud_click");
        return (axum::http::StatusCode::FORBIDDEN, "Invalid Traffic").into_response();
    }

    log_event(&params, deespee::tracking_event::InteractionType::Click);
    state.record_event("click");
    "Click Recorded".into_response()
}

pub async fn handle_viewability(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<TrackParams>,
) -> impl IntoResponse {
    if !state.verification.is_valid(&headers) {
        state.record_event("fraud_viewability");
        return (axum::http::StatusCode::FORBIDDEN, "Invalid Traffic").into_response();
    }

    log_event(
        &params,
        deespee::tracking_event::InteractionType::Viewability,
    );
    state.record_event("viewability");

    let pixel = general_purpose::STANDARD
        .decode("R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7")
        .unwrap();
    ([(axum::http::header::CONTENT_TYPE, "image/gif")], pixel).into_response()
}

pub async fn handle_conversion(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<TrackParams>,
) -> impl IntoResponse {
    // Conversions are usually higher security, but for now we follow same pattern
    if !state.verification.is_valid(&headers) {
        state.record_event("fraud_conversion");
        return (axum::http::StatusCode::FORBIDDEN, "Invalid Traffic").into_response();
    }

    log_event(
        &params,
        deespee::tracking_event::InteractionType::Conversion,
    );
    state.record_event("conversion");
    "Conversion Recorded".into_response()
}

fn log_event(params: &TrackParams, event_type: deespee::tracking_event::InteractionType) {
    println!(
        "📊 TRACKING: Type={:?}, BidID={}, UserID={}, CampaignID={}",
        event_type, params.bid_id, params.user_id, params.campaign_id
    );

    // TODO: Validate the bid_id (did we actually win this?)
    // TODO: Write to a high-throughput buffer (Kafka/PubSub)
    // TODO: Eventually land in BigQuery
}
