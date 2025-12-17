use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for HTTP client behavior
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub proxy: Option<String>,
    pub user_agent: String,
    pub default_headers: HashMap<String, String>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            proxy: None,
            user_agent: "litellm-rust/1.0".to_string(),
            default_headers: HashMap::new(),
        }
    }
}

/// Configuration for retry behavior with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Metrics for tracking request performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub start_time: std::time::SystemTime,
    pub end_time: Option<std::time::SystemTime>,
    pub duration: Option<Duration>,
    pub retry_count: u32,
    pub provider: String,
    pub model: String,
    pub status_code: Option<u16>,
}

impl RequestMetrics {
    pub fn new(provider: String, model: String) -> Self {
        Self {
            start_time: std::time::SystemTime::now(),
            end_time: None,
            duration: None,
            retry_count: 0,
            provider,
            model,
            status_code: None,
        }
    }

    pub fn finish(&mut self, status_code: Option<u16>) {
        let now = std::time::SystemTime::now();
        self.end_time = Some(now);
        self.duration = now.duration_since(self.start_time).ok();
        self.status_code = status_code;
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}
