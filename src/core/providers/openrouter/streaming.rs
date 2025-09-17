//! OpenRouter Streaming Implementation
//!
//! Uses the unified SSE parser with OpenAI-compatible transformer

use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};

/// OpenRouter uses OpenAI-compatible SSE format
pub type OpenRouterStream = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create OpenRouter stream
pub fn create_openrouter_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> OpenRouterStream {
    let transformer = OpenAICompatibleTransformer::new("openrouter");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use futures::stream;

    #[tokio::test]
    async fn test_openrouter_stream() {
        // Test data in OpenAI-compatible SSE format
        let test_data = vec![
            Ok(bytes::Bytes::from(
                "data: {\"id\":\"test-1\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(bytes::Bytes::from(
                "data: {\"id\":\"test-2\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" World\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(bytes::Bytes::from(
                "data: {\"id\":\"test-3\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            )),
            Ok(bytes::Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(test_data);
        let mut openrouter_stream = create_openrouter_stream(mock_stream);

        // First chunk
        let chunk1 = openrouter_stream.next().await;
        assert!(chunk1.is_some());
        let chunk1 = chunk1.unwrap().unwrap();
        assert_eq!(chunk1.choices[0].delta.content.as_ref().unwrap(), "Hello");

        // Second chunk
        let chunk2 = openrouter_stream.next().await;
        assert!(chunk2.is_some());
        let chunk2 = chunk2.unwrap().unwrap();
        assert_eq!(chunk2.choices[0].delta.content.as_ref().unwrap(), " World");

        // Final chunk with finish_reason
        let chunk3 = openrouter_stream.next().await;
        assert!(chunk3.is_some());
        let chunk3 = chunk3.unwrap().unwrap();
        assert!(chunk3.choices[0].finish_reason.is_some());

        // Stream should end after [DONE]
        let end = openrouter_stream.next().await;
        assert!(end.is_none());
    }

    #[tokio::test]
    async fn test_openrouter_stream_with_errors() {
        // Test with invalid JSON
        let test_data = vec![Ok(bytes::Bytes::from("data: {invalid json}\n\n"))];

        let mock_stream = stream::iter(test_data);
        let mut openrouter_stream = create_openrouter_stream(mock_stream);

        // Should return error for invalid JSON
        let result = openrouter_stream.next().await;
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }
}
