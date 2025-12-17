//! Provider-specific streaming implementations

use crate::utils::error::{GatewayError, Result};
use futures::stream::{BoxStream, StreamExt};

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
