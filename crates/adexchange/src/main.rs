use deespee_proto::deespee;
use prost::Message;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let dsp_host = "localhost:8001";

    println!("🚀 Ad Exchange Simulator (Rich Traffic) started...");
    println!("🎯 Targeting DSP at {}", dsp_host);

    loop {
        if let Err(e) = send_bid_request(dsp_host) {
            eprintln!("❌ Error sending bid request: {}", e);
        }
        std::thread::sleep(Duration::from_secs(2)); // Faster requests for analytics
    }
}

fn send_bid_request(host: &str) -> anyhow::Result<()> {
    let ts = os_timestamp();

    // Simulate some randomness
    let user_id = format!("user-{}", ts % 10); // 10 distinct users
    let (city, lat, lon) = if ts % 2 == 0 {
        ("San Francisco", 37.77, -122.41)
    } else {
        ("New York", 40.71, -74.00)
    };

    let req = deespee::BidRequest {
        id: format!("req-{}", ts),
        user: Some(deespee::User {
            id: user_id,
            segments: vec!["auto-shopper".to_string(), "tech-enthusiast".to_string()],
        }),
        device: Some(deespee::Device {
            ua: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string(),
            ip: "1.1.1.1".to_string(),
            devicetype: "desktop".to_string(),
            geo: Some(deespee::Geo {
                country: "USA".to_string(),
                city: city.to_string(),
                lat,
                lon,
            }),
        }),
        site: Some(deespee::Site {
            id: "site-456".to_string(),
            domain: "example-news.com".to_string(),
            cat: vec!["IAB12".to_string()], // News
        }),
        imp: Some(deespee::Impression {
            id: "imp-1".to_string(),
            w: 300,
            h: 250,
            pos: "above-the-fold".to_string(),
        }),
    };

    let mut body = Vec::new();
    req.encode(&mut body)?;

    let mut stream = TcpStream::connect(host)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;

    let http_request = format!(
        "POST /bid HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/x-protobuf\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        host,
        body.len()
    );

    stream.write_all(http_request.as_bytes())?;
    stream.write_all(&body)?;
    stream.flush()?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;

    if let Some(pos) = response.windows(4).position(|w| w == b"\r\n\r\n") {
        let headers = String::from_utf8_lossy(&response[..pos]);
        if headers.contains("200 OK") {
            let body_data = &response[pos + 4..];
            let bid_resp = deespee::BidResponse::decode(body_data)?;
            println!(
                "✅ Received Bid: ID={}, Price={:.2}, AdID={}",
                bid_resp.id, bid_resp.price, bid_resp.adid
            );
        } else {
            println!("ℹ️  No bid received (Non-200 response)");
        }
    }

    Ok(())
}

fn os_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
