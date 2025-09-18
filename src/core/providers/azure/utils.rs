//! Azure OpenAI Utilities
//!
//! Utility functions for Azure OpenAI Service

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

use super::config::{AzureConfig, AzureModelInfo};
use super::error::{azure_config_error, azure_header_error};
use crate::core::providers::unified_provider::ProviderError;

/// Azure endpoint types
#[derive(Debug, Clone, PartialEq)]
pub enum AzureEndpointType {
    ChatCompletions,
    Completions,
    Embeddings,
    Images,
    ImageEdits,
    ImageVariations,
    AudioSpeech,
    AudioTranscriptions,
    AudioTranslations,
    Files,
    FineTuning,
    Models,
}

/// Azure OpenAI utilities
pub struct AzureUtils;

impl AzureUtils {
    /// Build Azure OpenAI URL
    pub fn build_azure_url(
        azure_endpoint: &str,
        deployment_name: &str,
        api_version: &str,
        endpoint_type: AzureEndpointType,
    ) -> String {
        let base = azure_endpoint.trim_end_matches('/');
        let endpoint_path = match endpoint_type {
            AzureEndpointType::ChatCompletions => "chat/completions",
            AzureEndpointType::Completions => "completions",
            AzureEndpointType::Embeddings => "embeddings",
            AzureEndpointType::Images => "images/generations",
            AzureEndpointType::ImageEdits => "images/edits",
            AzureEndpointType::ImageVariations => "images/variations",
            AzureEndpointType::AudioSpeech => "audio/speech",
            AzureEndpointType::AudioTranscriptions => "audio/transcriptions",
            AzureEndpointType::AudioTranslations => "audio/translations",
            AzureEndpointType::Files => "files",
            AzureEndpointType::FineTuning => "fine_tuning/jobs",
            AzureEndpointType::Models => "models",
        };

        format!(
            "{}/openai/deployments/{}/{}?api-version={}",
            base, deployment_name, endpoint_path, api_version
        )
    }

    /// Process Azure headers to OpenAI format
    pub fn process_azure_headers(headers: &HeaderMap) -> HashMap<String, String> {
        let mut openai_headers = HashMap::new();

        // Rate limit headers
        if let Some(limit) = headers.get("x-ratelimit-limit-requests") {
            if let Ok(value) = limit.to_str() {
                openai_headers.insert("x-ratelimit-limit-requests".to_string(), value.to_string());
            }
        }

        if let Some(remaining) = headers.get("x-ratelimit-remaining-requests") {
            if let Ok(value) = remaining.to_str() {
                openai_headers.insert(
                    "x-ratelimit-remaining-requests".to_string(),
                    value.to_string(),
                );
            }
        }

        if let Some(reset) = headers.get("x-ratelimit-reset-requests") {
            if let Ok(value) = reset.to_str() {
                openai_headers.insert("x-ratelimit-reset-requests".to_string(), value.to_string());
            }
        }

        // Token rate limit headers
        if let Some(limit) = headers.get("x-ratelimit-limit-tokens") {
            if let Ok(value) = limit.to_str() {
                openai_headers.insert("x-ratelimit-limit-tokens".to_string(), value.to_string());
            }
        }

        if let Some(remaining) = headers.get("x-ratelimit-remaining-tokens") {
            if let Ok(value) = remaining.to_str() {
                openai_headers.insert(
                    "x-ratelimit-remaining-tokens".to_string(),
                    value.to_string(),
                );
            }
        }

        if let Some(reset) = headers.get("x-ratelimit-reset-tokens") {
            if let Ok(value) = reset.to_str() {
                openai_headers.insert("x-ratelimit-reset-tokens".to_string(), value.to_string());
            }
        }

        // Azure specific headers
        if let Some(request_id) = headers.get("x-request-id") {
            if let Ok(value) = request_id.to_str() {
                openai_headers.insert("x-request-id".to_string(), value.to_string());
            }
        }

        openai_headers
    }

    /// Create Azure request headers
    pub fn create_azure_headers(
        config: &AzureConfig,
        api_key: &str,
    ) -> Result<HeaderMap, ProviderError> {
        let mut headers = HeaderMap::new();

        // API key header (Azure uses api-key header, not Authorization)
        headers.insert(
            "api-key",
            HeaderValue::from_str(api_key)
                .map_err(|e| azure_header_error(format!("Invalid API key: {}", e)))?,
        );

        // Content type
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );

        // User agent
        headers.insert(
            HeaderName::from_static("user-agent"),
            HeaderValue::from_static("litellm-rust/1.0.0"),
        );

        // Add custom headers
        for (key, value) in &config.custom_headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| azure_header_error(format!("Invalid header name {}: {}", key, e)))?;
            let header_value = HeaderValue::from_str(value).map_err(|e| {
                azure_header_error(format!("Invalid header value for {}: {}", key, e))
            })?;
            headers.insert(header_name, header_value);
        }

        Ok(headers)
    }

    /// Validate Azure configuration
    pub fn validate_config(config: &AzureConfig) -> Result<(), ProviderError> {
        if config.get_effective_azure_endpoint().is_none() {
            return Err(azure_config_error("Azure endpoint is required"));
        }

        if config.api_version.is_empty() {
            return Err(azure_config_error("API version is required"));
        }

        Ok(())
    }

    /// Extract deployment name from model
    pub fn extract_deployment_from_model(model: &str) -> Option<String> {
        // Handle model names like "azure/gpt-4" or direct deployment names
        if let Some(stripped) = model.strip_prefix("azure/") {
            Some(stripped.to_string())
        } else if model.contains('/') {
            // Skip provider prefix
            model.split('/').next_back().map(|s| s.to_string())
        } else {
            // Use model name directly as deployment
            Some(model.to_string())
        }
    }

    /// Get model info from Azure deployment
    pub fn get_model_info_from_deployment(deployment_name: &str) -> AzureModelInfo {
        AzureModelInfo {
            deployment_name: deployment_name.to_string(),
            model_name: Self::infer_model_from_deployment(deployment_name),
            max_tokens: Self::get_max_tokens_for_model(deployment_name),
            supports_functions: Self::supports_functions(deployment_name),
            supports_streaming: true,
            api_version: "2024-02-01".to_string(),
        }
    }

    /// Infer base model from deployment name
    fn infer_model_from_deployment(deployment: &str) -> String {
        let lower = deployment.to_lowercase();

        if lower.contains("gpt-4") {
            if lower.contains("vision") || lower.contains("v") {
                "gpt-4-vision-preview".to_string()
            } else if lower.contains("turbo") || lower.contains("1106") {
                "gpt-4-1106-preview".to_string()
            } else {
                "gpt-4".to_string()
            }
        } else if lower.contains("gpt-35-turbo") || lower.contains("gpt-3.5-turbo") {
            if lower.contains("1106") {
                "gpt-3.5-turbo-1106".to_string()
            } else if lower.contains("instruct") {
                "gpt-3.5-turbo-instruct".to_string()
            } else {
                "gpt-3.5-turbo".to_string()
            }
        } else if lower.contains("text-embedding") {
            if lower.contains("ada-002") {
                "text-embedding-ada-002".to_string()
            } else {
                "text-embedding-3-small".to_string()
            }
        } else if lower.contains("dall-e") {
            if lower.contains("3") {
                "dall-e-3".to_string()
            } else {
                "dall-e-2".to_string()
            }
        } else {
            deployment.to_string()
        }
    }

    /// Get maximum tokens for model
    fn get_max_tokens_for_model(deployment: &str) -> Option<u32> {
        let lower = deployment.to_lowercase();

        if lower.contains("gpt-4") {
            if lower.contains("32k") {
                Some(32768)
            } else if lower.contains("turbo") || lower.contains("1106") {
                Some(128000)
            } else {
                Some(8192)
            }
        } else if lower.contains("gpt-35-turbo") || lower.contains("gpt-3.5-turbo") {
            if lower.contains("16k") {
                Some(16384)
            } else if lower.contains("1106") {
                Some(16385)
            } else {
                Some(4096)
            }
        } else {
            None
        }
    }

    /// Check if model supports function calling
    fn supports_functions(deployment: &str) -> bool {
        let lower = deployment.to_lowercase();

        lower.contains("gpt-4") || lower.contains("gpt-35-turbo") || lower.contains("gpt-3.5-turbo")
    }
}
