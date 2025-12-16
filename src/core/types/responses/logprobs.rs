//! Log probability and finish reason types

use serde::{Deserialize, Serialize};

/// Finish reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop
    Stop,
    /// Length limit reached
    Length,
    /// Tool call
    ToolCalls,
    /// Content filter
    ContentFilter,
    /// Function call (backward compatibility)
    FunctionCall,
}

/// Log probabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbs {
    /// Token log probabilities
    pub content: Vec<TokenLogProb>,

    /// Refusal sampling information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

/// Single token log probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLogProb {
    /// Token text
    pub token: String,

    /// Log probability
    pub logprob: f64,

    /// Token byte representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,

    /// Top log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogProb>>,
}

/// Top log probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogProb {
    /// Token text
    pub token: String,

    /// Log probability
    pub logprob: f64,

    /// Token byte representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}
