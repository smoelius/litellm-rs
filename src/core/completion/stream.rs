//! Completion streaming types

use crate::core::streaming::types::ChatCompletionChunk;
use crate::core::types::FinishReason;
use futures::stream::BoxStream;

/// Streaming completion response
pub type CompletionStream =
    BoxStream<'static, Result<CompletionChunk, crate::utils::error::GatewayError>>;

/// Chunk in a streaming completion response
#[derive(Debug, Clone)]
pub struct CompletionChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

/// Choice in a streaming chunk
#[derive(Debug, Clone)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: StreamDelta,
    pub finish_reason: Option<FinishReason>,
}

/// Delta content in streaming response
#[derive(Debug, Clone, Default)]
pub struct StreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<crate::core::completion::types::ToolCall>>,
}

/// Convert internal stream chunk to completion chunk
pub fn convert_stream_chunk(chunk: ChatCompletionChunk) -> CompletionChunk {
    CompletionChunk {
        id: chunk.id,
        object: chunk.object,
        created: chunk.created as i64,
        model: chunk.model,
        choices: chunk
            .choices
            .into_iter()
            .map(|c| StreamChoice {
                index: c.index,
                delta: StreamDelta {
                    role: c.delta.role.map(|r| r.to_string()),
                    content: c.delta.content,
                    tool_calls: None,
                },
                finish_reason: c.finish_reason.and_then(|s| parse_finish_reason(&s)),
            })
            .collect(),
    }
}

/// Parse finish reason string to FinishReason enum
fn parse_finish_reason(s: &str) -> Option<FinishReason> {
    match s.to_lowercase().as_str() {
        "stop" => Some(FinishReason::Stop),
        "length" => Some(FinishReason::Length),
        "tool_calls" | "function_call" => Some(FinishReason::ToolCalls),
        "content_filter" => Some(FinishReason::ContentFilter),
        _ => None,
    }
}
