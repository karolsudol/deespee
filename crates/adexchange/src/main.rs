use deespee_proto::deespee;
use prost::Message;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let dsp_host = "localhost:8001";

    println!("🚀 Ad Exchange Simulator (Rust + Std Net) started...");
    println!("🎯 Targeting DSP at {}", dsp_host);

    loop {
        if let Err(e) = send_bid_request(dsp_host) {
            eprintln!("❌ Error sending bid request: {}", e);
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}

fn send_bid_request(host: &str) -> anyhow::Result<()> {
    let req = deespee::BidRequest {
        id: format!("req-{}", os_timestamp()),
        user: Some(deespee::User {
            id: "user-123".to_string(),
        }),
        device: Some(deespee::Device {
            ua: "Mozilla/5.0".to_string(),
            ip: "1.1.1.1".to_string(),
        }),
    };

    let mut body = Vec::new();
    req.encode(&mut body)?;

    // Minimal HTTP POST over TCP
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

    // Basic HTTP response parsing (looking for \r\n\r\n)
    if let Some(pos) = response.windows(4).position(|w| w == b"\r\n\r\n") {
        let headers = String::from_utf8_lossy(&response[..pos]);
        if headers.contains("200 OK") {
            let body_data = &response[pos + 4..];
            let bid_resp = deespee::BidResponse::decode(body_data)?;
            println!(
                "✅ Received Bid: ID={}, Price={:.2}",
                bid_resp.id, bid_resp.price
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
