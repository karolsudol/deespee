use crate::state::AppState;
use axum::{http::header, response::IntoResponse};
use deespee_proto::deespee;
use prost::Message;
use std::sync::Arc;

pub async fn handle_campaigns(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    let campaigns = state.campaigns.lock().unwrap().clone();
    let states_map = state.campaign_states.lock().unwrap();

    let states = states_map
        .iter()
        .map(|(id, spent)| deespee::CampaignState {
            campaign_id: id.clone(),
            spent_today: *spent,
        })
        .collect();

    let resp = deespee::CampaignListResponse { campaigns, states };

    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}
