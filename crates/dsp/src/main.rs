mod handlers;
mod models;

use crate::handlers::{bid::handle_bid, pubsub::handle_pubsub_push};
use axum::{routing::post, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Setup routes
    let app = Router::new()
        .route("/pubsub/push", post(handle_pubsub_push))
        .route("/bid", post(handle_bid));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    println!("🚀 DSP Service (Rust + Protobuf) listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
