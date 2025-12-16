//! Request types for OpenAI-compatible API
//!
//! This module defines request structures for chat completions, text completions,
//! embeddings, and image generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::audio::AudioParams;
use super::messages::ChatMessage;
use super::tools::{Function, FunctionCall, Tool, ToolChoice};

/// Chat completion request (OpenAI compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model to use for completion
    pub model: String,
    /// List of messages
    pub messages: Vec<ChatMessage>,
    /// Temperature (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Maximum completion tokens (newer parameter)
    pub max_completion_tokens: Option<u32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Number of completions to generate
    pub n: Option<u32>,
    /// Whether to stream the response
    pub stream: Option<bool>,
    /// Stream options
    pub stream_options: Option<StreamOptions>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Logit bias
    pub logit_bias: Option<HashMap<String, f32>>,
    /// User identifier
    pub user: Option<String>,
    /// Function calling (legacy)
    pub functions: Option<Vec<Function>>,
    /// Function call (legacy)
    pub function_call: Option<FunctionCall>,
    /// Tools for function calling
    pub tools: Option<Vec<Tool>>,
    /// Tool choice
    pub tool_choice: Option<ToolChoice>,
    /// Response format
    pub response_format: Option<ResponseFormat>,
    /// Seed for deterministic outputs
    pub seed: Option<u32>,
    /// Logprobs
    pub logprobs: Option<bool>,
    /// Top logprobs
    pub top_logprobs: Option<u32>,
    /// Modalities (for multimodal models)
    pub modalities: Option<Vec<String>>,
    /// Audio parameters
    pub audio: Option<AudioParams>,
}

impl Default for ChatCompletionRequest {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            max_completion_tokens: None,
            top_p: None,
            n: None,
            stream: None,
            stream_options: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            functions: None,
            function_call: None,
            tools: None,
            tool_choice: None,
            response_format: None,
            seed: None,
            logprobs: None,
            top_logprobs: None,
            modalities: None,
            audio: None,
        }
    }
}

/// Stream options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    /// Include usage in stream
    pub include_usage: Option<bool>,
}

/// Response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// Format type
    #[serde(rename = "type")]
    pub format_type: String,
    /// JSON schema (for structured outputs)
    pub json_schema: Option<serde_json::Value>,
}

/// Text completion request (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use
    pub model: String,
    /// Prompt text
    pub prompt: String,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature
    pub temperature: Option<f64>,
    /// Top-p
    pub top_p: Option<f64>,
    /// Number of completions
    pub n: Option<u32>,
    /// Stream response
    pub stream: Option<bool>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Presence penalty
    pub presence_penalty: Option<f64>,
    /// Frequency penalty
    pub frequency_penalty: Option<f64>,
    /// Logit bias
    pub logit_bias: Option<HashMap<String, f64>>,
    /// User identifier
    pub user: Option<String>,
    /// Include the log probabilities
    pub logprobs: Option<u32>,
    /// Echo back the prompt
    pub echo: Option<bool>,
}

/// Embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model to use
    pub model: String,
    /// Input text or array of texts
    pub input: serde_json::Value,
    /// User identifier
    pub user: Option<String>,
}

/// Image generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Prompt for image generation
    pub prompt: String,
    /// Model to use
    pub model: Option<String>,
    /// Number of images
    pub n: Option<u32>,
    /// Image size
    pub size: Option<String>,
    /// Response format
    pub response_format: Option<String>,
    /// User identifier
    pub user: Option<String>,
}
