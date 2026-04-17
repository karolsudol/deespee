use crate::reader::LakehouseReader;
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct QueryRequest {
    #[allow(dead_code)]
    pub sql: String,
}

pub async fn handle_query(
    State(state): State<Arc<AppState>>,
    Json(_payload): Json<QueryRequest>,
) -> impl IntoResponse {
    println!("🔎 Manual Query: Reading all raw events from Parquet...");
    let reader = LakehouseReader::new(&state.base_path);

    match reader.read_all_events() {
        Ok(results) => (StatusCode::OK, Json(results)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read analytics: {}", e),
        )
            .into_response(),
    }
}
