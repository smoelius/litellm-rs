//! Bedrock Agents Module
//!
//! Handles agent invocation, session management, and tool calling

use crate::core::providers::unified_provider::ProviderError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Agent invocation request
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInvocationRequest {
    pub agent_id: String,
    pub agent_alias_id: String,
    pub session_id: String,
    pub input_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_state: Option<SessionState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_trace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_session: Option<bool>,
}

/// Session state
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub session_attributes: Option<Value>,
    pub prompt_session_attributes: Option<Value>,
}

/// Agent invocation response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInvocationResponse {
    pub completion: AgentCompletion,
    pub session_id: String,
    pub session_state: Option<SessionState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<AgentTrace>,
}

/// Agent completion
#[derive(Debug, Deserialize)]
pub struct AgentCompletion {
    pub text: String,
}

/// Agent trace
#[derive(Debug, Deserialize)]
pub struct AgentTrace {
    pub traces: Vec<TraceEntry>,
}

/// Trace entry
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceEntry {
    pub trace_id: String,
    pub trace_type: String,
    pub trace_data: Value,
}

/// Agent client for managing agent interactions
pub struct AgentClient<'a> {
    client: &'a crate::core::providers::bedrock::client::BedrockClient,
}

impl<'a> AgentClient<'a> {
    /// Create a new agent client
    pub fn new(client: &'a crate::core::providers::bedrock::client::BedrockClient) -> Self {
        Self { client }
    }

    /// Invoke an agent
    pub async fn invoke(
        &self,
        agent_id: &str,
        agent_alias_id: &str,
        session_id: &str,
        input_text: &str,
        enable_trace: bool,
    ) -> Result<AgentInvocationResponse, ProviderError> {
        let request = AgentInvocationRequest {
            agent_id: agent_id.to_string(),
            agent_alias_id: agent_alias_id.to_string(),
            session_id: session_id.to_string(),
            input_text: input_text.to_string(),
            session_state: None,
            enable_trace: Some(enable_trace),
            end_session: None,
        };

        let url = format!(
            "agents/{}/agentAliases/{}/sessions/{}/text",
            agent_id, agent_alias_id, session_id
        );

        let response = self
            .client
            .send_request("", &url, &serde_json::to_value(request)?)
            .await?;
        let agent_response: AgentInvocationResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("bedrock", e.to_string()))?;

        Ok(agent_response)
    }
}
