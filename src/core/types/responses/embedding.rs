//! Embedding response types

use serde::{Deserialize, Serialize};

use super::usage::Usage;

/// Embedding response (simple format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    /// Object type
    pub object: String,

    /// Embedding data list
    pub data: Vec<EmbeddingData>,

    /// Model used
    pub model: String,

    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
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

/// Embedding usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    /// Prompt token count
    pub prompt_tokens: u32,

    /// Total token count
    pub total_tokens: u32,
}

/// Embedding response (full format with backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Object type
    pub object: String,

    /// Embedding data list
    pub data: Vec<EmbeddingData>,

    /// Model used
    pub model: String,

    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,

    /// Embedding data list (backward compatibility field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<Vec<EmbeddingData>>,
}
