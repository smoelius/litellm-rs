//! Types
//!
//! Defines unified data structures for all API requests

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatRequest {
    /// Model
    pub model: String,

    /// List of chat messages
    pub messages: Vec<ChatMessage>,

    /// Sampling temperature (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Maximum completion tokens (new OpenAI parameter)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// Nucleus sampling parameter (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Frequency penalty (-2.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Presence penalty (-2.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Response
    #[serde(default)]
    pub stream: bool,

    /// Tool list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Tool selection strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Parallel tool calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Seed value (for reproducible generation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,

    /// Number of choices to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// Logit bias
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f32>>,

    /// Legacy function definitions (OpenAI Functions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<serde_json::Value>>,

    /// Legacy function call (OpenAI Function Call)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<serde_json::Value>,

    /// Whether to return logprobs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Number of top logprobs to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,

    /// Additional provider-specific parameters
    #[serde(flatten)]
    pub extra_params: HashMap<String, serde_json::Value>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role
    pub role: MessageRole,

    /// Message content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,

    /// Name of message sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tool call list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,

    /// Function call (backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

impl Default for ChatMessage {
    fn default() -> Self {
        Self {
            role: MessageRole::User,
            content: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
        }
    }
}

/// Message role enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool message
    Tool,
    /// Function message (backward compatibility)
    Function,
}

impl MessageRole {
    /// Check if the message role is effectively empty
    pub fn is_empty(&self) -> bool {
        // MessageRole is always non-empty by design
        false
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::Tool => write!(f, "tool"),
            MessageRole::Function => write!(f, "function"),
        }
    }
}

/// Message content (supports multimodal)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content
    Text(String),
    /// Multi-part content (supports text, images, audio, etc.)
    Parts(Vec<ContentPart>),
}

impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageContent::Text(text) => write!(f, "{}", text),
            MessageContent::Parts(parts) => {
                let texts: Vec<String> = parts
                    .iter()
                    .filter_map(|part| match part {
                        ContentPart::Text { text } => Some(text.clone()),
                        ContentPart::ImageUrl { .. } => None,
                        ContentPart::Audio { .. } => None,
                        ContentPart::Image { .. } => None,
                        ContentPart::Document { .. } => None,
                        ContentPart::ToolResult { .. } => None,
                        ContentPart::ToolUse { .. } => None,
                    })
                    .collect();
                write!(f, "{}", texts.join(" "))
            }
        }
    }
}

/// Content part (multimodal support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },

    /// Image URL
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },

    /// Audio data
    #[serde(rename = "audio")]
    Audio { audio: AudioData },

    /// Base64 encoded image
    #[serde(rename = "image")]
    Image {
        /// Base64 encoded image data
        source: ImageSource,
        /// Image detail level
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        /// Image URL (compatibility field)
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<ImageUrl>,
    },

    /// Document content (PDF etc)
    #[serde(rename = "document")]
    Document {
        /// Document source data
        source: DocumentSource,
        /// Cache control (Anthropic specific)
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Tool result
    #[serde(rename = "tool_result")]
    ToolResult {
        /// Tool usageID
        tool_use_id: String,
        /// Result content
        content: serde_json::Value,
        /// Error
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// Tool usage
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Tool usageID
        id: String,
        /// Tool name
        name: String,
        /// Tool input
        input: serde_json::Value,
    },
}

/// Image URL structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// Image URL
    pub url: String,
    /// Detail level ("auto", "low", "high")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Image source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// Media type
    pub media_type: String,
    /// Base64 encoded data
    pub data: String,
}

/// Audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// Base64 encoded audio data
    pub data: String,
    /// Audio format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Tool type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: ToolType,

    /// Function definition
    pub function: FunctionDefinition,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Parameter JSON Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool selection strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// String selection ("auto", "none")
    String(String),
    /// Specific tool selection
    Specific {
        #[serde(rename = "type")]
        choice_type: String,
        function: Option<FunctionChoice>,
    },
}

/// Specific function selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    pub name: String,
}

/// tool_call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Call ID
    pub id: String,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call details
    pub function: FunctionCall,
}

/// Function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function parameters (JSON string)
    pub arguments: String,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// Format type ("text", "json_object", "json_schema")
    #[serde(rename = "type")]
    pub format_type: String,

    /// JSON Schema (when type is json_schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_type: Option<String>,
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// Model
    pub model: String,

    /// Input text
    pub input: EmbedInput,

    /// Encoding format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,

    /// Dimensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,

    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Embedding input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbedInput {
    /// Single string
    Single(String),
    /// String array
    Multiple(Vec<String>),
    /// Integer array (token IDs)
    TokenIds(Vec<u32>),
    /// Array of integer arrays
    MultipleTokenIds(Vec<Vec<u32>>),
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRequest {
    /// Model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Image description prompt
    pub prompt: String,

    /// Number of images to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// Image quality
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// Image size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    /// Image style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionRequest {
    /// Audio file data
    pub file: Vec<u8>,

    /// Model
    pub model: String,

    /// Language (ISO-639-1 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

impl ChatRequest {
    /// Create
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Add message
    pub fn add_message(mut self, role: MessageRole, content: impl Into<MessageContent>) -> Self {
        self.messages.push(ChatMessage {
            role,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
        });
        self
    }

    /// Add system message
    pub fn add_system_message(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::System, MessageContent::Text(content.into()))
    }

    /// Add user message
    pub fn add_user_message(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::User, MessageContent::Text(content.into()))
    }

    /// Add assistant message
    pub fn add_assistant_message(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::Assistant, MessageContent::Text(content.into()))
    }

    /// Settings
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Settings
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Response
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    /// Add tools
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Estimate input token count
    pub fn estimate_input_tokens(&self) -> u32 {
        let mut total = 0;

        // Rough estimate: each message's role and content
        for message in &self.messages {
            total += 4; // message structure overhead

            if let Some(content) = &message.content {
                match content {
                    MessageContent::Text(text) => {
                        total += (text.len() as f64 / 4.0).ceil() as u32;
                    }
                    MessageContent::Parts(parts) => {
                        for part in parts {
                            match part {
                                ContentPart::Text { text } => {
                                    total += (text.len() as f64 / 4.0).ceil() as u32;
                                }
                                ContentPart::ImageUrl { .. } => {
                                    total += 85; // fixed image token consumption
                                }
                                ContentPart::Audio { .. } => {
                                    total += 100; // estimated audio token consumption
                                }
                                ContentPart::Image { .. } => {
                                    total += 85; // fixed image token consumption
                                }
                                ContentPart::Document { .. } => {
                                    total += 1000; // estimated document token consumption
                                }
                                ContentPart::ToolResult { .. } => {
                                    total += 50; // tool result token consumption
                                }
                                ContentPart::ToolUse { .. } => {
                                    total += 100; // tool usage token consumption
                                }
                            }
                        }
                    }
                }
            }
        }

        total
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

/// Input audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudio {
    /// Base64 encoded audio data
    pub data: String,
    /// Audio format
    pub format: String,
}

/// Function call selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallChoice {
    /// Do not call function
    None,
    /// Auto decide
    Auto,
    /// Specify function
    Function {
        /// Function name
        name: String,
    },
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model
    pub model: String,
    /// Input text prompt
    pub prompt: String,
    /// Sampling temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Nucleus sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Frequency penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Presence penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Number of choices to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Response
    #[serde(default)]
    pub stream: bool,
    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model
    pub model: String,
    /// Input text or text list
    pub input: EmbeddingInput,
    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Embedding format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    /// Dimensions count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    /// Task type (for Vertex AI etc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
}

/// Embedding input type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// Single text
    Text(String),
    /// Text list
    Array(Vec<String>),
}

impl EmbeddingInput {
    /// Get
    pub fn iter(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        match self {
            EmbeddingInput::Text(text) => Box::new(std::iter::once(text)),
            EmbeddingInput::Array(texts) => Box::new(texts.iter()),
        }
    }

    /// Convert to text vector
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            EmbeddingInput::Text(text) => vec![text.clone()],
            EmbeddingInput::Array(texts) => texts.clone(),
        }
    }
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Image description prompt
    pub prompt: String,
    /// Model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Number of images to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Image size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Image quality
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

// ============================================================================
// Anthropic-specific Types
// ============================================================================

/// Document source data (Anthropic PDF support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSource {
    /// Media type (application/pdf)
    pub media_type: String,
    /// Base64 encoded data
    pub data: String,
}

/// Cache control (Anthropic Cache Control)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    /// Cache type ("ephemeral", "persistent")
    #[serde(rename = "type")]
    pub cache_type: String,
}

/// Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    /// Enable thinking mode
    pub enabled: bool,
}

/// Settings
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

/// Configuration
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

/// Request
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

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,

    /// Configuration
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
