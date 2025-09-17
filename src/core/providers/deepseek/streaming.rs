//! DeepSeek Streaming Support
//!
//! Implementation

use futures::Stream;
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::ChatChunk;

/// Response
pub struct DeepSeekStreamParser {
    buffer: String,
}

impl Default for DeepSeekStreamParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DeepSeekStreamParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Handle
    pub fn process_chunk(&mut self, chunk: &[u8]) -> Result<Vec<ChatChunk>, ProviderError> {
        let chunk_str = std::str::from_utf8(chunk).map_err(|e| {
            ProviderError::response_parsing("deepseek", format!("Invalid UTF-8: {}", e))
        })?;

        self.buffer.push_str(chunk_str);
        let mut results = Vec::new();

        // Handle
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].trim().to_string();
            self.buffer.drain(..=newline_pos);

            if line.is_empty() {
                continue;
            }

            // Handle
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" {
                    break;
                }

                match self.parse_sse_data(data) {
                    Ok(Some(chunk)) => results.push(chunk),
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(results)
    }

    /// parseSSE数据为ChatChunk
    fn parse_sse_data(&self, data: &str) -> Result<Option<ChatChunk>, ProviderError> {
        let json: Value = serde_json::from_str(data).map_err(|e| {
            ProviderError::response_parsing("deepseek", format!("Invalid JSON: {}", e))
        })?;

        // Response
        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                if let Some(delta) = choice.get("delta") {
                    return Ok(Some(self.create_chat_chunk(&json, delta)?));
                }
            }
        }

        Ok(None)
    }

    /// Create
    fn create_chat_chunk(
        &self,
        response: &Value,
        delta: &Value,
    ) -> Result<ChatChunk, ProviderError> {
        use crate::core::types::requests::MessageRole;
        use crate::core::types::responses::{ChatDelta, ChatStreamChoice, FinishReason};
        use std::time::{SystemTime, UNIX_EPOCH};

        let content = delta
            .get("content")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        let role = delta
            .get("role")
            .and_then(|r| r.as_str())
            .and_then(|r| match r {
                "assistant" => Some(MessageRole::Assistant),
                "user" => Some(MessageRole::User),
                "system" => Some(MessageRole::System),
                _ => None,
            });

        let finish_reason = response
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("finish_reason"))
            .and_then(|fr| fr.as_str())
            .and_then(|s| match s {
                "stop" => Some(FinishReason::Stop),
                "length" => Some(FinishReason::Length),
                "content_filter" => Some(FinishReason::ContentFilter),
                "tool_calls" => Some(FinishReason::ToolCalls),
                _ => None,
            });

        let choice = ChatStreamChoice {
            index: 0,
            delta: ChatDelta {
                role,
                content,
                function_call: None,
                tool_calls: None,
            },
            finish_reason,
            logprobs: None,
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(ChatChunk {
            id: response
                .get("id")
                .and_then(|id| id.as_str())
                .unwrap_or("unknown")
                .to_string(),
            object: "chat.completion.chunk".to_string(),
            created: timestamp as i64,
            model: response
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("deepseek")
                .to_string(),
            choices: vec![choice],
            usage: None,
            system_fingerprint: None,
        })
    }
}

/// Response
pub struct DeepSeekStream {
    inner: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    parser: DeepSeekStreamParser,
}

impl DeepSeekStream {
    pub fn new(
        stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    ) -> Self {
        Self {
            inner: stream,
            parser: DeepSeekStreamParser::new(),
        }
    }
}

impl Stream for DeepSeekStream {
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                match self.parser.process_chunk(&chunk) {
                    Ok(chunks) => {
                        if chunks.is_empty() {
                            // 没有完整的chunk，继续etc待更多数据
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        } else {
                            // Handle
                            Poll::Ready(Some(Ok(chunks.into_iter().next().unwrap())))
                        }
                    }
                    Err(e) => Poll::Ready(Some(Err(e))),
                }
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(ProviderError::network("deepseek", e.to_string()))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_parsing() {
        let mut parser = DeepSeekStreamParser::new();

        let test_data = r#"data: {"id":"chatcmpl-test","object":"chat.completion.chunk","created":1640995200,"model":"deepseek-chat","choices":[{"index":0,"delta":{"role":"assistant","content":"Hello"},"finish_reason":null}]}
"#;

        let result = parser.process_chunk(test_data.as_bytes()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].choices[0].delta.content,
            Some("Hello".to_string())
        );
    }

    #[test]
    fn test_done_message() {
        let mut parser = DeepSeekStreamParser::new();

        let test_data = "data: [DONE]\n";
        let result = parser.process_chunk(test_data.as_bytes()).unwrap();
        assert_eq!(result.len(), 0);
    }
}
