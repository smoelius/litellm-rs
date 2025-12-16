//! Type definitions for vector storage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::error::Result;

/// Vector data for storage and retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorData {
    /// Unique identifier
    pub id: String,
    /// Vector embedding
    pub vector: Vec<f32>,
    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Search result from vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Associated metadata
    pub metadata: Option<serde_json::Value>,
    /// Vector data (optional)
    pub vector: Option<Vec<f32>>,
}

/// Vector point for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    /// Point ID
    pub id: String,
    /// Vector data
    pub vector: Vec<f32>,
    /// Associated metadata
    pub metadata: Option<serde_json::Value>,
}

/// Vector store trait
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync {
    /// Search for similar vectors
    async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>>;

    /// Insert vectors
    async fn insert(&self, vectors: Vec<VectorData>) -> Result<()>;

    /// Delete vectors by ID
    async fn delete(&self, ids: Vec<String>) -> Result<()>;
}
