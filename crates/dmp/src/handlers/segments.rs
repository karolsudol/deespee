use crate::state::AppState;
use axum::{
    body::Bytes,
    http::{header, StatusCode},
    response::IntoResponse,
};
use deespee_proto::deespee;
use prost::Message;
use std::sync::Arc;

pub async fn handle_segments(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    body: Bytes,
) -> impl IntoResponse {
    let req = match deespee::UserSegmentRequest::decode(body) {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid Protobuf").into_response(),
    };

    println!("🧠 DMP Segment Lookup for User: {}", req.user_id);

    let freq_map = state.frequency.lock().unwrap();
    let count = freq_map.get(&req.user_id).cloned().unwrap_or(0);

    let mut segments = if req.user_id.contains("1") || req.user_id.contains("3") {
        vec!["high-value-shopper".to_string()]
    } else {
        vec!["generic-audience".to_string()]
    };

    if count >= 3 {
        println!(
            "⚠️ User {} is frequency capped ({} impressions)",
            req.user_id, count
        );
        segments.push("capped".to_string());
    }

    let resp = deespee::UserSegmentResponse {
        user_id: req.user_id,
        segments,
    };

    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}
