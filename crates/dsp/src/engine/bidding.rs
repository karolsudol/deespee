use crate::engine::pacing::get_pacing_multiplier;
use deespee_proto::deespee;
use std::collections::HashMap;

pub struct BiddingEngine;

pub struct BidResult {
    pub campaign: deespee::Campaign,
    pub price: f32,
}

impl BiddingEngine {
    pub fn match_campaigns(
        req: &deespee::BidRequest,
        campaigns: Vec<deespee::Campaign>,
        campaign_states: &HashMap<String, f32>,
        segments: &[String],
    ) -> Option<BidResult> {
        let city = req
            .device
            .as_ref()
            .and_then(|d| d.geo.as_ref())
            .map(|g| g.city.as_str())
            .unwrap_or("unknown");

        let categories = req.site.as_ref().map(|s| s.cat.clone()).unwrap_or_default();

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
                    let predicted_ctr = 0.001;
                    base_bid = camp.target_value * predicted_ctr * 1000.0;
                }

                let pacing_mult = get_pacing_multiplier(spent_today, camp.daily_budget);
                let final_bid = base_bid * pacing_mult;

                if final_bid > best_bid_price {
                    best_bid_price = final_bid;
                    selected_campaign = Some(camp);
                }
            }
        }

        selected_campaign.map(|campaign| BidResult {
            campaign,
            price: best_bid_price,
        })
    }
}
