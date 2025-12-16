//! Rerank API for document relevance scoring
//!
//! This module provides reranking functionality to score and reorder documents
//! based on their relevance to a query. It's commonly used in RAG (Retrieval
//! Augmented Generation) systems to improve retrieval quality.
//!
//! ## Supported Providers
//! - Cohere (rerank-english-v3.0, rerank-multilingual-v3.0)
//! - Jina AI (jina-reranker-v2-base-multilingual)
//! - Voyage AI (rerank-2, rerank-2-lite)
//! - OpenAI (via embeddings + similarity)
//!
//! ## Example
//! ```rust,ignore
//! use litellm_rs::core::rerank::{RerankRequest, RerankProvider};
//!
//! let request = RerankRequest {
//!     model: "cohere/rerank-english-v3.0".to_string(),
//!     query: "What is the capital of France?".to_string(),
//!     documents: vec![
//!         "Paris is the capital of France.".to_string(),
//!         "London is the capital of England.".to_string(),
//!         "Berlin is the capital of Germany.".to_string(),
//!     ],
//!     top_n: Some(2),
//!     return_documents: Some(true),
//!     ..Default::default()
//! };
//!
//! let response = provider.rerank(request).await?;
//! ```

mod cache;
pub mod providers;
mod service;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use cache::{RerankCache, RerankCacheStats};
pub use providers::{CohereRerankProvider, JinaRerankProvider};
pub use service::{RerankProvider, RerankService};
pub use types::{RerankDocument, RerankRequest, RerankResponse, RerankResult, RerankUsage};
