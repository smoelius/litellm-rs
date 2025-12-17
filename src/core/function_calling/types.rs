//! Type definitions for function calling
//!
//! This module contains all the core types used in function calling.

use serde_json::Value;

/// Function definition for AI models
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Function parameters schema (JSON Schema)
    pub parameters: Value,
    /// Whether the function is strict (OpenAI specific)
    pub strict: Option<bool>,
}

/// Tool definition for AI models
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    /// Tool type (currently only "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition
    pub function: FunctionDefinition,
}

/// Tool choice options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tools should be called
    None,
    /// Let the model decide
    Auto,
    /// Force a specific tool
    Required,
    /// Specific tool to use
    Specific {
        /// Tool type identifier
        #[serde(rename = "type")]
        tool_type: String,
        /// Function choice details
        function: FunctionChoice,
    },
}

/// Specific function choice
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionChoice {
    /// Function name to call
    pub name: String,
}

/// Function call in a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments (JSON string)
    pub arguments: String,
}

/// Tool call in a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call details
    pub function: FunctionCall,
}
