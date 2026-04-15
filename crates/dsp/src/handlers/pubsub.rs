use crate::models::PubSubMessage;
use axum::extract::Json;
use base64::{engine::general_purpose, Engine as _};
use deespee_proto::deespee;
use prost::Message;

pub async fn handle_pubsub_push(Json(payload): Json<PubSubMessage>) -> &'static str {
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
