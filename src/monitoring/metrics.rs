//! Metrics collection and aggregation
//!
//! This module provides comprehensive metrics collection for monitoring and observability.

#![allow(dead_code)]

use crate::config::MonitoringConfig;
use crate::monitoring::{
    ErrorMetrics, LatencyPercentiles, PerformanceMetrics, ProviderMetrics, RequestMetrics,
    SystemResourceMetrics,
};
use crate::utils::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// Metrics collector for gathering and aggregating system metrics
#[derive(Debug)]
pub struct MetricsCollector {
    /// Configuration
    config: Arc<MonitoringConfig>,
    /// Request metrics storage
    request_metrics: Arc<RwLock<RequestMetricsStorage>>,
    /// Provider metrics storage
    provider_metrics: Arc<RwLock<ProviderMetricsStorage>>,
    /// System metrics storage
    system_metrics: Arc<RwLock<SystemMetricsStorage>>,
    /// Error metrics storage
    error_metrics: Arc<RwLock<ErrorMetricsStorage>>,
    /// Performance metrics storage
    performance_metrics: Arc<RwLock<PerformanceMetricsStorage>>,
    /// Collection start time
    start_time: Instant,
    /// Whether collection is active
    active: Arc<RwLock<bool>>,
}

/// Storage for request metrics
#[derive(Debug, Default)]
struct RequestMetricsStorage {
    total_requests: u64,
    response_times: Vec<f64>,
    status_codes: HashMap<u16, u64>,
    endpoints: HashMap<String, u64>,
    last_minute_requests: Vec<Instant>,
}

/// Storage for provider metrics
#[derive(Debug, Default)]
struct ProviderMetricsStorage {
    total_requests: u64,
    provider_requests: HashMap<String, u64>,
    provider_response_times: HashMap<String, Vec<f64>>,
    provider_errors: HashMap<String, u64>,
    token_usage: HashMap<String, u64>,
    costs: HashMap<String, f64>,
}

/// Storage for system metrics
#[derive(Debug, Default)]
struct SystemMetricsStorage {
    cpu_samples: Vec<f64>,
    memory_samples: Vec<u64>,
    disk_samples: Vec<u64>,
    network_in_samples: Vec<u64>,
    network_out_samples: Vec<u64>,
    connection_samples: Vec<u32>,
}

/// Storage for error metrics
#[derive(Debug, Default)]
struct ErrorMetricsStorage {
    total_errors: u64,
    error_types: HashMap<String, u64>,
    error_endpoints: HashMap<String, u64>,
    critical_errors: u64,
    warnings: u64,
    last_minute_errors: Vec<Instant>,
}

/// Storage for performance metrics
#[derive(Debug, Default)]
struct PerformanceMetricsStorage {
    cache_hits: u64,
    cache_misses: u64,
    db_query_times: Vec<f64>,
    queue_depths: Vec<u32>,
    throughput_samples: Vec<f64>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub async fn new(config: &MonitoringConfig) -> Result<Self> {
        Ok(Self {
            config: Arc::new(config.clone()),
            request_metrics: Arc::new(RwLock::new(RequestMetricsStorage::default())),
            provider_metrics: Arc::new(RwLock::new(ProviderMetricsStorage::default())),
            system_metrics: Arc::new(RwLock::new(SystemMetricsStorage::default())),
            error_metrics: Arc::new(RwLock::new(ErrorMetricsStorage::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetricsStorage::default())),
            start_time: Instant::now(),
            active: Arc::new(RwLock::new(false)),
        })
    }

    /// Start metrics collection
    pub async fn start(&self) -> Result<()> {
        debug!("Starting metrics collection");

        *self.active.write().await = true;

        // Start background collection tasks
        self.start_system_metrics_collection().await;
        self.start_cleanup_task().await;

        Ok(())
    }

    /// Stop metrics collection
    pub async fn stop(&self) -> Result<()> {
        debug!("Stopping metrics collection");
        *self.active.write().await = false;
        Ok(())
    }

    /// Record a request metric
    pub async fn record_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        response_time: Duration,
        _user_id: Option<uuid::Uuid>,
        _api_key_id: Option<uuid::Uuid>,
    ) -> Result<()> {
        let mut metrics = self.request_metrics.write().await;

        metrics.total_requests += 1;
        metrics
            .response_times
            .push(response_time.as_millis() as f64);
        *metrics.status_codes.entry(status_code).or_insert(0) += 1;

        let endpoint_key = format!("{} {}", method, path);
        *metrics.endpoints.entry(endpoint_key).or_insert(0) += 1;

        metrics.last_minute_requests.push(Instant::now());

        Ok(())
    }

    /// Record a provider request metric
    pub async fn record_provider_request(
        &self,
        provider: &str,
        _model: &str,
        tokens_used: u32,
        cost: f64,
        response_time: Duration,
        success: bool,
    ) -> Result<()> {
        let mut metrics = self.provider_metrics.write().await;

        metrics.total_requests += 1;
        *metrics
            .provider_requests
            .entry(provider.to_string())
            .or_insert(0) += 1;

        metrics
            .provider_response_times
            .entry(provider.to_string())
            .or_insert_with(Vec::new)
            .push(response_time.as_millis() as f64);

        if !success {
            *metrics
                .provider_errors
                .entry(provider.to_string())
                .or_insert(0) += 1;
        }

        *metrics.token_usage.entry(provider.to_string()).or_insert(0) += tokens_used as u64;
        *metrics.costs.entry(provider.to_string()).or_insert(0.0) += cost;

        Ok(())
    }

    /// Record an error metric
    pub async fn record_error(
        &self,
        error_type: &str,
        _error_message: &str,
        _context: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut metrics = self.error_metrics.write().await;

        metrics.total_errors += 1;
        *metrics
            .error_types
            .entry(error_type.to_string())
            .or_insert(0) += 1;

        // Classify error severity
        if error_type.contains("critical") || error_type.contains("fatal") {
            metrics.critical_errors += 1;
        } else if error_type.contains("warning") || error_type.contains("warn") {
            metrics.warnings += 1;
        }

        metrics.last_minute_errors.push(Instant::now());

        Ok(())
    }

    /// Record cache hit
    pub async fn record_cache_hit(&self) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        metrics.cache_hits += 1;
        Ok(())
    }

    /// Record cache miss
    pub async fn record_cache_miss(&self) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        metrics.cache_misses += 1;
        Ok(())
    }

    /// Record database query time
    pub async fn record_db_query_time(&self, duration: Duration) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        metrics.db_query_times.push(duration.as_millis() as f64);
        Ok(())
    }

    /// Get request metrics
    pub async fn get_request_metrics(&self) -> Result<RequestMetrics> {
        let metrics = self.request_metrics.read().await;
        let now = Instant::now();

        // Calculate requests per second (last minute)
        let recent_requests = metrics
            .last_minute_requests
            .iter()
            .filter(|&&time| now.duration_since(time) <= Duration::from_secs(60))
            .count();
        let requests_per_second = recent_requests as f64 / 60.0;

        // Calculate response time percentiles
        let mut sorted_times = metrics.response_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg_response_time = if sorted_times.is_empty() {
            0.0
        } else {
            sorted_times.iter().sum::<f64>() / sorted_times.len() as f64
        };

        let p95_response_time = calculate_percentile(&sorted_times, 0.95);
        let p99_response_time = calculate_percentile(&sorted_times, 0.99);

        // Calculate success rate
        let total_requests = metrics.total_requests;
        let error_requests = metrics
            .status_codes
            .iter()
            .filter(|(code, _)| **code >= 400)
            .map(|(_, count)| *count)
            .sum::<u64>();

        let success_rate = if total_requests > 0 {
            ((total_requests - error_requests) as f64 / total_requests as f64) * 100.0
        } else {
            100.0
        };

        Ok(RequestMetrics {
            total_requests,
            requests_per_second,
            avg_response_time_ms: avg_response_time,
            p95_response_time_ms: p95_response_time,
            p99_response_time_ms: p99_response_time,
            success_rate,
            status_codes: metrics.status_codes.clone(),
            endpoints: metrics.endpoints.clone(),
        })
    }

    /// Get provider metrics
    pub async fn get_provider_metrics(&self) -> Result<ProviderMetrics> {
        let metrics = self.provider_metrics.read().await;

        // Calculate success rates
        let mut provider_success_rates = HashMap::new();
        for (provider, &requests) in &metrics.provider_requests {
            let errors = metrics.provider_errors.get(provider).unwrap_or(&0);
            let success_rate = if requests > 0 {
                ((requests - errors) as f64 / requests as f64) * 100.0
            } else {
                100.0
            };
            provider_success_rates.insert(provider.clone(), success_rate);
        }

        // Calculate average response times
        let mut provider_response_times = HashMap::new();
        for (provider, times) in &metrics.provider_response_times {
            let avg_time = if times.is_empty() {
                0.0
            } else {
                times.iter().sum::<f64>() / times.len() as f64
            };
            provider_response_times.insert(provider.clone(), avg_time);
        }

        Ok(ProviderMetrics {
            total_provider_requests: metrics.total_requests,
            provider_success_rates,
            provider_response_times,
            provider_errors: metrics.provider_errors.clone(),
            provider_usage: metrics.provider_requests.clone(),
            token_usage: metrics.token_usage.clone(),
            costs: metrics.costs.clone(),
        })
    }

    /// Get system metrics
    pub async fn get_system_metrics(&self) -> Result<SystemResourceMetrics> {
        let metrics = self.system_metrics.read().await;

        // Calculate averages from samples
        let cpu_usage = calculate_average(&metrics.cpu_samples);
        let memory_usage = calculate_average_u64(&metrics.memory_samples);
        let disk_usage = calculate_average_u64(&metrics.disk_samples);
        let network_bytes_in = calculate_average_u64(&metrics.network_in_samples);
        let network_bytes_out = calculate_average_u64(&metrics.network_out_samples);
        let active_connections = calculate_average_u32(&metrics.connection_samples);

        Ok(SystemResourceMetrics {
            cpu_usage,
            memory_usage,
            memory_usage_percent: 0.0, // TODO: Calculate based on total memory
            disk_usage,
            disk_usage_percent: 0.0, // TODO: Calculate based on total disk
            network_bytes_in,
            network_bytes_out,
            active_connections,
            database_connections: 0, // TODO: Get from connection pool
            redis_connections: 0,    // TODO: Get from Redis pool
        })
    }

    /// Get error metrics
    pub async fn get_error_metrics(&self) -> Result<ErrorMetrics> {
        let metrics = self.error_metrics.read().await;
        let now = Instant::now();

        // Calculate error rate (errors per second in last minute)
        let recent_errors = metrics
            .last_minute_errors
            .iter()
            .filter(|&&time| now.duration_since(time) <= Duration::from_secs(60))
            .count();
        let error_rate = recent_errors as f64 / 60.0;

        Ok(ErrorMetrics {
            total_errors: metrics.total_errors,
            error_rate,
            error_types: metrics.error_types.clone(),
            error_endpoints: metrics.error_endpoints.clone(),
            critical_errors: metrics.critical_errors,
            warnings: metrics.warnings,
        })
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Result<PerformanceMetrics> {
        let metrics = self.performance_metrics.read().await;

        // Calculate cache hit/miss rates
        let total_cache_requests = metrics.cache_hits + metrics.cache_misses;
        let cache_hit_rate = if total_cache_requests > 0 {
            (metrics.cache_hits as f64 / total_cache_requests as f64) * 100.0
        } else {
            0.0
        };
        let cache_miss_rate = 100.0 - cache_hit_rate;

        // Calculate average DB query time
        let avg_db_query_time = calculate_average(&metrics.db_query_times);

        // Calculate throughput
        let throughput = calculate_average(&metrics.throughput_samples);

        // Calculate queue depth
        let queue_depth = calculate_average_u32(&metrics.queue_depths);

        Ok(PerformanceMetrics {
            cache_hit_rate,
            cache_miss_rate,
            avg_db_query_time_ms: avg_db_query_time,
            queue_depth,
            throughput,
            latency_percentiles: LatencyPercentiles {
                p50: 0.0, // TODO: Calculate from request metrics
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
                p999: 0.0,
            },
        })
    }

    /// Start system metrics collection
    async fn start_system_metrics_collection(&self) {
        let system_metrics = self.system_metrics.clone();
        let active = self.active.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                if !*active.read().await {
                    break;
                }

                // Collect system metrics
                let mut metrics = system_metrics.write().await;

                // TODO: Implement actual system metrics collection
                // For now, use placeholder values
                metrics.cpu_samples.push(get_cpu_usage());
                metrics.memory_samples.push(get_memory_usage());
                metrics.disk_samples.push(get_disk_usage());
                metrics.network_in_samples.push(get_network_bytes_in());
                metrics.network_out_samples.push(get_network_bytes_out());
                metrics.connection_samples.push(get_active_connections());

                // Keep only recent samples (last hour)
                const MAX_SAMPLES: usize = 360; // 1 hour at 10-second intervals
                let cpu_len = metrics.cpu_samples.len();
                if cpu_len > MAX_SAMPLES {
                    metrics.cpu_samples.drain(0..cpu_len - MAX_SAMPLES);
                }
                let memory_len = metrics.memory_samples.len();
                if memory_len > MAX_SAMPLES {
                    metrics.memory_samples.drain(0..memory_len - MAX_SAMPLES);
                }
                // ... similar for other metrics
            }
        });
    }

    /// Start cleanup task for old metrics
    async fn start_cleanup_task(&self) {
        let request_metrics = self.request_metrics.clone();
        let error_metrics = self.error_metrics.clone();
        let active = self.active.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                if !*active.read().await {
                    break;
                }

                let now = Instant::now();

                // Clean up old request timestamps
                {
                    let mut metrics = request_metrics.write().await;
                    metrics
                        .last_minute_requests
                        .retain(|&time| now.duration_since(time) <= Duration::from_secs(300));
                }

                // Clean up old error timestamps
                {
                    let mut metrics = error_metrics.write().await;
                    metrics
                        .last_minute_errors
                        .retain(|&time| now.duration_since(time) <= Duration::from_secs(300));
                }
            }
        });
    }
}

/// Calculate percentile from sorted values
fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    if percentile >= 1.0 {
        return *sorted_values.last().unwrap();
    }

    let index = percentile * (sorted_values.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;

    if lower == upper {
        sorted_values[lower]
    } else {
        let weight = index - lower as f64;
        sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
    }
}

/// Calculate average of f64 values
fn calculate_average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

/// Calculate average of u64 values
fn calculate_average_u64(values: &[u64]) -> u64 {
    if values.is_empty() {
        0
    } else {
        values.iter().sum::<u64>() / values.len() as u64
    }
}

/// Calculate average of u32 values
fn calculate_average_u32(values: &[u32]) -> u32 {
    if values.is_empty() {
        0
    } else {
        values.iter().sum::<u32>() / values.len() as u32
    }
}

// Placeholder functions for system metrics collection
// In a real implementation, these would use proper system monitoring libraries

fn get_cpu_usage() -> f64 {
    // Placeholder implementation
    rand::random::<f64>() * 100.0
}

fn get_memory_usage() -> u64 {
    // Placeholder implementation
    1024 * 1024 * 512 // 512MB
}

fn get_disk_usage() -> u64 {
    // Placeholder implementation
    1024 * 1024 * 1024 * 10 // 10GB
}

fn get_network_bytes_in() -> u64 {
    // Placeholder implementation
    1024 * 1024 // 1MB
}

fn get_network_bytes_out() -> u64 {
    // Placeholder implementation
    1024 * 512 // 512KB
}

fn get_active_connections() -> u32 {
    // Placeholder implementation
    100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 0.5), 3.0); // 50th percentile
        assert_eq!(calculate_percentile(&values, 0.95), 4.8); // 95th percentile (interpolated)
        assert_eq!(calculate_percentile(&values, 1.0), 5.0); // 100th percentile
        assert_eq!(calculate_percentile(&[], 0.5), 0.0); // empty array
    }

    #[test]
    fn test_calculate_average() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_average(&values), 3.0);
        assert_eq!(calculate_average(&[]), 0.0);
    }

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MonitoringConfig {
            metrics: crate::config::MetricsConfig {
                enabled: true,
                port: 9090,
                path: "/metrics".to_string(),
            },
            tracing: crate::config::TracingConfig {
                enabled: false,
                endpoint: None,
                service_name: "test".to_string(),
            },
            health: crate::config::HealthConfig {
                path: "/health".to_string(),
                detailed: true,
            },
        };

        let collector = MetricsCollector::new(&config).await.unwrap();
        assert!(!*collector.active.read().await);
    }
}
