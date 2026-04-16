use crate::verification::VerificationEngine;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct AppState {
    pub verification: VerificationEngine,
    // Simple in-memory storage for local testing
    pub event_counters: Mutex<HashMap<String, u64>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            verification: VerificationEngine::new(),
            event_counters: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_event(&self, event_type: &str) {
        let mut counters = self.event_counters.lock().unwrap();
        *counters.entry(event_type.to_string()).or_insert(0) += 1;
    }
}
