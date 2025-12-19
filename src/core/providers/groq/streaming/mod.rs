//! Streaming Module for Groq
//!
//! Handles streaming chat completions with support for fake streaming when
//! response_format is used (Groq limitation).

use super::error::GroqError;
use crate::core::types::requests::{MessageContent, MessageRole};
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatResponse, ChatStreamChoice};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Groq SSE stream implementation
pub struct GroqStream {
    inner: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buffer: String,
}

impl GroqStream {
    pub fn new(
        stream: impl Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
    ) -> Self {
        Self {
            inner: Box::pin(stream),
            buffer: String::new(),
        }
    }

    /// Parse SSE data into ChatChunk
    fn parse_sse_data(&self, data: &str) -> Option<ChatChunk> {
        // Skip empty data or [DONE] signal
        if data.is_empty() || data == "[DONE]" {
            return None;
        }

        // Parse JSON
        match serde_json::from_str::<ChatChunk>(data) {
            Ok(chunk) => Some(chunk),
            Err(e) => {
                tracing::warn!("Failed to parse SSE chunk: {}, data: {}", e, data);
                None
            }
        }
    }
}

impl Stream for GroqStream {
    type Item = Result<ChatChunk, GroqError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Check if we have a complete SSE message in the buffer
            if let Some(pos) = self.buffer.find("\n\n") {
                let message = self.buffer.drain(..pos + 2).collect::<String>();

                // Parse SSE message
                if let Some(data_line) = message.lines().find(|line| line.starts_with("data: ")) {
                    let data = &data_line[6..]; // Skip "data: " prefix
                    if let Some(chunk) = self.parse_sse_data(data) {
                        return Poll::Ready(Some(Ok(chunk)));
                    }
                }
                continue; // Try next message in buffer
            }

            // Need more data
            match self.inner.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    // Add new data to buffer
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        self.buffer.push_str(&text);
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(GroqError::StreamingError(e.to_string()))));
                }
                Poll::Ready(None) => {
                    // Stream ended
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Create a fake stream from a complete response
pub async fn create_fake_stream(
    response: ChatResponse,
) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, GroqError>> + Send>>, GroqError> {
    // Convert response to chunks
    let chunks = response_to_chunks(response);
    let stream = futures::stream::iter(chunks.into_iter().map(Ok));
    Ok(Box::pin(stream))
}

/// Convert a complete ChatResponse to stream chunks
fn response_to_chunks(response: ChatResponse) -> Vec<ChatChunk> {
    let mut chunks = Vec::new();

    // Create initial chunk with role
    chunks.push(ChatChunk {
        id: response.id.clone(),
        object: "chat.completion.chunk".to_string(),
        created: response.created,
        model: response.model.clone(),
        system_fingerprint: response.system_fingerprint.clone(),
        choices: vec![ChatStreamChoice {
            index: 0,
            delta: ChatDelta {
                role: Some(MessageRole::Assistant),
                content: None,
                            thinking: None,
                tool_calls: None,
                thinking: None,
                function_call: None,
                thinking: None,
            },
            finish_reason: None,
            logprobs: None,
        }],
        usage: None,
    });

    // Create content chunks
    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            let text = match content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::Parts(_) => content.to_string(), // Use Display impl
            };

            // Split content into smaller chunks for more natural streaming
            let words: Vec<&str> = text.split_whitespace().collect();
            let chunk_size = 5; // Words per chunk

            for word_chunk in words.chunks(chunk_size) {
                let chunk_text = word_chunk.join(" ") + " ";
                chunks.push(ChatChunk {
                    id: response.id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: response.created,
                    model: response.model.clone(),
                    system_fingerprint: response.system_fingerprint.clone(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: Some(chunk_text),
                            thinking: None,
                            tool_calls: None,
                            thinking: None,
                function_call: None,
                thinking: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    usage: None,
                });
            }
        }

        // Add final chunk with finish_reason
        chunks.push(ChatChunk {
            id: response.id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: response.created,
            model: response.model.clone(),
            system_fingerprint: response.system_fingerprint.clone(),
            choices: vec![ChatStreamChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: None,
                            thinking: None,
                    tool_calls: None,
                    thinking: None,
                function_call: None,
                thinking: None,
                },
                finish_reason: choice.finish_reason.clone(),
                logprobs: None,
            }],
            usage: response.usage.clone(),
        });
    }

    chunks
}
