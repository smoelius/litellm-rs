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

use crate::utils::error::{GatewayError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Rerank request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    /// Model to use for reranking (e.g., "cohere/rerank-english-v3.0")
    pub model: String,

    /// The query to compare documents against
    pub query: String,

    /// List of documents to rerank
    pub documents: Vec<RerankDocument>,

    /// Number of top results to return (default: all documents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<usize>,

    /// Whether to return the document text in results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_documents: Option<bool>,

    /// Maximum number of chunks per document (for long documents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chunks_per_doc: Option<usize>,

    /// Additional provider-specific parameters
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_params: HashMap<String, serde_json::Value>,
}

impl Default for RerankRequest {
    fn default() -> Self {
        Self {
            model: "cohere/rerank-english-v3.0".to_string(),
            query: String::new(),
            documents: Vec::new(),
            top_n: None,
            return_documents: Some(true),
            max_chunks_per_doc: None,
            extra_params: HashMap::new(),
        }
    }
}

/// Document for reranking - can be a simple string or structured
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RerankDocument {
    /// Simple text document
    Text(String),
    /// Structured document with metadata
    Structured {
        /// Document text content
        text: String,
        /// Optional document title
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// Optional document ID for tracking
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// Additional metadata
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        metadata: HashMap<String, serde_json::Value>,
    },
}

impl RerankDocument {
    /// Create a simple text document
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Create a structured document
    pub fn structured(text: impl Into<String>) -> Self {
        Self::Structured {
            text: text.into(),
            title: None,
            id: None,
            metadata: HashMap::new(),
        }
    }

    /// Get the text content of the document
    pub fn get_text(&self) -> &str {
        match self {
            Self::Text(t) => t,
            Self::Structured { text, .. } => text,
        }
    }

    /// Get the document ID if available
    pub fn get_id(&self) -> Option<&str> {
        match self {
            Self::Text(_) => None,
            Self::Structured { id, .. } => id.as_deref(),
        }
    }
}

impl From<String> for RerankDocument {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for RerankDocument {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

/// Rerank response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    /// Unique response ID
    pub id: String,

    /// Reranked results ordered by relevance (highest first)
    pub results: Vec<RerankResult>,

    /// Model used for reranking
    pub model: String,

    /// Token usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<RerankUsage>,

    /// Response metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, serde_json::Value>,
}

/// Individual rerank result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    /// Original index of the document in the input list
    pub index: usize,

    /// Relevance score (typically 0.0 to 1.0, higher is more relevant)
    pub relevance_score: f64,

    /// The document text (if return_documents was true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<RerankDocument>,
}

/// Token usage for reranking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RerankUsage {
    /// Number of tokens in the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_tokens: Option<u32>,

    /// Number of tokens in all documents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_tokens: Option<u32>,

    /// Total tokens processed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,

    /// Search units consumed (Cohere-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_units: Option<u32>,
}

/// Trait for rerank providers
#[async_trait]
pub trait RerankProvider: Send + Sync {
    /// Rerank documents based on query relevance
    async fn rerank(&self, request: RerankRequest) -> Result<RerankResponse>;

    /// Get the provider name
    fn provider_name(&self) -> &'static str;

    /// Check if a model is supported
    fn supports_model(&self, model: &str) -> bool;

    /// Get supported models
    fn supported_models(&self) -> Vec<&'static str>;
}

/// Rerank service that routes to appropriate providers
pub struct RerankService {
    /// Registered rerank providers
    providers: HashMap<String, Arc<dyn RerankProvider>>,

    /// Default provider name
    default_provider: Option<String>,

    /// Request timeout
    timeout: Duration,

    /// Enable caching
    enable_cache: bool,

    /// Cache for rerank results
    cache: Option<Arc<RerankCache>>,
}

impl RerankService {
    /// Create a new rerank service
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
            timeout: Duration::from_secs(30),
            enable_cache: false,
            cache: None,
        }
    }

    /// Register a rerank provider
    pub fn register_provider(
        &mut self,
        name: impl Into<String>,
        provider: Arc<dyn RerankProvider>,
    ) -> &mut Self {
        let name = name.into();
        info!("Registering rerank provider: {}", name);
        self.providers.insert(name, provider);
        self
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, name: impl Into<String>) -> &mut Self {
        self.default_provider = Some(name.into());
        self
    }

    /// Set request timeout
    pub fn set_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    /// Enable caching
    pub fn enable_cache(&mut self, cache: Arc<RerankCache>) -> &mut Self {
        self.enable_cache = true;
        self.cache = Some(cache);
        self
    }

    /// Rerank documents using the appropriate provider
    pub async fn rerank(&self, request: RerankRequest) -> Result<RerankResponse> {
        let start = Instant::now();

        // Validate request
        self.validate_request(&request)?;

        // Check cache if enabled
        if self.enable_cache {
            if let Some(cache) = &self.cache {
                if let Some(cached) = cache.get(&request).await {
                    debug!("Rerank cache hit for query: {}", request.query);
                    return Ok(cached);
                }
            }
        }

        // Determine provider from model name
        let provider_name = self.extract_provider_name(&request.model);
        let provider = self.get_provider(&provider_name)?;

        // Execute rerank with timeout
        let response = tokio::time::timeout(self.timeout, provider.rerank(request.clone()))
            .await
            .map_err(|_| {
                GatewayError::Timeout(format!("Rerank request timed out after {:?}", self.timeout))
            })??;

        // Cache result if enabled
        if self.enable_cache {
            if let Some(cache) = &self.cache {
                cache.set(&request, &response).await;
            }
        }

        let elapsed = start.elapsed();
        info!(
            "Rerank completed in {:?}: {} documents -> {} results",
            elapsed,
            request.documents.len(),
            response.results.len()
        );

        Ok(response)
    }

    /// Validate rerank request
    fn validate_request(&self, request: &RerankRequest) -> Result<()> {
        if request.query.is_empty() {
            return Err(GatewayError::InvalidRequest(
                "Query cannot be empty".to_string(),
            ));
        }

        if request.documents.is_empty() {
            return Err(GatewayError::InvalidRequest(
                "Documents list cannot be empty".to_string(),
            ));
        }

        if request.documents.len() > 10000 {
            return Err(GatewayError::InvalidRequest(
                "Too many documents (max 10000)".to_string(),
            ));
        }

        if let Some(top_n) = request.top_n {
            if top_n == 0 {
                return Err(GatewayError::InvalidRequest(
                    "top_n must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Extract provider name from model string (e.g., "cohere/rerank-v3" -> "cohere")
    fn extract_provider_name(&self, model: &str) -> String {
        if let Some(idx) = model.find('/') {
            model[..idx].to_string()
        } else {
            self.default_provider
                .clone()
                .unwrap_or_else(|| "cohere".to_string())
        }
    }

    /// Get provider by name
    fn get_provider(&self, name: &str) -> Result<&Arc<dyn RerankProvider>> {
        self.providers.get(name).ok_or_else(|| {
            GatewayError::ProviderNotFound(format!("Rerank provider not found: {}", name))
        })
    }

    /// Get all registered providers
    pub fn providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a model is supported by any provider
    pub fn supports_model(&self, model: &str) -> bool {
        let provider_name = self.extract_provider_name(model);
        if let Some(provider) = self.providers.get(&provider_name) {
            provider.supports_model(model)
        } else {
            false
        }
    }
}

impl Default for RerankService {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple in-memory cache for rerank results
pub struct RerankCache {
    /// Cache entries
    entries: tokio::sync::RwLock<HashMap<String, CacheEntry>>,
    /// Maximum cache size
    max_size: usize,
    /// Default TTL
    default_ttl: Duration,
}

struct CacheEntry {
    response: RerankResponse,
    created_at: Instant,
    ttl: Duration,
}

impl RerankCache {
    /// Create a new cache
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            entries: tokio::sync::RwLock::new(HashMap::new()),
            max_size,
            default_ttl,
        }
    }

    /// Generate cache key from request
    fn cache_key(request: &RerankRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.model.hash(&mut hasher);
        request.query.hash(&mut hasher);
        for doc in &request.documents {
            doc.get_text().hash(&mut hasher);
        }
        request.top_n.hash(&mut hasher);
        format!("rerank:{:x}", hasher.finish())
    }

    /// Get cached response
    pub async fn get(&self, request: &RerankRequest) -> Option<RerankResponse> {
        let key = Self::cache_key(request);
        let entries = self.entries.read().await;

        if let Some(entry) = entries.get(&key) {
            if entry.created_at.elapsed() < entry.ttl {
                return Some(entry.response.clone());
            }
        }
        None
    }

    /// Set cached response
    pub async fn set(&self, request: &RerankRequest, response: &RerankResponse) {
        let key = Self::cache_key(request);
        let mut entries = self.entries.write().await;

        // Evict if at capacity
        if entries.len() >= self.max_size {
            // Remove oldest entries (expired ones first)
            entries.retain(|_, entry| entry.created_at.elapsed() < entry.ttl);

            // If still at capacity, remove random entry
            if entries.len() >= self.max_size {
                if let Some(key_to_remove) = entries.keys().next().cloned() {
                    entries.remove(&key_to_remove);
                }
            }
        }

        entries.insert(
            key,
            CacheEntry {
                response: response.clone(),
                created_at: Instant::now(),
                ttl: self.default_ttl,
            },
        );
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> RerankCacheStats {
        let entries = self.entries.read().await;
        let valid_entries = entries
            .values()
            .filter(|e| e.created_at.elapsed() < e.ttl)
            .count();

        RerankCacheStats {
            total_entries: entries.len(),
            valid_entries,
            max_size: self.max_size,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct RerankCacheStats {
    /// Total entries in cache
    pub total_entries: usize,
    /// Valid (non-expired) entries
    pub valid_entries: usize,
    /// Maximum cache size
    pub max_size: usize,
}

/// Cohere rerank provider implementation
pub struct CohereRerankProvider {
    /// API key
    api_key: String,
    /// API base URL
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
}

impl CohereRerankProvider {
    /// Create a new Cohere rerank provider
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.cohere.ai/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Set custom base URL
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

#[async_trait]
impl RerankProvider for CohereRerankProvider {
    async fn rerank(&self, request: RerankRequest) -> Result<RerankResponse> {
        // Extract model name (remove provider prefix)
        let model = if request.model.contains('/') {
            request
                .model
                .split('/')
                .next_back()
                .unwrap_or(&request.model)
        } else {
            &request.model
        };

        // Build Cohere request
        let documents: Vec<String> = request
            .documents
            .iter()
            .map(|d| d.get_text().to_string())
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "query": request.query,
            "documents": documents,
        });

        if let Some(top_n) = request.top_n {
            body["top_n"] = serde_json::json!(top_n);
        }

        if let Some(return_docs) = request.return_documents {
            body["return_documents"] = serde_json::json!(return_docs);
        }

        // Send request
        let response = self
            .client
            .post(format!("{}/rerank", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| GatewayError::Network(format!("Cohere rerank request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GatewayError::External(format!(
                "Cohere rerank error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let cohere_response: serde_json::Value = response.json().await.map_err(|e| {
            GatewayError::Parsing(format!("Failed to parse Cohere response: {}", e))
        })?;

        // Convert to our response format
        let results = cohere_response["results"]
            .as_array()
            .ok_or_else(|| GatewayError::Parsing("Missing results in response".to_string()))?
            .iter()
            .map(|r| {
                let index = r["index"].as_u64().unwrap_or(0) as usize;
                let relevance_score = r["relevance_score"].as_f64().unwrap_or(0.0);
                let document = if request.return_documents.unwrap_or(true) {
                    request.documents.get(index).cloned()
                } else {
                    None
                };

                RerankResult {
                    index,
                    relevance_score,
                    document,
                }
            })
            .collect();

        let usage = cohere_response.get("meta").and_then(|m| {
            m.get("billed_units").map(|bu| RerankUsage {
                query_tokens: None,
                document_tokens: None,
                total_tokens: None,
                search_units: bu
                    .get("search_units")
                    .and_then(|s| s.as_u64())
                    .map(|s| s as u32),
            })
        });

        Ok(RerankResponse {
            id: cohere_response["id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            results,
            model: model.to_string(),
            usage,
            meta: HashMap::new(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "cohere"
    }

    fn supports_model(&self, model: &str) -> bool {
        let model_name = model.split('/').next_back().unwrap_or(model);
        matches!(
            model_name,
            "rerank-english-v3.0"
                | "rerank-multilingual-v3.0"
                | "rerank-english-v2.0"
                | "rerank-multilingual-v2.0"
        )
    }

    fn supported_models(&self) -> Vec<&'static str> {
        vec![
            "rerank-english-v3.0",
            "rerank-multilingual-v3.0",
            "rerank-english-v2.0",
            "rerank-multilingual-v2.0",
        ]
    }
}

/// Jina AI rerank provider implementation
pub struct JinaRerankProvider {
    /// API key
    api_key: String,
    /// API base URL
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
}

impl JinaRerankProvider {
    /// Create a new Jina rerank provider
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.jina.ai/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl RerankProvider for JinaRerankProvider {
    async fn rerank(&self, request: RerankRequest) -> Result<RerankResponse> {
        let model = if request.model.contains('/') {
            request
                .model
                .split('/')
                .next_back()
                .unwrap_or(&request.model)
        } else {
            &request.model
        };

        let documents: Vec<String> = request
            .documents
            .iter()
            .map(|d| d.get_text().to_string())
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "query": request.query,
            "documents": documents,
        });

        if let Some(top_n) = request.top_n {
            body["top_n"] = serde_json::json!(top_n);
        }

        let response = self
            .client
            .post(format!("{}/rerank", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| GatewayError::Network(format!("Jina rerank request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GatewayError::External(format!(
                "Jina rerank error ({}): {}",
                status, error_text
            )));
        }

        let jina_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GatewayError::Parsing(format!("Failed to parse Jina response: {}", e)))?;

        let results = jina_response["results"]
            .as_array()
            .ok_or_else(|| GatewayError::Parsing("Missing results in response".to_string()))?
            .iter()
            .map(|r| {
                let index = r["index"].as_u64().unwrap_or(0) as usize;
                let relevance_score = r["relevance_score"].as_f64().unwrap_or(0.0);
                let document = if request.return_documents.unwrap_or(true) {
                    request.documents.get(index).cloned()
                } else {
                    None
                };

                RerankResult {
                    index,
                    relevance_score,
                    document,
                }
            })
            .collect();

        let usage = jina_response.get("usage").map(|u| RerankUsage {
            query_tokens: u
                .get("prompt_tokens")
                .and_then(|t| t.as_u64())
                .map(|t| t as u32),
            document_tokens: None,
            total_tokens: u
                .get("total_tokens")
                .and_then(|t| t.as_u64())
                .map(|t| t as u32),
            search_units: None,
        });

        Ok(RerankResponse {
            id: uuid::Uuid::new_v4().to_string(),
            results,
            model: model.to_string(),
            usage,
            meta: HashMap::new(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "jina"
    }

    fn supports_model(&self, model: &str) -> bool {
        let model_name = model.split('/').next_back().unwrap_or(model);
        matches!(
            model_name,
            "jina-reranker-v2-base-multilingual"
                | "jina-reranker-v1-base-en"
                | "jina-reranker-v1-turbo-en"
        )
    }

    fn supported_models(&self) -> Vec<&'static str> {
        vec![
            "jina-reranker-v2-base-multilingual",
            "jina-reranker-v1-base-en",
            "jina-reranker-v1-turbo-en",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rerank_document_creation() {
        let doc1 = RerankDocument::text("Hello world");
        assert_eq!(doc1.get_text(), "Hello world");
        assert!(doc1.get_id().is_none());

        let doc2 = RerankDocument::Structured {
            text: "Test document".to_string(),
            title: Some("Title".to_string()),
            id: Some("doc-1".to_string()),
            metadata: HashMap::new(),
        };
        assert_eq!(doc2.get_text(), "Test document");
        assert_eq!(doc2.get_id(), Some("doc-1"));
    }

    #[test]
    fn test_rerank_document_from_string() {
        let doc: RerankDocument = "Test".into();
        assert_eq!(doc.get_text(), "Test");

        let doc2: RerankDocument = String::from("Test2").into();
        assert_eq!(doc2.get_text(), "Test2");
    }

    #[test]
    fn test_rerank_request_default() {
        let request = RerankRequest::default();
        assert_eq!(request.model, "cohere/rerank-english-v3.0");
        assert!(request.query.is_empty());
        assert!(request.documents.is_empty());
        assert!(request.top_n.is_none());
        assert_eq!(request.return_documents, Some(true));
    }

    #[test]
    fn test_rerank_service_extract_provider() {
        let service = RerankService::new();

        assert_eq!(
            service.extract_provider_name("cohere/rerank-english-v3.0"),
            "cohere"
        );
        assert_eq!(
            service.extract_provider_name("jina/jina-reranker-v2"),
            "jina"
        );
        assert_eq!(service.extract_provider_name("voyage/rerank-2"), "voyage");
        // No provider prefix - uses default
        assert_eq!(
            service.extract_provider_name("rerank-english-v3.0"),
            "cohere"
        );
    }

    #[test]
    fn test_rerank_service_validation() {
        let service = RerankService::new();

        // Empty query
        let request = RerankRequest {
            query: "".to_string(),
            documents: vec![RerankDocument::text("doc")],
            ..Default::default()
        };
        assert!(service.validate_request(&request).is_err());

        // Empty documents
        let request = RerankRequest {
            query: "query".to_string(),
            documents: vec![],
            ..Default::default()
        };
        assert!(service.validate_request(&request).is_err());

        // top_n = 0
        let request = RerankRequest {
            query: "query".to_string(),
            documents: vec![RerankDocument::text("doc")],
            top_n: Some(0),
            ..Default::default()
        };
        assert!(service.validate_request(&request).is_err());

        // Valid request
        let request = RerankRequest {
            query: "query".to_string(),
            documents: vec![RerankDocument::text("doc")],
            top_n: Some(1),
            ..Default::default()
        };
        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_cohere_provider_supports_model() {
        let provider = CohereRerankProvider::new("test-key");

        assert!(provider.supports_model("rerank-english-v3.0"));
        assert!(provider.supports_model("cohere/rerank-english-v3.0"));
        assert!(provider.supports_model("rerank-multilingual-v3.0"));
        assert!(!provider.supports_model("unknown-model"));
    }

    #[test]
    fn test_jina_provider_supports_model() {
        let provider = JinaRerankProvider::new("test-key");

        assert!(provider.supports_model("jina-reranker-v2-base-multilingual"));
        assert!(provider.supports_model("jina/jina-reranker-v2-base-multilingual"));
        assert!(!provider.supports_model("unknown-model"));
    }

    #[tokio::test]
    async fn test_rerank_cache() {
        let cache = RerankCache::new(100, Duration::from_secs(3600));

        let request = RerankRequest {
            model: "cohere/rerank-english-v3.0".to_string(),
            query: "test query".to_string(),
            documents: vec![RerankDocument::text("test doc")],
            ..Default::default()
        };

        let response = RerankResponse {
            id: "test-id".to_string(),
            results: vec![RerankResult {
                index: 0,
                relevance_score: 0.95,
                document: Some(RerankDocument::text("test doc")),
            }],
            model: "rerank-english-v3.0".to_string(),
            usage: None,
            meta: HashMap::new(),
        };

        // Initially empty
        assert!(cache.get(&request).await.is_none());

        // Set and get
        cache.set(&request, &response).await;
        let cached = cache.get(&request).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().id, "test-id");

        // Stats
        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.valid_entries, 1);
    }

    #[test]
    fn test_rerank_result_ordering() {
        let mut results = [
            RerankResult {
                index: 0,
                relevance_score: 0.5,
                document: None,
            },
            RerankResult {
                index: 1,
                relevance_score: 0.9,
                document: None,
            },
            RerankResult {
                index: 2,
                relevance_score: 0.7,
                document: None,
            },
        ];

        // Sort by relevance descending
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        assert_eq!(results[0].index, 1); // 0.9
        assert_eq!(results[1].index, 2); // 0.7
        assert_eq!(results[2].index, 0); // 0.5
    }
}
