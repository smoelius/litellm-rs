//! Utility functions for streaming

use super::types::Event;
use serde_json::json;

/// Parse SSE data line
pub fn parse_sse_line(line: &str) -> Option<String> {
    line.strip_prefix("data: ")
        .map(|stripped| stripped.to_string())
}

/// Check if SSE line indicates end of stream
pub fn is_done_line(line: &str) -> bool {
    line.trim() == "data: [DONE]" || line.trim() == "[DONE]"
}

/// Create an error event for SSE
pub fn create_error_event(error: &str) -> Event {
    Event::default()
        .event("error")
        .data(&json!({"error": error}).to_string())
}

/// Create a heartbeat event for SSE
pub fn create_heartbeat_event() -> Event {
    Event::default().event("heartbeat").data("ping")
}
