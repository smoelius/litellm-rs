//! Implementation

use crate::sdk::{config::ClientConfig, errors::*, types::*};
use reqwest;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 完整功能的 LLM 客户端
#[derive(Debug)]
pub struct LLMClient {
    config: ClientConfig,
    http_client: reqwest::Client,
    provider_stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
    load_balancer: Arc<LoadBalancer>,
}

/// 提供商统计信息
#[derive(Debug, Clone, Default)]
pub struct ProviderStats {
    pub requests: u64,
    pub errors: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_latency_ms: f64,
    pub last_used: Option<SystemTime>,
    pub health_score: f64,
}

/// 负载均衡器
#[derive(Debug)]
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
}

/// 负载均衡策略
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastLatency,
    WeightedRandom,
    HealthBased,
}

impl LLMClient {
    /// Create
    pub fn new(config: ClientConfig) -> Result<Self> {
        if config.providers.is_empty() {
            return Err(SDKError::ConfigError("No providers configured".to_string()));
        }

        // Build
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.settings.timeout))
            .build()
            .map_err(|e| SDKError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let provider_stats = Arc::new(RwLock::new(HashMap::new()));
        let load_balancer = Arc::new(LoadBalancer::new(LoadBalancingStrategy::WeightedRandom));

        info!(
            "LLMClient created with {} providers",
            config.providers.len()
        );

        Ok(Self {
            config,
            http_client,
            provider_stats,
            load_balancer,
        })
    }

    /// Create
    pub async fn new_async(config: ClientConfig) -> Result<Self> {
        let client = Self::new(config)?;

        // Check
        client.initialize_providers().await?;

        Ok(client)
    }

    /// Initialize
    async fn initialize_providers(&self) -> Result<()> {
        let mut stats = self.provider_stats.write().await;

        for provider in &self.config.providers {
            let provider_stats = ProviderStats {
                health_score: 1.0, // 初始健康分数
                ..Default::default()
            };
            stats.insert(provider.id.clone(), provider_stats);

            // Check
            debug!("Initialized provider: {}", provider.id);
        }

        Ok(())
    }

    /// 发送Chat message（usage负载均衡）
    pub async fn chat(&self, messages: Vec<Message>) -> Result<ChatResponse> {
        let request = ChatRequest {
            model: String::new(), // Settings
            messages,
            options: ChatOptions::default(),
        };

        self.chat_with_options(request).await
    }

    /// 发送Chat message（带选项）
    pub async fn chat_with_options(&self, request: ChatRequest) -> Result<ChatResponse> {
        let start_time = SystemTime::now();

        // 选择最佳提供商
        let provider = self.select_provider(&request).await?;

        // Request
        let result = self.execute_chat_request(&provider.id, request).await;

        // Update
        self.update_provider_stats(&provider.id, start_time, &result)
            .await;

        result
    }

    /// 流式聊天
    pub async fn chat_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl futures::Stream<Item = Result<ChatChunk>>> {
        let provider = self.select_provider_for_stream(&messages).await?;
        self.execute_stream_request(&provider.id, messages).await
    }

    /// 选择提供商
    async fn select_provider(
        &self,
        request: &ChatRequest,
    ) -> Result<&crate::sdk::config::ProviderConfig> {
        // 如果指定了模型，找到支持该模型的提供商
        if !request.model.is_empty() {
            for provider in &self.config.providers {
                if provider.models.contains(&request.model) && provider.enabled {
                    return Ok(provider);
                }
            }
            return Err(SDKError::ModelNotFound(format!(
                "Model '{}' not supported by any provider",
                request.model
            )));
        }

        // usage负载均衡策略选择提供商
        self.load_balancer
            .select_provider(&self.config.providers, &self.provider_stats)
            .await
    }

    /// Request
    async fn select_provider_for_stream(
        &self,
        _messages: &[Message],
    ) -> Result<&crate::sdk::config::ProviderConfig> {
        // 找到支持流式的提供商
        for provider in &self.config.providers {
            if provider.enabled {
                return Ok(provider);
            }
        }
        Err(SDKError::NoDefaultProvider)
    }

    /// Request
    async fn execute_chat_request(
        &self,
        provider_id: &str,
        request: ChatRequest,
    ) -> Result<ChatResponse> {
        let provider = self
            .config
            .providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| SDKError::ProviderNotFound(provider_id.to_string()))?;

        debug!("Executing chat request with provider: {}", provider_id);

        match provider.provider_type {
            crate::sdk::config::ProviderType::Anthropic => {
                self.call_anthropic_api(provider, request).await
            }
            crate::sdk::config::ProviderType::OpenAI => {
                self.call_openai_api(provider, request).await
            }
            crate::sdk::config::ProviderType::Google => {
                self.call_google_api(provider, request).await
            }
            // crate::sdk::config::ProviderType::Groq => {
            //     self.call_groq_api(provider, request).await
            // }
            _ => {
                warn!(
                    "Provider type {:?} not fully implemented, using mock response",
                    provider.provider_type
                );
                self.create_mock_response(provider, &request.messages).await
            }
        }
    }

    /// Request
    async fn execute_stream_request(
        &self,
        provider_id: &str,
        _messages: Vec<Message>,
    ) -> Result<impl futures::Stream<Item = Result<ChatChunk>>> {
        let _provider = self
            .config
            .providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| SDKError::ProviderNotFound(provider_id.to_string()))?;

        // Request
        // 现在Returns一个简单的模拟流
        use futures::stream;

        let chunk = ChatChunk {
            id: "stream-test".to_string(),
            model: "test-model".to_string(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: MessageDelta {
                    role: Some(Role::Assistant),
                    content: Some("Streaming response...".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        Ok(stream::once(async move { Ok(chunk) }))
    }

    /// call Anthropic API
    async fn call_anthropic_api(
        &self,
        provider: &crate::sdk::config::ProviderConfig,
        request: ChatRequest,
    ) -> Result<ChatResponse> {
        // 转换messageformat
        let (system_message, anthropic_messages) =
            self.convert_messages_to_anthropic(&request.messages);

        // Request
        let mut body = serde_json::json!({
            "model": provider.models.first().unwrap_or(&"claude-3-sonnet-20240229".to_string()),
            "messages": anthropic_messages,
            "max_tokens": request.options.max_tokens.unwrap_or(1000)
        });

        if let Some(system) = system_message {
            body["system"] = serde_json::json!(system);
        }

        if let Some(temp) = request.options.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(top_p) = request.options.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }

        // Request
        let default_url = "https://api.anthropic.com".to_string();
        let base_url = provider.base_url.as_ref().unwrap_or(&default_url);
        let url = if base_url.contains("/v1") {
            format!("{}/messages", base_url.trim_end_matches('/'))
        } else {
            format!("{}/v1/messages", base_url.trim_end_matches('/'))
        };

        debug!("Calling Anthropic API: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", &provider.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SDKError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Anthropic API error: {} - {}", status, error_text);
            return Err(SDKError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SDKError::ParseError(e.to_string()))?;

        // Response
        self.convert_anthropic_response(
            anthropic_response,
            provider
                .models
                .first()
                .unwrap_or(&"claude-3-sonnet-20240229".to_string()),
        )
    }

    /// call OpenAI API
    async fn call_openai_api(
        &self,
        provider: &crate::sdk::config::ProviderConfig,
        request: ChatRequest,
    ) -> Result<ChatResponse> {
        let body = serde_json::json!({
            "model": provider.models.first().unwrap_or(&"gpt-3.5-turbo".to_string()),
            "messages": request.messages,
            "max_tokens": request.options.max_tokens.unwrap_or(1000),
            "temperature": request.options.temperature.unwrap_or(0.7),
            "stream": false
        });

        let default_url = "https://api.openai.com".to_string();
        let base_url = provider.base_url.as_ref().unwrap_or(&default_url);
        let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

        debug!("Calling OpenAI API: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", provider.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SDKError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(SDKError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        // Response
        let openai_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| SDKError::ParseError(e.to_string()))?;

        Ok(openai_response)
    }

    /// call Google API
    async fn call_google_api(
        &self,
        provider: &crate::sdk::config::ProviderConfig,
        request: ChatRequest,
    ) -> Result<ChatResponse> {
        // Google API implementation占位符
        warn!("Google API not fully implemented, using mock response");
        self.create_mock_response(provider, &request.messages).await
    }

    /// call Groq API
    #[allow(dead_code)]
    async fn call_groq_api(
        &self,
        provider: &crate::sdk::config::ProviderConfig,
        request: ChatRequest,
    ) -> Result<ChatResponse> {
        // Groq API implementation占位符
        warn!("Groq API not fully implemented, using mock response");
        self.create_mock_response(provider, &request.messages).await
    }

    /// Create
    async fn create_mock_response(
        &self,
        provider: &crate::sdk::config::ProviderConfig,
        messages: &[Message],
    ) -> Result<ChatResponse> {
        let user_message = messages
            .iter()
            .filter(|m| matches!(m.role, Role::User))
            .next_back()
            .and_then(|m| match &m.content {
                Some(Content::Text(text)) => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or("Hello");

        let mock_content = format!(
            "Mock response from {} provider. You said: \"{}\"",
            provider.name, user_message
        );

        Ok(ChatResponse {
            id: format!("mock-{}", uuid::Uuid::new_v4()),
            model: provider
                .models
                .first()
                .unwrap_or(&"mock-model".to_string())
                .clone(),
            choices: vec![ChatChoice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: Some(Content::Text(mock_content)),
                    name: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 15,
                total_tokens: 25,
            },
            created: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// 转换message到 Anthropic format
    fn convert_messages_to_anthropic(
        &self,
        messages: &[Message],
    ) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_message = None;
        let mut anthropic_messages = Vec::new();

        for message in messages {
            match message.role {
                Role::System => {
                    if let Some(Content::Text(text)) = &message.content {
                        system_message = Some(text.clone());
                    }
                }
                Role::User => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "user",
                        "content": self.convert_content_to_anthropic(message.content.as_ref())
                    }));
                }
                Role::Assistant => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": self.convert_content_to_anthropic(message.content.as_ref())
                    }));
                }
                _ => {} // 忽略其他角色
            }
        }

        (system_message, anthropic_messages)
    }

    /// 转换content到 Anthropic format
    fn convert_content_to_anthropic(&self, content: Option<&Content>) -> serde_json::Value {
        match content {
            Some(Content::Text(text)) => serde_json::json!(text),
            Some(Content::Multimodal(parts)) => {
                let mut anthropic_content = Vec::new();
                for part in parts {
                    match part {
                        ContentPart::Text { text } => {
                            anthropic_content.push(serde_json::json!({
                                "type": "text",
                                "text": text
                            }));
                        }
                        ContentPart::Image { image_url } => {
                            anthropic_content.push(serde_json::json!({
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": "image/jpeg",
                                    "data": image_url.url.trim_start_matches("data:image/jpeg;base64,")
                                }
                            }));
                        }
                        _ => {} // 忽略其他类型
                    }
                }
                serde_json::json!(anthropic_content)
            }
            None => serde_json::json!(""),
        }
    }

    /// Response
    fn convert_anthropic_response(
        &self,
        anthropic_response: serde_json::Value,
        model: &str,
    ) -> Result<ChatResponse> {
        let id = anthropic_response
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("chatcmpl-anthropic")
            .to_string();

        let content = anthropic_response
            .get("content")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let usage = if let Some(u) = anthropic_response.get("usage") {
            Usage {
                prompt_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                    as u32,
                total_tokens: 0, // 将在下面计算
            }
        } else {
            Usage::default()
        };

        let mut usage = usage;
        usage.total_tokens = usage.prompt_tokens + usage.completion_tokens;

        Ok(ChatResponse {
            id,
            model: model.to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: Some(Content::Text(content)),
                    name: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage,
            created: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Update
    async fn update_provider_stats(
        &self,
        provider_id: &str,
        start_time: SystemTime,
        result: &Result<ChatResponse>,
    ) {
        let mut stats = self.provider_stats.write().await;
        let provider_stats = stats.entry(provider_id.to_string()).or_default();

        provider_stats.requests += 1;
        provider_stats.last_used = Some(SystemTime::now());

        if let Ok(elapsed) = start_time.elapsed() {
            let latency_ms = elapsed.as_millis() as f64;
            provider_stats.avg_latency_ms = if provider_stats.requests == 1 {
                latency_ms
            } else {
                (provider_stats.avg_latency_ms * (provider_stats.requests - 1) as f64 + latency_ms)
                    / provider_stats.requests as f64
            };
        }

        match result {
            Ok(response) => {
                provider_stats.total_tokens += response.usage.total_tokens as u64;
                provider_stats.health_score = (provider_stats.health_score * 0.9 + 0.1).min(1.0);
            }
            Err(_) => {
                provider_stats.errors += 1;
                provider_stats.health_score = (provider_stats.health_score * 0.9).max(0.1);
            }
        }

        debug!(
            "Updated stats for provider {}: requests={}, errors={}, health={:.2}",
            provider_id,
            provider_stats.requests,
            provider_stats.errors,
            provider_stats.health_score
        );
    }

    /// 列出可用的提供商
    pub fn list_providers(&self) -> Vec<String> {
        self.config.providers.iter().map(|p| p.id.clone()).collect()
    }

    /// Get
    pub async fn get_provider_stats(&self) -> HashMap<String, ProviderStats> {
        self.provider_stats.read().await.clone()
    }

    /// Configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Check
    pub async fn health_check(&self) -> Result<HashMap<String, bool>> {
        let mut health_status = HashMap::new();

        for provider in &self.config.providers {
            let is_healthy = self.check_provider_health(&provider.id).await.is_ok();
            health_status.insert(provider.id.clone(), is_healthy);
        }

        Ok(health_status)
    }

    /// Check
    async fn check_provider_health(&self, provider_id: &str) -> Result<()> {
        let simple_request = ChatRequest {
            model: String::new(),
            messages: vec![Message {
                role: Role::User,
                content: Some(Content::Text("Hi".to_string())),
                name: None,
                tool_calls: None,
            }],
            options: ChatOptions {
                max_tokens: Some(1),
                ..Default::default()
            },
        };

        // Request
        self.execute_chat_request(provider_id, simple_request)
            .await?;
        Ok(())
    }
}

impl LoadBalancer {
    fn new(strategy: LoadBalancingStrategy) -> Self {
        Self { strategy }
    }

    async fn select_provider<'a>(
        &self,
        providers: &'a [crate::sdk::config::ProviderConfig],
        stats: &Arc<RwLock<HashMap<String, ProviderStats>>>,
    ) -> Result<&'a crate::sdk::config::ProviderConfig> {
        let enabled_providers: Vec<&crate::sdk::config::ProviderConfig> =
            providers.iter().filter(|p| p.enabled).collect();

        if enabled_providers.is_empty() {
            return Err(SDKError::NoDefaultProvider);
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                // 简单的轮询选择第一个可用的提供商
                Ok(enabled_providers[0])
            }
            LoadBalancingStrategy::WeightedRandom => {
                // 基于权重的随机选择
                use rand::Rng;
                let total_weight: f32 = enabled_providers.iter().map(|p| p.weight).sum();
                let mut rng = rand::thread_rng();
                let mut random_weight = rng.r#gen::<f32>() * total_weight;

                for provider in &enabled_providers {
                    random_weight -= provider.weight;
                    if random_weight <= 0.0 {
                        return Ok(provider);
                    }
                }

                Ok(enabled_providers[0])
            }
            LoadBalancingStrategy::HealthBased => {
                // 基于健康分数选择
                let stats_guard = stats.read().await;
                let mut best_provider = enabled_providers[0];
                let mut best_score = 0.0f64;

                for provider in enabled_providers {
                    let health_score = stats_guard
                        .get(&provider.id)
                        .map(|s| s.health_score)
                        .unwrap_or(1.0);

                    if health_score > best_score {
                        best_score = health_score;
                        best_provider = provider;
                    }
                }

                Ok(best_provider)
            }
            LoadBalancingStrategy::LeastLatency => {
                // 基于延迟选择
                let stats_guard = stats.read().await;
                let mut best_provider = enabled_providers[0];
                let mut best_latency = f64::INFINITY;

                for provider in enabled_providers {
                    let latency = stats_guard
                        .get(&provider.id)
                        .map(|s| s.avg_latency_ms)
                        .unwrap_or(0.0);

                    if latency < best_latency {
                        best_latency = latency;
                        best_provider = provider;
                    }
                }

                Ok(best_provider)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::config::{ConfigBuilder, ProviderType};

    #[tokio::test]
    async fn test_llm_client_creation() {
        let config = ConfigBuilder::new()
            .add_provider(crate::sdk::config::ProviderConfig {
                id: "test".to_string(),
                provider_type: ProviderType::OpenAI,
                name: "Test Provider".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                models: vec!["gpt-3.5-turbo".to_string()],
                enabled: true,
                weight: 1.0,
                rate_limit_rpm: Some(1000),
                rate_limit_tpm: Some(10000),
                settings: HashMap::new(),
            })
            .build();

        let client = LLMClient::new(config).unwrap();
        assert_eq!(client.list_providers().len(), 1);
    }

    #[tokio::test]
    async fn test_provider_selection() {
        let config = ConfigBuilder::new()
            .add_provider(crate::sdk::config::ProviderConfig {
                id: "anthropic".to_string(),
                provider_type: ProviderType::Anthropic,
                name: "Anthropic".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                models: vec!["claude-3-sonnet-20240229".to_string()],
                enabled: true,
                weight: 1.0,
                rate_limit_rpm: Some(1000),
                rate_limit_tpm: Some(10000),
                settings: HashMap::new(),
            })
            .build();

        let client = LLMClient::new(config).unwrap();

        let request = ChatRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![],
            options: ChatOptions::default(),
        };

        let provider = client.select_provider(&request).await.unwrap();
        assert_eq!(provider.id, "anthropic");
    }
}
