use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

pub struct RateLimiter {
    state: Mutex<RateLimiterState>,
}

struct RateLimiterState {
    window_counts: HashMap<String, Vec<Instant>>,
    last_stop: Option<Instant>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(RateLimiterState {
                window_counts: HashMap::new(),
                last_stop: None,
            }),
        }
    }

    /// Check if a notification should be rate limited.
    /// Returns true if the notification should be sent (not limited).
    pub fn check(&self, event_type: &str, max_per_minute: u32, cooldown_seconds: u64) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        // Stop cooldown check
        if event_type == "stop" {
            if let Some(last) = state.last_stop {
                if now.duration_since(last).as_secs() < cooldown_seconds {
                    return false;
                }
            }
            state.last_stop = Some(now);
        }

        // Per-minute rate limit
        let counts = state
            .window_counts
            .entry(event_type.to_string())
            .or_default();
        counts.retain(|t| now.duration_since(*t).as_secs() < 60);
        if counts.len() >= max_per_minute as usize {
            return false;
        }
        counts.push(now);
        true
    }
}
