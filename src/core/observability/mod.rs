//! Advanced observability and monitoring system
//!
//! This module provides comprehensive monitoring, logging, and alerting capabilities.

mod alerting;
mod destinations;
mod histogram;
mod logging;
mod metrics;
mod tracing;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use alerting::AlertManager;
pub use destinations::{AlertChannel, AlertRule, LogDestination, TraceExporter};
pub use histogram::{BoundedHistogram, HISTOGRAM_MAX_SAMPLES};
pub use logging::LogAggregator;
pub use metrics::{DataDogClient, MetricsCollector, OtelExporter, PrometheusMetrics};
pub use tracing::PerformanceTracer;
pub use types::{
    AlertCondition, AlertSeverity, AlertState, ErrorDetails, LogEntry, LogLevel, MetricValue,
    SpanLog, TokenUsage, TraceSpan,
};
