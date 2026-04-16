use deespee_proto::deespee;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct AppState {
    pub frequency: Mutex<HashMap<String, u32>>,
    pub campaigns: Mutex<Vec<deespee::Campaign>>,
    pub campaign_states: Mutex<HashMap<String, f32>>, // campaign_id -> spent_today
}

impl AppState {
    pub fn new() -> Self {
        // Initialize with some dummy campaigns
        let campaigns = vec![
            deespee::Campaign {
                id: "camp-123".to_string(),
                name: "High Value NYC Shoppers (CPM)".to_string(),
                total_budget: 10000.0,
                daily_budget: 100.0,
                targeted_segments: vec!["high-value-shopper".to_string()],
                targeted_cities: vec!["New York".to_string()],
                targeted_categories: vec![],
                bid_type: deespee::BidType::Cpm as i32,
                target_value: 5.0, // $5.00 CPM
            },
            deespee::Campaign {
                id: "camp-456".to_string(),
                name: "Tech Enthusiasts (eCPC)".to_string(),
                total_budget: 5000.0,
                daily_budget: 50.0,
                targeted_segments: vec!["generic-audience".to_string()],
                targeted_cities: vec![],
                targeted_categories: vec!["IAB19".to_string()], // Tech
                bid_type: deespee::BidType::Ecpc as i32,
                target_value: 2.5, // $2.50 target cost-per-click
            },
        ];

        Self {
            frequency: Mutex::new(HashMap::new()),
            campaigns: Mutex::new(campaigns),
            campaign_states: Mutex::new(HashMap::new()),
        }
    }
}
