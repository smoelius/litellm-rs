//! Semantic caching for AI responses
//!
//! This module provides intelligent caching based on semantic similarity of prompts.

use crate::core::models::openai::*;
use crate::storage::vector::VectorStore;
use crate::utils::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Semantic cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCacheEntry {
    /// Unique cache entry ID
    pub id: String,
    /// Original prompt/messages hash
    pub prompt_hash: String,
    /// Prompt embedding vector
    pub embedding: Vec<f32>,
    /// Cached response
    pub response: ChatCompletionResponse,
    /// Model used for the response
    pub model: String,
    /// Cache creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// TTL in seconds
    pub ttl_seconds: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Semantic cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCacheConfig {
    /// Similarity threshold (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Default TTL in seconds
    pub default_ttl_seconds: u64,
    /// Embedding model to use
    pub embedding_model: String,
    /// Enable cache for streaming responses
    pub enable_streaming_cache: bool,
    /// Minimum prompt length to cache
    pub min_prompt_length: usize,
    /// Cache hit boost factor
    pub cache_hit_boost: f64,
}

impl Default for SemanticCacheConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            max_cache_size: 10000,
            default_ttl_seconds: 3600, // 1 hour
            embedding_model: "text-embedding-ada-002".to_string(),
            enable_streaming_cache: false,
            min_prompt_length: 10,
            cache_hit_boost: 1.1,
        }
    }
}

/// Semantic cache implementation
pub struct SemanticCache {
    /// Cache configuration
    config: SemanticCacheConfig,
    /// Vector store for embeddings
    vector_store: Arc<dyn VectorStore>,
    /// Embedding provider for generating embeddings
    embedding_provider: Arc<dyn EmbeddingProvider>,
    /// Consolidated cache data - single lock for cache entries and statistics
    cache_data: Arc<RwLock<CacheData>>,
}

/// Consolidated cache data - single lock for cache entries and statistics
#[derive(Debug, Default)]
struct CacheData {
    /// In-memory cache for recent entries
    entries: HashMap<String, SemanticCacheEntry>,
    /// Cache statistics
    stats: CacheStats,
}

/// Cache statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total cache entries
    pub total_entries: u64,
    /// Average similarity score for hits
    pub avg_hit_similarity: f64,
    /// Cache size in bytes (approximate)
    pub cache_size_bytes: u64,
}

/// Trait for embedding providers
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for text
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;

    /// Get embedding dimension
    fn embedding_dimension(&self) -> usize;
}

impl SemanticCache {
    /// Create a new semantic cache
    pub async fn new(
        config: SemanticCacheConfig,
        vector_store: Arc<dyn VectorStore>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<Self> {
        info!(
            "Initializing semantic cache with threshold: {}",
            config.similarity_threshold
        );

        Ok(Self {
            config,
            vector_store,
            embedding_provider,
            cache_data: Arc::new(RwLock::new(CacheData::default())),
        })
    }

    /// Try to get a cached response for the given request
    pub async fn get_cached_response(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Option<ChatCompletionResponse>> {
        // Check if caching is appropriate for this request
        if !self.should_cache_request(request) {
            return Ok(None);
        }

        // Generate prompt text for embedding
        let prompt_text = self.extract_prompt_text(&request.messages);

        if prompt_text.len() < self.config.min_prompt_length {
            debug!("Prompt too short for caching: {} chars", prompt_text.len());
            return Ok(None);
        }

        // Generate embedding for the prompt
        let embedding = match self
            .embedding_provider
            .generate_embedding(&prompt_text)
            .await
        {
            Ok(emb) => emb,
            Err(e) => {
                warn!("Failed to generate embedding for cache lookup: {}", e);
                return Ok(None);
            }
        };

        // Search for similar entries in vector store
        let search_results = self.vector_store.search(embedding, 10).await?;

        // Find the best match
        for result in search_results {
            if result.score >= self.config.similarity_threshold as f32 {
                if let Some(entry) = self.get_cache_entry(&result.id).await? {
                    // Check if entry is still valid
                    if self.is_entry_valid(&entry) {
                        // Update access and hit statistics with single lock
                        {
                            let mut data = self.cache_data.write().await;
                            if let Some(cache_entry) = data.entries.get_mut(&result.id) {
                                cache_entry.last_accessed = chrono::Utc::now();
                                cache_entry.access_count += 1;
                            }
                            data.stats.hits += 1;
                            data.stats.avg_hit_similarity = (data.stats.avg_hit_similarity
                                * (data.stats.hits - 1) as f64
                                + result.score as f64)
                                / data.stats.hits as f64;
                        }

                        info!(
                            "Cache hit! Similarity: {:.3}, Entry: {}",
                            result.score, result.id
                        );
                        return Ok(Some(entry.response));
                    } else {
                        // Remove expired entry
                        self.remove_cache_entry(&result.id).await?;
                    }
                }
            }
        }

        // No cache hit
        {
            let mut data = self.cache_data.write().await;
            data.stats.misses += 1;
        }

        debug!(
            "Cache miss for prompt: {}",
            prompt_text.chars().take(100).collect::<String>()
        );
        Ok(None)
    }

    /// Cache a response for the given request
    pub async fn cache_response(
        &self,
        request: &ChatCompletionRequest,
        response: &ChatCompletionResponse,
    ) -> Result<()> {
        // Check if caching is appropriate
        if !self.should_cache_request(request) {
            return Ok(());
        }

        let prompt_text = self.extract_prompt_text(&request.messages);

        if prompt_text.len() < self.config.min_prompt_length {
            return Ok(());
        }

        // Generate embedding for the prompt
        let embedding = self
            .embedding_provider
            .generate_embedding(&prompt_text)
            .await?;

        // Create cache entry
        let entry = SemanticCacheEntry {
            id: Uuid::new_v4().to_string(),
            prompt_hash: self.hash_prompt(&prompt_text),
            embedding: embedding.clone(),
            response: response.clone(),
            model: request.model.clone(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            access_count: 0,
            ttl_seconds: Some(self.config.default_ttl_seconds),
            metadata: HashMap::new(),
        };

        // Store in vector store
        let vector_data = crate::storage::vector::VectorData {
            id: entry.id.clone(),
            vector: embedding,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "prompt_hash".to_string(),
                    serde_json::to_value(&entry.prompt_hash)?,
                );
                metadata.insert(
                    "created_at".to_string(),
                    serde_json::to_value(entry.created_at)?,
                );
                metadata
            },
        };
        self.vector_store.insert(vec![vector_data]).await?;

        // Store in memory cache and update statistics with single lock
        let should_evict = {
            let mut data = self.cache_data.write().await;
            data.entries.insert(entry.id.clone(), entry);
            data.stats.total_entries += 1;
            data.entries.len() > self.config.max_cache_size
        };

        // Check cache size limits (eviction outside lock)
        if should_evict {
            self.evict_old_entries().await?;
        }

        info!("Cached response for model: {}", request.model);
        Ok(())
    }

    /// Check if a request should be cached
    fn should_cache_request(&self, request: &ChatCompletionRequest) -> bool {
        // Don't cache streaming requests unless explicitly enabled
        if request.stream.unwrap_or(false) && !self.config.enable_streaming_cache {
            return false;
        }

        // Don't cache requests with function calls (they might have side effects)
        if request.tools.is_some() || request.tool_choice.is_some() {
            return false;
        }

        // Don't cache requests with high randomness
        if let Some(temperature) = request.temperature {
            if temperature > 0.7 {
                return false;
            }
        }

        true
    }

    /// Extract prompt text from messages
    fn extract_prompt_text(&self, messages: &[ChatMessage]) -> String {
        messages
            .iter()
            .filter_map(|msg| match &msg.content {
                Some(MessageContent::Text(text)) => Some(text.clone()),
                Some(MessageContent::Parts(parts)) => {
                    let text = parts
                        .iter()
                        .filter_map(|part| match part {
                            ContentPart::Text { text } => Some(text.clone()),
                            _ => None,
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    if text.is_empty() { None } else { Some(text) }
                }
                None => None,
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Hash a prompt for quick lookup
    fn hash_prompt(&self, prompt: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get cache entry by ID
    async fn get_cache_entry(&self, entry_id: &str) -> Result<Option<SemanticCacheEntry>> {
        let data = self.cache_data.read().await;
        Ok(data.entries.get(entry_id).cloned())
    }

    /// Check if cache entry is still valid
    fn is_entry_valid(&self, entry: &SemanticCacheEntry) -> bool {
        if let Some(ttl_seconds) = entry.ttl_seconds {
            let expiry_time = entry.created_at + chrono::Duration::seconds(ttl_seconds as i64);
            chrono::Utc::now() < expiry_time
        } else {
            true // No TTL means never expires
        }
    }

    /// Update access statistics for a cache entry
    async fn update_access_stats(&self, entry_id: &str, _similarity: f64) -> Result<()> {
        let mut data = self.cache_data.write().await;
        if let Some(entry) = data.entries.get_mut(entry_id) {
            entry.last_accessed = chrono::Utc::now();
            entry.access_count += 1;
        }
        Ok(())
    }

    /// Remove cache entry
    async fn remove_cache_entry(&self, entry_id: &str) -> Result<()> {
        // Remove from memory cache
        {
            let mut data = self.cache_data.write().await;
            data.entries.remove(entry_id);
        }

        // Remove from vector store
        self.vector_store.delete(vec![entry_id.to_string()]).await?;

        Ok(())
    }

    /// Evict old entries when cache is full
    async fn evict_old_entries(&self) -> Result<()> {
        let entries_to_remove: Vec<String> = {
            let data = self.cache_data.read().await;

            // Sort entries by last access time and remove oldest 10%
            let mut entries: Vec<_> = data
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.last_accessed))
                .collect();
            entries.sort_by_key(|(_, last_accessed)| *last_accessed);

            let evict_count = (entries.len() as f64 * 0.1).ceil() as usize;
            entries
                .iter()
                .take(evict_count)
                .map(|(id, _)| id.clone())
                .collect()
        };

        let evict_count = entries_to_remove.len();

        // Remove from cache
        {
            let mut data = self.cache_data.write().await;
            for entry_id in &entries_to_remove {
                data.entries.remove(entry_id);
            }
        }

        // Also remove from vector store (async)
        for entry_id in entries_to_remove {
            let vector_store = self.vector_store.clone();
            tokio::spawn(async move {
                if let Err(e) = vector_store.delete(vec![entry_id]).await {
                    warn!("Failed to delete entry from vector store: {}", e);
                }
            });
        }

        info!("Evicted {} old cache entries", evict_count);
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.cache_data.read().await.stats.clone()
    }

    /// Clear all cache entries
    pub async fn clear_cache(&self) -> Result<()> {
        // Clear cache and reset statistics with single lock
        {
            let mut data = self.cache_data.write().await;
            data.entries.clear();
            data.stats = CacheStats::default();
        }

        // Note: Vector store doesn't have clear_all method in current implementation
        // In a full implementation, you would delete all vectors or recreate the collection

        info!("Cleared all cache entries");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::openai::{MessageContent, MessageRole};

    #[test]
    fn test_semantic_cache_config_default() {
        let config = SemanticCacheConfig::default();
        assert_eq!(config.similarity_threshold, 0.85);
        assert_eq!(config.max_cache_size, 10000);
        assert_eq!(config.default_ttl_seconds, 3600);
    }

    #[tokio::test]
    async fn test_extract_prompt_text() {
        let cache = create_test_cache().await;

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a helpful assistant".to_string(),
                )),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello world".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
        ];

        let prompt_text = cache.extract_prompt_text(&messages);
        assert!(prompt_text.contains("You are a helpful assistant"));
        assert!(prompt_text.contains("Hello world"));
    }

    #[tokio::test]
    async fn test_should_cache_request() {
        let cache = create_test_cache().await;

        let mut request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            max_tokens: None,
            max_completion_tokens: None,
            temperature: Some(0.1),
            top_p: None,
            n: None,
            stream: Some(false),
            stream_options: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            functions: None,
            function_call: None,
            tools: None,
            tool_choice: None,
            response_format: None,
            seed: None,
            logprobs: None,
            top_logprobs: None,
            modalities: None,
            audio: None,
        };

        // Should cache low temperature request
        assert!(cache.should_cache_request(&request));

        // Should not cache high temperature request
        request.temperature = Some(0.9);
        assert!(!cache.should_cache_request(&request));

        // Should not cache streaming request (by default)
        request.temperature = Some(0.1);
        request.stream = Some(true);
        assert!(!cache.should_cache_request(&request));
    }

    async fn create_test_cache() -> SemanticCache {
        // For testing purposes, create a dummy cache
        let config = SemanticCacheConfig {
            similarity_threshold: 0.85,
            max_cache_size: 1000,
            default_ttl_seconds: 3600,
            embedding_model: "text-embedding-ada-002".to_string(),
            enable_streaming_cache: false,
            min_prompt_length: 10,
            cache_hit_boost: 1.1,
        };

        // Create a simple test implementation
        SemanticCache {
            config,
            vector_store: Arc::new(TestVectorStore),
            embedding_provider: Arc::new(TestEmbeddingProvider),
            cache_data: Arc::new(RwLock::new(CacheData::default())),
        }
    }

    // Simple test implementations
    struct TestVectorStore;
    struct TestEmbeddingProvider;

    #[async_trait::async_trait]
    impl VectorStore for TestVectorStore {
        async fn search(
            &self,
            _vector: Vec<f32>,
            _limit: usize,
        ) -> Result<Vec<crate::storage::vector::SearchResult>> {
            Ok(vec![])
        }

        async fn insert(&self, _vectors: Vec<crate::storage::vector::VectorData>) -> Result<()> {
            Ok(())
        }

        async fn delete(&self, _ids: Vec<String>) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl EmbeddingProvider for TestEmbeddingProvider {
        async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.1; 1536])
        }

        fn embedding_dimension(&self) -> usize {
            1536
        }
    }
}
