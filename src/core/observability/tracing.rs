//! Performance tracing system

use super::destinations::TraceExporter;
use super::types::TraceSpan;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Performance tracer for request tracing
pub struct PerformanceTracer {
    /// Active traces
    traces: Arc<RwLock<HashMap<String, TraceSpan>>>,
    /// Trace exporters
    exporters: Vec<TraceExporter>,
}

impl Default for PerformanceTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceTracer {
    /// Create a new performance tracer
    pub fn new() -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            exporters: Vec::new(),
        }
    }

    /// Add a trace exporter
    pub fn add_exporter(&mut self, exporter: TraceExporter) {
        self.exporters.push(exporter);
    }

    /// Start a new trace span
    pub async fn start_span(&self, span: TraceSpan) {
        let mut traces = self.traces.write().await;
        traces.insert(span.span_id.clone(), span);
    }

    /// End a trace span
    pub async fn end_span(&self, span_id: &str) -> Option<TraceSpan> {
        let mut traces = self.traces.write().await;
        traces.remove(span_id).map(|mut span| {
            span.end_time = Some(std::time::Instant::now());
            span
        })
    }

    /// Get active traces
    pub async fn get_active_traces(&self) -> Vec<TraceSpan> {
        let traces = self.traces.read().await;
        traces.values().cloned().collect()
    }

    /// Get exporters
    pub fn exporters(&self) -> &[TraceExporter] {
        &self.exporters
    }
}
