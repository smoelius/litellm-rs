//! Unified Cost Calculator
//!
//! Core cost calculation logic that all providers delegate to.
//! This eliminates code duplication and ensures consistent behavior.

use async_trait::async_trait;

use crate::core::cost::types::{
    CostBreakdown, CostError, CostEstimate, ModelCostComparison, ModelPricing, UsageTokens,
};
use crate::core::cost::utils::select_tiered_pricing;

/// Unified Cost Calculator Trait
///
/// All providers should implement this trait by delegating to the generic functions
#[async_trait]
pub trait CostCalculator {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Calculate cost for a completed request
    async fn calculate_cost(
        &self,
        model: &str,
        usage: &UsageTokens,
    ) -> Result<CostBreakdown, Self::Error>;

    /// Estimate cost before making a request
    async fn estimate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        max_output_tokens: Option<u32>,
    ) -> Result<CostEstimate, Self::Error>;

    /// Get pricing information for a model
    fn get_model_pricing(&self, model: &str) -> Result<ModelPricing, Self::Error>;

    /// Get provider name
    fn provider_name(&self) -> &str;
}

/// Generic cost calculation function (like Python's generic_cost_per_token)
///
/// This is the core cost calculation logic that all providers delegate to
pub fn generic_cost_per_token(
    model: &str,
    usage: &UsageTokens,
    provider: &str,
) -> Result<CostBreakdown, CostError> {
    // Get model pricing information
    let pricing = get_model_pricing(model, provider)?;

    // Initialize cost breakdown
    let mut breakdown = CostBreakdown::new(model.to_string(), provider.to_string(), usage.clone());

    // Calculate tiered pricing if applicable
    let (input_cost_per_1k, output_cost_per_1k, cache_creation_cost_per_1k, cache_read_cost_per_1k) =
        select_tiered_pricing(&pricing, usage);

    // Calculate input cost
    breakdown.input_cost = calculate_input_cost(usage, input_cost_per_1k);

    // Calculate output cost
    breakdown.output_cost = calculate_output_cost(usage, output_cost_per_1k);

    // Calculate cache costs if applicable
    if let Some(cached_tokens) = usage.cached_tokens {
        breakdown.cache_cost = calculate_cache_cost(
            cached_tokens,
            cache_creation_cost_per_1k,
            cache_read_cost_per_1k,
        );
    }

    // Calculate audio costs if applicable
    if let Some(audio_tokens) = usage.audio_tokens {
        breakdown.audio_cost = calculate_audio_cost(&pricing, audio_tokens);
    }

    // Calculate image costs if applicable
    if let Some(image_tokens) = usage.image_tokens {
        breakdown.image_cost = calculate_image_cost(&pricing, image_tokens);
    }

    // Calculate reasoning tokens cost if applicable (for o1 models)
    if let Some(reasoning_tokens) = usage.reasoning_tokens {
        breakdown.reasoning_cost = calculate_reasoning_cost(&pricing, reasoning_tokens);
    }

    // Calculate total
    breakdown.calculate_total();

    Ok(breakdown)
}

/// Get model pricing information
pub fn get_model_pricing(model: &str, provider: &str) -> Result<ModelPricing, CostError> {
    // This will be populated with actual pricing data
    // For now, return a basic implementation

    match provider.to_lowercase().as_str() {
        "openai" => get_openai_pricing(model),
        "anthropic" => get_anthropic_pricing(model),
        "azure" => get_azure_pricing(model),
        "vertex_ai" | "vertexai" => get_vertex_ai_pricing(model),
        "deepseek" => get_deepseek_pricing(model),
        "moonshot" => get_moonshot_pricing(model),
        _ => Err(CostError::ProviderNotSupported {
            provider: provider.to_string(),
        }),
    }
}

/// Calculate input cost
fn calculate_input_cost(usage: &UsageTokens, cost_per_1k: f64) -> f64 {
    let non_cached_tokens = if let Some(cached) = usage.cached_tokens {
        usage.prompt_tokens.saturating_sub(cached)
    } else {
        usage.prompt_tokens
    };

    (non_cached_tokens as f64 / 1000.0) * cost_per_1k
}

/// Calculate output cost
fn calculate_output_cost(usage: &UsageTokens, cost_per_1k: f64) -> f64 {
    (usage.completion_tokens as f64 / 1000.0) * cost_per_1k
}

/// Calculate cache cost
fn calculate_cache_cost(cached_tokens: u32, _creation_cost: f64, read_cost: f64) -> f64 {
    // Assume all cached tokens are read (typical case)
    (cached_tokens as f64 / 1000.0) * read_cost
}

/// Calculate audio cost
fn calculate_audio_cost(pricing: &ModelPricing, audio_tokens: u32) -> f64 {
    if let Some(audio_cost_per_token) = pricing.input_cost_per_audio_token {
        audio_tokens as f64 * audio_cost_per_token
    } else {
        0.0
    }
}

/// Calculate image cost
fn calculate_image_cost(pricing: &ModelPricing, image_tokens: u32) -> f64 {
    if let Some(image_cost_per_token) = pricing.image_cost_per_token {
        image_tokens as f64 * image_cost_per_token
    } else {
        0.0
    }
}

/// Calculate reasoning tokens cost (for o1 models)
fn calculate_reasoning_cost(pricing: &ModelPricing, reasoning_tokens: u32) -> f64 {
    if let Some(reasoning_cost_per_token) = pricing.reasoning_cost_per_token {
        reasoning_tokens as f64 * reasoning_cost_per_token
    } else {
        0.0
    }
}

/// Estimate cost for a request
pub fn estimate_cost(
    model: &str,
    provider: &str,
    input_tokens: u32,
    max_output_tokens: Option<u32>,
) -> Result<CostEstimate, CostError> {
    let pricing = get_model_pricing(model, provider)?;

    let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k_tokens;

    let estimated_output_tokens = max_output_tokens.unwrap_or(100); // Default estimate
    let max_output_cost =
        (estimated_output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k_tokens;

    Ok(CostEstimate {
        min_cost: input_cost,
        max_cost: input_cost + max_output_cost,
        input_cost,
        estimated_output_cost: max_output_cost,
        currency: pricing.currency,
    })
}

/// Compare costs between different models
pub fn compare_model_costs(
    models: &[(String, String)], // (model, provider) pairs
    input_tokens: u32,
    output_tokens: u32,
) -> Vec<ModelCostComparison> {
    let mut comparisons = Vec::new();
    let usage = UsageTokens::new(input_tokens, output_tokens);

    for (model, provider) in models {
        if let Ok(breakdown) = generic_cost_per_token(model, &usage, provider) {
            let cost_per_token = breakdown.total_cost / (input_tokens + output_tokens) as f64;
            let efficiency_score = if breakdown.total_cost > 0.0 {
                (input_tokens + output_tokens) as f64 / breakdown.total_cost
            } else {
                0.0
            };

            comparisons.push(ModelCostComparison {
                model: model.clone(),
                provider: provider.clone(),
                total_cost: breakdown.total_cost,
                cost_per_token,
                efficiency_score,
            });
        }
    }

    // Sort by cost (lowest first)
    comparisons.sort_by(|a, b| {
        a.total_cost
            .partial_cmp(&b.total_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    comparisons
}

// Provider-specific pricing functions
// These will be populated with actual pricing data from JSON or database

fn get_openai_pricing(model: &str) -> Result<ModelPricing, CostError> {
    use chrono::Utc;

    let pricing = match model.to_lowercase().as_str() {
        m if m.contains("gpt-4o-mini") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.00015,
            output_cost_per_1k_tokens: 0.0006,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        m if m.contains("gpt-4o") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.005,
            output_cost_per_1k_tokens: 0.015,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        m if m.contains("gpt-4-turbo") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.01,
            output_cost_per_1k_tokens: 0.03,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        m if m.contains("gpt-3.5-turbo") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.0005,
            output_cost_per_1k_tokens: 0.0015,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        _ => {
            return Err(CostError::ModelNotSupported {
                model: model.to_string(),
                provider: "openai".to_string(),
            });
        }
    };

    Ok(pricing)
}

fn get_anthropic_pricing(model: &str) -> Result<ModelPricing, CostError> {
    use chrono::Utc;

    let pricing = match model.to_lowercase().as_str() {
        m if m.contains("claude-3-5-sonnet") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.003,
            output_cost_per_1k_tokens: 0.015,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        m if m.contains("claude-3-5-haiku") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.001,
            output_cost_per_1k_tokens: 0.005,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        m if m.contains("claude-3-haiku") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.00025,
            output_cost_per_1k_tokens: 0.00125,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        _ => {
            return Err(CostError::ModelNotSupported {
                model: model.to_string(),
                provider: "anthropic".to_string(),
            });
        }
    };

    Ok(pricing)
}

fn get_azure_pricing(model: &str) -> Result<ModelPricing, CostError> {
    // Azure pricing is typically the same as OpenAI but may have regional differences
    get_openai_pricing(model).map(|mut pricing| {
        pricing.model = model.to_string();
        pricing
    })
}

fn get_vertex_ai_pricing(model: &str) -> Result<ModelPricing, CostError> {
    use chrono::Utc;

    let pricing = match model.to_lowercase().as_str() {
        m if m.contains("gemini-pro") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.00125,
            output_cost_per_1k_tokens: 0.00375,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        m if m.contains("gemini-flash") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.000075,
            output_cost_per_1k_tokens: 0.0003,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        _ => {
            return Err(CostError::ModelNotSupported {
                model: model.to_string(),
                provider: "vertex_ai".to_string(),
            });
        }
    };

    Ok(pricing)
}

fn get_deepseek_pricing(model: &str) -> Result<ModelPricing, CostError> {
    use chrono::Utc;

    let pricing = match model.to_lowercase().as_str() {
        m if m.contains("deepseek-chat") => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.00014,
            output_cost_per_1k_tokens: 0.00028,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        _ => {
            return Err(CostError::ModelNotSupported {
                model: model.to_string(),
                provider: "deepseek".to_string(),
            });
        }
    };

    Ok(pricing)
}

fn get_moonshot_pricing(model: &str) -> Result<ModelPricing, CostError> {
    use chrono::Utc;

    let pricing = match model.to_lowercase().as_str() {
        "moonshot-v1-8k" => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.01,
            output_cost_per_1k_tokens: 0.02,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        "moonshot-v1-32k" => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.02,
            output_cost_per_1k_tokens: 0.04,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        "moonshot-v1-128k" => ModelPricing {
            model: model.to_string(),
            input_cost_per_1k_tokens: 0.03,
            output_cost_per_1k_tokens: 0.06,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        _ => {
            return Err(CostError::ModelNotSupported {
                model: model.to_string(),
                provider: "moonshot".to_string(),
            });
        }
    };

    Ok(pricing)
}
