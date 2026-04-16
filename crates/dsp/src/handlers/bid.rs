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
    pub campaign_id: Option<String>,
    pub price: Option<f32>,
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

async fn fetch_campaigns() -> (
    Vec<deespee::Campaign>,
    std::collections::HashMap<String, f32>,
) {
    let host = "localhost:8002";
    let mut stream = match TcpStream::connect(host) {
        Ok(s) => s,
        Err(_) => return (vec![], std::collections::HashMap::new()),
    };

    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));

    let http_request = format!(
        "GET /campaigns HTTP/1.1\r\n\
         Host: {}\r\n\
         Connection: close\r\n\r\n",
        host
    );

    if stream.write_all(http_request.as_bytes()).is_err() {
        return (vec![], std::collections::HashMap::new());
    }

    let mut response = Vec::new();
    if stream.read_to_end(&mut response).is_ok() {
        if let Some(pos) = response.windows(4).position(|w| w == b"\r\n\r\n") {
            let body_data = &response[pos + 4..];
            if let Ok(campaign_resp) = deespee::CampaignListResponse::decode(body_data) {
                let mut states = std::collections::HashMap::new();
                for state in campaign_resp.states {
                    states.insert(state.campaign_id, state.spent_today);
                }
                return (campaign_resp.campaigns, states);
            }
        }
    }
    (vec![], std::collections::HashMap::new())
}

fn get_pacing_multiplier(spent_today: f32, daily_budget: f32) -> f32 {
    // Simple even pacing: target spend is proportional to time of day
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let seconds_since_midnight = now.as_secs() % 86400;
    let target_spend_ratio = seconds_since_midnight as f32 / 86400.0;
    let current_spend_ratio = spent_today / daily_budget;

    if current_spend_ratio > target_spend_ratio + 0.1 {
        // Over-spending: reduce bid price
        0.5
    } else if current_spend_ratio < target_spend_ratio - 0.1 {
        // Under-spending: increase bid price (aggressive)
        1.2
    } else {
        1.0
    }
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

    // Fetch data from DMP (Parallel-ish)
    let segments = fetch_segments(user_id).await;
    let (campaigns, campaign_states) = fetch_campaigns().await;

    let city = req
        .device
        .as_ref()
        .and_then(|d| d.geo.as_ref())
        .map(|g| g.city.as_str())
        .unwrap_or("unknown");

    let categories = req.site.as_ref().map(|s| s.cat.clone()).unwrap_or_default();

    println!(
        "🎯 Bid Request: ID={}, User={}, City={}, Categories={:?}, Segments={:?}",
        req.id, user_id, city, categories, segments
    );

    if segments.contains(&"capped".to_string()) {
        println!("🛑 CAPPED: Not bidding for user {}", user_id);
        return (axum::http::StatusCode::NO_CONTENT, "Capped").into_response();
    }

    // Phase 2: Budget Pacing & Campaign Matching
    let mut selected_campaign = None;
    let mut best_bid_price = 0.0;

    for camp in campaigns {
        // 1. Targeting Check
        let segment_match = camp.targeted_segments.is_empty()
            || camp.targeted_segments.iter().any(|s| segments.contains(s));
        let city_match =
            camp.targeted_cities.is_empty() || camp.targeted_cities.contains(&city.to_string());
        let category_match = camp.targeted_categories.is_empty()
            || camp
                .targeted_categories
                .iter()
                .any(|c| categories.contains(c));

        if segment_match && city_match && category_match {
            // 2. Budget Check
            let spent_today = campaign_states.get(&camp.id).cloned().unwrap_or(0.0);
            if spent_today >= camp.daily_budget {
                println!(
                    "💸 Budget Exhausted for campaign {}: ${}",
                    camp.id, spent_today
                );
                continue;
            }

            // 3. Pacing & Bidding Model
            let mut base_bid = 0.0;

            if camp.bid_type == deespee::BidType::Cpm as i32 {
                base_bid = camp.target_value;
                // Apply Targeting Premiums for CPM
                if city == "New York" {
                    base_bid += 2.0;
                }
                if segments.contains(&"high-value-shopper".to_string()) {
                    base_bid += 1.5;
                }
            } else if camp.bid_type == deespee::BidType::Ecpc as i32 {
                // eCPC Model: bid = target_ecpc * predicted_ctr
                // For this MVP, we use a fixed CTR (e.g., 0.1% or 0.001)
                let predicted_ctr = 0.001;
                base_bid = camp.target_value * predicted_ctr * 1000.0; // Multiply by 1000 for CPM basis
                println!(
                    "📈 eCPC Bidding: Target=${:.2}, Predicted CTR={:.3}, Base Bid=${:.2} CPM",
                    camp.target_value, predicted_ctr, base_bid
                );
            }

            let pacing_mult = get_pacing_multiplier(spent_today, camp.daily_budget);
            let final_bid = base_bid * pacing_mult;

            if final_bid > best_bid_price {
                best_bid_price = final_bid;
                selected_campaign = Some(camp);
            }
        }
    }

    let camp = match selected_campaign {
        Some(c) => c,
        None => {
            println!("🤷 No matching campaigns for request {}", req.id);
            return (axum::http::StatusCode::NO_CONTENT, "No Match").into_response();
        }
    };

    println!(
        "✅ BIDDING: Campaign={} Price=${:.2}",
        camp.id, best_bid_price
    );

    let tracking_params = format!(
        "bid_id={}&user_id={}&campaign_id={}",
        req.id, user_id, camp.id
    );

    let adm = format!(
        "<html>\
           <body style='margin:0;padding:0;'>\
             <div style='width:300px;height:250px;background:#f0f0f0;display:flex;align-items:center;justify-content:center;flex-direction:column;border:1px solid #ccc;'>\
               <h2 style='margin:0;'>{}</h2>\
               <p style='font-size:12px;'>Targeted for: {}</p>\
               <a href='http://localhost:8003/c?{}' target='_blank' style='display:inline-block;margin-top:10px;padding:10px 20px;background:#007bff;color:white;text-decoration:none;border-radius:5px;'>\
                 Click Here\
               </a>\
               <img src='http://localhost:8003/i?{}' width='1' height='1' style='display:none;' />\
               <script>\
                 (function() {{\
                   var tracked = false;\
                   var observer = new IntersectionObserver(function(entries) {{\
                     entries.forEach(function(entry) {{\
                       if (entry.isIntersecting && entry.intersectionRatio >= 0.5 && !tracked) {{\
                         setTimeout(function() {{\
                           /* Check again after 1s for IAB standard */\
                           if (!tracked) {{\
                             fetch('http://localhost:8003/v?{}');\
                             tracked = true;\
                           }}\
                         }}, 1000);\
                       }}\
                     }});\
                   }}, {{ threshold: [0.5] }});\
                   observer.observe(document.body);\
                 }})();\
               </script>\
             </div>\
           </body>\
         </html>",
        camp.name, user_id, tracking_params, tracking_params, tracking_params
    );

    let resp = deespee::BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        price: best_bid_price,
        adid: format!("ad-{}", camp.id),
        crid: "cr-456".to_string(),
        adm,
        nurl: format!(
            "http://localhost:8001/win?id={}&user_id={}&campaign_id={}&price={}",
            req.id, user_id, camp.id, best_bid_price
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
    });

    "Win Recorded"
}
