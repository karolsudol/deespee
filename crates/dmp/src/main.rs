use axum::{
    body::Bytes,
    http::header,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use deespee_proto::deespee;
use prost::Message;
use std::net::SocketAddr;

async fn handle_segments(body: Bytes) -> impl IntoResponse {
    // Decode Protobuf Request
    let req = match deespee::UserSegmentRequest::decode(body) {
        Ok(r) => r,
        Err(_) => return (axum::http::StatusCode::BAD_REQUEST, "Invalid Protobuf").into_response(),
    };

    println!("🧠 DMP Segment Lookup for User: {}", req.user_id);

    // Mock Segment Logic
    // In a real scenario, this would check Firestore/Redis
    let segments = if req.user_id.contains("1") || req.user_id.contains("3") {
        vec![
            "high-value-shopper".to_string(),
            "tech-enthusiast".to_string(),
        ]
    } else {
        vec!["generic-audience".to_string()]
    };

    let resp = deespee::UserSegmentResponse {
        user_id: req.user_id,
        segments,
    };

    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/segments", post(handle_segments));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8002));
    println!("🚀 DMP Service (Rust + Protobuf) listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
