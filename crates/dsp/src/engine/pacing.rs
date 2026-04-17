use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_pacing_multiplier(spent_today: f32, daily_budget: f32) -> f32 {
    // Simple even pacing: target spend is proportional to time of day
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
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
