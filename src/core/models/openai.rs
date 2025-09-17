//! OpenAI-compatible API models
//!
//! This module defines all the data structures that are compatible with OpenAI's API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: Option<MessageContent>,
    /// Message name (for function/tool messages)
    pub name: Option<String>,
    /// Function call (legacy)
    pub function_call: Option<FunctionCall>,
    /// Tool calls
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Tool call ID (for tool messages)
    pub tool_call_id: Option<String>,
    /// Audio content
    pub audio: Option<AudioContent>,
}

/// Message role
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message role
    System,
    /// User message role
    User,
    /// Assistant message role
    Assistant,
    /// Function call message role
    Function,
    /// Tool call message role
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::Function => write!(f, "function"),
            MessageRole::Tool => write!(f, "tool"),
        }
    }
}

/// Message content (can be string or array of content parts)
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content
    Text(String),
    /// Multi-part content (text, images, audio)
    Parts(Vec<ContentPart>),
}

/// Content part for multimodal messages
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content part
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
    /// Image URL content part
    #[serde(rename = "image_url")]
    ImageUrl {
        /// Image URL details
        image_url: ImageUrl,
    },
    /// Audio content part
    #[serde(rename = "audio")]
    Audio {
        /// Audio content details
        audio: AudioContent,
    },
}

/// Image URL content
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ImageUrl {
    /// Image URL
    pub url: String,
    /// Detail level
    pub detail: Option<String>,
}

/// Audio content
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct AudioContent {
    /// Audio data (base64 encoded)
    pub data: String,
    /// Audio format
    pub format: String,
}

/// Stream options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    /// Include usage in stream
    pub include_usage: Option<bool>,
}

/// Function definition (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Function parameters schema
    pub parameters: Option<serde_json::Value>,
}

/// Function call (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments (JSON string)
    pub arguments: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition
    pub function: Function,
}

/// Tool choice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tool calls allowed
    None(String), // "none"
    /// Automatic tool selection
    Auto(String), // "auto"
    /// Tool calls required
    Required(String), // "required"
    /// Specific tool to use
    Specific(ToolChoiceFunction),
}

/// Specific tool choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function specification
    pub function: ToolChoiceFunctionSpec,
}

/// Tool choice function specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunctionSpec {
    /// Function name
    pub name: String,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call
    pub function: FunctionCall,
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

/// Audio parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParams {
    /// Voice to use
    pub voice: String,
    /// Audio format
    pub format: String,
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Response ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used
    pub model: String,
    /// System fingerprint
    pub system_fingerprint: Option<String>,
    /// Choices
    pub choices: Vec<ChatChoice>,
    /// Usage statistics
    pub usage: Option<Usage>,
}

/// Chat choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// Choice index
    pub index: u32,
    /// Message
    pub message: ChatMessage,
    /// Logprobs
    pub logprobs: Option<Logprobs>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Logprobs information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logprobs {
    /// Content logprobs
    pub content: Option<Vec<ContentLogprob>>,
}

/// Content logprob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLogprob {
    /// Token
    pub token: String,
    /// Log probability
    pub logprob: f64,
    /// Bytes
    pub bytes: Option<Vec<u8>>,
    /// Top logprobs
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

/// Top logprob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogprob {
    /// Token
    pub token: String,
    /// Log probability
    pub logprob: f64,
    /// Bytes
    pub bytes: Option<Vec<u8>>,
}

/// Usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Completion tokens
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
    /// Prompt tokens details
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    /// Completion tokens details
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Prompt tokens details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    /// Cached tokens
    pub cached_tokens: Option<u32>,
    /// Audio tokens
    pub audio_tokens: Option<u32>,
}

/// Completion tokens details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    /// Reasoning tokens
    pub reasoning_tokens: Option<u32>,
    /// Audio tokens
    pub audio_tokens: Option<u32>,
}

/// Chat completion chunk (for streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Response ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used
    pub model: String,
    /// System fingerprint
    pub system_fingerprint: Option<String>,
    /// Choices
    pub choices: Vec<ChatChoiceDelta>,
    /// Usage statistics (only in final chunk)
    pub usage: Option<Usage>,
}

/// Chat choice delta (for streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoiceDelta {
    /// Choice index
    pub index: u32,
    /// Delta message
    pub delta: ChatMessageDelta,
    /// Logprobs
    pub logprobs: Option<Logprobs>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Chat message delta (for streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageDelta {
    /// Message role (only in first chunk)
    pub role: Option<MessageRole>,
    /// Content delta
    pub content: Option<String>,
    /// Function call delta (legacy)
    pub function_call: Option<FunctionCallDelta>,
    /// Tool calls delta
    pub tool_calls: Option<Vec<ToolCallDelta>>,
    /// Audio delta
    pub audio: Option<AudioDelta>,
}

/// Function call delta (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    /// Function name
    pub name: Option<String>,
    /// Function arguments delta
    pub arguments: Option<String>,
}

/// Tool call delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    /// Tool call index
    pub index: u32,
    /// Tool call ID
    pub id: Option<String>,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    /// Function call delta
    pub function: Option<FunctionCallDelta>,
}

/// Audio delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDelta {
    /// Audio data delta
    pub data: Option<String>,
    /// Transcript delta
    pub transcript: Option<String>,
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

/// Text completion response
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
    /// Completion choices
    pub choices: Vec<CompletionChoice>,
    /// Usage statistics
    pub usage: Option<Usage>,
}

/// Completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// Generated text
    pub text: String,
    /// Choice index
    pub index: u32,
    /// Log probabilities
    pub logprobs: Option<serde_json::Value>,
    /// Finish reason
    pub finish_reason: Option<String>,
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

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Object type
    pub object: String,
    /// Embedding data
    pub data: Vec<EmbeddingObject>,
    /// Model used
    pub model: String,
    /// Usage statistics
    pub usage: EmbeddingUsage,
}

/// Embedding object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingObject {
    /// Object type
    pub object: String,
    /// Embedding vector
    pub embedding: Vec<f64>,
    /// Index
    pub index: u32,
}

/// Embedding usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
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

/// Image generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Creation timestamp
    pub created: u64,
    /// Generated images
    pub data: Vec<ImageObject>,
}

/// Image object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    /// Image URL
    pub url: Option<String>,
    /// Base64 encoded image
    pub b64_json: Option<String>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Model ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Owner
    pub owned_by: String,
}

/// Model list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelListResponse {
    /// Object type
    pub object: String,
    /// List of models
    pub data: Vec<Model>,
}

/// Chat completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    /// Choice index
    pub index: u32,
    /// Message content
    pub message: ChatMessage,
    /// Finish reason
    pub finish_reason: Option<String>,
    /// Log probabilities
    pub logprobs: Option<serde_json::Value>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_completion_request_serialization() {
        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            }],
            temperature: Some(0.7),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_message_content_variants() {
        let text_content = MessageContent::Text("Hello".to_string());
        let json = serde_json::to_string(&text_content).unwrap();
        assert_eq!(json, "\"Hello\"");

        let parts_content = MessageContent::Parts(vec![ContentPart::Text {
            text: "Hello".to_string(),
        }]);
        let json = serde_json::to_string(&parts_content).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("Hello"));
    }
}
