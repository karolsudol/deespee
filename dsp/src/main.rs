use axum::{body::Bytes, http::header, response::IntoResponse, routing::post, Router};
use base64::{engine::general_purpose, Engine as _};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// Include generated protobuf code
pub mod deespee {
    include!(concat!(env!("OUT_DIR"), "/deespee.rs"));
}

#[derive(Debug, Serialize, Deserialize)]
struct PubSubMessage {
    message: MessageData,
    subscription: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageData {
    data: String, // Base64 encoded
    message_id: String,
    publish_time: String,
}

async fn handle_pubsub_push(
    axum::extract::Json(payload): axum::extract::Json<PubSubMessage>,
) -> &'static str {
    println!("Received Pub/Sub message: {:?}", payload.message.message_id);

    let decoded_data = match general_purpose::STANDARD.decode(&payload.message.data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to decode base64: {}", e);
            return "Error: Invalid base64";
        }
    };

    match deespee::AgentRequest::decode(&decoded_data[..]) {
        Ok(request) => {
            println!(
                "Decoded AgentRequest: ID={}, Query={}",
                request.request_id, request.query
            );
        }
        Err(e) => {
            eprintln!("Failed to decode protobuf: {}", e);
            return "Error: Invalid protobuf";
        }
    }

    "OK"
}

async fn handle_bid(body: Bytes) -> impl IntoResponse {
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/pubsub/push", post(handle_pubsub_push))
        .route("/bid", post(handle_bid));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    println!("🚀 DSP Service (Rust + Protobuf) listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
