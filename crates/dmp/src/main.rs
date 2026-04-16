type AxResponse = axum::response::Response;
use axum::{
    body::Bytes,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use deespee_proto::deespee;
use prost::Message;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

// Shared state for User Frequency and Campaign Budgets
struct AppState {
    frequency: Mutex<HashMap<String, u32>>,
    campaigns: Mutex<Vec<deespee::Campaign>>,
    campaign_states: Mutex<HashMap<String, f32>>, // campaign_id -> spent_today
}

async fn handle_segments(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    body: Bytes,
) -> AxResponse {
    let req = match deespee::UserSegmentRequest::decode(body) {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid Protobuf").into_response(),
    };

    println!("🧠 DMP Segment Lookup for User: {}", req.user_id);

    let freq_map = state.frequency.lock().unwrap();
    let count = freq_map.get(&req.user_id).cloned().unwrap_or(0);

    let mut segments = if req.user_id.contains("1") || req.user_id.contains("3") {
        vec!["high-value-shopper".to_string()]
    } else {
        vec!["generic-audience".to_string()]
    };

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

async fn handle_campaigns(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> AxResponse {
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

async fn handle_pubsub_push(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    body: Bytes,
) -> AxResponse {
    let event = match deespee::EventNotification::decode(body) {
        Ok(e) => e,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    if event.r#type == deespee::event_notification::EventType::Win as i32 {
        // Increment user frequency
        {
            let mut freq_map = state.frequency.lock().unwrap();
            let counter = freq_map.entry(event.user_id.clone()).or_insert(0);
            *counter += 1;
            println!(
                "📈 Incrementing frequency for user {}: new count = {}",
                event.user_id, counter
            );
        }

        // Increment campaign spend
        if !event.campaign_id.is_empty() {
            let mut states_map = state.campaign_states.lock().unwrap();
            let spend = states_map.entry(event.campaign_id.clone()).or_insert(0.0);
            *spend += event.cost;
            println!(
                "💰 Campaign {} spend updated: ${:.2}",
                event.campaign_id, spend
            );
        }
    }

    StatusCode::OK.into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize with some dummy campaigns
    let campaigns = vec![
        deespee::Campaign {
            id: "camp-123".to_string(),
            name: "High Value NYC Shoppers (CPM)".to_string(),
            total_budget: 10000.0,
            daily_budget: 100.0,
            targeted_segments: vec!["high-value-shopper".to_string()],
            targeted_cities: vec!["New York".to_string()],
            targeted_categories: vec![],
            bid_type: deespee::BidType::Cpm as i32,
            target_value: 5.0, // $5.00 CPM
        },
        deespee::Campaign {
            id: "camp-456".to_string(),
            name: "Tech Enthusiasts (eCPC)".to_string(),
            total_budget: 5000.0,
            daily_budget: 50.0,
            targeted_segments: vec!["generic-audience".to_string()],
            targeted_cities: vec![],
            targeted_categories: vec!["IAB19".to_string()], // Tech
            bid_type: deespee::BidType::Ecpc as i32,
            target_value: 2.5, // $2.50 target cost-per-click
        },
    ];

    let state = Arc::new(AppState {
        frequency: Mutex::new(HashMap::new()),
        campaigns: Mutex::new(campaigns),
        campaign_states: Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/segments", post(handle_segments))
        .route("/campaigns", get(handle_campaigns))
        .route("/pubsub/push", post(handle_pubsub_push))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8002));
    println!(
        "🚀 DMP Service (Rust + Explicit Types) listening on {}",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
