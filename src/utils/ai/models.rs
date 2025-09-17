use crate::core::providers::unified_provider::ProviderError;
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

pub struct ModelUtils;

impl ModelUtils {
    pub fn get_model_capabilities(model: &str) -> ModelCapabilities {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-4") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: true,
                supports_tool_choice: true,
                supports_response_schema: true,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: true,
                supports_vision: model_lower.contains("vision") || model_lower.contains("turbo"),
                supports_streaming: true,
                max_tokens: Some(if model_lower.contains("32k") {
                    32768
                } else {
                    8192
                }),
                context_window: Some(if model_lower.contains("32k") {
                    32768
                } else {
                    8192
                }),
            }
        } else if model_lower.starts_with("gpt-3.5") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: false,
                supports_tool_choice: true,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: false,
                supports_vision: false,
                supports_streaming: true,
                max_tokens: Some(if model_lower.contains("16k") {
                    16384
                } else {
                    4096
                }),
                context_window: Some(if model_lower.contains("16k") {
                    16384
                } else {
                    4096
                }),
            }
        } else if model_lower.starts_with("claude-3") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: false,
                supports_tool_choice: true,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: true,
                supports_vision: true,
                supports_streaming: true,
                max_tokens: Some(200000),
                context_window: Some(200000),
            }
        } else if model_lower.starts_with("claude-2") || model_lower.starts_with("claude-instant") {
            ModelCapabilities {
                supports_function_calling: false,
                supports_parallel_function_calling: false,
                supports_tool_choice: false,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: false,
                supports_vision: false,
                supports_streaming: true,
                max_tokens: Some(100000),
                context_window: Some(100000),
            }
        } else if model_lower.starts_with("gemini") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: false,
                supports_tool_choice: false,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: true,
                supports_url_context: true,
                supports_vision: model_lower.contains("vision") || model_lower.contains("pro"),
                supports_streaming: true,
                max_tokens: Some(32768),
                context_window: Some(32768),
            }
        } else {
            ModelCapabilities::default()
        }
    }

    pub fn supports_function_calling(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_function_calling
    }

    pub fn supports_parallel_function_calling(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_parallel_function_calling
    }

    pub fn supports_tool_choice(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_tool_choice
    }

    pub fn supports_response_schema(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_response_schema
    }

    pub fn supports_system_messages(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_system_messages
    }

    pub fn supports_web_search(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_web_search
    }

    pub fn supports_url_context(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_url_context
    }

    pub fn supports_vision(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_vision
    }

    pub fn supports_streaming(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_streaming
    }

    pub fn get_provider_from_model(model: &str) -> Option<String> {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-") || model_lower.contains("openai") {
            Some("openai".to_string())
        } else if model_lower.starts_with("claude-") || model_lower.contains("anthropic") {
            Some("anthropic".to_string())
        } else if model_lower.starts_with("gemini-") || model_lower.contains("google") {
            Some("google".to_string())
        } else if model_lower.starts_with("command") || model_lower.contains("cohere") {
            Some("cohere".to_string())
        } else if model_lower.contains("mistral") {
            Some("mistral".to_string())
        } else if model_lower.contains("llama") {
            Some("meta".to_string())
        } else {
            None
        }
    }

    pub fn get_base_model(model: &str) -> String {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-4") {
            if model_lower.contains("32k") {
                "gpt-4-32k".to_string()
            } else if model_lower.contains("turbo") {
                "gpt-4-turbo".to_string()
            } else {
                "gpt-4".to_string()
            }
        } else if model_lower.starts_with("gpt-3.5") {
            if model_lower.contains("16k") {
                "gpt-3.5-turbo-16k".to_string()
            } else {
                "gpt-3.5-turbo".to_string()
            }
        } else if model_lower.starts_with("claude-3") {
            if model_lower.contains("opus") {
                "claude-3-opus".to_string()
            } else if model_lower.contains("sonnet") {
                "claude-3-sonnet".to_string()
            } else if model_lower.contains("haiku") {
                "claude-3-haiku".to_string()
            } else {
                "claude-3".to_string()
            }
        } else {
            model.to_string()
        }
    }

    pub fn is_valid_model(model: &str) -> bool {
        let known_providers = [
            "openai",
            "anthropic",
            "google",
            "cohere",
            "mistral",
            "meta",
            "azure",
            "replicate",
        ];

        let known_models = [
            "gpt-4",
            "gpt-3.5-turbo",
            "claude-3",
            "claude-2",
            "gemini",
            "command",
            "mistral",
        ];

        let model_lower = model.to_lowercase();

        for provider in &known_providers {
            if model_lower.contains(provider) {
                return true;
            }
        }

        for base_model in &known_models {
            if model_lower.starts_with(base_model) {
                return true;
            }
        }

        false
    }

    pub fn get_model_family(model: &str) -> String {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-") {
            "gpt".to_string()
        } else if model_lower.starts_with("claude-") {
            "claude".to_string()
        } else if model_lower.starts_with("gemini-") {
            "gemini".to_string()
        } else if model_lower.starts_with("command") {
            "command".to_string()
        } else if model_lower.contains("llama") {
            "llama".to_string()
        } else if model_lower.contains("mistral") {
            "mistral".to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub fn get_model_pricing(model: &str) -> Option<(f64, f64)> {
        let model_lower = model.to_lowercase();

        match model_lower.as_str() {
            m if m.starts_with("gpt-4-turbo") => Some((0.01, 0.03)),
            m if m.starts_with("gpt-4") => Some((0.03, 0.06)),
            m if m.starts_with("gpt-3.5-turbo") => Some((0.0015, 0.002)),
            m if m.contains("claude-3-opus") => Some((0.015, 0.075)),
            m if m.contains("claude-3-sonnet") => Some((0.003, 0.015)),
            m if m.contains("claude-3-haiku") => Some((0.00025, 0.00125)),
            m if m.starts_with("gemini-pro") => Some((0.0005, 0.0015)),
            _ => None,
        }
    }

    pub fn get_compatible_models_for_provider(provider: &str) -> Vec<String> {
        match provider.to_lowercase().as_str() {
            "openai" => vec![
                "gpt-4".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-4-32k".to_string(),
                "gpt-3.5-turbo".to_string(),
                "gpt-3.5-turbo-16k".to_string(),
            ],
            "anthropic" => vec![
                "claude-3-opus".to_string(),
                "claude-3-sonnet".to_string(),
                "claude-3-haiku".to_string(),
                "claude-2".to_string(),
                "claude-instant".to_string(),
            ],
            "google" => vec![
                "gemini-pro".to_string(),
                "gemini-pro-vision".to_string(),
                "gemini-1.5-pro".to_string(),
            ],
            "cohere" => vec![
                "command".to_string(),
                "command-r".to_string(),
                "command-r-plus".to_string(),
            ],
            "mistral" => vec![
                "mistral-tiny".to_string(),
                "mistral-small".to_string(),
                "mistral-medium".to_string(),
                "mistral-large".to_string(),
            ],
            _ => vec![],
        }
    }

    pub fn validate_model_with_provider(model: &str, provider: &str) -> Result<(), ProviderError> {
        let compatible_models = Self::get_compatible_models_for_provider(provider);

        if compatible_models.is_empty() {
            return Ok(());
        }

        let model_matches = compatible_models.iter().any(|compatible_model| {
            model
                .to_lowercase()
                .starts_with(&compatible_model.to_lowercase())
        });

        if !model_matches {
            return Err(ProviderError::ModelNotFound {
                provider: "unknown",
                model: format!(
                    "Model '{}' is not compatible with provider '{}'",
                    model, provider
                ),
            });
        }

        Ok(())
    }

    pub fn get_model_aliases(model: &str) -> Vec<String> {
        let model_lower = model.to_lowercase();
        let mut aliases = vec![];

        match model_lower.as_str() {
            "gpt-4" => {
                aliases.extend_from_slice(&[
                    "openai/gpt-4".to_string(),
                    "gpt-4-0314".to_string(),
                    "gpt-4-0613".to_string(),
                ]);
            }
            "claude-3-opus" => {
                aliases.extend_from_slice(&[
                    "anthropic/claude-3-opus".to_string(),
                    "claude-3-opus-20240229".to_string(),
                ]);
            }
            "gemini-pro" => {
                aliases.extend_from_slice(&[
                    "google/gemini-pro".to_string(),
                    "gemini-1.0-pro".to_string(),
                ]);
            }
            _ => {}
        }

        aliases
    }

    pub fn is_chat_model(model: &str) -> bool {
        let model_lower = model.to_lowercase();

        let chat_patterns = ["gpt-", "claude-", "gemini-", "command", "llama", "mistral"];

        chat_patterns
            .iter()
            .any(|pattern| model_lower.contains(pattern))
    }

    pub fn is_completion_model(model: &str) -> bool {
        let model_lower = model.to_lowercase();

        let completion_patterns = [
            "text-davinci",
            "text-curie",
            "text-babbage",
            "text-ada",
            "davinci",
            "curie",
        ];

        completion_patterns
            .iter()
            .any(|pattern| model_lower.contains(pattern))
    }

    pub fn get_recommended_temperature(model: &str) -> f32 {
        match Self::get_model_family(model).as_str() {
            "gpt" => 0.7,
            "claude" => 0.9,
            "gemini" => 0.8,
            "command" => 0.8,
            _ => 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_capabilities() {
        let caps = ModelUtils::get_model_capabilities("gpt-4");
        assert!(caps.supports_function_calling);
        assert!(caps.supports_parallel_function_calling);

        let caps_35 = ModelUtils::get_model_capabilities("gpt-3.5-turbo");
        assert!(caps_35.supports_function_calling);
        assert!(!caps_35.supports_parallel_function_calling);

        let caps_claude = ModelUtils::get_model_capabilities("claude-3-opus");
        assert!(caps_claude.supports_function_calling);
        assert!(caps_claude.supports_vision);
    }

    #[test]
    fn test_provider_detection() {
        assert_eq!(
            ModelUtils::get_provider_from_model("gpt-4"),
            Some("openai".to_string())
        );
        assert_eq!(
            ModelUtils::get_provider_from_model("claude-3-opus"),
            Some("anthropic".to_string())
        );
        assert_eq!(
            ModelUtils::get_provider_from_model("gemini-pro"),
            Some("google".to_string())
        );
        assert_eq!(ModelUtils::get_provider_from_model("unknown-model"), None);
    }

    #[test]
    fn test_base_model_extraction() {
        assert_eq!(ModelUtils::get_base_model("gpt-4-0314"), "gpt-4");
        assert_eq!(
            ModelUtils::get_base_model("gpt-4-turbo-preview"),
            "gpt-4-turbo"
        );
        assert_eq!(
            ModelUtils::get_base_model("claude-3-opus-20240229"),
            "claude-3-opus"
        );
    }

    #[test]
    fn test_model_validation() {
        assert!(ModelUtils::is_valid_model("gpt-4"));
        assert!(ModelUtils::is_valid_model("claude-3-opus"));
        assert!(ModelUtils::is_valid_model("gemini-pro"));
        assert!(!ModelUtils::is_valid_model("unknown-model-xyz"));
    }

    #[test]
    fn test_model_family() {
        assert_eq!(ModelUtils::get_model_family("gpt-4-turbo"), "gpt");
        assert_eq!(ModelUtils::get_model_family("claude-3-opus"), "claude");
        assert_eq!(ModelUtils::get_model_family("gemini-pro"), "gemini");
    }

    #[test]
    fn test_model_pricing() {
        let pricing = ModelUtils::get_model_pricing("gpt-4");
        assert!(pricing.is_some());
        assert_eq!(pricing.unwrap(), (0.03, 0.06));

        assert!(ModelUtils::get_model_pricing("unknown-model").is_none());
    }

    #[test]
    fn test_compatible_models() {
        let openai_models = ModelUtils::get_compatible_models_for_provider("openai");
        assert!(openai_models.contains(&"gpt-4".to_string()));

        let anthropic_models = ModelUtils::get_compatible_models_for_provider("anthropic");
        assert!(anthropic_models.contains(&"claude-3-opus".to_string()));

        let unknown_models = ModelUtils::get_compatible_models_for_provider("unknown");
        assert!(unknown_models.is_empty());
    }

    #[test]
    fn test_model_type_detection() {
        assert!(ModelUtils::is_chat_model("gpt-4"));
        assert!(ModelUtils::is_chat_model("claude-3-opus"));
        assert!(ModelUtils::is_completion_model("text-davinci-003"));
        assert!(!ModelUtils::is_completion_model("gpt-4"));
    }

    #[test]
    fn test_recommended_temperature() {
        assert_eq!(ModelUtils::get_recommended_temperature("gpt-4"), 0.7);
        assert_eq!(
            ModelUtils::get_recommended_temperature("claude-3-opus"),
            0.9
        );
        assert_eq!(ModelUtils::get_recommended_temperature("gemini-pro"), 0.8);
    }
}
