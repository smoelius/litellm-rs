//! Cloudflare Workers AI Model Information
//!
//! Model configurations for Cloudflare's Workers AI models

use std::collections::HashMap;
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};

/// Cloudflare Workers AI model identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudflareModel {
    // Llama models
    Llama3_8B,
    Llama3_8BInstruct,
    Llama3_70B,
    Llama3_70BInstruct,
    Llama2_7B,
    Llama2_13B,

    // Mistral models
    Mistral7BInstruct,
    Mixtral8x7BInstruct,

    // Other open models
    Qwen15_7BChat,
    Deepseek1_5B,
    Phi2,
    Gemma7BIT,

    // Code models
    CodeLlama7B,
    DeepseekCoder6_7B,
}

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model ID as used in API
    pub model_id: &'static str,
    /// Display name
    pub display_name: &'static str,
    /// Maximum context length
    pub context_length: u32,
    /// Maximum output tokens
    pub max_output_tokens: u32,
    /// Whether the model supports tools/functions
    pub supports_tools: bool,
    /// Whether the model supports vision
    pub supports_vision: bool,
    /// Whether the model supports streaming
    pub supports_streaming: bool,
    /// Input cost per million tokens (in USD)
    pub input_cost_per_million: f64,
    /// Output cost per million tokens (in USD)
    pub output_cost_per_million: f64,
}

/// Static model configurations
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Llama 3 models
    configs.insert("@cf/meta/llama-3-8b-instruct", ModelInfo {
        model_id: "@cf/meta/llama-3-8b-instruct",
        display_name: "Llama 3 8B Instruct",
        context_length: 8192,
        max_output_tokens: 2048,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0, // Free on Cloudflare Workers
        output_cost_per_million: 0.0,
    });

    configs.insert("@cf/meta/llama-3-70b-instruct", ModelInfo {
        model_id: "@cf/meta/llama-3-70b-instruct",
        display_name: "Llama 3 70B Instruct",
        context_length: 8192,
        max_output_tokens: 2048,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    configs.insert("@cf/meta/llama-2-7b-chat-int8", ModelInfo {
        model_id: "@cf/meta/llama-2-7b-chat-int8",
        display_name: "Llama 2 7B Chat",
        context_length: 4096,
        max_output_tokens: 2048,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    // Mistral models
    configs.insert("@cf/mistral/mistral-7b-instruct-v0.1", ModelInfo {
        model_id: "@cf/mistral/mistral-7b-instruct-v0.1",
        display_name: "Mistral 7B Instruct",
        context_length: 8192,
        max_output_tokens: 2048,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    configs.insert("@hf/thebloke/mixtral-8x7b-instruct-v0.1-awq", ModelInfo {
        model_id: "@hf/thebloke/mixtral-8x7b-instruct-v0.1-awq",
        display_name: "Mixtral 8x7B Instruct",
        context_length: 32768,
        max_output_tokens: 4096,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    // Qwen model
    configs.insert("@cf/qwen/qwen1.5-7b-chat-awq", ModelInfo {
        model_id: "@cf/qwen/qwen1.5-7b-chat-awq",
        display_name: "Qwen 1.5 7B Chat",
        context_length: 32768,
        max_output_tokens: 4096,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    // Code models
    configs.insert("@cf/meta/codellama-7b-instruct", ModelInfo {
        model_id: "@cf/meta/codellama-7b-instruct",
        display_name: "Code Llama 7B",
        context_length: 16384,
        max_output_tokens: 4096,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    configs.insert("@cf/deepseek-ai/deepseek-coder-6.7b-instruct-awq", ModelInfo {
        model_id: "@cf/deepseek-ai/deepseek-coder-6.7b-instruct-awq",
        display_name: "DeepSeek Coder 6.7B",
        context_length: 16384,
        max_output_tokens: 4096,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    // Smaller models
    configs.insert("@cf/microsoft/phi-2", ModelInfo {
        model_id: "@cf/microsoft/phi-2",
        display_name: "Phi-2",
        context_length: 2048,
        max_output_tokens: 1024,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    configs.insert("@cf/google/gemma-7b-it", ModelInfo {
        model_id: "@cf/google/gemma-7b-it",
        display_name: "Gemma 7B IT",
        context_length: 8192,
        max_output_tokens: 2048,
        supports_tools: false,
        supports_vision: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    });

    configs
});

/// Get model information by ID
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    // Handle cloudflare/ prefix
    let model_id = model_id.strip_prefix("cloudflare/").unwrap_or(model_id);
    MODEL_CONFIGS.get(model_id)
}

/// Get all available model IDs
pub fn get_available_models() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

/// Calculate cost (always 0 for Cloudflare Workers AI as it's free within limits)
pub fn calculate_cost(
    model_id: &str,
    _input_tokens: u32,
    _output_tokens: u32,
) -> Option<f64> {
    // Cloudflare Workers AI is free within usage limits
    get_model_info(model_id).map(|_| 0.0)
}

impl CloudflareModel {
    /// Get the API model ID
    pub fn model_id(&self) -> &'static str {
        match self {
            CloudflareModel::Llama3_8B => "@cf/meta/llama-3-8b",
            CloudflareModel::Llama3_8BInstruct => "@cf/meta/llama-3-8b-instruct",
            CloudflareModel::Llama3_70B => "@cf/meta/llama-3-70b",
            CloudflareModel::Llama3_70BInstruct => "@cf/meta/llama-3-70b-instruct",
            CloudflareModel::Llama2_7B => "@cf/meta/llama-2-7b-chat-int8",
            CloudflareModel::Llama2_13B => "@cf/meta/llama-2-13b-chat",
            CloudflareModel::Mistral7BInstruct => "@cf/mistral/mistral-7b-instruct-v0.1",
            CloudflareModel::Mixtral8x7BInstruct => "@hf/thebloke/mixtral-8x7b-instruct-v0.1-awq",
            CloudflareModel::Qwen15_7BChat => "@cf/qwen/qwen1.5-7b-chat-awq",
            CloudflareModel::Deepseek1_5B => "@cf/deepseek-ai/deepseek-1.5b",
            CloudflareModel::Phi2 => "@cf/microsoft/phi-2",
            CloudflareModel::Gemma7BIT => "@cf/google/gemma-7b-it",
            CloudflareModel::CodeLlama7B => "@cf/meta/codellama-7b-instruct",
            CloudflareModel::DeepseekCoder6_7B => "@cf/deepseek-ai/deepseek-coder-6.7b-instruct-awq",
        }
    }

    /// Get model information
    pub fn info(&self) -> Option<&'static ModelInfo> {
        get_model_info(self.model_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info() {
        let info = get_model_info("@cf/meta/llama-3-8b-instruct").unwrap();
        assert_eq!(info.model_id, "@cf/meta/llama-3-8b-instruct");
        assert_eq!(info.context_length, 8192);
        assert!(info.supports_streaming);

        // Test with cloudflare/ prefix
        let info = get_model_info("cloudflare/@cf/meta/llama-3-8b-instruct").unwrap();
        assert_eq!(info.model_id, "@cf/meta/llama-3-8b-instruct");
    }

    #[test]
    fn test_available_models() {
        let models = get_available_models();
        assert!(models.contains(&"@cf/meta/llama-3-8b-instruct"));
        assert!(models.contains(&"@cf/mistral/mistral-7b-instruct-v0.1"));
    }

    #[test]
    fn test_cost_calculation() {
        // Cloudflare Workers AI is free
        let cost = calculate_cost("@cf/meta/llama-3-8b-instruct", 1000, 500).unwrap();
        assert_eq!(cost, 0.0);
    }
}