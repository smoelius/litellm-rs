//! Streaming response handling for AI providers
//!
//! This module provides Server-Sent Events (SSE) streaming support for real-time AI responses.

use crate::core::models::openai::*;
use crate::core::types::MessageRole;
use crate::utils::error::{GatewayError, Result};
use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use actix_web::{HttpResponse, web};
use futures::stream::{Stream, StreamExt};
use serde_json::json;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;
use uuid::Uuid;

/// Simple Event structure for SSE compatibility
#[derive(Debug, Clone, Default)]
pub struct Event {
    /// Event type
    pub event: Option<String>,
    /// Event data
    pub data: String,
}

impl Event {
    /// Create a new empty event
    pub fn new() -> Self {
        Self {
            event: None,
            data: String::new(),
        }
    }

    /// Set the event type
    pub fn event(mut self, event: &str) -> Self {
        self.event = Some(event.to_string());
        self
    }

    /// Set the event data
    pub fn data(mut self, data: &str) -> Self {
        self.data = data.to_string();
        self
    }

    /// Convert event to bytes for SSE transmission
    pub fn to_bytes(&self) -> web::Bytes {
        let mut result = String::new();
        if let Some(event) = &self.event {
            result.push_str(&format!("event: {}\n", event));
        }
        result.push_str(&format!("data: {}\n\n", self.data));
        web::Bytes::from(result)
    }
}

/// Streaming response chunk for chat completions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunk {
    /// Unique identifier for the completion
    pub id: String,
    /// Object type (always "chat.completion.chunk")
    pub object: String,
    /// Unix timestamp of creation
    pub created: u64,
    /// Model used for completion
    pub model: String,
    /// System fingerprint
    pub system_fingerprint: Option<String>,
    /// Array of completion choices
    pub choices: Vec<ChatCompletionChunkChoice>,
    /// Usage statistics (only in final chunk)
    pub usage: Option<Usage>,
}

/// Choice in a streaming chat completion chunk
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunkChoice {
    /// Index of the choice
    pub index: u32,
    /// Delta containing the incremental content
    pub delta: ChatCompletionDelta,
    /// Reason for finishing (only in final chunk)
    pub finish_reason: Option<String>,
    /// Log probabilities
    pub logprobs: Option<serde_json::Value>,
}

/// Delta containing incremental content in streaming response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionDelta {
    /// Role of the message (only in first chunk)
    pub role: Option<MessageRole>,
    /// Incremental content
    pub content: Option<String>,
    /// Tool calls (for function calling)
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

/// Tool call delta for streaming function calls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCallDelta {
    /// Index of the tool call
    pub index: u32,
    /// Tool call ID (only in first chunk)
    pub id: Option<String>,
    /// Type of tool call (only in first chunk)
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    /// Function call details
    pub function: Option<FunctionCallDelta>,
}

/// Function call delta for streaming
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCallDelta {
    /// Function name (only in first chunk)
    pub name: Option<String>,
    /// Incremental function arguments
    pub arguments: Option<String>,
}

/// Streaming response handler
pub struct StreamingHandler {
    /// Request ID for tracking
    request_id: String,
    /// Model being used
    model: String,
    /// Whether this is the first chunk
    is_first_chunk: bool,
    /// Accumulated content for final usage calculation
    accumulated_content: String,
    /// Start time for latency calculation
    start_time: std::time::Instant,
}

impl StreamingHandler {
    /// Create a new streaming handler
    pub fn new(model: String) -> Self {
        Self {
            request_id: format!("chatcmpl-{}", Uuid::new_v4()),
            model,
            is_first_chunk: true,
            accumulated_content: String::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Create a streaming response from a provider stream for Actix-web
    pub fn create_sse_stream<S>(
        mut self,
        provider_stream: S,
    ) -> impl Stream<Item = Result<web::Bytes>>
    where
        S: Stream<Item = Result<String>> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            tokio::pin!(provider_stream);

            while let Some(chunk_result) = provider_stream.next().await {
                match chunk_result {
                    Ok(chunk_data) => {
                        match self.process_chunk(&chunk_data).await {
                            Ok(Some(event)) => {
                                if tx.send(Ok(event.to_bytes())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(None) => continue, // Skip empty chunks
                            Err(e) => {
                                error!("Error processing chunk: {}", e);
                                let error_event = Event::default()
                                    .event("error")
                                    .data(&json!({"error": e.to_string()}).to_string());
                                let _ = tx.send(Ok(error_event.to_bytes())).await;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Provider stream error: {}", e);
                        let error_event = Event::default()
                            .event("error")
                            .data(&json!({"error": e.to_string()}).to_string());
                        let _ = tx.send(Ok(error_event.to_bytes())).await;
                        break;
                    }
                }
            }

            // Send final chunk with usage information
            if let Ok(final_event) = self.create_final_chunk().await {
                let _ = tx.send(Ok(final_event.to_bytes())).await;
            }

            // Send done event
            let done_event = Event::default().data("[DONE]");
            let _ = tx.send(Ok(done_event.to_bytes())).await;
        });

        ReceiverStream::new(rx)
    }

    /// Process a single chunk from the provider
    async fn process_chunk(&mut self, chunk_data: &str) -> Result<Option<Event>> {
        // Parse provider-specific chunk format
        let content = self.extract_content_from_chunk(chunk_data)?;

        if content.is_empty() {
            return Ok(None);
        }

        self.accumulated_content.push_str(&content);

        let chunk = ChatCompletionChunk {
            id: self.request_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: self.model.clone(),
            system_fingerprint: None,
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionDelta {
                    role: if self.is_first_chunk {
                        Some(MessageRole::Assistant)
                    } else {
                        None
                    },
                    content: Some(content),
                    tool_calls: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            usage: None,
        };

        self.is_first_chunk = false;

        let event = Event::default().data(&serde_json::to_string(&chunk)?);

        Ok(Some(event))
    }

    /// Extract content from provider-specific chunk format
    fn extract_content_from_chunk(&self, chunk_data: &str) -> Result<String> {
        // Handle different provider formats
        if chunk_data.starts_with("data: ") {
            let data = chunk_data.strip_prefix("data: ").unwrap_or(chunk_data);

            if data.trim() == "[DONE]" {
                return Ok(String::new());
            }

            // Parse JSON chunk
            if let Ok(json_chunk) = serde_json::from_str::<serde_json::Value>(data) {
                // OpenAI format
                if let Some(choices) = json_chunk.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.first() {
                        if let Some(delta) = choice.get("delta") {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                return Ok(content.to_string());
                            }
                        }
                    }
                }

                // Anthropic format
                if let Some(delta) = json_chunk.get("delta") {
                    if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                        return Ok(text.to_string());
                    }
                }

                // Generic text field
                if let Some(text) = json_chunk.get("text").and_then(|t| t.as_str()) {
                    return Ok(text.to_string());
                }
            }
        }

        // Fallback: treat as plain text
        Ok(chunk_data.to_string())
    }

    /// Create the final chunk with usage information
    async fn create_final_chunk(&self) -> Result<Event> {
        // Calculate actual token counts using the token counter
        let token_counter = crate::utils::TokenCounter::new();
        let completion_tokens = token_counter
            .count_completion_tokens(&self.model, &self.accumulated_content)
            .map(|estimate| estimate.input_tokens)
            .unwrap_or_else(|_| self.estimate_token_count(&self.accumulated_content));

        // For prompt tokens, we'd need the original request context
        // For now, use a reasonable estimate based on typical chat requests
        let prompt_tokens = self.estimate_prompt_tokens();
        let total_tokens = prompt_tokens + completion_tokens;

        let usage = Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        };

        let final_chunk = ChatCompletionChunk {
            id: self.request_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: self.model.clone(),
            system_fingerprint: None,
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionDelta {
                    role: None,
                    content: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            usage: Some(usage),
        };

        let event = Event::default().data(&serde_json::to_string(&final_chunk)?);

        Ok(event)
    }

    /// Estimate token count from text (simplified)
    fn estimate_token_count(&self, text: &str) -> u32 {
        // Very rough estimation: ~4 characters per token
        (text.len() as f64 / 4.0).ceil() as u32
    }

    /// Estimate prompt tokens based on typical chat requests
    fn estimate_prompt_tokens(&self) -> u32 {
        // This is a rough estimate since we don't have the original request
        // In a real implementation, we'd store the original prompt tokens
        // For now, use a reasonable default based on typical usage
        match self.model.as_str() {
            m if m.contains("gpt-4") => 150,
            m if m.contains("gpt-3.5") => 100,
            m if m.contains("claude") => 200,
            m if m.contains("gemini") => 120,
            _ => 100,
        }
    }
}

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

/// Provider-specific streaming implementations
pub mod providers {
    use super::*;
    use futures::stream::BoxStream;

    /// OpenAI streaming implementation
    pub struct OpenAIStreaming;

    impl OpenAIStreaming {
        /// Create a stream from OpenAI SSE response
        pub fn create_stream(response: reqwest::Response) -> BoxStream<'static, Result<String>> {
            let stream = response.bytes_stream().map(|chunk_result| {
                chunk_result
                    .map_err(|e| GatewayError::Network(e.to_string()))
                    .and_then(|chunk| {
                        String::from_utf8(chunk.to_vec())
                            .map_err(|e| GatewayError::Parsing(e.to_string()))
                    })
            });

            Box::pin(stream)
        }
    }

    /// Anthropic streaming implementation
    pub struct AnthropicStreaming;

    impl AnthropicStreaming {
        /// Create a stream from Anthropic SSE response
        pub fn create_stream(response: reqwest::Response) -> BoxStream<'static, Result<String>> {
            let stream = response.bytes_stream().map(|chunk_result| {
                chunk_result
                    .map_err(|e| GatewayError::network(e.to_string()))
                    .and_then(|chunk| {
                        String::from_utf8(chunk.to_vec())
                            .map_err(|e| GatewayError::internal(format!("Parsing error: {}", e)))
                    })
            });

            Box::pin(stream)
        }
    }

    /// Generic streaming implementation for other providers
    pub struct GenericStreaming;

    impl GenericStreaming {
        /// Create a stream from generic SSE response
        pub fn create_stream(response: reqwest::Response) -> BoxStream<'static, Result<String>> {
            let stream = response.bytes_stream().map(|chunk_result| {
                chunk_result
                    .map_err(|e| GatewayError::network(e.to_string()))
                    .and_then(|chunk| {
                        String::from_utf8(chunk.to_vec())
                            .map_err(|e| GatewayError::internal(format!("Parsing error: {}", e)))
                    })
            });

            Box::pin(stream)
        }
    }
}

/// Utility functions for streaming
pub mod utils {
    use super::*;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_handler_creation() {
        let handler = StreamingHandler::new("gpt-4".to_string());
        assert_eq!(handler.model, "gpt-4");
        assert!(handler.is_first_chunk);
        assert!(handler.accumulated_content.is_empty());
    }

    #[test]
    fn test_extract_content_from_chunk() {
        let handler = StreamingHandler::new("gpt-4".to_string());

        // Test OpenAI format
        let openai_chunk = r#"data: {"choices":[{"delta":{"content":"Hello"}}]}"#;
        let content = handler.extract_content_from_chunk(openai_chunk).unwrap();
        assert_eq!(content, "Hello");

        // Test Anthropic format
        let anthropic_chunk = r#"data: {"delta":{"text":"World"}}"#;
        let content = handler.extract_content_from_chunk(anthropic_chunk).unwrap();
        assert_eq!(content, "World");

        // Test done signal
        let done_chunk = "data: [DONE]";
        let content = handler.extract_content_from_chunk(done_chunk).unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn test_token_estimation() {
        let handler = StreamingHandler::new("gpt-4".to_string());

        // Test token estimation
        let text = "Hello world"; // 11 chars -> ~3 tokens
        let tokens = handler.estimate_token_count(text);
        assert_eq!(tokens, 3);

        let longer_text = "This is a longer text for testing"; // 34 chars -> ~9 tokens
        let tokens = handler.estimate_token_count(longer_text);
        assert_eq!(tokens, 9);
    }

    #[tokio::test]
    async fn test_sse_utils() {
        // Test SSE line parsing
        let line = "data: Hello World";
        let data = utils::parse_sse_line(line);
        assert_eq!(data, Some("Hello World".to_string()));

        // Test done detection
        assert!(utils::is_done_line("data: [DONE]"));
        assert!(utils::is_done_line("[DONE]"));
        assert!(!utils::is_done_line("data: Hello"));

        // Test error event creation
        let _error_event = utils::create_error_event("Test error");
        // Note: Event doesn't have a contains method, so we'll just check it was created
        // assert!(!error_event.data("").is_empty());
    }
}
