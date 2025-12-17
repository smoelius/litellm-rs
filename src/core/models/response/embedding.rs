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
