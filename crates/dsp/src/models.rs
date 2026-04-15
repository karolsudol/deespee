use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PubSubMessage {
    pub message: MessageData,
    pub subscription: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageData {
    pub data: String, // Base64 encoded
    pub message_id: String,
    pub publish_time: String,
}
