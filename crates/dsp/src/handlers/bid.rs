use axum::{body::Bytes, http::header, response::IntoResponse};
use deespee_proto::deespee;
use prost::Message;

pub async fn handle_bid(body: Bytes) -> impl IntoResponse {
    // Decode Protobuf BidRequest
    let req = match deespee::BidRequest::decode(body) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to decode BidRequest: {}", e);
            return (axum::http::StatusCode::BAD_REQUEST, "Invalid Protobuf").into_response();
        }
    };

    let user_id = req
        .user
        .as_ref()
        .map(|u| u.id.as_str())
        .unwrap_or("unknown");
    let device_info = req
        .device
        .as_ref()
        .map(|d| format!("{} {}", d.make, d.model))
        .unwrap_or("unknown".to_string());

    println!(
        "🎯 Bid Request: ID={}, User={}, Device={}",
        req.id, user_id, device_info
    );

    // Basic Bidding Logic
    let bid_price = 1.25;

    let resp = deespee::BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        price: bid_price,
        adid: "creative-123".to_string(),
        crid: "cr-456".to_string(),
        adm: "<html><body><h1>Ad Served</h1></body></html>".to_string(),
        nurl: format!("http://localhost:8001/win?id={}", req.id),
        cat: vec!["IAB1".to_string()],
    };

    // Encode Protobuf BidResponse
    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}
