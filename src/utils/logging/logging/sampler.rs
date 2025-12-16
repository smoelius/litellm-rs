//! Log sampling manager for high-frequency events

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Log sampling manager for high-frequency events
#[allow(dead_code)]
pub struct LogSampler {
    sample_rates: HashMap<String, f64>,
    counters: HashMap<String, AtomicU64>,
}

#[allow(dead_code)]
impl Default for LogSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSampler {
    /// Create a new log sampler
    pub fn new() -> Self {
        Self {
            sample_rates: HashMap::new(),
            counters: HashMap::new(),
        }
    }

    /// Configure sampling rate for a log category
    #[allow(dead_code)]
    pub fn set_sample_rate(&mut self, category: &str, rate: f64) {
        self.sample_rates
            .insert(category.to_string(), rate.clamp(0.0, 1.0));
        self.counters
            .insert(category.to_string(), AtomicU64::new(0));
    }

    /// Check if a log should be sampled
    #[allow(dead_code)]
    pub fn should_log(&self, category: &str) -> bool {
        if let Some(&rate) = self.sample_rates.get(category) {
            if rate >= 1.0 {
                return true;
            }
            if rate <= 0.0 {
                return false;
            }

            if let Some(counter) = self.counters.get(category) {
                let count = counter.fetch_add(1, Ordering::Relaxed);
                let sample_threshold = (1.0 / rate) as u64;
                count % sample_threshold == 0
            } else {
                true
            }
        } else {
            true
        }
    }
}
