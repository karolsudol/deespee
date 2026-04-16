mod handlers;
mod state;

use crate::handlers::{
    campaigns::handle_campaigns, events::handle_pubsub_push, segments::handle_segments,
};
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize application state
    let state = Arc::new(AppState::new());

    // Setup routes
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/segments", post(handle_segments))
        .route("/campaigns", get(handle_campaigns))
        .route("/pubsub/push", post(handle_pubsub_push))
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8002));
    println!("🚀 DMP Service (Rust + Modularized) listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
