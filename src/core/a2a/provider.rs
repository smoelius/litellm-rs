//! A2A Provider Adapters
//!
//! Platform-specific adapters for different agent providers.

use async_trait::async_trait;
use std::sync::Arc;

use super::config::{AgentConfig, AgentProvider};
use super::error::{A2AError, A2AResult};
use super::message::{A2AMessage, A2AResponse, Message, TaskResult};

/// Trait for A2A provider implementations
#[async_trait]
pub trait A2AProviderAdapter: Send + Sync {
    /// Get provider type
    fn provider_type(&self) -> AgentProvider;

    /// Send a message to the agent
    async fn send_message(
        &self,
        config: &AgentConfig,
        message: A2AMessage,
    ) -> A2AResult<A2AResponse>;

    /// Get task status
    async fn get_task(&self, config: &AgentConfig, task_id: &str) -> A2AResult<TaskResult>;

    /// Cancel a task
    async fn cancel_task(&self, config: &AgentConfig, task_id: &str) -> A2AResult<()>;

    /// Check if provider supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Check if provider supports async tasks
    fn supports_async_tasks(&self) -> bool {
        true
    }
}

/// Generic A2A provider (standard JSON-RPC 2.0)
pub struct GenericA2AProvider {
    client: reqwest::Client,
}

impl GenericA2AProvider {
    /// Create a new generic provider
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Create with custom HTTP client
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Build request with authentication
    fn build_request(
        &self,
        config: &AgentConfig,
        message: &A2AMessage,
    ) -> reqwest::RequestBuilder {
        let mut request = self
            .client
            .post(&config.url)
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .json(message);

        // Add API key if present
        if let Some(ref api_key) = config.api_key {
            request = request.bearer_auth(api_key);
        }

        // Add custom headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        request
    }
}

impl Default for GenericA2AProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl A2AProviderAdapter for GenericA2AProvider {
    fn provider_type(&self) -> AgentProvider {
        AgentProvider::A2A
    }

    async fn send_message(
        &self,
        config: &AgentConfig,
        message: A2AMessage,
    ) -> A2AResult<A2AResponse> {
        let response = self
            .build_request(config, &message)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    A2AError::Timeout {
                        agent_name: config.name.clone(),
                        timeout_ms: config.timeout_ms,
                    }
                } else if e.is_connect() {
                    A2AError::ConnectionError {
                        agent_name: config.name.clone(),
                        message: e.to_string(),
                    }
                } else {
                    A2AError::ProtocolError {
                        message: e.to_string(),
                    }
                }
            })?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(A2AError::AuthenticationError {
                agent_name: config.name.clone(),
                message: "Unauthorized".to_string(),
            });
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(|s| s * 1000);

            return Err(A2AError::RateLimitExceeded {
                agent_name: config.name.clone(),
                retry_after_ms: retry_after,
            });
        }

        let a2a_response: A2AResponse = response.json().await.map_err(|e| {
            A2AError::ProtocolError {
                message: format!("Failed to parse response: {}", e),
            }
        })?;

        Ok(a2a_response)
    }

    async fn get_task(&self, config: &AgentConfig, task_id: &str) -> A2AResult<TaskResult> {
        let message = A2AMessage::get_task(task_id);
        let response = self.send_message(config, message).await?;

        response.result.ok_or_else(|| {
            if let Some(error) = response.error {
                if error.code == -32001 {
                    A2AError::TaskNotFound {
                        agent_name: config.name.clone(),
                        task_id: task_id.to_string(),
                    }
                } else {
                    A2AError::ProtocolError {
                        message: error.message,
                    }
                }
            } else {
                A2AError::ProtocolError {
                    message: "Empty response".to_string(),
                }
            }
        })
    }

    async fn cancel_task(&self, config: &AgentConfig, task_id: &str) -> A2AResult<()> {
        let message = A2AMessage::cancel_task(task_id);
        let response = self.send_message(config, message).await?;

        if response.is_error() {
            if let Some(error) = response.error {
                return Err(A2AError::TaskFailed {
                    agent_name: config.name.clone(),
                    task_id: task_id.to_string(),
                    message: error.message,
                });
            }
        }

        Ok(())
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_async_tasks(&self) -> bool {
        true
    }
}

/// LangGraph provider adapter
pub struct LangGraphProvider {
    inner: GenericA2AProvider,
}

impl LangGraphProvider {
    pub fn new() -> Self {
        Self {
            inner: GenericA2AProvider::new(),
        }
    }
}

impl Default for LangGraphProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl A2AProviderAdapter for LangGraphProvider {
    fn provider_type(&self) -> AgentProvider {
        AgentProvider::LangGraph
    }

    async fn send_message(
        &self,
        config: &AgentConfig,
        message: A2AMessage,
    ) -> A2AResult<A2AResponse> {
        // LangGraph uses standard A2A protocol
        self.inner.send_message(config, message).await
    }

    async fn get_task(&self, config: &AgentConfig, task_id: &str) -> A2AResult<TaskResult> {
        self.inner.get_task(config, task_id).await
    }

    async fn cancel_task(&self, config: &AgentConfig, task_id: &str) -> A2AResult<()> {
        self.inner.cancel_task(config, task_id).await
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_async_tasks(&self) -> bool {
        true
    }
}

/// Get provider adapter for a given agent type
pub fn get_provider_adapter(provider: AgentProvider) -> Arc<dyn A2AProviderAdapter> {
    match provider {
        AgentProvider::LangGraph => Arc::new(LangGraphProvider::new()),
        AgentProvider::VertexAI => Arc::new(GenericA2AProvider::new()), // TODO: Add specific adapter
        AgentProvider::AzureAIFoundry => Arc::new(GenericA2AProvider::new()),
        AgentProvider::BedrockAgentCore => Arc::new(GenericA2AProvider::new()),
        AgentProvider::PydanticAI => Arc::new(GenericA2AProvider::new()),
        AgentProvider::A2A | AgentProvider::Custom => Arc::new(GenericA2AProvider::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_provider_creation() {
        let provider = GenericA2AProvider::new();
        assert_eq!(provider.provider_type(), AgentProvider::A2A);
        assert!(provider.supports_streaming());
        assert!(provider.supports_async_tasks());
    }

    #[test]
    fn test_langgraph_provider_creation() {
        let provider = LangGraphProvider::new();
        assert_eq!(provider.provider_type(), AgentProvider::LangGraph);
    }

    #[test]
    fn test_get_provider_adapter() {
        let adapter = get_provider_adapter(AgentProvider::LangGraph);
        assert_eq!(adapter.provider_type(), AgentProvider::LangGraph);

        let adapter = get_provider_adapter(AgentProvider::A2A);
        assert_eq!(adapter.provider_type(), AgentProvider::A2A);
    }
}
