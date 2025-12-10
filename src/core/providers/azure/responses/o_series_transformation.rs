//! O-Series Response Transformation for Azure

use super::{AzureProcessedResponse, AzureResponseMetadata, ResponseMetrics};
use serde::{Deserialize, Serialize};

/// O-Series specific response processor
pub struct OSeriesResponseProcessor;

impl OSeriesResponseProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process O-series model responses
    pub fn process_response<T: Serialize + for<'de> Deserialize<'de> + Clone>(
        &self,
        response: T,
    ) -> Result<AzureProcessedResponse<T>, String> {
        let start_time = std::time::Instant::now();

        // Convert to JSON for processing
        let json_response = serde_json::to_value(&response)
            .map_err(|e| format!("Failed to serialize response: {}", e))?;

        // Extract O-series specific metadata
        let metadata = self.extract_o_series_metadata(&json_response);

        // Check for reasoning tokens in usage
        let has_reasoning = self.has_reasoning_tokens(&json_response);

        let processing_time = start_time.elapsed().as_millis() as u64;

        let response_size = serde_json::to_vec(&response).map_or(0, |v| v.len());

        Ok(AzureProcessedResponse {
            data: response,
            metadata,
            content_filtered: false, // O-series models typically have different filtering
            warnings: if has_reasoning {
                vec!["Response includes reasoning tokens".to_string()]
            } else {
                vec![]
            },
            metrics: ResponseMetrics {
                total_time_ms: processing_time,
                transformation_time_ms: processing_time,
                filtering_time_ms: 0,
                response_size_bytes: response_size,
            },
        })
    }

    fn extract_o_series_metadata(&self, response: &serde_json::Value) -> AzureResponseMetadata {
        let mut metadata = AzureResponseMetadata::default();

        // Extract deployment info
        if let Some(model) = response.get("model").and_then(|m| m.as_str()) {
            metadata.deployment_id = Some(model.to_string());
        }

        metadata
    }

    fn has_reasoning_tokens(&self, response: &serde_json::Value) -> bool {
        if let Some(usage) = response.get("usage") {
            return usage.get("reasoning_tokens").is_some();
        }
        false
    }
}

/// O-Series response transformation
pub struct OSeriesResponseTransformation;

impl OSeriesResponseTransformation {
    pub fn new() -> Self {
        Self
    }

    /// Transform O-series response for compatibility
    /// Takes ownership to avoid unnecessary cloning
    pub fn transform_response(
        &self,
        mut response: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // Handle reasoning tokens in usage
        if let Some(usage) = response.get_mut("usage") {
            self.transform_usage_with_reasoning(usage)?;
        }

        // Handle choices with reasoning content
        if let Some(choices) = response.get_mut("choices").and_then(|c| c.as_array_mut()) {
            for choice in choices {
                self.transform_o_series_choice(choice)?;
            }
        }

        Ok(response)
    }

    fn transform_usage_with_reasoning(&self, usage: &mut serde_json::Value) -> Result<(), String> {
        // O-series models may include reasoning_tokens
        // Ensure they're properly accounted for in total_tokens

        if let Some(usage_obj) = usage.as_object_mut() {
            let prompt_tokens = usage_obj
                .get("prompt_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);

            let completion_tokens = usage_obj
                .get("completion_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);

            let reasoning_tokens = usage_obj
                .get("reasoning_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);

            // Update total_tokens to include reasoning tokens
            let total_tokens = prompt_tokens + completion_tokens + reasoning_tokens;
            usage_obj.insert("total_tokens".to_string(), serde_json::json!(total_tokens));
        }

        Ok(())
    }

    fn transform_o_series_choice(&self, _choice: &mut serde_json::Value) -> Result<(), String> {
        // O-series models might have special handling for reasoning
        // For now, pass through as-is
        Ok(())
    }
}

impl Default for OSeriesResponseProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OSeriesResponseTransformation {
    fn default() -> Self {
        Self::new()
    }
}
