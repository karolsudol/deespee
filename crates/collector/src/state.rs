use crate::verification::VerificationEngine;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct AppState {
    pub verification: VerificationEngine,
    // campaign_id -> { "wins": X, "impressions": Y, ... }
    pub stats: Mutex<HashMap<String, HashMap<String, u64>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            verification: VerificationEngine::new(),
            stats: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_metric(&self, campaign_id: &str, metric: &str) {
        let mut stats = self.stats.lock().unwrap();
        let campaign_stats = stats.entry(campaign_id.to_string()).or_default();
        *campaign_stats.entry(metric.to_string()).or_insert(0) += 1;
    }

    pub fn get_discrepancy_report(&self) -> String {
        let stats = self.stats.lock().unwrap();
        let mut report = String::from("📊 DISCREPANCY REPORT\n");
        report.push_str("-------------------------------------------\n");

        for (camp_id, metrics) in stats.iter() {
            let wins = metrics.get("win").cloned().unwrap_or(0);
            let imps = metrics.get("impression").cloned().unwrap_or(0);
            let clicks = metrics.get("click").cloned().unwrap_or(0);

            let discrepancy = if wins > 0 {
                let diff = (wins as i64 - imps as i64).abs();
                (diff as f64 / wins as f64) * 100.0
            } else {
                0.0
            };

            report.push_str(&format!(
                "Campaign: {}\n  Wins: {}\n  Imps: {}\n  Clicks: {}\n  Discrepancy: {:.2}%\n",
                camp_id, wins, imps, clicks, discrepancy
            ));
        }
        report
    }
}
