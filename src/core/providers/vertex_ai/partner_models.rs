//! Partner models support (Anthropic, Meta, AI21, etc.)

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Partner model provider
#[derive(Debug, Clone)]
pub enum PartnerProvider {
    Anthropic,
    Meta,
    AI21,
    Mistral,
    Cohere,
}

/// Anthropic-specific parameters via Vertex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicVertexParams {
    pub anthropic_version: String,
    pub max_tokens: i32,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub stop_sequences: Option<Vec<String>>,
    pub system: Option<String>,
}

/// Meta Llama-specific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaVertexParams {
    pub max_gen_len: Option<i32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
}

/// AI21 Jamba-specific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JambaVertexParams {
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
}

/// Transform partner model request
pub fn transform_partner_request(
    provider: &PartnerProvider,
    messages: Vec<Value>,
    params: Value,
) -> Value {
    match provider {
        PartnerProvider::Anthropic => {
            serde_json::json!({
                "anthropic_version": "vertex-2023-10-16",
                "messages": messages,
                "parameters": params
            })
        }
        PartnerProvider::Meta => {
            serde_json::json!({
                "instances": [{
                    "messages": messages
                }],
                "parameters": params
            })
        }
        PartnerProvider::AI21 => {
            serde_json::json!({
                "instances": [{
                    "messages": messages
                }],
                "parameters": params
            })
        }
        _ => {
            serde_json::json!({
                "instances": [{
                    "messages": messages
                }],
                "parameters": params
            })
        }
    }
}
