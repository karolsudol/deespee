use axum::{body::Bytes, extract::Query, http::header, response::IntoResponse};
use deespee_proto::deespee;
use prost::Message;
use serde::Deserialize;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct WinQuery {
    pub id: String,
    pub user_id: Option<String>,
}

async fn fetch_segments(user_id: &str) -> Vec<String> {
    let host = "localhost:8002";

    let req = deespee::UserSegmentRequest {
        user_id: user_id.to_string(),
    };

    let mut body = Vec::new();
    req.encode(&mut body).unwrap();

    let mut stream = match TcpStream::connect(host) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));

    let http_request = format!(
        "POST /segments HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/x-protobuf\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        host,
        body.len()
    );

    if stream.write_all(http_request.as_bytes()).is_err() || stream.write_all(&body).is_err() {
        return vec![];
    }

    let mut response = Vec::new();
    if stream.read_to_end(&mut response).is_ok() {
        if let Some(pos) = response.windows(4).position(|w| w == b"\r\n\r\n") {
            let body_data = &response[pos + 4..];
            if let Ok(segment_resp) = deespee::UserSegmentResponse::decode(body_data) {
                return segment_resp.segments;
            }
        }
    }

    vec![]
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
    let segments = fetch_segments(user_id).await;

    println!(
        "🎯 Bid Request: ID={}, User={}, Segments={:?}",
        req.id, user_id, segments
    );

    // 3. Decision Logic based on segments
    let mut bid_price = 0.50;
    if segments.contains(&"capped".to_string()) {
        println!("🛑 CAPPED: Not bidding for user {}", user_id);
        bid_price = 0.00;
    } else if segments.contains(&"high-value-shopper".to_string()) {
        bid_price = 2.50;
    }

    let resp = deespee::BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        price: bid_price as f32,
        adid: "creative-123".to_string(),
        crid: "cr-456".to_string(),
        adm: format!("<html><body><h1>Ad for {:?}</h1></body></html>", segments),
        nurl: format!(
            "http://localhost:8001/win?id={}&user_id={}",
            req.id, user_id
        ),
        cat: vec!["IAB1".to_string()],
    };

    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}

pub async fn handle_win(Query(params): Query<WinQuery>) -> impl IntoResponse {
    println!("🏆 Win Notice Received for Bid: {}", params.id);

    // In a real scenario, we'd publish to the REAL Pub/Sub topic.
    // Since we're using the emulator and want to avoid heavy dependencies,
    // we'll simulate the "Pub/Sub" push by calling the DMP directly for this POC,
    // OR we can implement a minimal Pub/Sub REST publisher.

    // Let's use the DMP direct call for the POC feedback loop to keep it simple and fast.
    let user_id = params.user_id.unwrap_or_else(|| "unknown".to_string());

    // Asynchronously update DMP (Fire and Forget)
    tokio::spawn(async move {
        let host = "localhost:8002";
        let event = deespee::EventNotification {
            event_id: uuid::Uuid::new_v4().to_string(),
            r#type: deespee::event_notification::EventType::Win as i32,
            user_id: user_id.clone(),
            bid_id: params.id,
            ad_id: "creative-123".to_string(),
            cost: 1.25, // This would be dynamic in production
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mut body = Vec::new();
        event.encode(&mut body).unwrap();

        // Push to DMP's Pub/Sub handler
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
    });

    "Win Recorded"
}
