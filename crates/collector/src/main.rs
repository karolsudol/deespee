mod handlers;
mod state;
mod verification;

use crate::handlers::events::{
    handle_click, handle_conversion, handle_impression, handle_viewability,
};
use crate::state::AppState;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize application state
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/i", get(handle_impression)) // Impression
        .route("/c", get(handle_click)) // Click
        .route("/v", get(handle_viewability)) // Viewability
        .route("/conv", get(handle_conversion)) // Conversion
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8003));
    println!(
        "🚀 Measurement Collector (Modular + Bot Detection) listening on {}",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
