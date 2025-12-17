//! Tests for monitoring module

#[cfg(test)]
mod tests {
    use super::super::types::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            id: "test-alert".to_string(),
            severity: AlertSeverity::Warning,
            title: "Test Alert".to_string(),
            description: "This is a test alert".to_string(),
            timestamp: chrono::Utc::now(),
            source: "test".to_string(),
            metadata: serde_json::json!({"test": true}),
            resolved: false,
        };

        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert!(!alert.resolved);
    }

    #[test]
    fn test_system_metrics_structure() {
        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            requests: RequestMetrics {
                total_requests: 1000,
                requests_per_second: 10.5,
                avg_response_time_ms: 150.0,
                p95_response_time_ms: 300.0,
                p99_response_time_ms: 500.0,
                success_rate: 99.5,
                status_codes: std::collections::HashMap::new(),
                endpoints: std::collections::HashMap::new(),
            },
            providers: ProviderMetrics {
                total_provider_requests: 800,
                provider_success_rates: std::collections::HashMap::new(),
                provider_response_times: std::collections::HashMap::new(),
                provider_errors: std::collections::HashMap::new(),
                provider_usage: std::collections::HashMap::new(),
                token_usage: std::collections::HashMap::new(),
                costs: std::collections::HashMap::new(),
            },
            system: SystemResourceMetrics {
                cpu_usage: 45.2,
                memory_usage: 1024 * 1024 * 512, // 512MB
                memory_usage_percent: 25.0,
                disk_usage: 1024 * 1024 * 1024 * 10, // 10GB
                disk_usage_percent: 50.0,
                network_bytes_in: 1024 * 1024,
                network_bytes_out: 1024 * 512,
                active_connections: 100,
                database_connections: 10,
                redis_connections: 5,
            },
            errors: ErrorMetrics {
                total_errors: 5,
                error_rate: 0.1,
                error_types: std::collections::HashMap::new(),
                error_endpoints: std::collections::HashMap::new(),
                critical_errors: 1,
                warnings: 4,
            },
            performance: PerformanceMetrics {
                cache_hit_rate: 85.5,
                cache_miss_rate: 14.5,
                avg_db_query_time_ms: 25.0,
                queue_depth: 0,
                throughput: 10.5,
                latency_percentiles: LatencyPercentiles {
                    p50: 100.0,
                    p90: 200.0,
                    p95: 300.0,
                    p99: 500.0,
                    p999: 800.0,
                },
            },
        };

        assert_eq!(metrics.requests.total_requests, 1000);
        assert_eq!(metrics.system.cpu_usage, 45.2);
        assert_eq!(metrics.errors.critical_errors, 1);
    }
}
