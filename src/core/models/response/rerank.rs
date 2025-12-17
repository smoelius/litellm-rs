//! Rerank response types

use serde::{Deserialize, Serialize};

/// Rerank response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    /// Response ID
    pub id: String,
    /// Model used
    pub model: String,
    /// Reranked results
    pub results: Vec<RerankResult>,
    /// Usage statistics
    pub usage: Option<RerankUsage>,
}

/// Rerank result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    /// Document index
    pub index: u32,
    /// Relevance score
    pub relevance_score: f64,
    /// Document text (if requested)
    pub document: Option<String>,
}

/// Rerank usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankUsage {
    /// Total tokens
    pub total_tokens: u32,
}
