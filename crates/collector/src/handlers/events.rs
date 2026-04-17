use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use base64::{engine::general_purpose, Engine as _};
use deespee_proto::deespee;
use prost::Message;
use serde::Deserialize;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
        state.record_metric(&params.campaign_id, "fraud_impression");
        return (axum::http::StatusCode::FORBIDDEN, "Invalid Traffic").into_response();
    }

    log_event(
        &params,
        deespee::tracking_event::InteractionType::Impression,
    );
    state.record_metric(&params.campaign_id, "impression");

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
        state.record_metric(&params.campaign_id, "fraud_click");
        return (axum::http::StatusCode::FORBIDDEN, "Invalid Traffic").into_response();
    }

    log_event(&params, deespee::tracking_event::InteractionType::Click);
    state.record_metric(&params.campaign_id, "click");
    "Click Recorded".into_response()
}

pub async fn handle_viewability(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<TrackParams>,
) -> impl IntoResponse {
    if !state.verification.is_valid(&headers) {
        state.record_metric(&params.campaign_id, "fraud_viewability");
        return (axum::http::StatusCode::FORBIDDEN, "Invalid Traffic").into_response();
    }

    log_event(
        &params,
        deespee::tracking_event::InteractionType::Viewability,
    );
    state.record_metric(&params.campaign_id, "viewability");

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
    if !state.verification.is_valid(&headers) {
        state.record_metric(&params.campaign_id, "fraud_conversion");
        return (axum::http::StatusCode::FORBIDDEN, "Invalid Traffic").into_response();
    }

    log_event(
        &params,
        deespee::tracking_event::InteractionType::Conversion,
    );
    state.record_metric(&params.campaign_id, "conversion");
    "Conversion Recorded".into_response()
}

pub async fn handle_win_notify(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    if let Ok(event) = deespee::EventNotification::decode(body) {
        println!(
            "🔔 RECONCILIATION: Win reported for Campaign={}",
            event.campaign_id
        );
        state.record_metric(&event.campaign_id, "win");
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::BAD_REQUEST
    }
}

pub async fn handle_report(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.get_discrepancy_report()
}

fn log_event(params: &TrackParams, event_type: deespee::tracking_event::InteractionType) {
    println!(
        "📊 TRACKING: Type={:?}, BidID={}, UserID={}, CampaignID={}",
        event_type, params.bid_id, params.user_id, params.campaign_id
    );

    let event = deespee::TrackingEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        r#type: event_type as i32,
        user_id: params.user_id.clone(),
        campaign_id: params.campaign_id.clone(),
        bid_id: params.bid_id.clone(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        metadata: std::collections::HashMap::new(),
    };

    let mut body = Vec::new();
    event.encode(&mut body).unwrap();

    // Async notify Analytics
    tokio::spawn(async move {
        let analytics_host = "localhost:8004";
        if let Ok(mut stream) = TcpStream::connect(analytics_host) {
            let http_request = format!(
                "POST /tracking HTTP/1.1\r\n\
                 Host: {}\r\n\
                 Content-Type: application/x-protobuf\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\r\n",
                analytics_host,
                body.len()
            );
            let _ = stream.write_all(http_request.as_bytes());
            let _ = stream.write_all(&body);
        }
    });

    // TODO: Validate the bid_id (did we actually win this?)
    // TODO: Write to a high-throughput buffer (Kafka/PubSub)
    // TODO: Eventually land in BigQuery
}
