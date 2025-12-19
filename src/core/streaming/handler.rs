//! Streaming response handler implementation

use super::types::{
    ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionDelta, Event,
};
use crate::core::models::openai::Usage;
use crate::core::types::MessageRole;
use crate::utils::error::Result;
use actix_web::web;
use futures::stream::{Stream, StreamExt};
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;
use uuid::Uuid;

/// Streaming response handler
pub struct StreamingHandler {
    /// Request ID for tracking
    request_id: String,
    /// Model being used
    pub(crate) model: String,
    /// Whether this is the first chunk
    pub(crate) is_first_chunk: bool,
    /// Accumulated content for final usage calculation
    pub(crate) accumulated_content: String,
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
    pub(crate) fn extract_content_from_chunk(&self, chunk_data: &str) -> Result<String> {
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
        let token_counter = crate::utils::ai::counter::token_counter::TokenCounter::new();
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
                thinking_usage: None,
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
    pub(crate) fn estimate_token_count(&self, text: &str) -> u32 {
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
