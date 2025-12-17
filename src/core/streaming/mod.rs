//! Streaming response handling for AI providers
//!
//! This module provides Server-Sent Events (SSE) streaming support for real-time AI responses.

use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use actix_web::{HttpResponse, web};
use crate::utils::error::Result;
use futures::stream::Stream;

// Module declarations
mod types;
mod handler;
pub mod providers;
pub mod utils;

#[cfg(test)]
mod tests;

// Re-export types for backward compatibility
pub use types::{
    ChatCompletionChunk,
    ChatCompletionChunkChoice,
    ChatCompletionDelta,
    Event,
    FunctionCallDelta,
    ToolCallDelta,
};

// Re-export handler
pub use handler::StreamingHandler;

// Re-export provider implementations
pub use providers::{
    AnthropicStreaming,
    GenericStreaming,
    OpenAIStreaming,
};

// Re-export utils
pub use utils::{
    create_error_event,
    create_heartbeat_event,
    is_done_line,
    parse_sse_line,
};

/// Create a Server-Sent Events response for Actix-web
pub fn create_sse_response<S>(stream: S) -> HttpResponse
where
    S: Stream<Item = Result<web::Bytes>> + Send + 'static,
{
    HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}
