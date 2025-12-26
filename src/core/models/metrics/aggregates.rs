//! Aggregated metrics models

use super::super::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Provider metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    /// Metrics metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Provider name
    pub provider: String,
    /// Time period start
    pub period_start: chrono::DateTime<chrono::Utc>,
    /// Time period end
    pub period_end: chrono::DateTime<chrono::Utc>,
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// P50 response time
    pub p50_response_time_ms: f64,
    /// P95 response time
    pub p95_response_time_ms: f64,
    /// P99 response time
    pub p99_response_time_ms: f64,
    /// Total tokens processed
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Error breakdown
    pub error_breakdown: HashMap<String, u64>,
    /// Model breakdown
    pub model_breakdown: HashMap<String, ModelMetrics>,
}

/// Model-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Model name
    pub model: String,
    /// Request count
    pub requests: u64,
    /// Success count
    pub successes: u64,
    /// Total tokens
    pub tokens: u64,
    /// Total cost
    pub cost: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Metrics metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// Network I/O
    pub network_io: NetworkIO,
    /// Active connections
    pub active_connections: u32,
    /// Queue sizes
    pub queue_sizes: HashMap<String, u32>,
    /// Thread pool stats
    pub thread_pool: ThreadPoolStats,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkIO {
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Packets sent
    pub packets_sent: u64,
}

/// Thread pool statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadPoolStats {
    /// Active threads
    pub active_threads: u32,
    /// Total threads
    pub total_threads: u32,
    /// Queued tasks
    pub queued_tasks: u32,
    /// Completed tasks
    pub completed_tasks: u64,
}

/// Usage analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    /// Analytics metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Time period
    pub period: TimePeriod,
    /// User ID (if user-specific)
    pub user_id: Option<Uuid>,
    /// Team ID (if team-specific)
    pub team_id: Option<Uuid>,
    /// Total requests
    pub total_requests: u64,
    /// Total tokens
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Model usage breakdown
    pub model_usage: HashMap<String, ModelUsage>,
    /// Provider usage breakdown
    pub provider_usage: HashMap<String, ProviderUsage>,
    /// Daily breakdown
    pub daily_breakdown: Vec<DailyUsage>,
    /// Top endpoints
    pub top_endpoints: Vec<EndpointUsage>,
}

/// Time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Period start
    pub start: chrono::DateTime<chrono::Utc>,
    /// Period end
    pub end: chrono::DateTime<chrono::Utc>,
    /// Period type
    pub period_type: PeriodType,
}

/// Period type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodType {
    /// Hourly period
    Hour,
    /// Daily period
    Day,
    /// Weekly period
    Week,
    /// Monthly period
    Month,
    /// Yearly period
    Year,
    /// Custom period
    Custom,
}

/// Model usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Model name
    pub model: String,
    /// Request count
    pub requests: u64,
    /// Token count
    pub tokens: u64,
    /// Cost
    pub cost: f64,
    /// Success rate
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// Provider usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    /// Provider name
    pub provider: String,
    /// Request count
    pub requests: u64,
    /// Token count
    pub tokens: u64,
    /// Cost
    pub cost: f64,
    /// Success rate
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// Daily usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    /// Date
    pub date: chrono::NaiveDate,
    /// Request count
    pub requests: u64,
    /// Token count
    pub tokens: u64,
    /// Cost
    pub cost: f64,
    /// Unique users
    pub unique_users: u32,
}

/// Endpoint usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointUsage {
    /// Endpoint path
    pub endpoint: String,
    /// Request count
    pub requests: u64,
    /// Success rate
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Alert metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Alert name
    pub name: String,
    /// Alert description
    pub description: Option<String>,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert threshold
    pub threshold: f64,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert channels
    pub channels: Vec<String>,
    /// Alert enabled
    pub enabled: bool,
    /// Cooldown period in seconds
    pub cooldown_seconds: u64,
}

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCondition {
    /// High error rate condition
    ErrorRateHigh,
    /// Slow response time condition
    ResponseTimeSlow,
    /// High request volume condition
    RequestVolumeHigh,
    /// High cost condition
    CostHigh,
    /// Provider down condition
    ProviderDown,
    /// Quota exceeded condition
    QuotaExceeded,
    /// Custom alert condition
    Custom(String),
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Utc};

    fn create_test_metadata() -> Metadata {
        Metadata {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            extra: HashMap::new(),
        }
    }

    // ==================== ModelMetrics Tests ====================

    #[test]
    fn test_model_metrics_structure() {
        let metrics = ModelMetrics {
            model: "gpt-4".to_string(),
            requests: 100,
            successes: 95,
            tokens: 50000,
            cost: 5.0,
            avg_response_time_ms: 150.0,
        };

        assert_eq!(metrics.model, "gpt-4");
        assert_eq!(metrics.requests, 100);
        assert_eq!(metrics.successes, 95);
    }

    #[test]
    fn test_model_metrics_serialization() {
        let metrics = ModelMetrics {
            model: "claude-3".to_string(),
            requests: 50,
            successes: 48,
            tokens: 25000,
            cost: 2.5,
            avg_response_time_ms: 200.0,
        };

        let json = serde_json::to_value(&metrics).unwrap();
        assert_eq!(json["model"], "claude-3");
        assert_eq!(json["requests"], 50);
        assert_eq!(json["cost"], 2.5);
    }

    #[test]
    fn test_model_metrics_clone() {
        let metrics = ModelMetrics {
            model: "test".to_string(),
            requests: 10,
            successes: 10,
            tokens: 1000,
            cost: 0.5,
            avg_response_time_ms: 100.0,
        };

        let cloned = metrics.clone();
        assert_eq!(metrics.model, cloned.model);
        assert_eq!(metrics.requests, cloned.requests);
    }

    // ==================== NetworkIO Tests ====================

    #[test]
    fn test_network_io_default() {
        let io = NetworkIO::default();
        assert_eq!(io.bytes_received, 0);
        assert_eq!(io.bytes_sent, 0);
        assert_eq!(io.packets_received, 0);
        assert_eq!(io.packets_sent, 0);
    }

    #[test]
    fn test_network_io_serialization() {
        let io = NetworkIO {
            bytes_received: 1024,
            bytes_sent: 2048,
            packets_received: 100,
            packets_sent: 200,
        };

        let json = serde_json::to_value(&io).unwrap();
        assert_eq!(json["bytes_received"], 1024);
        assert_eq!(json["bytes_sent"], 2048);
    }

    // ==================== ThreadPoolStats Tests ====================

    #[test]
    fn test_thread_pool_stats_default() {
        let stats = ThreadPoolStats::default();
        assert_eq!(stats.active_threads, 0);
        assert_eq!(stats.total_threads, 0);
        assert_eq!(stats.queued_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
    }

    #[test]
    fn test_thread_pool_stats_serialization() {
        let stats = ThreadPoolStats {
            active_threads: 4,
            total_threads: 8,
            queued_tasks: 10,
            completed_tasks: 1000,
        };

        let json = serde_json::to_value(&stats).unwrap();
        assert_eq!(json["active_threads"], 4);
        assert_eq!(json["total_threads"], 8);
    }

    // ==================== PeriodType Tests ====================

    #[test]
    fn test_period_type_variants() {
        let types = vec![
            PeriodType::Hour,
            PeriodType::Day,
            PeriodType::Week,
            PeriodType::Month,
            PeriodType::Year,
            PeriodType::Custom,
        ];

        for period_type in types {
            let json = serde_json::to_string(&period_type).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_period_type_serialization() {
        assert!(serde_json::to_string(&PeriodType::Hour).unwrap().contains("hour"));
        assert!(serde_json::to_string(&PeriodType::Day).unwrap().contains("day"));
        assert!(serde_json::to_string(&PeriodType::Week).unwrap().contains("week"));
        assert!(serde_json::to_string(&PeriodType::Month).unwrap().contains("month"));
    }

    // ==================== TimePeriod Tests ====================

    #[test]
    fn test_time_period_structure() {
        let period = TimePeriod {
            start: Utc::now(),
            end: Utc::now(),
            period_type: PeriodType::Day,
        };

        assert!(matches!(period.period_type, PeriodType::Day));
    }

    #[test]
    fn test_time_period_serialization() {
        let period = TimePeriod {
            start: Utc::now(),
            end: Utc::now(),
            period_type: PeriodType::Week,
        };

        let json = serde_json::to_value(&period).unwrap();
        assert!(json["start"].is_string());
        assert!(json["end"].is_string());
        assert!(json["period_type"].is_string());
    }

    // ==================== ModelUsage Tests ====================

    #[test]
    fn test_model_usage_structure() {
        let usage = ModelUsage {
            model: "gpt-4".to_string(),
            requests: 100,
            tokens: 50000,
            cost: 5.0,
            success_rate: 0.95,
            avg_response_time_ms: 150.0,
        };

        assert_eq!(usage.model, "gpt-4");
        assert!((usage.success_rate - 0.95).abs() < f64::EPSILON);
    }

    // ==================== ProviderUsage Tests ====================

    #[test]
    fn test_provider_usage_structure() {
        let usage = ProviderUsage {
            provider: "openai".to_string(),
            requests: 500,
            tokens: 100000,
            cost: 10.0,
            success_rate: 0.98,
            avg_response_time_ms: 120.0,
        };

        assert_eq!(usage.provider, "openai");
        assert_eq!(usage.requests, 500);
    }

    // ==================== DailyUsage Tests ====================

    #[test]
    fn test_daily_usage_structure() {
        let usage = DailyUsage {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            requests: 1000,
            tokens: 500000,
            cost: 50.0,
            unique_users: 25,
        };

        assert_eq!(usage.requests, 1000);
        assert_eq!(usage.unique_users, 25);
    }

    #[test]
    fn test_daily_usage_serialization() {
        let usage = DailyUsage {
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            requests: 100,
            tokens: 10000,
            cost: 1.0,
            unique_users: 5,
        };

        let json = serde_json::to_value(&usage).unwrap();
        assert!(json["date"].is_string());
        assert_eq!(json["requests"], 100);
    }

    // ==================== EndpointUsage Tests ====================

    #[test]
    fn test_endpoint_usage_structure() {
        let usage = EndpointUsage {
            endpoint: "/v1/chat/completions".to_string(),
            requests: 5000,
            success_rate: 0.99,
            avg_response_time_ms: 100.0,
        };

        assert_eq!(usage.endpoint, "/v1/chat/completions");
        assert_eq!(usage.requests, 5000);
    }

    // ==================== AlertCondition Tests ====================

    #[test]
    fn test_alert_condition_variants() {
        let conditions = vec![
            AlertCondition::ErrorRateHigh,
            AlertCondition::ResponseTimeSlow,
            AlertCondition::RequestVolumeHigh,
            AlertCondition::CostHigh,
            AlertCondition::ProviderDown,
            AlertCondition::QuotaExceeded,
            AlertCondition::Custom("custom_condition".to_string()),
        ];

        for condition in conditions {
            let json = serde_json::to_string(&condition).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_alert_condition_serialization() {
        let condition = AlertCondition::ErrorRateHigh;
        let json = serde_json::to_string(&condition).unwrap();
        assert!(json.contains("error_rate_high"));
    }

    #[test]
    fn test_alert_condition_custom() {
        let condition = AlertCondition::Custom("my_custom_alert".to_string());
        let json = serde_json::to_string(&condition).unwrap();
        assert!(json.contains("my_custom_alert"));
    }

    // ==================== AlertSeverity Tests ====================

    #[test]
    fn test_alert_severity_variants() {
        let severities = vec![
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Error,
            AlertSeverity::Critical,
        ];

        for severity in severities {
            let json = serde_json::to_string(&severity).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_alert_severity_serialization() {
        assert!(serde_json::to_string(&AlertSeverity::Info).unwrap().contains("info"));
        assert!(serde_json::to_string(&AlertSeverity::Warning).unwrap().contains("warning"));
        assert!(serde_json::to_string(&AlertSeverity::Error).unwrap().contains("error"));
        assert!(serde_json::to_string(&AlertSeverity::Critical).unwrap().contains("critical"));
    }

    // ==================== AlertConfig Tests ====================

    #[test]
    fn test_alert_config_structure() {
        let config = AlertConfig {
            metadata: create_test_metadata(),
            name: "High Error Rate".to_string(),
            description: Some("Alert when error rate exceeds threshold".to_string()),
            condition: AlertCondition::ErrorRateHigh,
            threshold: 0.05,
            severity: AlertSeverity::Warning,
            channels: vec!["slack".to_string(), "email".to_string()],
            enabled: true,
            cooldown_seconds: 300,
        };

        assert_eq!(config.name, "High Error Rate");
        assert!(config.enabled);
        assert_eq!(config.channels.len(), 2);
    }

    #[test]
    fn test_alert_config_no_description() {
        let config = AlertConfig {
            metadata: create_test_metadata(),
            name: "Simple Alert".to_string(),
            description: None,
            condition: AlertCondition::CostHigh,
            threshold: 100.0,
            severity: AlertSeverity::Info,
            channels: vec![],
            enabled: false,
            cooldown_seconds: 0,
        };

        assert!(config.description.is_none());
        assert!(!config.enabled);
    }

    // ==================== ProviderMetrics Tests ====================

    #[test]
    fn test_provider_metrics_structure() {
        let mut error_breakdown = HashMap::new();
        error_breakdown.insert("rate_limit".to_string(), 10u64);
        error_breakdown.insert("timeout".to_string(), 5u64);

        let metrics = ProviderMetrics {
            metadata: create_test_metadata(),
            provider: "openai".to_string(),
            period_start: Utc::now(),
            period_end: Utc::now(),
            total_requests: 1000,
            successful_requests: 950,
            failed_requests: 50,
            success_rate: 0.95,
            avg_response_time_ms: 150.0,
            p50_response_time_ms: 100.0,
            p95_response_time_ms: 300.0,
            p99_response_time_ms: 500.0,
            total_tokens: 500000,
            total_cost: 50.0,
            error_breakdown,
            model_breakdown: HashMap::new(),
        };

        assert_eq!(metrics.provider, "openai");
        assert_eq!(metrics.total_requests, 1000);
        assert_eq!(metrics.error_breakdown.len(), 2);
    }

    #[test]
    fn test_provider_metrics_with_model_breakdown() {
        let mut model_breakdown = HashMap::new();
        model_breakdown.insert(
            "gpt-4".to_string(),
            ModelMetrics {
                model: "gpt-4".to_string(),
                requests: 500,
                successes: 480,
                tokens: 250000,
                cost: 30.0,
                avg_response_time_ms: 200.0,
            },
        );

        let metrics = ProviderMetrics {
            metadata: create_test_metadata(),
            provider: "openai".to_string(),
            period_start: Utc::now(),
            period_end: Utc::now(),
            total_requests: 500,
            successful_requests: 480,
            failed_requests: 20,
            success_rate: 0.96,
            avg_response_time_ms: 200.0,
            p50_response_time_ms: 150.0,
            p95_response_time_ms: 400.0,
            p99_response_time_ms: 600.0,
            total_tokens: 250000,
            total_cost: 30.0,
            error_breakdown: HashMap::new(),
            model_breakdown,
        };

        assert!(metrics.model_breakdown.contains_key("gpt-4"));
    }

    // ==================== SystemMetrics Tests ====================

    #[test]
    fn test_system_metrics_structure() {
        let metrics = SystemMetrics {
            metadata: create_test_metadata(),
            timestamp: Utc::now(),
            cpu_usage: 45.5,
            memory_usage: 8_000_000_000,
            memory_usage_percent: 50.0,
            disk_usage: 100_000_000_000,
            disk_usage_percent: 40.0,
            network_io: NetworkIO::default(),
            active_connections: 100,
            queue_sizes: HashMap::new(),
            thread_pool: ThreadPoolStats::default(),
        };

        assert!((metrics.cpu_usage - 45.5).abs() < f64::EPSILON);
        assert_eq!(metrics.active_connections, 100);
    }

    // ==================== UsageAnalytics Tests ====================

    #[test]
    fn test_usage_analytics_structure() {
        let analytics = UsageAnalytics {
            metadata: create_test_metadata(),
            period: TimePeriod {
                start: Utc::now(),
                end: Utc::now(),
                period_type: PeriodType::Month,
            },
            user_id: Some(Uuid::new_v4()),
            team_id: None,
            total_requests: 10000,
            total_tokens: 5000000,
            total_cost: 500.0,
            model_usage: HashMap::new(),
            provider_usage: HashMap::new(),
            daily_breakdown: vec![],
            top_endpoints: vec![],
        };

        assert!(analytics.user_id.is_some());
        assert!(analytics.team_id.is_none());
        assert_eq!(analytics.total_requests, 10000);
    }

    #[test]
    fn test_usage_analytics_with_breakdowns() {
        let mut model_usage = HashMap::new();
        model_usage.insert(
            "gpt-4".to_string(),
            ModelUsage {
                model: "gpt-4".to_string(),
                requests: 100,
                tokens: 50000,
                cost: 5.0,
                success_rate: 0.95,
                avg_response_time_ms: 150.0,
            },
        );

        let analytics = UsageAnalytics {
            metadata: create_test_metadata(),
            period: TimePeriod {
                start: Utc::now(),
                end: Utc::now(),
                period_type: PeriodType::Day,
            },
            user_id: None,
            team_id: None,
            total_requests: 100,
            total_tokens: 50000,
            total_cost: 5.0,
            model_usage,
            provider_usage: HashMap::new(),
            daily_breakdown: vec![],
            top_endpoints: vec![],
        };

        assert!(analytics.model_usage.contains_key("gpt-4"));
    }

    // ==================== Deserialization Tests ====================

    #[test]
    fn test_period_type_deserialization() {
        let json = r#""day""#;
        let period_type: PeriodType = serde_json::from_str(json).unwrap();
        assert!(matches!(period_type, PeriodType::Day));
    }

    #[test]
    fn test_alert_severity_deserialization() {
        let json = r#""critical""#;
        let severity: AlertSeverity = serde_json::from_str(json).unwrap();
        assert!(matches!(severity, AlertSeverity::Critical));
    }

    #[test]
    fn test_alert_condition_deserialization() {
        let json = r#""provider_down""#;
        let condition: AlertCondition = serde_json::from_str(json).unwrap();
        assert!(matches!(condition, AlertCondition::ProviderDown));
    }
}
