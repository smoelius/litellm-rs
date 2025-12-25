//! A2A Agent Registry
//!
//! Registry for managing and discovering agents.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::AgentConfig;
use super::error::{A2AError, A2AResult};

/// Agent registry entry
#[derive(Debug, Clone)]
pub struct AgentEntry {
    /// Agent configuration
    pub config: AgentConfig,

    /// Agent state
    pub state: AgentState,

    /// Last health check timestamp
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,

    /// Total invocation count
    pub invocation_count: u64,

    /// Total cost (USD)
    pub total_cost: f64,
}

impl AgentEntry {
    /// Create a new agent entry
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            state: AgentState::Unknown,
            last_health_check: None,
            invocation_count: 0,
            total_cost: 0.0,
        }
    }

    /// Update invocation stats
    pub fn record_invocation(&mut self, cost: f64) {
        self.invocation_count += 1;
        self.total_cost += cost;
    }
}

/// Agent state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentState {
    /// State unknown (not checked)
    #[default]
    Unknown,

    /// Agent is healthy
    Healthy,

    /// Agent is degraded (slow response)
    Degraded,

    /// Agent is unhealthy (errors)
    Unhealthy,

    /// Agent is disabled
    Disabled,
}

impl AgentState {
    /// Check if agent is available for requests
    pub fn is_available(&self) -> bool {
        matches!(self, AgentState::Healthy | AgentState::Degraded | AgentState::Unknown)
    }
}

/// Agent registry for discovery and management
#[derive(Debug)]
pub struct AgentRegistry {
    /// Registered agents
    agents: RwLock<HashMap<String, AgentEntry>>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    /// Register an agent
    pub async fn register(&self, config: AgentConfig) -> A2AResult<()> {
        let name = config.name.clone();

        config.validate().map_err(|e| A2AError::ConfigurationError { message: e })?;

        let mut agents = self.agents.write().await;
        if agents.contains_key(&name) {
            return Err(A2AError::AgentAlreadyExists { agent_name: name });
        }

        agents.insert(name, AgentEntry::new(config));
        Ok(())
    }

    /// Unregister an agent
    pub async fn unregister(&self, name: &str) -> Option<AgentEntry> {
        self.agents.write().await.remove(name)
    }

    /// Get an agent by name
    pub async fn get(&self, name: &str) -> Option<AgentEntry> {
        self.agents.read().await.get(name).cloned()
    }

    /// Get agent configuration
    pub async fn get_config(&self, name: &str) -> Option<AgentConfig> {
        self.agents.read().await.get(name).map(|e| e.config.clone())
    }

    /// Update agent state
    pub async fn update_state(&self, name: &str, state: AgentState) {
        if let Some(entry) = self.agents.write().await.get_mut(name) {
            entry.state = state;
            entry.last_health_check = Some(chrono::Utc::now());
        }
    }

    /// Record an invocation
    pub async fn record_invocation(&self, name: &str, cost: f64) {
        if let Some(entry) = self.agents.write().await.get_mut(name) {
            entry.record_invocation(cost);
        }
    }

    /// List all agent names
    pub async fn list_names(&self) -> Vec<String> {
        self.agents.read().await.keys().cloned().collect()
    }

    /// List all available agents (healthy or unknown state)
    pub async fn list_available(&self) -> Vec<AgentConfig> {
        self.agents
            .read()
            .await
            .values()
            .filter(|e| e.state.is_available() && e.config.enabled)
            .map(|e| e.config.clone())
            .collect()
    }

    /// List agents by tags
    pub async fn list_by_tag(&self, tag: &str) -> Vec<AgentConfig> {
        self.agents
            .read()
            .await
            .values()
            .filter(|e| e.config.tags.contains(&tag.to_string()))
            .map(|e| e.config.clone())
            .collect()
    }

    /// Get agent count
    pub async fn count(&self) -> usize {
        self.agents.read().await.len()
    }

    /// Get registry statistics
    pub async fn stats(&self) -> RegistryStats {
        let agents = self.agents.read().await;

        let mut stats = RegistryStats::default();
        stats.total_agents = agents.len();

        for entry in agents.values() {
            match entry.state {
                AgentState::Healthy => stats.healthy_agents += 1,
                AgentState::Degraded => stats.degraded_agents += 1,
                AgentState::Unhealthy => stats.unhealthy_agents += 1,
                AgentState::Disabled => stats.disabled_agents += 1,
                AgentState::Unknown => stats.unknown_agents += 1,
            }

            if entry.config.enabled {
                stats.enabled_agents += 1;
            }

            stats.total_invocations += entry.invocation_count;
            stats.total_cost += entry.total_cost;
        }

        stats
    }
}

/// Registry statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct RegistryStats {
    /// Total registered agents
    pub total_agents: usize,
    /// Enabled agents
    pub enabled_agents: usize,
    /// Healthy agents
    pub healthy_agents: usize,
    /// Degraded agents
    pub degraded_agents: usize,
    /// Unhealthy agents
    pub unhealthy_agents: usize,
    /// Disabled agents
    pub disabled_agents: usize,
    /// Unknown state agents
    pub unknown_agents: usize,
    /// Total invocations across all agents
    pub total_invocations: u64,
    /// Total cost across all agents
    pub total_cost: f64,
}

/// Thread-safe registry handle
pub type AgentRegistryHandle = Arc<AgentRegistry>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_register_agent() {
        let registry = AgentRegistry::new();

        let config = AgentConfig::new("test-agent", "https://example.com/agent");
        registry.register(config).await.unwrap();

        assert_eq!(registry.count().await, 1);
        assert!(registry.get("test-agent").await.is_some());
    }

    #[tokio::test]
    async fn test_register_duplicate() {
        let registry = AgentRegistry::new();

        let config = AgentConfig::new("test-agent", "https://example.com/agent");
        registry.register(config.clone()).await.unwrap();

        let result = registry.register(config).await;
        assert!(matches!(result, Err(A2AError::AgentAlreadyExists { .. })));
    }

    #[tokio::test]
    async fn test_unregister_agent() {
        let registry = AgentRegistry::new();

        let config = AgentConfig::new("test-agent", "https://example.com/agent");
        registry.register(config).await.unwrap();

        let removed = registry.unregister("test-agent").await;
        assert!(removed.is_some());
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_update_state() {
        let registry = AgentRegistry::new();

        let config = AgentConfig::new("test-agent", "https://example.com/agent");
        registry.register(config).await.unwrap();

        registry.update_state("test-agent", AgentState::Healthy).await;

        let entry = registry.get("test-agent").await.unwrap();
        assert_eq!(entry.state, AgentState::Healthy);
        assert!(entry.last_health_check.is_some());
    }

    #[tokio::test]
    async fn test_record_invocation() {
        let registry = AgentRegistry::new();

        let config = AgentConfig::new("test-agent", "https://example.com/agent");
        registry.register(config).await.unwrap();

        registry.record_invocation("test-agent", 0.01).await;
        registry.record_invocation("test-agent", 0.02).await;

        let entry = registry.get("test-agent").await.unwrap();
        assert_eq!(entry.invocation_count, 2);
        assert!((entry.total_cost - 0.03).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_list_available() {
        let registry = AgentRegistry::new();

        // Add enabled agent
        let config1 = AgentConfig::new("agent1", "https://example.com/agent1");
        registry.register(config1).await.unwrap();
        registry.update_state("agent1", AgentState::Healthy).await;

        // Add disabled agent
        let mut config2 = AgentConfig::new("agent2", "https://example.com/agent2");
        config2.enabled = false;
        registry.register(config2).await.unwrap();

        let available = registry.list_available().await;
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].name, "agent1");
    }

    #[tokio::test]
    async fn test_list_by_tag() {
        let registry = AgentRegistry::new();

        let mut config1 = AgentConfig::new("agent1", "https://example.com/agent1");
        config1.tags = vec!["production".to_string()];
        registry.register(config1).await.unwrap();

        let mut config2 = AgentConfig::new("agent2", "https://example.com/agent2");
        config2.tags = vec!["staging".to_string()];
        registry.register(config2).await.unwrap();

        let production = registry.list_by_tag("production").await;
        assert_eq!(production.len(), 1);
        assert_eq!(production[0].name, "agent1");
    }

    #[tokio::test]
    async fn test_registry_stats() {
        let registry = AgentRegistry::new();

        let config1 = AgentConfig::new("agent1", "https://example.com/agent1");
        registry.register(config1).await.unwrap();
        registry.update_state("agent1", AgentState::Healthy).await;
        registry.record_invocation("agent1", 0.10).await;

        let config2 = AgentConfig::new("agent2", "https://example.com/agent2");
        registry.register(config2).await.unwrap();
        registry.update_state("agent2", AgentState::Unhealthy).await;

        let stats = registry.stats().await;
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.healthy_agents, 1);
        assert_eq!(stats.unhealthy_agents, 1);
        assert_eq!(stats.total_invocations, 1);
    }

    #[test]
    fn test_agent_state_availability() {
        assert!(AgentState::Healthy.is_available());
        assert!(AgentState::Degraded.is_available());
        assert!(AgentState::Unknown.is_available());
        assert!(!AgentState::Unhealthy.is_available());
        assert!(!AgentState::Disabled.is_available());
    }
}
