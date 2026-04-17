use crate::clients::dmp::DmpClient;
use crate::engine::bidding::BiddingEngine;
use crate::render::render_ad_markup;
use axum::{body::Bytes, extract::Query, http::header, response::IntoResponse};
use deespee_proto::deespee;
use prost::Message;
use serde::Deserialize;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, Deserialize)]
pub struct WinQuery {
    pub id: String,
    pub user_id: Option<String>,
    pub campaign_id: Option<String>,
    pub price: Option<f32>,
}

pub async fn handle_bid(body: Bytes) -> impl IntoResponse {
    let req = match deespee::BidRequest::decode(body) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to decode BidRequest: {}", e);
            return (axum::http::StatusCode::BAD_REQUEST, "Invalid Protobuf").into_response();
        }
    };

    let user_id = req
        .user
        .as_ref()
        .map(|u| u.id.as_str())
        .unwrap_or("unknown");

    // Clients
    let dmp = DmpClient::new("localhost:8002");

    // Fetch data from DMP (Parallel-ish)
    let segments = dmp.fetch_segments(user_id).await;
    let (campaigns, campaign_states) = dmp.fetch_campaigns().await;

    if segments.contains(&"capped".to_string()) {
        println!("🛑 CAPPED: Not bidding for user {}", user_id);
        return (axum::http::StatusCode::NO_CONTENT, "Capped").into_response();
    }

    // Matching Engine
    let bid_result = BiddingEngine::match_campaigns(&req, campaigns, &campaign_states, &segments);

    let result = match bid_result {
        Some(r) => r,
        None => {
            println!("🤷 No matching campaigns for request {}", req.id);
            return (axum::http::StatusCode::NO_CONTENT, "No Match").into_response();
        }
    };

    println!(
        "✅ BIDDING: Campaign={} Price=${:.2}",
        result.campaign.id, result.price
    );

    let tracking_params = format!(
        "bid_id={}&user_id={}&campaign_id={}",
        req.id, user_id, result.campaign.id
    );

    let adm = render_ad_markup(&result.campaign.name, user_id, &tracking_params);

    let resp = deespee::BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        price: result.price,
        adid: format!("ad-{}", result.campaign.id),
        crid: "cr-456".to_string(),
        adm,
        nurl: format!(
            "http://localhost:8001/win?id={}&user_id={}&campaign_id={}&price={}",
            req.id, user_id, result.campaign.id, result.price
        ),
        cat: vec!["IAB1".to_string()],
    };

    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}

pub async fn handle_win(Query(params): Query<WinQuery>) -> impl IntoResponse {
    println!(
        "🏆 Win Notice Received for Bid: {} (Campaign: {:?}, Price: {:?})",
        params.id, params.campaign_id, params.price
    );
    let user_id = params.user_id.unwrap_or_else(|| "unknown".to_string());
    let campaign_id = params.campaign_id.unwrap_or_default();
    let price = params.price.unwrap_or(0.0);

    tokio::spawn(async move {
        let host = "localhost:8002";
        let event = deespee::EventNotification {
            event_id: uuid::Uuid::new_v4().to_string(),
            r#type: deespee::event_notification::EventType::Win as i32,
            user_id: user_id.clone(),
            bid_id: params.id,
            ad_id: "creative-123".to_string(),
            cost: price, // Use the actual bid price
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            campaign_id,
        };

        let mut body = Vec::new();
        event.encode(&mut body).unwrap();

        if let Ok(mut stream) = TcpStream::connect(host) {
            let http_request = format!(
                "POST /pubsub/push HTTP/1.1\r\n\
                 Host: {}\r\n\
                 Content-Type: application/x-protobuf\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\r\n",
                host,
                body.len()
            );
            let _ = stream.write_all(http_request.as_bytes());
            let _ = stream.write_all(&body);
        }

        // Notify Collector for Reconciliation (Discrepancy Engine)
        let collector_host = "localhost:8003";
        if let Ok(mut stream) = TcpStream::connect(collector_host) {
            let http_request = format!(
                "POST /win HTTP/1.1\r\n\
                 Host: {}\r\n\
                 Content-Type: application/x-protobuf\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\r\n",
                collector_host,
                body.len()
            );
            let _ = stream.write_all(http_request.as_bytes());
            let _ = stream.write_all(&body);
        }

        // Notify Analytics (Lakehouse Ingestion)
        let analytics_host = "localhost:8004";
        if let Ok(mut stream) = TcpStream::connect(analytics_host) {
            let http_request = format!(
                "POST /events HTTP/1.1\r\n\
                 Host: {}\r\n\
                 Content-Type: application/x-protobuf\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\r\n",
                analytics_host,
                body.len()
            );
            let _ = stream.write_all(http_request.as_bytes());
            let _ = stream.write_all(&body);
        }
    });

    "Win Recorded"
}
