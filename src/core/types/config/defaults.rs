//! Default value functions for configuration

use super::observability::LogFormat;

pub fn default_true() -> bool {
    true
}

pub fn default_weight() -> f64 {
    1.0
}

pub fn default_timeout_seconds() -> u64 {
    30
}

pub fn default_max_retries() -> u32 {
    3
}

pub fn default_health_check_interval() -> u64 {
    30
}

pub fn default_health_check_timeout() -> u64 {
    5
}

pub fn default_health_threshold() -> u32 {
    2
}

pub fn default_unhealthy_threshold() -> u32 {
    3
}

pub fn default_failure_threshold() -> u32 {
    5
}

pub fn default_recovery_timeout() -> u64 {
    60
}

pub fn default_half_open_requests() -> u32 {
    3
}

pub fn default_session_timeout() -> u64 {
    3600
}

pub fn default_initial_delay_ms() -> u64 {
    100
}

pub fn default_max_delay_ms() -> u64 {
    30000
}

pub fn default_pool_size() -> u32 {
    10
}

pub fn default_jwt_algorithm() -> String {
    "HS256".to_string()
}

pub fn default_jwt_expiration() -> u64 {
    3600
}

pub fn default_api_key_header() -> String {
    "Authorization".to_string()
}

pub fn default_cors_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
    ]
}

pub fn default_cors_headers() -> Vec<String> {
    vec!["Content-Type".to_string(), "Authorization".to_string()]
}

pub fn default_cors_max_age() -> u64 {
    3600
}

pub fn default_metrics_endpoint() -> String {
    "/metrics".to_string()
}

pub fn default_metrics_interval() -> u64 {
    15
}

pub fn default_sampling_rate() -> f64 {
    0.1
}

pub fn default_log_level() -> String {
    "info".to_string()
}

pub fn default_log_format() -> LogFormat {
    LogFormat::Json
}
