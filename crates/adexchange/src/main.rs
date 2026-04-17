use deespee_proto::deespee;
use prost::Message;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let dsp_host = "localhost:8001";

    println!("🚀 Ad Exchange Simulator (Production Traffic) started...");
    println!("🎯 Targeting DSP at {}", dsp_host);

    loop {
        if let Err(e) = send_bid_request(dsp_host) {
            eprintln!("❌ Error sending bid request: {}", e);
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn send_bid_request(host: &str) -> anyhow::Result<()> {
    let ts = os_timestamp();

    let user_id = format!("user-{}", ts % 10);
    let (city, lat, lon) = if ts % 2 == 0 {
        ("San Francisco", 37.77, -122.41)
    } else {
        ("New York", 40.71, -74.00)
    };

    let req = deespee::BidRequest {
        id: format!("req-{}", ts),
        user: Some(deespee::User {
            id: user_id.clone(),
            segments: vec!["auto-shopper".to_string()],
        }),
        device: Some(deespee::Device {
            ua: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)".to_string(),
            ip: "1.1.1.1".to_string(),
            devicetype: "phone".to_string(),
            make: "Apple".to_string(),
            model: "iPhone 15".to_string(),
            os: "iOS".to_string(),
            geo: Some(deespee::Geo {
                country: "USA".to_string(),
                city: city.to_string(),
                lat,
                lon,
            }),
        }),
        site: Some(deespee::Site {
            id: "site-456".to_string(),
            domain: "news-portal.com".to_string(),
            page: "https://news-portal.com/tech/latest-ai-news".to_string(),
            cat: vec!["IAB12".to_string()],
        }),
        imp: Some(deespee::Impression {
            id: "imp-1".to_string(),
            w: 320,
            h: 50,
            pos: "1".to_string(),
        }),
        test: "0".to_string(),
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
                "✅ Bid Received: Price={:.2}, AdID={}, Creative={}",
                bid_resp.price, bid_resp.adid, bid_resp.crid
            );

            // SIMULATION: 50% Win Rate, 10% Click Rate
            let win = ts % 2 == 0;
            if win {
                println!("🏆 Simulating WIN notice for {}...", bid_resp.id);
                if let Err(e) = trigger_win_notice(&bid_resp.nurl) {
                    eprintln!("❌ Error triggering win notice: {}", e);
                }

                // Simulate an impression ping to Collector
                if let Err(e) = trigger_pixel_ping("i", &req.id, &user_id) {
                    eprintln!("❌ Error triggering impression ping: {}", e);
                }

                // 10% chance of a click
                if ts % 10 == 0 {
                    println!("🖱️ Simulating CLICK for {}...", req.id);
                    if let Err(e) = trigger_pixel_ping("c", &req.id, &user_id) {
                        eprintln!("❌ Error triggering click ping: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

fn trigger_win_notice(nurl: &str) -> anyhow::Result<()> {
    // Win notices are usually simple GET requests from the Exchange
    reqwest::blocking::get(nurl)?;
    Ok(())
}

fn trigger_pixel_ping(pixel_type: &str, bid_id: &str, user_id: &str) -> anyhow::Result<()> {
    let url = format!(
        "http://localhost:8003/{}?bid_id={}&user_id={}&campaign_id=camp-auto",
        pixel_type, bid_id, user_id
    );
    reqwest::blocking::get(url)?;
    Ok(())
}

fn os_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
