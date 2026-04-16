use axum::{body::Bytes, http::header, response::IntoResponse};
use deespee_proto::deespee;
use prost::Message;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

async fn fetch_segments(user_id: &str) -> Vec<String> {
    let host = "localhost:8002";

    let req = deespee::UserSegmentRequest {
        user_id: user_id.to_string(),
    };

    let mut body = Vec::new();
    req.encode(&mut body).unwrap();

    // Minimal HTTP POST over TCP to DMP
    let mut stream = match TcpStream::connect(host) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ DMP Connection Error: {}", e);
            return vec![];
        }
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
    let _ = stream.flush();

    let mut response = Vec::new();
    if stream.read_to_end(&mut response).is_ok() {
        if let Some(pos) = response.windows(4).position(|w| w == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&response[..pos]);
            if headers.contains("200 OK") {
                let body_data = &response[pos + 4..];
                if let Ok(segment_resp) = deespee::UserSegmentResponse::decode(body_data) {
                    return segment_resp.segments;
                }
            }
        }
    }

    vec![]
}

pub async fn handle_bid(body: Bytes) -> impl IntoResponse {
    // 1. Decode Protobuf BidRequest from Exchange
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

    // 2. Query DMP for User Segments (The Hot Path)
    let segments = fetch_segments(user_id).await;

    println!(
        "🎯 Bid Request: ID={}, User={}, Segments={:?}",
        req.id, user_id, segments
    );

    // 3. Decision Logic based on segments
    let mut bid_price = 0.50; // Base price
    if segments.contains(&"high-value-shopper".to_string()) {
        bid_price = 2.50; // Bid higher for valuable users
    }

    let resp = deespee::BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        price: bid_price as f32,
        adid: "creative-123".to_string(),
        crid: "cr-456".to_string(),
        adm: format!("<html><body><h1>Ad for {:?}</h1></body></html>", segments),
        nurl: format!("http://localhost:8001/win?id={}", req.id),
        cat: vec!["IAB1".to_string()],
    };

    // 4. Encode Protobuf BidResponse
    let mut buf = Vec::new();
    resp.encode(&mut buf).unwrap();

    ([(header::CONTENT_TYPE, "application/x-protobuf")], buf).into_response()
}
