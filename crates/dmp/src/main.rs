use axum::{
    body::Bytes,
    http::header,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use deespee_proto::deespee;
use prost::Message;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

// Shared state for User Frequency
struct AppState {
    frequency: Mutex<HashMap<String, u32>>,
}

async fn handle_segments(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    body: Bytes,
) -> impl IntoResponse {
    let req = match deespee::UserSegmentRequest::decode(body) {
        Ok(r) => r,
        Err(_) => return (axum::http::StatusCode::BAD_REQUEST, "Invalid Protobuf").into_response(),
    };

    println!("🧠 DMP Segment Lookup for User: {}", req.user_id);

    let freq_map = state.frequency.lock().unwrap();
    let count = freq_map.get(&req.user_id).cloned().unwrap_or(0);

    let mut segments = if req.user_id.contains("1") || req.user_id.contains("3") {
        vec!["high-value-shopper".to_string()]
    } else {
        vec!["generic-audience".to_string()]
    };

    // Add Capping Segment
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

async fn handle_pubsub_push(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    body: Bytes,
) -> impl IntoResponse {
    // Note: In real Cloud Run, Pub/Sub sends a JSON envelope.
    // For this internal direct call loop, we decode the binary EventNotification directly.
    let event = match deespee::EventNotification::decode(body) {
        Ok(e) => e,
        Err(_) => return (axum::http::StatusCode::BAD_REQUEST, "Invalid Protobuf").into_response(),
    };

    if event.r#type == deespee::event_notification::EventType::Win as i32 {
        let mut freq_map = state.frequency.lock().unwrap();
        let counter = freq_map.entry(event.user_id.clone()).or_insert(0);
        *counter += 1;
        println!(
            "📈 Incrementing frequency for user {}: new count = {}",
            event.user_id, counter
        );
    }

    "Event Processed".into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState {
        frequency: Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/segments", post(handle_segments))
        .route("/pubsub/push", post(handle_pubsub_push))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8002));
    println!("🚀 DMP Service (Rust + Memory State) listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
