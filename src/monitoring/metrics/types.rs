//! Types for metrics storage

use std::collections::HashMap;
use std::collections::VecDeque;

/// Consolidated metrics storage - single lock for all metrics
#[derive(Debug, Default)]
pub(super) struct MetricsStorage {
    pub(super) request: RequestMetricsStorage,
    pub(super) provider: ProviderMetricsStorage,
    pub(super) system: SystemMetricsStorage,
    pub(super) error: ErrorMetricsStorage,
    pub(super) performance: PerformanceMetricsStorage,
}

/// Storage for request metrics
#[derive(Debug, Default)]
pub(super) struct RequestMetricsStorage {
    pub(super) total_requests: u64,
    pub(super) response_times: VecDeque<f64>,
    pub(super) status_codes: HashMap<u16, u64>,
    pub(super) endpoints: HashMap<String, u64>,
    pub(super) last_minute_requests: VecDeque<std::time::Instant>,
}

/// Storage for provider metrics
#[derive(Debug, Default)]
pub(super) struct ProviderMetricsStorage {
    pub(super) total_requests: u64,
    pub(super) provider_requests: HashMap<String, u64>,
    pub(super) provider_response_times: HashMap<String, VecDeque<f64>>,
    pub(super) provider_errors: HashMap<String, u64>,
    pub(super) token_usage: HashMap<String, u64>,
    pub(super) costs: HashMap<String, f64>,
}

/// Storage for system metrics
#[derive(Debug, Default)]
pub(super) struct SystemMetricsStorage {
    pub(super) cpu_samples: VecDeque<f64>,
    pub(super) memory_samples: VecDeque<u64>,
    pub(super) disk_samples: VecDeque<u64>,
    pub(super) network_in_samples: VecDeque<u64>,
    pub(super) network_out_samples: VecDeque<u64>,
    pub(super) connection_samples: VecDeque<u32>,
}

/// Storage for error metrics
#[derive(Debug, Default)]
pub(super) struct ErrorMetricsStorage {
    pub(super) total_errors: u64,
    pub(super) error_types: HashMap<String, u64>,
    pub(super) error_endpoints: HashMap<String, u64>,
    pub(super) critical_errors: u64,
    pub(super) warnings: u64,
    pub(super) last_minute_errors: VecDeque<std::time::Instant>,
}

/// Storage for performance metrics
#[derive(Debug, Default)]
pub(super) struct PerformanceMetricsStorage {
    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,
    pub(super) db_query_times: VecDeque<f64>,
    pub(super) queue_depths: VecDeque<u32>,
    pub(super) throughput_samples: VecDeque<f64>,
}
