//! OpenAI Streaming Response Handler
//!
//! Server-Sent Events (SSE) stream processing for OpenAI API

use futures::Stream;
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::warn;

use super::error::OpenAIError;
use crate::core::types::responses::ChatChunk;

/// OpenAI streaming response handler
pub struct OpenAIStream {
    inner: Pin<
        Box<
            dyn Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>>
                + Send,
        >,
    >,
    parser: OpenAIStreamParser,
}

impl OpenAIStream {
    /// Create new OpenAI stream
    pub fn new(
        stream: Pin<
            Box<
                dyn Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>>
                    + Send,
            >,
        >,
    ) -> Self {
        Self {
            inner: stream,
            parser: OpenAIStreamParser::new(),
        }
    }
}

impl Stream for OpenAIStream {
    type Item = Result<ChatChunk, OpenAIError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let chunk_str = match String::from_utf8(bytes.to_vec()) {
                    Ok(s) => s,
                    Err(e) => {
                        return Poll::Ready(Some(Err(OpenAIError::Other {
                            provider: "openai",
                            message: format!("Invalid UTF-8 in stream chunk: {}", e),
                        })));
                    }
                };

                match self.parser.parse_chunk(&chunk_str) {
                    Ok(Some(chunk)) => Poll::Ready(Some(Ok(chunk))),
                    Ok(None) => {
                        // No complete chunk yet, continue polling
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(e) => Poll::Ready(Some(Err(e))),
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(OpenAIError::Other {
                provider: "openai",
                message: format!("Stream error: {}", e),
            }))),
            Poll::Ready(None) => {
                // Stream ended
                if self.parser.is_finished() {
                    Poll::Ready(None)
                } else {
                    // Process any remaining buffered data
                    match self.parser.finalize() {
                        Ok(Some(chunk)) => Poll::Ready(Some(Ok(chunk))),
                        Ok(None) => Poll::Ready(None),
                        Err(e) => Poll::Ready(Some(Err(e))),
                    }
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// OpenAI Server-Sent Events parser
pub struct OpenAIStreamParser {
    buffer: String,
    finished: bool,
}

impl OpenAIStreamParser {
    /// Create new parser
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            finished: false,
        }
    }

    /// Parse incoming chunk data
    pub fn parse_chunk(&mut self, data: &str) -> Result<Option<ChatChunk>, OpenAIError> {
        if self.finished {
            return Ok(None);
        }

        // Add new data to buffer
        self.buffer.push_str(data);

        // Process complete lines
        while let Some(line_end) = self.buffer.find('\n') {
            let line = self.buffer[..line_end].trim_end_matches('\r').to_string();
            self.buffer.drain(..=line_end);

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(':') {
                continue;
            }

            // Parse SSE data lines
            if let Some(data_content) = line.strip_prefix("data: ") {
                if data_content.trim() == "[DONE]" {
                    self.finished = true;
                    return Ok(None);
                }

                // Parse JSON chunk
                match self.parse_json_chunk(data_content) {
                    Ok(chunk) => return Ok(Some(chunk)),
                    Err(e) => {
                        // Log parsing error but continue processing
                        warn!(
                            provider = "openai",
                            error = %e,
                            data = %data_content,
                            "Failed to parse streaming chunk, continuing with next chunk"
                        );
                        continue;
                    }
                }
            }
        }

        // No complete chunk available yet
        Ok(None)
    }

    /// Parse JSON chunk data
    fn parse_json_chunk(&self, data: &str) -> Result<ChatChunk, OpenAIError> {
        let json_value: Value =
            serde_json::from_str(data).map_err(|e| OpenAIError::ResponseParsing {
                provider: "openai",
                message: format!("Invalid JSON in stream: {}", e),
            })?;

        // Transform OpenAI streaming response to standard format
        self.transform_streaming_chunk(json_value)
    }

    /// Transform OpenAI streaming chunk to standard ChatChunk format
    fn transform_streaming_chunk(&self, chunk: Value) -> Result<ChatChunk, OpenAIError> {
        let id = chunk
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let object = chunk
            .get("object")
            .and_then(|v| v.as_str())
            .unwrap_or("chat.completion.chunk")
            .to_string();

        let created = chunk.get("created").and_then(|v| v.as_i64()).unwrap_or(0);

        let model = chunk
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let choices = chunk
            .get("choices")
            .and_then(|v| v.as_array())
            .map(|choices| {
                choices
                    .iter()
                    .filter_map(|choice| self.transform_choice(choice).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(ChatChunk {
            id,
            object,
            created,
            model,
            choices,
            system_fingerprint: None, // Not always available in streaming
            usage: None,              // Usage typically provided in last chunk
        })
    }

    /// Transform individual choice in streaming response
    fn transform_choice(
        &self,
        choice: &Value,
    ) -> Result<crate::core::types::responses::ChatStreamChoice, OpenAIError> {
        use crate::core::types::responses::{ChatDelta, ChatStreamChoice};

        let index = choice.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let delta = choice
            .get("delta")
            .map(|delta| {
                let role = delta
                    .get("role")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "system" => Some(crate::core::types::requests::MessageRole::System),
                        "user" => Some(crate::core::types::requests::MessageRole::User),
                        "assistant" => Some(crate::core::types::requests::MessageRole::Assistant),
                        "tool" => Some(crate::core::types::requests::MessageRole::Tool),
                        "function" => Some(crate::core::types::requests::MessageRole::Function),
                        _ => None,
                    });

                let content = delta
                    .get("content")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // TODO: Handle tool_calls properly
                let tool_calls = None;
                let function_call = None;

                ChatDelta {
                    role,
                    content,
                    tool_calls,
                    function_call,
                }
            })
            .unwrap_or(ChatDelta {
                role: None,
                content: None,
                tool_calls: None,
                function_call: None,
            });

        let finish_reason = choice
            .get("finish_reason")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "stop" => Some(crate::core::types::responses::FinishReason::Stop),
                "length" => Some(crate::core::types::responses::FinishReason::Length),
                "content_filter" => {
                    Some(crate::core::types::responses::FinishReason::ContentFilter)
                }
                "function_call" => Some(crate::core::types::responses::FinishReason::FunctionCall),
                "tool_calls" => Some(crate::core::types::responses::FinishReason::ToolCalls),
                _ => None,
            });

        Ok(ChatStreamChoice {
            index,
            delta,
            finish_reason,
            logprobs: None, // OpenAI may include this in future
        })
    }

    /// Check if parsing is finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Finalize parsing and return any remaining chunk
    pub fn finalize(&mut self) -> Result<Option<ChatChunk>, OpenAIError> {
        if !self.buffer.is_empty() && !self.finished {
            // Try to parse any remaining data
            let remaining = self.buffer.trim().to_string();
            if !remaining.is_empty() {
                if let Some(data_content) = remaining.strip_prefix("data: ") {
                    if data_content.trim() != "[DONE]" {
                        self.buffer.clear();
                        return self.parse_json_chunk(data_content).map(Some);
                    }
                }
            }
        }
        Ok(None)
    }
}

impl Default for OpenAIStreamParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{StreamExt, stream};

    #[test]
    fn test_parser_creation() {
        let parser = OpenAIStreamParser::new();
        assert!(!parser.is_finished());
    }

    #[test]
    fn test_sse_parsing() {
        let mut parser = OpenAIStreamParser::new();

        let test_data = r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

"#;

        let result = parser.parse_chunk(test_data);
        assert!(result.is_ok());

        if let Ok(Some(chunk)) = result {
            assert_eq!(chunk.id, "chatcmpl-123");
            assert_eq!(chunk.model, "gpt-4");
            assert_eq!(chunk.choices.len(), 1);
            assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
        } else {
            panic!("Expected successful parsing");
        }
    }

    #[test]
    fn test_done_message() {
        let mut parser = OpenAIStreamParser::new();

        let done_data = "data: [DONE]\n\n";
        let result = parser.parse_chunk(done_data);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert!(parser.is_finished());
    }

    #[test]
    fn test_invalid_json() {
        let mut parser = OpenAIStreamParser::new();

        let invalid_data = "data: {invalid json}\n\n";
        let result = parser.parse_chunk(invalid_data);

        // Should handle invalid JSON gracefully
        assert!(result.is_ok());
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = OpenAIStreamParser::new();

        // Send data in parts
        let part1 = "data: {\"id\":\"test\",\"object\":\"chat.completion.chunk\"";
        let part2 = ",\"created\":123,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\n";

        // First part should not produce a chunk
        let result1 = parser.parse_chunk(part1);
        assert!(result1.is_ok());
        assert!(result1.unwrap().is_none());

        // Second part should complete the chunk
        let result2 = parser.parse_chunk(part2);
        assert!(result2.is_ok());

        if let Ok(Some(chunk)) = result2 {
            assert_eq!(chunk.id, "test");
            assert_eq!(chunk.choices[0].delta.content, Some("Hi".to_string()));
        } else {
            panic!("Expected successful parsing of complete chunk");
        }
    }

    #[tokio::test]
    async fn test_stream_wrapper() {
        use bytes::Bytes;

        // Create a mock byte stream
        let data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(data)
            .map(|item| item.map_err(|e: Box<dyn std::error::Error + Send + Sync>| e));

        let mut openai_stream = OpenAIStream::new(Box::pin(mock_stream));

        // Should produce one chunk
        let first_chunk = openai_stream.next().await;
        assert!(first_chunk.is_some());

        if let Some(Ok(chunk)) = first_chunk {
            assert_eq!(chunk.id, "test");
            assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
        }

        // Should end after [DONE]
        let second_chunk = openai_stream.next().await;
        assert!(second_chunk.is_none());
    }
}
