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

    println!(
        "🎯 Received Bid Request: ID={}, User={}",
        req.id,
        req.user
            .as_ref()
            .map(|u| u.id.as_str())
            .unwrap_or("unknown")
    );

    // Bidding Logic
    let bid_price = 1.25;

    let resp = deespee::BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        price: bid_price,
    };

    // Encode Protobuf BidResponse
    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}
