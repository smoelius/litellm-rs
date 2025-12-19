//! Unified SSE (Server-Sent Events) Parser
//!
//! A centralized SSE parsing system that eliminates code duplication across providers.
//! All providers can use this parser and only need to implement the transformation logic.

use bytes::Bytes;
use futures::Stream;
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatStreamChoice, FinishReason};

/// SSE Event Types
#[derive(Debug, Clone, PartialEq)]
pub enum SSEEventType {
    Data,
    Event,
    Id,
    Retry,
    Comment,
}

/// Parsed SSE Event
#[derive(Debug, Clone)]
pub struct SSEEvent {
    pub event_type: Option<String>,
    pub data: String,
    pub id: Option<String>,
    pub retry: Option<u64>,
}

impl SSEEvent {
    /// Parse SSE event from a line
    pub fn from_line(line: &str) -> Option<Self> {
        if line.is_empty() || line.starts_with(':') {
            return None;
        }

        if let Some(colon_pos) = line.find(':') {
            let field = &line[..colon_pos];
            let value = line[colon_pos + 1..].trim_start();

            match field {
                "data" => Some(SSEEvent {
                    event_type: None,
                    data: value.to_string(),
                    id: None,
                    retry: None,
                }),
                "event" => Some(SSEEvent {
                    event_type: Some(value.to_string()),
                    data: String::new(),
                    id: None,
                    retry: None,
                }),
                "id" => Some(SSEEvent {
                    event_type: None,
                    data: String::new(),
                    id: Some(value.to_string()),
                    retry: None,
                }),
                "retry" => {
                    if let Ok(retry_ms) = value.parse::<u64>() {
                        Some(SSEEvent {
                            event_type: None,
                            data: String::new(),
                            id: None,
                            retry: Some(retry_ms),
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Trait for provider-specific SSE transformation
pub trait SSETransformer: Send + Sync {
    /// Provider name for error reporting
    fn provider_name(&self) -> &'static str;

    /// Check if this is the end-of-stream marker
    fn is_end_marker(&self, data: &str) -> bool {
        data.trim() == "[DONE]"
    }

    /// Transform raw SSE data into ChatChunk
    fn transform_chunk(&self, data: &str) -> Result<Option<ChatChunk>, ProviderError>;

    /// Parse finish reason from string (provider-specific)
    fn parse_finish_reason(&self, reason: &str) -> Option<FinishReason> {
        match reason {
            "stop" => Some(FinishReason::Stop),
            "length" | "max_tokens" => Some(FinishReason::Length),
            "tool_calls" | "function_call" => Some(FinishReason::ToolCalls),
            "content_filter" => Some(FinishReason::ContentFilter),
            _ => None,
        }
    }
}

/// Unified SSE Parser
pub struct UnifiedSSEParser<T: SSETransformer> {
    transformer: T,
    buffer: String,
    current_event: Option<SSEEvent>,
}

impl<T: SSETransformer> UnifiedSSEParser<T> {
    /// Create new SSE parser with a transformer
    pub fn new(transformer: T) -> Self {
        Self {
            transformer,
            buffer: String::new(),
            current_event: None,
        }
    }

    /// Process raw bytes into SSE events
    pub fn process_bytes(&mut self, bytes: &[u8]) -> Result<Vec<ChatChunk>, ProviderError> {
        let text = String::from_utf8_lossy(bytes);
        self.buffer.push_str(&text);

        let mut chunks = Vec::new();
        let mut lines: Vec<String> = self.buffer.lines().map(|s| s.to_string()).collect();

        // Keep incomplete line in buffer
        if !self.buffer.ends_with('\n') {
            if let Some(last_line) = lines.pop() {
                self.buffer = last_line;
            } else {
                self.buffer.clear();
            }
        } else {
            self.buffer.clear();
        }

        // Process complete lines
        for line in lines {
            if let Some(chunk) = self.process_line(&line)? {
                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    /// Process a single SSE line
    fn process_line(&mut self, line: &str) -> Result<Option<ChatChunk>, ProviderError> {
        // Empty line signals end of event
        if line.is_empty() {
            if let Some(event) = self.current_event.take() {
                return self.process_event(event);
            }
            return Ok(None);
        }

        // Parse SSE field
        if let Some(event) = SSEEvent::from_line(line) {
            // For data fields, accumulate or merge
            if !event.data.is_empty() {
                if let Some(ref mut current) = self.current_event {
                    // Append to existing data
                    if !current.data.is_empty() {
                        current.data.push('\n');
                    }
                    current.data.push_str(&event.data);
                } else {
                    self.current_event = Some(event);
                }
            } else if event.event_type.is_some() || event.id.is_some() || event.retry.is_some() {
                // Merge other fields
                if let Some(ref mut current) = self.current_event {
                    if event.event_type.is_some() {
                        current.event_type = event.event_type;
                    }
                    if event.id.is_some() {
                        current.id = event.id;
                    }
                    if event.retry.is_some() {
                        current.retry = event.retry;
                    }
                } else {
                    self.current_event = Some(event);
                }
            }
        }

        Ok(None)
    }

    /// Process a complete SSE event
    fn process_event(&self, event: SSEEvent) -> Result<Option<ChatChunk>, ProviderError> {
        // Skip empty events
        if event.data.is_empty() {
            return Ok(None);
        }

        // Check for end marker
        if self.transformer.is_end_marker(&event.data) {
            return Ok(None);
        }

        // Transform to ChatChunk
        self.transformer.transform_chunk(&event.data)
    }
}

/// Streaming wrapper that uses UnifiedSSEParser
pub struct UnifiedSSEStream<S, T>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin,
    T: SSETransformer + Clone,
{
    inner: S,
    parser: UnifiedSSEParser<T>,
    chunk_buffer: Vec<ChatChunk>,
}

impl<S, T> UnifiedSSEStream<S, T>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin,
    T: SSETransformer + Clone,
{
    pub fn new(stream: S, transformer: T) -> Self {
        Self {
            inner: stream,
            parser: UnifiedSSEParser::new(transformer),
            chunk_buffer: Vec::new(),
        }
    }
}

impl<S, T> Stream for UnifiedSSEStream<S, T>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin,
    T: SSETransformer + Clone + Unpin,
{
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // Return buffered chunks first
        if !this.chunk_buffer.is_empty() {
            return Poll::Ready(Some(Ok(this.chunk_buffer.remove(0))));
        }

        // Poll inner stream for more data
        match Pin::new(&mut this.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                match this.parser.process_bytes(&bytes) {
                    Ok(chunks) => {
                        if chunks.is_empty() {
                            // No chunks yet, poll again
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        } else {
                            // Buffer chunks and return first one
                            this.chunk_buffer = chunks;
                            if !this.chunk_buffer.is_empty() {
                                Poll::Ready(Some(Ok(this.chunk_buffer.remove(0))))
                            } else {
                                cx.waker().wake_by_ref();
                                Poll::Pending
                            }
                        }
                    }
                    Err(e) => Poll::Ready(Some(Err(e))),
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(ProviderError::network(
                this.parser.transformer.provider_name(),
                format!("Stream error: {}", e),
            )))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// OpenAI-compatible SSE Transformer (can be reused by many providers)
#[derive(Debug, Clone)]
pub struct OpenAICompatibleTransformer {
    provider: &'static str,
}

impl OpenAICompatibleTransformer {
    pub fn new(provider: &'static str) -> Self {
        Self { provider }
    }
}

impl SSETransformer for OpenAICompatibleTransformer {
    fn provider_name(&self) -> &'static str {
        self.provider
    }

    fn transform_chunk(&self, data: &str) -> Result<Option<ChatChunk>, ProviderError> {
        // Parse JSON
        let json_value: Value = serde_json::from_str(data).map_err(|e| {
            ProviderError::response_parsing(
                self.provider,
                format!("Failed to parse SSE JSON: {}", e),
            )
        })?;

        // Extract fields
        let id = json_value
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("stream-chunk")
            .to_string();

        let model = json_value
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let created = json_value
            .get("created")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp()) as u64;

        // Parse choices
        let choices = json_value
            .get("choices")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ProviderError::response_parsing(
                    self.provider,
                    "No choices in SSE chunk".to_string(),
                )
            })?;

        let mut stream_choices = Vec::new();

        for (index, choice) in choices.iter().enumerate() {
            let delta = choice.get("delta").ok_or_else(|| {
                ProviderError::response_parsing(self.provider, "No delta in choice".to_string())
            })?;

            let delta_obj: ChatDelta = serde_json::from_value(delta.clone()).map_err(|e| {
                ProviderError::response_parsing(
                    self.provider,
                    format!("Failed to parse delta: {}", e),
                )
            })?;

            let finish_reason = choice
                .get("finish_reason")
                .and_then(|v| v.as_str())
                .and_then(|s| self.parse_finish_reason(s));

            let logprobs = choice
                .get("logprobs")
                .and_then(|v| serde_json::from_value(v.clone()).ok());

            stream_choices.push(ChatStreamChoice {
                index: index as u32,
                delta: delta_obj,
                finish_reason,
                logprobs,
            });
        }

        // Parse usage (optional)
        let usage = json_value
            .get("usage")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        Ok(Some(ChatChunk {
            id,
            object: "chat.completion.chunk".to_string(),
            created: created as i64,
            model,
            choices: stream_choices,
            usage,
            system_fingerprint: json_value
                .get("system_fingerprint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }))
    }
}

/// Provider-specific transformers can extend the base
#[derive(Debug, Clone)]
pub struct AnthropicTransformer;

impl SSETransformer for AnthropicTransformer {
    fn provider_name(&self) -> &'static str {
        "anthropic"
    }

    fn is_end_marker(&self, data: &str) -> bool {
        // Anthropic uses different end markers
        data.contains("event: message_stop") || data.trim() == "data: {\"type\": \"message_stop\"}"
    }

    fn transform_chunk(&self, data: &str) -> Result<Option<ChatChunk>, ProviderError> {
        // Anthropic-specific transformation logic
        let json_value: Value = serde_json::from_str(data).map_err(|e| {
            ProviderError::response_parsing(
                "anthropic",
                format!("Failed to parse Anthropic SSE: {}", e),
            )
        })?;

        // Handle Anthropic's event types
        let event_type = json_value
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match event_type {
            "content_block_delta" => {
                // Extract text delta
                let text = json_value
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                let delta = ChatDelta {
                    role: None,
                    content: Some(text.to_string()),
                    thinking: None,
                    tool_calls: None,
                    thinking: None,
                function_call: None,
                thinking: None,
                };

                let choice = ChatStreamChoice {
                    index: 0,
                    delta,
                    finish_reason: None,
                    logprobs: None,
                };

                Ok(Some(ChatChunk {
                    id: "anthropic-stream".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: "claude".to_string(),
                    choices: vec![choice],
                    usage: None,
                    system_fingerprint: None,
                }))
            }
            "message_stop" => Ok(None),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_parsing() {
        let event = SSEEvent::from_line("data: test data").unwrap();
        assert_eq!(event.data, "test data");

        let event = SSEEvent::from_line("event: message").unwrap();
        assert_eq!(event.event_type, Some("message".to_string()));

        let event = SSEEvent::from_line("id: 123").unwrap();
        assert_eq!(event.id, Some("123".to_string()));

        let event = SSEEvent::from_line("retry: 5000").unwrap();
        assert_eq!(event.retry, Some(5000));

        // Comments should be ignored
        assert!(SSEEvent::from_line(": comment").is_none());
    }

    #[test]
    fn test_openai_transformer() {
        let transformer = OpenAICompatibleTransformer::new("test");

        // Test end marker
        assert!(transformer.is_end_marker("[DONE]"));
        assert!(!transformer.is_end_marker("data: {\"test\": 1}"));

        // Test JSON transformation
        let json_data = r#"{
            "id": "test-id",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {"content": "Hello"},
                "finish_reason": null
            }]
        }"#;

        let result = transformer.transform_chunk(json_data).unwrap().unwrap();
        assert_eq!(result.id, "test-id");
        assert_eq!(result.model, "gpt-4");
        assert_eq!(result.choices[0].delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_sse_parser_multiline() {
        let transformer = OpenAICompatibleTransformer::new("test");
        let mut parser = UnifiedSSEParser::new(transformer);

        // Simulate receiving data in chunks
        let chunk1 = b"data: {\"id\": \"test\", ";
        let chunk2 = b"\"choices\": [{\"delta\": {\"content\": \"Hi\"}, \"index\": 0}], ";
        let chunk3 = b"\"model\": \"gpt-4\", \"created\": 123}\n\n";

        let results1 = parser.process_bytes(chunk1).unwrap();
        assert!(results1.is_empty()); // Not complete yet

        let results2 = parser.process_bytes(chunk2).unwrap();
        assert!(results2.is_empty()); // Still not complete

        let results3 = parser.process_bytes(chunk3).unwrap();
        assert_eq!(results3.len(), 1); // Now we have a complete event
        assert_eq!(results3[0].choices[0].delta.content, Some("Hi".to_string()));
    }
}
