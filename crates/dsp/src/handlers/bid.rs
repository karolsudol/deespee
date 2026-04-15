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
    let city = req
        .device
        .as_ref()
        .and_then(|d| d.geo.as_ref())
        .map(|g| g.city.as_str())
        .unwrap_or("unknown");

    println!(
        "🎯 Bid Request: ID={}, User={}, City={}",
        req.id, user_id, city
    );

    // Basic Bidding Logic
    let bid_price = 1.25;

    let resp = deespee::BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        price: bid_price,
        adid: "creative-789".to_string(),
        nurl: format!("http://localhost:8001/win?id={}", req.id),
    };

    // Encode Protobuf BidResponse
    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}
