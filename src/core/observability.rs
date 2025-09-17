//! Advanced observability and monitoring system
//!
//! This module provides comprehensive monitoring, logging, and alerting capabilities.

use crate::utils::error::{GatewayError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Metrics collector and exporter
pub struct MetricsCollector {
    /// Prometheus metrics
    prometheus_metrics: Arc<RwLock<PrometheusMetrics>>,
    /// DataDog metrics
    datadog_client: Option<DataDogClient>,
    /// OpenTelemetry exporter
    otel_exporter: Option<OtelExporter>,
    /// Custom metrics storage
    custom_metrics: Arc<RwLock<HashMap<String, MetricValue>>>,
}

/// Prometheus metrics structure
#[derive(Debug, Default)]
pub struct PrometheusMetrics {
    /// Request counters
    pub request_total: HashMap<String, u64>,
    /// Request duration histograms
    pub request_duration: HashMap<String, Vec<f64>>,
    /// Error counters
    pub error_total: HashMap<String, u64>,
    /// Token usage counters
    pub token_usage: HashMap<String, u64>,
    /// Cost tracking
    pub cost_total: HashMap<String, f64>,
    /// Provider health status
    pub provider_health: HashMap<String, f64>,
    /// Cache hit/miss ratios
    pub cache_hits: u64,
    pub cache_misses: u64,
    /// Active connections
    pub active_connections: u64,
    /// Queue sizes
    pub queue_size: HashMap<String, u64>,
}

/// DataDog client for metrics
pub struct DataDogClient {
    /// API key
    api_key: String,
    /// Base URL
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
    /// Tags to add to all metrics
    default_tags: Vec<String>,
}

/// OpenTelemetry exporter
pub struct OtelExporter {
    /// Endpoint URL
    endpoint: String,
    /// Headers
    headers: HashMap<String, String>,
    /// HTTP client
    client: reqwest::Client,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary { sum: f64, count: u64 },
}

/// Log aggregator for centralized logging
pub struct LogAggregator {
    /// Configured log destinations
    destinations: Vec<LogDestination>,
    /// Log buffer
    buffer: Arc<RwLock<Vec<LogEntry>>>,
    /// Buffer flush interval
    flush_interval: Duration,
}

/// Log destinations
#[derive(Debug, Clone)]
pub enum LogDestination {
    /// Elasticsearch
    Elasticsearch {
        url: String,
        index: String,
        auth: Option<String>,
    },
    /// Splunk
    Splunk {
        url: String,
        token: String,
        index: String,
    },
    /// AWS CloudWatch
    CloudWatch {
        region: String,
        log_group: String,
        log_stream: String,
    },
    /// Google Cloud Logging
    GCPLogging {
        project_id: String,
        log_name: String,
    },
    /// Datadog Logs
    DatadogLogs {
        api_key: String,
        site: String,
    },
    /// Custom webhook
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Message
    pub message: String,
    /// Request ID
    pub request_id: Option<String>,
    /// User ID
    pub user_id: Option<String>,
    /// Provider
    pub provider: Option<String>,
    /// Model
    pub model: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Token usage
    pub tokens: Option<TokenUsage>,
    /// Cost
    pub cost: Option<f64>,
    /// Error details
    pub error: Option<ErrorDetails>,
    /// Additional fields
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Error details for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_message: String,
    pub error_code: Option<String>,
    pub stack_trace: Option<String>,
}

/// Alert manager for notifications
pub struct AlertManager {
    /// Alert channels
    channels: Vec<AlertChannel>,
    /// Alert rules
    rules: Vec<AlertRule>,
    /// Alert state tracking
    alert_states: Arc<RwLock<HashMap<String, AlertState>>>,
}

/// Alert channels
#[derive(Debug, Clone)]
pub enum AlertChannel {
    /// Slack webhook
    Slack {
        webhook_url: String,
        channel: String,
        username: String,
    },
    /// Email SMTP
    Email {
        smtp_host: String,
        smtp_port: u16,
        username: String,
        password: String,
        from: String,
        to: Vec<String>,
    },
    /// PagerDuty
    PagerDuty {
        integration_key: String,
        severity: String,
    },
    /// Discord webhook
    Discord {
        webhook_url: String,
    },
    /// Microsoft Teams
    Teams {
        webhook_url: String,
    },
    /// Custom webhook
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
}

/// Alert rules
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Condition
    pub condition: AlertCondition,
    /// Threshold value
    pub threshold: f64,
    /// Evaluation window
    pub window: Duration,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Channels to notify
    pub channels: Vec<String>,
    /// Whether rule is enabled
    pub enabled: bool,
}

/// Alert conditions
#[derive(Debug, Clone)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Alert state tracking
#[derive(Debug, Clone)]
pub struct AlertState {
    /// Whether alert is currently firing
    pub firing: bool,
    /// When alert started firing
    pub fired_at: Option<DateTime<Utc>>,
    /// Last notification sent
    pub last_notification: Option<DateTime<Utc>>,
    /// Notification count
    pub notification_count: u32,
}

/// Performance tracer for request tracing
pub struct PerformanceTracer {
    /// Active traces
    traces: Arc<RwLock<HashMap<String, TraceSpan>>>,
    /// Trace exporters
    exporters: Vec<TraceExporter>,
}

/// Trace span
#[derive(Debug, Clone)]
pub struct TraceSpan {
    /// Span ID
    pub span_id: String,
    /// Parent span ID
    pub parent_id: Option<String>,
    /// Trace ID
    pub trace_id: String,
    /// Operation name
    pub operation: String,
    /// Start time
    pub start_time: Instant,
    /// End time
    pub end_time: Option<Instant>,
    /// Tags
    pub tags: HashMap<String, String>,
    /// Logs
    pub logs: Vec<SpanLog>,
}

/// Span log entry
#[derive(Debug, Clone)]
pub struct SpanLog {
    pub timestamp: Instant,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// Trace exporters
#[derive(Debug, Clone)]
pub enum TraceExporter {
    /// Jaeger
    Jaeger {
        endpoint: String,
        service_name: String,
    },
    /// Zipkin
    Zipkin {
        endpoint: String,
        service_name: String,
    },
    /// OpenTelemetry
    OpenTelemetry {
        endpoint: String,
        headers: HashMap<String, String>,
    },
    /// DataDog APM
    DataDogAPM {
        api_key: String,
        service_name: String,
    },
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            prometheus_metrics: Arc::new(RwLock::new(PrometheusMetrics::default())),
            datadog_client: None,
            otel_exporter: None,
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure DataDog integration
    pub fn with_datadog(mut self, api_key: String, site: String) -> Self {
        self.datadog_client = Some(DataDogClient {
            api_key,
            base_url: format!("https://api.{}", site),
            client: reqwest::Client::new(),
            default_tags: vec![
                "service:litellm-gateway".to_string(),
                "env:production".to_string(),
            ],
        });
        self
    }

    /// Configure OpenTelemetry integration
    pub fn with_otel(mut self, endpoint: String, headers: HashMap<String, String>) -> Self {
        self.otel_exporter = Some(OtelExporter {
            endpoint,
            headers,
            client: reqwest::Client::new(),
        });
        self
    }

    /// Record request metrics
    pub async fn record_request(
        &self,
        provider: &str,
        model: &str,
        duration: Duration,
        tokens: Option<TokenUsage>,
        cost: Option<f64>,
        success: bool,
    ) {
        let mut metrics = self.prometheus_metrics.write().await;
        
        // Request counter
        let key = format!("{}:{}", provider, model);
        *metrics.request_total.entry(key.clone()).or_insert(0) += 1;
        
        // Duration histogram
        metrics.request_duration.entry(key.clone())
            .or_insert_with(Vec::new)
            .push(duration.as_secs_f64());
        
        // Error counter
        if !success {
            *metrics.error_total.entry(key.clone()).or_insert(0) += 1;
        }
        
        // Token usage
        if let Some(token_usage) = tokens {
            *metrics.token_usage.entry(format!("{}:prompt", key)).or_insert(0) += token_usage.prompt_tokens as u64;
            *metrics.token_usage.entry(format!("{}:completion", key)).or_insert(0) += token_usage.completion_tokens as u64;
        }
        
        // Cost tracking
        if let Some(request_cost) = cost {
            *metrics.cost_total.entry(key).or_insert(0.0) += request_cost;
        }
    }

    /// Record cache metrics
    pub async fn record_cache_hit(&self, hit: bool) {
        let mut metrics = self.prometheus_metrics.write().await;
        if hit {
            metrics.cache_hits += 1;
        } else {
            metrics.cache_misses += 1;
        }
    }

    /// Update provider health
    pub async fn update_provider_health(&self, provider: &str, health_score: f64) {
        let mut metrics = self.prometheus_metrics.write().await;
        metrics.provider_health.insert(provider.to_string(), health_score);
    }

    /// Export metrics to Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.prometheus_metrics.read().await;
        let mut output = String::new();
        
        // Request total
        output.push_str("# HELP litellm_requests_total Total number of requests\n");
        output.push_str("# TYPE litellm_requests_total counter\n");
        for (key, value) in &metrics.request_total {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                output.push_str(&format!(
                    "litellm_requests_total{{provider=\"{}\",model=\"{}\"}} {}\n",
                    parts[0], parts[1], value
                ));
            }
        }
        
        // Error total
        output.push_str("# HELP litellm_errors_total Total number of errors\n");
        output.push_str("# TYPE litellm_errors_total counter\n");
        for (key, value) in &metrics.error_total {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                output.push_str(&format!(
                    "litellm_errors_total{{provider=\"{}\",model=\"{}\"}} {}\n",
                    parts[0], parts[1], value
                ));
            }
        }
        
        // Cache metrics
        output.push_str("# HELP litellm_cache_hits_total Total cache hits\n");
        output.push_str("# TYPE litellm_cache_hits_total counter\n");
        output.push_str(&format!("litellm_cache_hits_total {}\n", metrics.cache_hits));
        
        output.push_str("# HELP litellm_cache_misses_total Total cache misses\n");
        output.push_str("# TYPE litellm_cache_misses_total counter\n");
        output.push_str(&format!("litellm_cache_misses_total {}\n", metrics.cache_misses));
        
        // Provider health
        output.push_str("# HELP litellm_provider_health Provider health score\n");
        output.push_str("# TYPE litellm_provider_health gauge\n");
        for (provider, health) in &metrics.provider_health {
            output.push_str(&format!(
                "litellm_provider_health{{provider=\"{}\"}} {}\n",
                provider, health
            ));
        }
        
        output
    }

    /// Send metrics to DataDog
    pub async fn send_to_datadog(&self) -> Result<()> {
        if let Some(client) = &self.datadog_client {
            let metrics = self.prometheus_metrics.read().await;
            
            // Convert metrics to DataDog format and send
            // Implementation would depend on DataDog API format
            debug!("Sending metrics to DataDog");
        }
        Ok(())
    }
}

impl LogAggregator {
    /// Create a new log aggregator
    pub fn new() -> Self {
        Self {
            destinations: vec![],
            buffer: Arc::new(RwLock::new(Vec::new())),
            flush_interval: Duration::from_secs(10),
        }
    }

    /// Add log destination
    pub fn add_destination(mut self, destination: LogDestination) -> Self {
        self.destinations.push(destination);
        self
    }

    /// Log an entry
    pub async fn log(&self, entry: LogEntry) {
        let mut buffer = self.buffer.write().await;
        buffer.push(entry);
        
        // Flush if buffer is full
        if buffer.len() >= 100 {
            self.flush_buffer().await;
        }
    }

    /// Flush log buffer
    async fn flush_buffer(&self) {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return;
        }
        
        let entries = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer);
        
        // Send to all destinations
        for destination in &self.destinations {
            if let Err(e) = self.send_to_destination(destination, &entries).await {
                error!("Failed to send logs to destination: {}", e);
            }
        }
    }

    /// Send logs to a specific destination
    async fn send_to_destination(
        &self,
        destination: &LogDestination,
        entries: &[LogEntry],
    ) -> Result<()> {
        match destination {
            LogDestination::Elasticsearch { url, index, auth } => {
                // Send to Elasticsearch
                debug!("Sending {} logs to Elasticsearch", entries.len());
            }
            LogDestination::Splunk { url, token, index } => {
                // Send to Splunk
                debug!("Sending {} logs to Splunk", entries.len());
            }
            LogDestination::DatadogLogs { api_key, site } => {
                // Send to Datadog Logs
                debug!("Sending {} logs to Datadog", entries.len());
            }
            LogDestination::Webhook { url, headers } => {
                // Send to webhook
                let client = reqwest::Client::new();
                let mut request = client.post(url).json(entries);
                
                for (key, value) in headers {
                    request = request.header(key, value);
                }
                
                request.send().await
                    .map_err(|e| GatewayError::Network(e.to_string()))?;
            }
            _ => {
                // Other destinations would be implemented similarly
                debug!("Sending {} logs to destination", entries.len());
            }
        }
        Ok(())
    }

    /// Start background flushing
    pub async fn start_background_flush(&self) {
        let aggregator = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(aggregator.flush_interval);
            loop {
                interval.tick().await;
                aggregator.flush_buffer().await;
            }
        });
    }
}

impl Clone for LogAggregator {
    fn clone(&self) -> Self {
        Self {
            destinations: self.destinations.clone(),
            buffer: self.buffer.clone(),
            flush_interval: self.flush_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();
        
        collector.record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(500),
            Some(TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            Some(0.01),
            true,
        ).await;
        
        let prometheus_output = collector.export_prometheus().await;
        assert!(prometheus_output.contains("litellm_requests_total"));
        assert!(prometheus_output.contains("provider=\"openai\""));
        assert!(prometheus_output.contains("model=\"gpt-4\""));
    }

    #[tokio::test]
    async fn test_log_aggregation() {
        let aggregator = LogAggregator::new();
        
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Test log entry".to_string(),
            request_id: Some("req-123".to_string()),
            user_id: Some("user-456".to_string()),
            provider: Some("openai".to_string()),
            model: Some("gpt-4".to_string()),
            duration_ms: Some(500),
            tokens: None,
            cost: None,
            error: None,
            fields: HashMap::new(),
        };
        
        aggregator.log(entry).await;
        
        let buffer = aggregator.buffer.read().await;
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0].message, "Test log entry");
    }
}
