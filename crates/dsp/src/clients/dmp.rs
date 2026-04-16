use deespee_proto::deespee;
use prost::Message;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct DmpClient {
    host: String,
}

impl DmpClient {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
        }
    }

    pub async fn fetch_segments(&self, user_id: &str) -> Vec<String> {
        let req = deespee::UserSegmentRequest {
            user_id: user_id.to_string(),
        };
        let mut body = Vec::new();
        req.encode(&mut body).unwrap();

        let mut stream = match TcpStream::connect(&self.host) {
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
            self.host,
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

    pub async fn fetch_campaigns(&self) -> (Vec<deespee::Campaign>, HashMap<String, f32>) {
        let mut stream = match TcpStream::connect(&self.host) {
            Ok(s) => s,
            Err(_) => return (vec![], HashMap::new()),
        };

        let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));

        let http_request = format!(
            "GET /campaigns HTTP/1.1\r\n\
             Host: {}\r\n\
             Connection: close\r\n\r\n",
            self.host
        );

        if stream.write_all(http_request.as_bytes()).is_err() {
            return (vec![], HashMap::new());
        }

        let mut response = Vec::new();
        if stream.read_to_end(&mut response).is_ok() {
            if let Some(pos) = response.windows(4).position(|w| w == b"\r\n\r\n") {
                let body_data = &response[pos + 4..];
                if let Ok(campaign_resp) = deespee::CampaignListResponse::decode(body_data) {
                    let mut states = HashMap::new();
                    for state in campaign_resp.states {
                        states.insert(state.campaign_id, state.spent_today);
                    }
                    return (campaign_resp.campaigns, states);
                }
            }
        }
        (vec![], HashMap::new())
    }
}
