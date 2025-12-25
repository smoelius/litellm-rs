//! A2A (Agent-to-Agent) Protocol Gateway
//!
//! This module implements A2A protocol support for litellm-rs, enabling
//! invocation and management of AI agents across multiple platforms.
//!
//! # Overview
//!
//! A2A (Agent-to-Agent) Protocol enables communication between AI agents
//! using JSON-RPC 2.0 specification. This implementation supports:
//!
//! - Multiple agent platforms (LangGraph, Vertex AI, Azure AI Foundry, etc.)
//! - Agent discovery and registration
//! - Request/response logging
//! - Access controls and load balancing
//! - Cost tracking for agent invocations
//!
//! # Supported Platforms
//!
//! - **LangGraph**: LangChain-based agent workflows
//! - **Vertex AI Agent Engine**: Google Cloud agents
//! - **Azure AI Foundry**: Microsoft AI agents
//! - **Bedrock AgentCore**: AWS agent runtime
//! - **Pydantic AI**: Python-based agents
//!
//! # Usage
//!
//! ```rust,ignore
//! use litellm_rs::core::a2a::{A2AGateway, AgentConfig, AgentProvider};
//!
//! // Configure an agent
//! let config = AgentConfig {
//!     name: "my-agent".to_string(),
//!     provider: AgentProvider::LangGraph,
//!     url: "https://my-agent.example.com".to_string(),
//!     ..Default::default()
//! };
//!
//! // Create gateway and register agent
//! let gateway = A2AGateway::new();
//! gateway.register_agent(config).await?;
//!
//! // Invoke the agent
//! let response = gateway.send_message("my-agent", message).await?;
//! ```

pub mod config;
pub mod error;
pub mod gateway;
pub mod message;
pub mod provider;
pub mod registry;

// Re-export commonly used types
pub use config::{AgentConfig, AgentProvider};
pub use error::{A2AError, A2AResult};
pub use gateway::A2AGateway;
pub use message::{A2AMessage, A2AResponse, MessagePart, TaskState};
pub use registry::AgentRegistry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all public types are accessible
        let _ = AgentProvider::LangGraph;
        let _ = TaskState::Pending;
    }
}
