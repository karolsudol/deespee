use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct QueryRequest {
    pub sql: String,
}

pub async fn handle_query(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    println!("🔎 Executing SQL: {}", payload.sql);

    match state.ctx.sql(&payload.sql).await {
        Ok(df) => match df.collect().await {
            Ok(batches) => {
                let mut results = Vec::new();
                for batch in batches {
                    let mut buf = Vec::new();
                    let mut writer = arrow_json::ArrayWriter::new(&mut buf);
                    writer.write(&batch).unwrap();
                    writer.finish().unwrap();

                    if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&buf) {
                        if let Some(arr) = json_val.as_array() {
                            results.extend(arr.clone());
                        }
                    }
                }
                (StatusCode::OK, Json(results)).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Execution Error: {}", e),
            )
                .into_response(),
        },
        Err(e) => (StatusCode::BAD_REQUEST, format!("SQL Error: {}", e)).into_response(),
    }
}
