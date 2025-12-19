//! Anthropic-specific request types

use super::chat::ChatRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Anthropic-specific thinking configuration (legacy)
///
/// Note: For the unified thinking config, use `crate::core::types::thinking::ThinkingConfig`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicThinkingConfig {
    /// Enable thinking mode
    pub enabled: bool,
}

/// Computer tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerToolConfig {
    /// Screen width
    pub display_width: u32,
    /// Screen height
    pub display_height: u32,
    /// Display density
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_density: Option<u32>,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name
    pub name: String,
    /// Server endpoint
    pub endpoint: String,
    /// Authentication info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<serde_json::Value>,
}

/// Anthropic request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicRequestParams {
    /// System message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Top K sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<AnthropicMetadata>,
    /// Thinking configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<AnthropicThinkingConfig>,
    /// Computer use configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computer_use: Option<ComputerToolConfig>,
    /// MCP server list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<McpServerConfig>>,
}

/// Anthropic metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMetadata {
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Custom data
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Enhanced ChatRequest to support Anthropic features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicChatRequest {
    #[serde(flatten)]
    pub base: ChatRequest,
    #[serde(flatten)]
    pub anthropic_params: AnthropicRequestParams,
}
