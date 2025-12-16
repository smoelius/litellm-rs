//! Performance logging utilities

use crate::utils::logging::logging::async_logger::async_logger;
use crate::utils::logging::logging::types::RequestMetrics;
use std::collections::HashMap;
use tracing::Level;

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
