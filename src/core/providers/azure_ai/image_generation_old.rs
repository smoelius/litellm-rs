//! Azure AI Image Generation Handler
//!
//! Complete FLUX image generation functionality for Azure AI services following unified architecture

use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::config::{AzureAIConfig, AzureAIEndpointType};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{
    common::RequestContext,
    requests::ImageGenerationRequest,
    responses::{ImageGenerationResponse, ImageData},
};

/// Azure AI image generation handler following unified architecture
#[derive(Debug)]
pub struct AzureAIImageHandler {
    config: AzureAIConfig,
    client: reqwest::Client,
}

impl AzureAIImageHandler {
    /// Create new image generation handler
    pub fn new(config: AzureAIConfig) -> Result<Self, ProviderError> {
        // Create headers for the client
        let mut headers = HeaderMap::new();
        let default_headers = config.create_default_headers()
            .map_err(|e| ProviderError::configuration_error("azure_ai", &e))?;
        
        for (key, value) in default_headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| ProviderError::configuration_error("azure_ai", &format!("Invalid header name: {}", e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(&value)
                .map_err(|e| ProviderError::configuration_error("azure_ai", &format!("Invalid header value: {}", e)))?;
            headers.insert(header_name, header_value);
        }
        
        let client = reqwest::Client::builder()
            .timeout(config.timeout())
            .default_headers(headers)
            .build()
            .map_err(|e| ProviderError::configuration_error("azure_ai", &format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self { config, client })
    }

    /// Generate image using FLUX models
    pub async fn generate_image(
        &self,
        request: ImageGenerationRequest,
        context: RequestContext,
    ) -> Result<ImageGenerationResponse, ProviderError> {
        // Validate request
        AzureAIImageUtils::validate_request(&request)?;

        // Transform request to Azure AI format
        let azure_request = AzureAIImageUtils::transform_request(&request)?;

        // Build URL
        let url = self.config.build_endpoint_url(AzureAIEndpointType::ImageGeneration.as_path())
            .map_err(|e| ProviderError::configuration_error("azure_ai", &e))?;

        // Execute request (image generation can take longer)
        let response = self
            .client
            .post(&url)
            .json(&azure_request)
            .timeout(std::time::Duration::from_secs(180)) // 3 minutes for image generation
            .send()
            .await
            .map_err(|e| ProviderError::network_error("azure_ai", &format!("Request failed: {}", e)))?;

        // Handle error responses
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ProviderError::api_error("azure_ai", status, &error_body));
        }

        // Parse response
        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("azure_ai", &format!("Failed to parse response: {}", e)))?;

        // Transform to standard format
        AzureAIImageUtils::transform_response(response_json, &request.model)
    }


}

/// Utility struct for Azure AI image generation operations
pub struct AzureAIImageUtils;

impl AzureAIImageUtils {
    /// Validate image generation request
    pub fn validate_request(request: &ImageGenerationRequest) -> Result<(), ProviderError> {
        if request.prompt.trim().is_empty() {
            return Err(ProviderError::invalid_request("azure_ai", "Prompt cannot be empty"));
        }

        if request.model.is_empty() {
            return Err(ProviderError::invalid_request("azure_ai", "Model cannot be empty"));
        }

        // Validate number of images
        if request.n.unwrap_or(1) == 0 || request.n.unwrap_or(1) > 4 {
            return Err(ProviderError::invalid_request(
                "azure_ai",
                "Number of images must be between 1 and 4",
            ));
        }

        // Validate image size if specified
        if let Some(ref size) = request.size {
            if !Self::is_valid_size(size) {
                return Err(ProviderError::invalid_request(
                    "azure_ai",
                    "Invalid image size. Supported sizes: 256x256, 512x512, 1024x1024, 1024x1792, 1792x1024",
                ));
            }
        }

        // Validate quality if specified
        if let Some(ref quality) = request.quality {
            if !matches!(quality.as_str(), "standard" | "hd") {
                return Err(ProviderError::invalid_request(
                    "azure_ai",
                    "Quality must be 'standard' or 'hd'",
                ));
            }
        }

        Ok(())
    }

    /// Transform ImageGenerationRequest to Azure AI format
    pub fn transform_request(request: &ImageGenerationRequest) -> Result<Value, ProviderError> {
        let mut azure_request = json!({
            "model": request.model,
            "prompt": request.prompt,
        });

        // Add optional parameters
        if let Some(n) = request.n {
            azure_request["n"] = json!(n);
        }

        if let Some(ref size) = request.size {
            azure_request["size"] = json!(size);
        }

        if let Some(ref quality) = request.quality {
            azure_request["quality"] = json!(quality);
        }

        if let Some(ref style) = request.style {
            azure_request["style"] = json!(style);
        }

        if let Some(ref user) = request.user {
            azure_request["user"] = json!(user);
        }

        // FLUX-specific parameters
        if request.model.contains("flux") {
            // Set default size for FLUX if not specified
            if request.size.is_none() {
                azure_request["size"] = json!("1024x1024");
            }
            
            // FLUX models typically generate high quality by default
            if request.quality.is_none() {
                azure_request["quality"] = json!("hd");
            }
        }

        Ok(azure_request)
    }

    /// Transform Azure AI response to ImageGenerationResponse
    pub fn transform_response(response: Value, model: &str) -> Result<ImageGenerationResponse, ProviderError> {
        // Parse data array
        let data_array = response["data"]
            .as_array()
            .ok_or_else(|| ProviderError::response_parsing("azure_ai", "Missing or invalid 'data' field"))?;

        let mut image_data = Vec::new();
        
        for item in data_array.iter() {
            let image = if let Some(url) = item["url"].as_str() {
                ImageData::Url { url: url.to_string() }
            } else if let Some(b64_json) = item["b64_json"].as_str() {
                ImageData::Base64 { b64_json: b64_json.to_string() }
            } else {
                return Err(ProviderError::response_parsing("azure_ai", "Missing image URL or base64 data"));
            };

            image_data.push(image);
        }

        // Note: Image generation responses don't typically include usage information

        Ok(ImageGenerationResponse {
            created: response["created"]
                .as_i64()
                .unwrap_or_else(|| chrono::Utc::now().timestamp()) as u32,
            data: image_data,
            model: model.to_string(),
        })
    }

    /// Check if image size is valid
    pub fn is_valid_size(size: &str) -> bool {
        matches!(
            size,
            "256x256" | "512x512" | "1024x1024" | "1024x1792" | "1792x1024" | "1080x1080" | "1080x1920" | "1920x1080"
        )
    }

    /// Get supported sizes for model
    pub fn get_supported_sizes(model: &str) -> Vec<&'static str> {
        if model.contains("flux") {
            vec![
                "512x512",
                "768x768", 
                "1024x1024",
                "1024x1792",
                "1792x1024",
                "1080x1080",
                "1080x1920",
                "1920x1080",
            ]
        } else {
            vec!["256x256", "512x512", "1024x1024"]
        }
    }

    /// Get default size for model
    pub fn get_default_size(model: &str) -> &'static str {
        if model.contains("flux") {
            "1024x1024"
        } else {
            "1024x1024"
        }
    }

    /// Get cost per image for model
    pub fn get_cost_per_image(model: &str, quality: Option<&str>) -> f64 {
        match model {
            m if m.contains("flux-1.1-pro") => 0.04,
            m if m.contains("flux.1-kontext-pro") => 0.04,
            m if m.contains("flux") => 0.04, // Default FLUX pricing
            _ => 0.02, // Default pricing for other models
        }
    }

    /// Estimate generation time based on model and parameters
    pub fn estimate_generation_time(model: &str, n: Option<u32>, quality: Option<&str>) -> u32 {
        let base_time = if model.contains("flux") {
            60 // FLUX models typically take ~60 seconds per image
        } else {
            30 // Other models ~30 seconds
        };
        
        let image_count = n.unwrap_or(1);
        let quality_multiplier = if quality == Some("hd") { 1.5 } else { 1.0 };
        
        (base_time as f64 * image_count as f64 * quality_multiplier) as u32
    }

    /// Validate prompt length and content
    pub fn validate_prompt(prompt: &str) -> Result<(), ProviderError> {
        if prompt.trim().is_empty() {
            return Err(ProviderError::invalid_request("azure_ai", "Prompt cannot be empty"));
        }
        
        if prompt.len() > 4000 {
            return Err(ProviderError::invalid_request(
                "azure_ai",
                "Prompt too long. Maximum length is 4000 characters",
            ));
        }
        
        // Check for potentially problematic content
        let prohibited_terms = ["nsfw", "adult", "violence", "gore"];
        let prompt_lower = prompt.to_lowercase();
        
        for term in prohibited_terms {
            if prompt_lower.contains(term) {
                return Err(ProviderError::invalid_request(
                    "azure_ai",
                    "Prompt contains prohibited content",
                ));
            }
        }
        
        Ok(())
    }
}

/// FLUX model capabilities
#[derive(Debug, Clone)]
pub struct FluxModelCapabilities {
    pub model_name: String,
    pub max_images_per_request: u32,
    pub supported_sizes: Vec<String>,
    pub default_size: String,
    pub supports_hd_quality: bool,
    pub cost_per_image: f64,
    pub estimated_time_seconds: u32,
}

impl FluxModelCapabilities {
    /// Get capabilities for FLUX models
    pub fn for_model(model: &str) -> Self {
        match model {
            m if m.contains("flux-1.1-pro") => Self {
                model_name: "FLUX-1.1-pro".to_string(),
                max_images_per_request: 4,
                supported_sizes: vec![
                    "512x512".to_string(),
                    "768x768".to_string(),
                    "1024x1024".to_string(),
                    "1024x1792".to_string(),
                    "1792x1024".to_string(),
                ],
                default_size: "1024x1024".to_string(),
                supports_hd_quality: true,
                cost_per_image: 0.04,
                estimated_time_seconds: 60,
            },
            m if m.contains("flux.1-kontext-pro") => Self {
                model_name: "FLUX.1-Kontext-pro".to_string(),
                max_images_per_request: 4,
                supported_sizes: vec![
                    "512x512".to_string(),
                    "768x768".to_string(),
                    "1024x1024".to_string(),
                    "1024x1792".to_string(),
                    "1792x1024".to_string(),
                    "1080x1080".to_string(),
                    "1080x1920".to_string(),
                    "1920x1080".to_string(),
                ],
                default_size: "1024x1024".to_string(),
                supports_hd_quality: true,
                cost_per_image: 0.04,
                estimated_time_seconds: 60,
            },
            _ => Self {
                model_name: "Unknown".to_string(),
                max_images_per_request: 1,
                supported_sizes: vec!["1024x1024".to_string()],
                default_size: "1024x1024".to_string(),
                supports_hd_quality: false,
                cost_per_image: 0.02,
                estimated_time_seconds: 30,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::azure_ai::config::AzureAIConfig;

    #[test]
    fn test_image_utils_validation() {
        let mut request = ImageGenerationRequest {
            prompt: "A cute baby sea otter".to_string(),
            model: "azure_ai/flux-1.1-pro".to_string(),
            n: Some(2),
            quality: Some("hd".to_string()),
            size: Some("1024x1024".to_string()),
            style: None,
            response_format: None,
            user: None,
        };

        // Valid request should pass
        assert!(AzureAIImageUtils::validate_request(&request).is_ok());

        // Empty prompt should fail
        request.prompt = "".to_string();
        assert!(AzureAIImageUtils::validate_request(&request).is_err());

        // Too many images should fail
        request.prompt = "test".to_string();
        request.n = Some(5);
        assert!(AzureAIImageUtils::validate_request(&request).is_err());

        // Invalid size should fail
        request.n = Some(1);
        request.size = Some("invalid_size".to_string());
        assert!(AzureAIImageUtils::validate_request(&request).is_err());
    }

    #[test]
    fn test_size_validation() {
        assert!(AzureAIImageUtils::is_valid_size("1024x1024"));
        assert!(AzureAIImageUtils::is_valid_size("1792x1024"));
        assert!(!AzureAIImageUtils::is_valid_size("invalid"));
        assert!(!AzureAIImageUtils::is_valid_size("123x456"));
    }

    #[test]
    fn test_flux_capabilities() {
        let caps = FluxModelCapabilities::for_model("flux-1.1-pro");
        assert_eq!(caps.model_name, "FLUX-1.1-pro");
        assert_eq!(caps.max_images_per_request, 4);
        assert_eq!(caps.cost_per_image, 0.04);
        assert!(caps.supports_hd_quality);

        let kontext_caps = FluxModelCapabilities::for_model("flux.1-kontext-pro");
        assert_eq!(kontext_caps.model_name, "FLUX.1-Kontext-pro");
        assert!(kontext_caps.supported_sizes.len() > 5);
    }

    #[test]
    fn test_cost_calculation() {
        assert_eq!(AzureAIImageUtils::get_cost_per_image("flux-1.1-pro", None), 0.04);
        assert_eq!(AzureAIImageUtils::get_cost_per_image("flux.1-kontext-pro", None), 0.04);
    }

    #[test]
    fn test_generation_time_estimation() {
        let time = AzureAIImageUtils::estimate_generation_time("flux-1.1-pro", Some(2), Some("hd"));
        assert!(time > 60); // Should be more than base time due to multiple images and HD quality
        
        let standard_time = AzureAIImageUtils::estimate_generation_time("flux-1.1-pro", Some(1), None);
        assert_eq!(standard_time, 60); // Base time for single image
    }

    #[test]
    fn test_prompt_validation() {
        assert!(AzureAIImageUtils::validate_prompt("A beautiful landscape").is_ok());
        assert!(AzureAIImageUtils::validate_prompt("").is_err());
        assert!(AzureAIImageUtils::validate_prompt(&"x".repeat(5000)).is_err());
    }

    #[test]
    fn test_request_transformation() {
        let request = ImageGenerationRequest {
            prompt: "A cute baby sea otter".to_string(),
            model: "flux-1.1-pro".to_string(),
            n: Some(2),
            quality: Some("hd".to_string()),
            size: Some("1024x1024".to_string()),
            style: Some("vivid".to_string()),
            response_format: None,
            user: Some("test-user".to_string()),
        };

        let result = AzureAIImageUtils::transform_request(&request);
        assert!(result.is_ok());
        
        let azure_request = result.unwrap();
        assert_eq!(azure_request["model"], "flux-1.1-pro");
        assert_eq!(azure_request["prompt"], "A cute baby sea otter");
        assert_eq!(azure_request["n"], 2);
        assert_eq!(azure_request["quality"], "hd");
        assert_eq!(azure_request["size"], "1024x1024");
        assert_eq!(azure_request["user"], "test-user");
    }

    #[test]
    fn test_handler_creation() {
        let config = AzureAIConfig::new("azure_ai");
        // Test that handler can be created without errors
        let _result = AzureAIImageHandler::new(config);
    }
}