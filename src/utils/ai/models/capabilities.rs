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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_capabilities_default() {
        let caps = ModelCapabilities::default();
        assert!(!caps.supports_function_calling);
        assert!(!caps.supports_parallel_function_calling);
        assert!(!caps.supports_tool_choice);
        assert!(!caps.supports_response_schema);
        assert!(caps.supports_system_messages);
        assert!(!caps.supports_web_search);
        assert!(!caps.supports_url_context);
        assert!(!caps.supports_vision);
        assert!(caps.supports_streaming);
        assert!(caps.max_tokens.is_none());
        assert!(caps.context_window.is_none());
    }

    #[test]
    fn test_model_capabilities_custom() {
        let caps = ModelCapabilities {
            supports_function_calling: true,
            supports_parallel_function_calling: true,
            supports_tool_choice: true,
            supports_response_schema: true,
            supports_system_messages: true,
            supports_web_search: true,
            supports_url_context: true,
            supports_vision: true,
            supports_streaming: true,
            max_tokens: Some(4096),
            context_window: Some(128000),
        };
        assert!(caps.supports_function_calling);
        assert!(caps.supports_vision);
        assert_eq!(caps.max_tokens, Some(4096));
        assert_eq!(caps.context_window, Some(128000));
    }

    #[test]
    fn test_model_capabilities_clone() {
        let caps = ModelCapabilities {
            supports_function_calling: true,
            supports_vision: true,
            max_tokens: Some(8192),
            ..ModelCapabilities::default()
        };
        let cloned = caps.clone();
        assert_eq!(caps.supports_function_calling, cloned.supports_function_calling);
        assert_eq!(caps.max_tokens, cloned.max_tokens);
    }

    #[test]
    fn test_model_capabilities_serialization() {
        let caps = ModelCapabilities {
            supports_function_calling: true,
            max_tokens: Some(4096),
            ..ModelCapabilities::default()
        };
        let json = serde_json::to_value(&caps).unwrap();
        assert_eq!(json["supports_function_calling"], true);
        assert_eq!(json["max_tokens"], 4096);
        assert_eq!(json["supports_streaming"], true);
    }

    #[test]
    fn test_model_capabilities_deserialization() {
        let json = r#"{
            "supports_function_calling": true,
            "supports_parallel_function_calling": true,
            "supports_tool_choice": true,
            "supports_response_schema": false,
            "supports_system_messages": true,
            "supports_web_search": false,
            "supports_url_context": false,
            "supports_vision": true,
            "supports_streaming": true,
            "max_tokens": 8192,
            "context_window": 200000
        }"#;
        let caps: ModelCapabilities = serde_json::from_str(json).unwrap();
        assert!(caps.supports_function_calling);
        assert!(caps.supports_vision);
        assert_eq!(caps.max_tokens, Some(8192));
        assert_eq!(caps.context_window, Some(200000));
    }

    #[test]
    fn test_model_capabilities_partial_deserialization() {
        let json = r#"{
            "supports_function_calling": true,
            "supports_parallel_function_calling": false,
            "supports_tool_choice": false,
            "supports_response_schema": false,
            "supports_system_messages": true,
            "supports_web_search": false,
            "supports_url_context": false,
            "supports_vision": false,
            "supports_streaming": true
        }"#;
        let caps: ModelCapabilities = serde_json::from_str(json).unwrap();
        assert!(caps.max_tokens.is_none());
        assert!(caps.context_window.is_none());
    }
}
