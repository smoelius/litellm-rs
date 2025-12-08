//! Enhanced logging utilities with structured logging and performance optimizations
//!
//! This module provides improved logging capabilities including structured logging,
//! log sampling, and async logging to minimize performance impact.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tracing::{Level, debug, error, info, warn};
use uuid::Uuid;

/// Log entry for async processing
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: String,
    /// Logger name/component
    pub logger: String,
    /// Log message
    pub message: String,
    /// Structured fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// User ID if available
    pub user_id: Option<Uuid>,
    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,
}

/// Async logger configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AsyncLoggerConfig {
    /// Buffer size for log entries
    pub buffer_size: usize,
    /// Whether to drop logs on buffer overflow
    pub drop_on_overflow: bool,
    /// Sampling rate for high-frequency logs (0.0 to 1.0)
    pub sample_rate: f64,
    /// Maximum log message length
    pub max_message_length: usize,
}

impl Default for AsyncLoggerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            drop_on_overflow: false,
            sample_rate: 1.0,
            max_message_length: 1024,
        }
    }
}

/// Async logger for high-performance logging
#[allow(dead_code)]
pub struct AsyncLogger {
    sender: mpsc::Sender<LogEntry>,
    config: AsyncLoggerConfig,
    sample_counter: AtomicU64,
}

#[allow(dead_code)]
impl AsyncLogger {
    /// Create a new async logger with bounded channel to prevent memory leaks
    pub fn new(config: AsyncLoggerConfig) -> Self {
        // Use bounded channel with configured buffer size to prevent OOM
        let (sender, mut receiver) = mpsc::channel::<LogEntry>(config.buffer_size);

        // Spawn background task to process log entries
        tokio::spawn(async move {
            while let Some(entry) = receiver.recv().await {
                Self::process_log_entry(entry).await;
            }
        });

        Self {
            sender,
            config,
            sample_counter: AtomicU64::new(0),
        }
    }

    /// Try to send a log entry, handling backpressure
    fn try_send(&self, entry: LogEntry) -> bool {
        match self.sender.try_send(entry) {
            Ok(()) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                if !self.config.drop_on_overflow {
                    // Log overflow warning (but don't recurse)
                    warn!("Async logger buffer full, log entry dropped");
                }
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                error!("Async logger channel closed");
                false
            }
        }
    }

    /// Log a message with structured fields
    pub fn log_structured(
        &self,
        level: Level,
        logger: &str,
        message: &str,
        fields: HashMap<String, serde_json::Value>,
        request_id: Option<String>,
        user_id: Option<Uuid>,
    ) {
        // Apply sampling if configured (rate < 1.0)
        if self.config.sample_rate < 1.0 {
            if self.config.sample_rate <= 0.0 {
                return; // 0% sampling = drop all
            }
            let counter = self.sample_counter.fetch_add(1, Ordering::Relaxed);
            // Correct sampling: sample every N logs where N = 1/rate
            // e.g., rate=0.1 means keep 1 in 10, rate=0.5 means keep 1 in 2
            let sample_interval = (1.0 / self.config.sample_rate) as u64;
            if counter % sample_interval != 0 {
                return;
            }
        }

        // Truncate message if too long
        let truncated_message = if message.len() > self.config.max_message_length {
            format!("{}...", &message[..self.config.max_message_length - 3])
        } else {
            message.to_string()
        };

        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: level.to_string(),
            logger: logger.to_string(),
            message: truncated_message,
            fields,
            request_id,
            user_id,
            trace_id: Self::current_trace_id(),
        };

        // Use try_send with backpressure handling instead of blocking send
        self.try_send(entry);
    }

    /// Log a simple message
    pub fn log(&self, level: Level, logger: &str, message: &str) {
        self.log_structured(level, logger, message, HashMap::new(), None, None);
    }

    /// Log with request context
    pub fn log_with_context(
        &self,
        level: Level,
        logger: &str,
        message: &str,
        request_id: Option<String>,
        user_id: Option<Uuid>,
    ) {
        self.log_structured(level, logger, message, HashMap::new(), request_id, user_id);
    }

    /// Process a log entry (background task)
    async fn process_log_entry(entry: LogEntry) {
        // In a real implementation, you might:
        // - Write to files
        // - Send to external logging services
        // - Store in databases
        // - Forward to monitoring systems

        // For now, just output to tracing
        let level = match entry.level.as_str() {
            "ERROR" => Level::ERROR,
            "WARN" => Level::WARN,
            "INFO" => Level::INFO,
            "DEBUG" => Level::DEBUG,
            _ => Level::INFO,
        };

        match level {
            Level::ERROR => error!(
                logger = entry.logger,
                request_id = entry.request_id,
                user_id = ?entry.user_id,
                trace_id = entry.trace_id,
                fields = ?entry.fields,
                "{}",
                entry.message
            ),
            Level::WARN => warn!(
                logger = entry.logger,
                request_id = entry.request_id,
                user_id = ?entry.user_id,
                trace_id = entry.trace_id,
                fields = ?entry.fields,
                "{}",
                entry.message
            ),
            Level::INFO => info!(
                logger = entry.logger,
                request_id = entry.request_id,
                user_id = ?entry.user_id,
                trace_id = entry.trace_id,
                fields = ?entry.fields,
                "{}",
                entry.message
            ),
            Level::DEBUG => debug!(
                logger = entry.logger,
                request_id = entry.request_id,
                user_id = ?entry.user_id,
                trace_id = entry.trace_id,
                fields = ?entry.fields,
                "{}",
                entry.message
            ),
            _ => info!(
                logger = entry.logger,
                request_id = entry.request_id,
                user_id = ?entry.user_id,
                trace_id = entry.trace_id,
                fields = ?entry.fields,
                "{}",
                entry.message
            ),
        }
    }

    /// Get current trace ID from tracing context
    fn current_trace_id() -> Option<String> {
        // In a real implementation, extract from tracing span context
        // For now, return None
        None
    }
}

/// Global async logger instance
#[allow(dead_code)]
static ASYNC_LOGGER: OnceLock<AsyncLogger> = OnceLock::new();

/// Initialize the global async logger
#[allow(dead_code)]
pub fn init_async_logger(config: AsyncLoggerConfig) {
    ASYNC_LOGGER.get_or_init(|| AsyncLogger::new(config));
}

/// Get the global async logger
#[allow(dead_code)]
pub fn async_logger() -> Option<&'static AsyncLogger> {
    ASYNC_LOGGER.get()
}

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

/// Security-aware logging utilities
#[allow(dead_code)]
pub struct SecurityLogger;

#[allow(dead_code)]
impl SecurityLogger {
    /// Log authentication events
    pub fn log_auth_event(
        event_type: &str,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        success: bool,
        details: Option<&str>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "event_type".to_string(),
            serde_json::Value::String(event_type.to_string()),
        );
        fields.insert("success".to_string(), serde_json::Value::Bool(success));

        if let Some(ip) = ip_address {
            fields.insert(
                "ip_address".to_string(),
                serde_json::Value::String(ip.to_string()),
            );
        }

        if let Some(ua) = user_agent {
            // Truncate user agent to prevent log injection
            let safe_ua = ua.chars().take(200).collect::<String>();
            fields.insert("user_agent".to_string(), serde_json::Value::String(safe_ua));
        }

        if let Some(details) = details {
            fields.insert(
                "details".to_string(),
                serde_json::Value::String(details.to_string()),
            );
        }

        let level = if success { Level::INFO } else { Level::WARN };
        let message = format!(
            "Authentication {}: {}",
            if success { "success" } else { "failure" },
            event_type
        );

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", &message, fields, None, user_id);
        }
    }

    /// Log authorization events
    pub fn log_authz_event(
        user_id: Uuid,
        resource: &str,
        action: &str,
        granted: bool,
        reason: Option<&str>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "resource".to_string(),
            serde_json::Value::String(resource.to_string()),
        );
        fields.insert(
            "action".to_string(),
            serde_json::Value::String(action.to_string()),
        );
        fields.insert("granted".to_string(), serde_json::Value::Bool(granted));

        if let Some(reason) = reason {
            fields.insert(
                "reason".to_string(),
                serde_json::Value::String(reason.to_string()),
            );
        }

        let level = if granted { Level::DEBUG } else { Level::WARN };
        let message = format!(
            "Authorization {}: {} on {}",
            if granted { "granted" } else { "denied" },
            action,
            resource
        );

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", &message, fields, None, Some(user_id));
        }
    }

    /// Log security violations
    pub fn log_security_violation(
        violation_type: &str,
        severity: &str,
        description: &str,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        additional_data: Option<HashMap<String, serde_json::Value>>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "violation_type".to_string(),
            serde_json::Value::String(violation_type.to_string()),
        );
        fields.insert(
            "severity".to_string(),
            serde_json::Value::String(severity.to_string()),
        );

        if let Some(ip) = ip_address {
            fields.insert(
                "ip_address".to_string(),
                serde_json::Value::String(ip.to_string()),
            );
        }

        if let Some(data) = additional_data {
            for (key, value) in data {
                fields.insert(key, value);
            }
        }

        let level = match severity.to_lowercase().as_str() {
            "critical" | "high" => Level::ERROR,
            "medium" => Level::WARN,
            _ => Level::INFO,
        };

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", description, fields, None, user_id);
        }
    }
}

/// Request metrics for performance logging
#[derive(Debug)]
pub struct RequestMetrics {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path
    pub path: String,
    /// HTTP status code
    pub status_code: u16,
    /// Request duration in milliseconds
    pub duration_ms: u64,
    /// Request size in bytes
    pub request_size: u64,
    /// Response size in bytes
    pub response_size: u64,
    /// Optional user ID
    pub user_id: Option<Uuid>,
    /// Optional request ID for tracing
    pub request_id: Option<String>,
}

/// Performance logging utilities
#[allow(dead_code)]
pub struct PerformanceLogger;

#[allow(dead_code)]
impl PerformanceLogger {
    /// Log request performance metrics
    pub fn log_request_metrics(metrics: RequestMetrics) {
        let mut fields = HashMap::new();
        fields.insert(
            "method".to_string(),
            serde_json::Value::String(metrics.method.clone()),
        );
        fields.insert(
            "path".to_string(),
            serde_json::Value::String(metrics.path.clone()),
        );
        fields.insert(
            "status_code".to_string(),
            serde_json::Value::Number(metrics.status_code.into()),
        );
        fields.insert(
            "duration_ms".to_string(),
            serde_json::Value::Number(metrics.duration_ms.into()),
        );
        fields.insert(
            "request_size".to_string(),
            serde_json::Value::Number(metrics.request_size.into()),
        );
        fields.insert(
            "response_size".to_string(),
            serde_json::Value::Number(metrics.response_size.into()),
        );

        let message = format!(
            "{} {} {} {}ms",
            metrics.method, metrics.path, metrics.status_code, metrics.duration_ms
        );

        // Use different log levels based on performance
        let level = if metrics.duration_ms > 5000 {
            Level::WARN // Very slow requests
        } else if metrics.duration_ms > 1000 {
            Level::INFO // Slow requests
        } else {
            Level::DEBUG // Normal requests
        };

        if let Some(logger) = async_logger() {
            logger.log_structured(
                level,
                "performance",
                &message,
                fields,
                metrics.request_id,
                metrics.user_id,
            );
        }
    }

    /// Log provider performance metrics
    pub fn log_provider_metrics(
        provider: &str,
        model: &str,
        duration_ms: u64,
        token_count: Option<u32>,
        success: bool,
        error: Option<&str>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "provider".to_string(),
            serde_json::Value::String(provider.to_string()),
        );
        fields.insert(
            "model".to_string(),
            serde_json::Value::String(model.to_string()),
        );
        fields.insert(
            "duration_ms".to_string(),
            serde_json::Value::Number(duration_ms.into()),
        );
        fields.insert("success".to_string(), serde_json::Value::Bool(success));

        if let Some(tokens) = token_count {
            fields.insert(
                "token_count".to_string(),
                serde_json::Value::Number(tokens.into()),
            );
        }

        if let Some(err) = error {
            fields.insert(
                "error".to_string(),
                serde_json::Value::String(err.to_string()),
            );
        }

        let level = if success { Level::DEBUG } else { Level::WARN };
        let message = format!(
            "Provider {} {} {}ms {}",
            provider,
            model,
            duration_ms,
            if success { "success" } else { "failed" }
        );

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "performance", &message, fields, None, None);
        }
    }
}

/// Convenience macros for structured logging
#[macro_export]
macro_rules! log_structured {
    ($level:expr, $logger:expr, $message:expr, $($key:expr => $value:expr),*) => {
        {
            let mut fields = std::collections::HashMap::new();
            $(
                fields.insert($key.to_string(), serde_json::to_value($value).unwrap_or(serde_json::Value::Null));
            )*

            if let Some(logger) = $crate::utils::logging::async_logger() {
                logger.log_structured($level, $logger, $message, fields, None, None);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_sampler() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("test", 0.5);

        // Should sample approximately half the logs
        let mut sampled_count = 0;
        for _ in 0..1000 {
            if sampler.should_log("test") {
                sampled_count += 1;
            }
        }

        // Allow some variance due to sampling
        assert!(sampled_count > 400 && sampled_count < 600);
    }

    #[test]
    fn test_log_sampler_edge_cases() {
        let mut sampler = LogSampler::new();

        // Test 100% sampling
        sampler.set_sample_rate("full", 1.0);
        let mut count = 0;
        for _ in 0..100 {
            if sampler.should_log("full") {
                count += 1;
            }
        }
        assert_eq!(count, 100);

        // Test 0% sampling
        sampler.set_sample_rate("none", 0.0);
        count = 0;
        for _ in 0..100 {
            if sampler.should_log("none") {
                count += 1;
            }
        }
        assert_eq!(count, 0);

        // Test 10% sampling
        sampler.set_sample_rate("ten_percent", 0.1);
        count = 0;
        for _ in 0..1000 {
            if sampler.should_log("ten_percent") {
                count += 1;
            }
        }
        // Should be exactly 100 (every 10th log)
        assert_eq!(count, 100);
    }

    #[test]
    fn test_async_logger_config() {
        let config = AsyncLoggerConfig {
            buffer_size: 5000,
            drop_on_overflow: true,
            sample_rate: 0.8,
            max_message_length: 512,
        };

        assert_eq!(config.buffer_size, 5000);
        assert!(config.drop_on_overflow);
        assert_eq!(config.sample_rate, 0.8);
        assert_eq!(config.max_message_length, 512);
    }

    #[tokio::test]
    async fn test_async_logger_creation() {
        let config = AsyncLoggerConfig::default();
        let logger = AsyncLogger::new(config);

        // Test basic logging
        logger.log(Level::INFO, "test", "test message");

        // Give background task time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_async_logger_bounded_channel() {
        // Create logger with small buffer to test backpressure
        let config = AsyncLoggerConfig {
            buffer_size: 10,
            drop_on_overflow: true,
            sample_rate: 1.0,
            max_message_length: 100,
        };
        let logger = AsyncLogger::new(config);

        // Send more messages than buffer can hold
        for i in 0..100 {
            logger.log(Level::INFO, "test", &format!("message {}", i));
        }

        // Should not panic or hang - messages are dropped when buffer full
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_async_logger_sampling() {
        let config = AsyncLoggerConfig {
            buffer_size: 1000,
            drop_on_overflow: false,
            sample_rate: 0.5, // 50% sampling
            max_message_length: 100,
        };
        let logger = AsyncLogger::new(config);

        // The sampling counter is internal, so we just verify no panic
        for i in 0..100 {
            logger.log(Level::INFO, "test", &format!("sampled message {}", i));
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
}
