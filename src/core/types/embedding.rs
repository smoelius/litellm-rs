//! Embedding request types

use serde::{Deserialize, Serialize};

/// Embedding request (short form)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// Model name
    pub model: String,
    /// Input text
    pub input: EmbedInput,
    /// Encoding format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    /// Dimensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Embedding input (short form)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbedInput {
    /// Single string
    Single(String),
    /// String array
    Multiple(Vec<String>),
    /// Integer array (token IDs)
    TokenIds(Vec<u32>),
    /// Array of integer arrays
    MultipleTokenIds(Vec<Vec<u32>>),
}

/// Embedding request (full form)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model name
    pub model: String,
    /// Input text or text list
    pub input: EmbeddingInput,
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Embedding format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    /// Dimensions count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    /// Task type (for Vertex AI etc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
}

/// Embedding input type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// Single text
    Text(String),
    /// Text list
    Array(Vec<String>),
}

impl EmbeddingInput {
    /// Get iterator over texts
    pub fn iter(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        match self {
            EmbeddingInput::Text(text) => Box::new(std::iter::once(text)),
            EmbeddingInput::Array(texts) => Box::new(texts.iter()),
        }
    }

    /// Convert to text vector
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            EmbeddingInput::Text(text) => vec![text.clone()],
            EmbeddingInput::Array(texts) => texts.clone(),
        }
    }
}
