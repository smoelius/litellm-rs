//! Embedding response types

use serde::{Deserialize, Serialize};

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Object type
    pub object: String,
    /// Embeddings
    pub data: Vec<EmbeddingData>,
    /// Model used
    pub model: String,
    /// Usage statistics
    pub usage: Option<EmbeddingUsage>,
}

/// Embedding data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    /// Object type
    pub object: String,
    /// Index
    pub index: u32,
    /// Embedding vector
    pub embedding: Vec<f32>,
}

/// Embedding usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== EmbeddingUsage Tests ====================

    #[test]
    fn test_embedding_usage_structure() {
        let usage = EmbeddingUsage {
            prompt_tokens: 100,
            total_tokens: 100,
        };

        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.total_tokens, 100);
    }

    #[test]
    fn test_embedding_usage_zero() {
        let usage = EmbeddingUsage {
            prompt_tokens: 0,
            total_tokens: 0,
        };

        assert_eq!(usage.prompt_tokens, 0);
    }

    #[test]
    fn test_embedding_usage_serialization() {
        let usage = EmbeddingUsage {
            prompt_tokens: 50,
            total_tokens: 50,
        };

        let json = serde_json::to_value(&usage).unwrap();
        assert_eq!(json["prompt_tokens"], 50);
        assert_eq!(json["total_tokens"], 50);
    }

    #[test]
    fn test_embedding_usage_deserialization() {
        let json = r#"{"prompt_tokens": 75, "total_tokens": 75}"#;
        let usage: EmbeddingUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.prompt_tokens, 75);
    }

    #[test]
    fn test_embedding_usage_clone() {
        let usage = EmbeddingUsage {
            prompt_tokens: 25,
            total_tokens: 25,
        };

        let cloned = usage.clone();
        assert_eq!(usage.prompt_tokens, cloned.prompt_tokens);
    }

    // ==================== EmbeddingData Tests ====================

    #[test]
    fn test_embedding_data_structure() {
        let data = EmbeddingData {
            object: "embedding".to_string(),
            index: 0,
            embedding: vec![0.1, 0.2, 0.3],
        };

        assert_eq!(data.object, "embedding");
        assert_eq!(data.index, 0);
        assert_eq!(data.embedding.len(), 3);
    }

    #[test]
    fn test_embedding_data_empty_vector() {
        let data = EmbeddingData {
            object: "embedding".to_string(),
            index: 0,
            embedding: vec![],
        };

        assert!(data.embedding.is_empty());
    }

    #[test]
    fn test_embedding_data_large_vector() {
        let data = EmbeddingData {
            object: "embedding".to_string(),
            index: 5,
            embedding: vec![0.0; 1536], // OpenAI ada-002 dimension
        };

        assert_eq!(data.embedding.len(), 1536);
        assert_eq!(data.index, 5);
    }

    #[test]
    fn test_embedding_data_serialization() {
        let data = EmbeddingData {
            object: "embedding".to_string(),
            index: 1,
            embedding: vec![0.5, -0.5],
        };

        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["object"], "embedding");
        assert_eq!(json["index"], 1);
        assert!(json["embedding"].is_array());
    }

    #[test]
    fn test_embedding_data_deserialization() {
        let json = r#"{"object": "embedding", "index": 2, "embedding": [0.1, 0.2, 0.3]}"#;
        let data: EmbeddingData = serde_json::from_str(json).unwrap();
        assert_eq!(data.index, 2);
        assert_eq!(data.embedding.len(), 3);
    }

    #[test]
    fn test_embedding_data_clone() {
        let data = EmbeddingData {
            object: "embedding".to_string(),
            index: 0,
            embedding: vec![1.0, 2.0, 3.0],
        };

        let cloned = data.clone();
        assert_eq!(data.object, cloned.object);
        assert_eq!(data.embedding, cloned.embedding);
    }

    // ==================== EmbeddingResponse Tests ====================

    #[test]
    fn test_embedding_response_structure() {
        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![],
            model: "text-embedding-ada-002".to_string(),
            usage: None,
        };

        assert_eq!(response.object, "list");
        assert_eq!(response.model, "text-embedding-ada-002");
        assert!(response.data.is_empty());
        assert!(response.usage.is_none());
    }

    #[test]
    fn test_embedding_response_with_data() {
        let data = EmbeddingData {
            object: "embedding".to_string(),
            index: 0,
            embedding: vec![0.1, 0.2, 0.3],
        };

        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![data],
            model: "text-embedding-3-small".to_string(),
            usage: Some(EmbeddingUsage {
                prompt_tokens: 5,
                total_tokens: 5,
            }),
        };

        assert_eq!(response.data.len(), 1);
        assert!(response.usage.is_some());
    }

    #[test]
    fn test_embedding_response_multiple_embeddings() {
        let data1 = EmbeddingData {
            object: "embedding".to_string(),
            index: 0,
            embedding: vec![0.1, 0.2],
        };
        let data2 = EmbeddingData {
            object: "embedding".to_string(),
            index: 1,
            embedding: vec![0.3, 0.4],
        };

        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![data1, data2],
            model: "text-embedding-3-large".to_string(),
            usage: Some(EmbeddingUsage {
                prompt_tokens: 10,
                total_tokens: 10,
            }),
        };

        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].index, 0);
        assert_eq!(response.data[1].index, 1);
    }

    #[test]
    fn test_embedding_response_serialization() {
        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![EmbeddingData {
                object: "embedding".to_string(),
                index: 0,
                embedding: vec![0.5],
            }],
            model: "model".to_string(),
            usage: Some(EmbeddingUsage {
                prompt_tokens: 1,
                total_tokens: 1,
            }),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["object"], "list");
        assert_eq!(json["model"], "model");
        assert!(json["data"].is_array());
        assert!(json["usage"].is_object());
    }

    #[test]
    fn test_embedding_response_deserialization() {
        let json = r#"{
            "object": "list",
            "data": [{"object": "embedding", "index": 0, "embedding": [0.1]}],
            "model": "test-model",
            "usage": {"prompt_tokens": 5, "total_tokens": 5}
        }"#;

        let response: EmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.model, "test-model");
        assert_eq!(response.data.len(), 1);
    }

    #[test]
    fn test_embedding_response_clone() {
        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![],
            model: "clone-test".to_string(),
            usage: None,
        };

        let cloned = response.clone();
        assert_eq!(response.object, cloned.object);
        assert_eq!(response.model, cloned.model);
    }

    #[test]
    fn test_embedding_response_no_usage() {
        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![],
            model: "model".to_string(),
            usage: None,
        };

        assert!(response.usage.is_none());
    }
}
