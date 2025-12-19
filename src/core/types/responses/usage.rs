//! Usage statistics types

use serde::{Deserialize, Serialize};

use super::super::thinking::ThinkingUsage;

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    /// Prompt token count
    pub prompt_tokens: u32,

    /// Completion token count
    pub completion_tokens: u32,

    /// Total token count
    pub total_tokens: u32,

    /// Prompt token details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    /// Completion token details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,

    /// Thinking/reasoning usage statistics
    ///
    /// Contains detailed breakdown of thinking tokens and costs
    /// for thinking-enabled models (OpenAI o-series, Claude thinking,
    /// DeepSeek R1, Gemini thinking).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_usage: Option<ThinkingUsage>,
}

impl Usage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        }
    }

    /// Create usage with thinking statistics
    pub fn with_thinking(mut self, thinking: ThinkingUsage) -> Self {
        self.thinking_usage = Some(thinking);
        self
    }

    /// Get thinking tokens count (convenience method)
    pub fn thinking_tokens(&self) -> Option<u32> {
        self.thinking_usage
            .as_ref()
            .and_then(|t| t.thinking_tokens)
            .or_else(|| {
                // Fallback to completion_tokens_details.reasoning_tokens
                self.completion_tokens_details
                    .as_ref()
                    .and_then(|d| d.reasoning_tokens)
            })
    }
}

/// Prompt token details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    /// Cached token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,

    /// Audio token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}

/// Completion token details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    /// Reasoning token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,

    /// Audio token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}
