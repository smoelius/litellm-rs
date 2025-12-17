use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supports_function_calling: bool,
    pub supports_parallel_function_calling: bool,
    pub supports_tool_choice: bool,
    pub supports_response_schema: bool,
    pub supports_system_messages: bool,
    pub supports_web_search: bool,
    pub supports_url_context: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
    pub max_tokens: Option<usize>,
    pub context_window: Option<usize>,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            supports_function_calling: false,
            supports_parallel_function_calling: false,
            supports_tool_choice: false,
            supports_response_schema: false,
            supports_system_messages: true,
            supports_web_search: false,
            supports_url_context: false,
            supports_vision: false,
            supports_streaming: true,
            max_tokens: None,
            context_window: None,
        }
    }
}
