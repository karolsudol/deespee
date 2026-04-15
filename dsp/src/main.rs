use axum::{
    extract::Json,
    routing::post,
    Router,
};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use base64::{engine::general_purpose, Engine as _};

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

async fn handle_pubsub_push(Json(payload): Json<PubSubMessage>) -> &'static str {
    println!("Received Pub/Sub message: {:?}", payload.message.message_id);

    // Decode base64 data
    let decoded_data = match general_purpose::STANDARD.decode(&payload.message.data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to decode base64: {}", e);
            return "Error: Invalid base64";
        }
    };

    // Decode Protobuf
    match deespee::AgentRequest::decode(&decoded_data[..]) {
        Ok(request) => {
            println!("Decoded AgentRequest: ID={}, Query={}", request.request_id, request.query);
            // Here you would add bidding logic and publish to dsp-actions topic
        }
        Err(e) => {
            eprintln!("Failed to decode protobuf: {}", e);
            return "Error: Invalid protobuf";
        }
    }

    "OK"
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/pubsub/push", post(handle_pubsub_push));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    println!("DSP Service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
