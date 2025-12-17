//! Completion response types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::openai::Usage;

/// Completion response (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Response ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used
    pub model: String,
    /// Choices
    pub choices: Vec<CompletionChoice>,
    /// Usage statistics
    pub usage: Option<Usage>,
}

/// Completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// Choice index
    pub index: u32,
    /// Generated text
    pub text: String,
    /// Logprobs
    pub logprobs: Option<CompletionLogprobs>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Completion logprobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionLogprobs {
    /// Tokens
    pub tokens: Vec<String>,
    /// Token logprobs
    pub token_logprobs: Vec<f64>,
    /// Top logprobs
    pub top_logprobs: Vec<HashMap<String, f64>>,
    /// Text offset
    pub text_offset: Vec<u32>,
}
