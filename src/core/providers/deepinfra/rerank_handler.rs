//! DeepInfra Rerank Handler

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::common_utils::{DeepInfraClient, DeepInfraConfig, DeepInfraError};

/// Rerank request for DeepInfra
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: Option<usize>,
}

/// Rerank response from DeepInfra
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    pub results: Vec<RerankResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub relevance_score: f64,
    pub document: String,
}

/// DeepInfra rerank handler
pub struct DeepInfraRerankHandler {
    client: DeepInfraClient,
    config: DeepInfraConfig,
}

impl DeepInfraRerankHandler {
    pub fn new(config: DeepInfraConfig) -> Result<Self, DeepInfraError> {
        let client = DeepInfraClient::new(config.clone())?;
        Ok(Self { client, config })
    }
    
    /// Rerank documents
    pub async fn rerank(
        &self,
        request: RerankRequest,
        api_key: Option<&str>,
    ) -> Result<RerankResponse, DeepInfraError> {
        let api_key = api_key
            .map(|s| s.to_string())
            .or_else(|| self.config.api_key.clone())
            .ok_or_else(|| DeepInfraError::Authentication("DeepInfra API key required".to_string()))?;
        
        let url = format!("{}/rerank", self.config.api_base.as_deref().unwrap_or("https://api.deepinfra.com"));
        
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        // Send request to DeepInfra
        self.client.send_rerank_request(request, url, headers).await
    }
}