//! OpenAI Streaming Response Handler
//!
//! Uses the unified SSE parser for consistent streaming across providers.

use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

use super::error::OpenAIError;
use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::ChatChunk;

/// OpenAI uses OpenAI-compatible SSE format (naturally)
pub type OpenAIStream = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create OpenAI stream
pub fn create_openai_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> OpenAIStream {
    let transformer = OpenAICompatibleTransformer::new("openai");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

/// Wrapper stream that converts ProviderError to OpenAIError for backward compatibility
pub struct OpenAIStreamCompat {
    inner: OpenAIStream,
}

impl OpenAIStreamCompat {
    pub fn new(stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: create_openai_stream(stream),
        }
    }
}

impl Stream for OpenAIStreamCompat {
    type Item = Result<ChatChunk, OpenAIError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::pin::Pin;
        use std::task::Poll;

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => Poll::Ready(Some(Ok(chunk))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(provider_error_to_openai(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Convert ProviderError to OpenAIError
fn provider_error_to_openai(e: ProviderError) -> OpenAIError {
    match e {
        ProviderError::ResponseParsing { message, .. } => OpenAIError::ResponseParsing {
            provider: "openai",
            message,
        },
        ProviderError::Network { message, .. } => OpenAIError::Other {
            provider: "openai",
            message,
        },
        _ => OpenAIError::Other {
            provider: "openai",
            message: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::base::sse::UnifiedSSEParser;
    use futures::StreamExt;

    #[test]
    fn test_sse_parsing() {
        let transformer = OpenAICompatibleTransformer::new("openai");
        let mut parser = UnifiedSSEParser::new(transformer);

        let test_data = b"data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n";

        let result = parser.process_bytes(test_data);
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].id, "chatcmpl-123");
        assert_eq!(chunks[0].model, "gpt-4");
        assert_eq!(chunks[0].choices.len(), 1);
        assert_eq!(
            chunks[0].choices[0].delta.content,
            Some("Hello".to_string())
        );
    }

    #[test]
    fn test_done_message() {
        let transformer = OpenAICompatibleTransformer::new("openai");
        let mut parser = UnifiedSSEParser::new(transformer);

        let done_data = b"data: [DONE]\n\n";
        let result = parser.process_bytes(done_data);

        assert!(result.is_ok());
        // [DONE] should not produce any chunks
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_incremental_parsing() {
        let transformer = OpenAICompatibleTransformer::new("openai");
        let mut parser = UnifiedSSEParser::new(transformer);

        // Send data in parts
        let part1 = b"data: {\"id\":\"test\",\"object\":\"chat.completion.chunk\"";
        let part2 = b",\"created\":123,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\n";

        // First part should not produce a chunk
        let result1 = parser.process_bytes(part1);
        assert!(result1.is_ok());
        assert!(result1.unwrap().is_empty());

        // Second part should complete the chunk
        let result2 = parser.process_bytes(part2);
        assert!(result2.is_ok());

        let chunks = result2.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].id, "test");
        assert_eq!(chunks[0].choices[0].delta.content, Some("Hi".to_string()));
    }

    #[tokio::test]
    async fn test_stream_wrapper() {
        use futures::stream;

        // Create a mock byte stream
        let data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(data);
        let mut openai_stream = create_openai_stream(mock_stream);

        // Should produce one chunk
        let first_chunk = openai_stream.next().await;
        assert!(first_chunk.is_some());

        if let Some(Ok(chunk)) = first_chunk {
            assert_eq!(chunk.id, "test");
            assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
        }

        // Stream should end after [DONE]
        let second_chunk = openai_stream.next().await;
        assert!(second_chunk.is_none());
    }
}
