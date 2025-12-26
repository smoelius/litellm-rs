//! Partner models support (Anthropic, Meta, AI21, etc.)

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Partner model provider
#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partner_provider_equality() {
        assert_eq!(PartnerProvider::Anthropic, PartnerProvider::Anthropic);
        assert_ne!(PartnerProvider::Anthropic, PartnerProvider::Meta);
    }

    #[test]
    fn test_anthropic_vertex_params() {
        let params = AnthropicVertexParams {
            anthropic_version: "vertex-2023-10-16".to_string(),
            max_tokens: 1024,
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            stop_sequences: Some(vec!["STOP".to_string()]),
            system: Some("You are helpful".to_string()),
        };

        assert_eq!(params.anthropic_version, "vertex-2023-10-16");
        assert_eq!(params.max_tokens, 1024);
        assert_eq!(params.temperature, Some(0.7));
        assert!(params.stop_sequences.is_some());
    }

    #[test]
    fn test_anthropic_vertex_params_serialization() {
        let params = AnthropicVertexParams {
            anthropic_version: "vertex-2023-10-16".to_string(),
            max_tokens: 1024,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            system: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["max_tokens"], 1024);
        assert_eq!(json["anthropic_version"], "vertex-2023-10-16");
    }

    #[test]
    fn test_llama_vertex_params() {
        let params = LlamaVertexParams {
            max_gen_len: Some(512),
            temperature: Some(0.8),
            top_p: Some(0.95),
        };

        assert_eq!(params.max_gen_len, Some(512));
        assert_eq!(params.temperature, Some(0.8));
    }

    #[test]
    fn test_llama_vertex_params_serialization() {
        let params = LlamaVertexParams {
            max_gen_len: Some(256),
            temperature: Some(0.5),
            top_p: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["max_gen_len"], 256);
        assert_eq!(json["temperature"], 0.5);
    }

    #[test]
    fn test_jamba_vertex_params() {
        let params = JambaVertexParams {
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(50),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.3),
        };

        assert_eq!(params.max_tokens, Some(1000));
        assert_eq!(params.frequency_penalty, Some(0.5));
        assert_eq!(params.presence_penalty, Some(0.3));
    }

    #[test]
    fn test_transform_partner_request_anthropic() {
        let messages = vec![serde_json::json!({"role": "user", "content": "Hello"})];
        let params = serde_json::json!({"max_tokens": 100});

        let result = transform_partner_request(&PartnerProvider::Anthropic, messages, params);

        assert_eq!(result["anthropic_version"], "vertex-2023-10-16");
        assert!(result["messages"].is_array());
        assert!(result["parameters"].is_object());
    }

    #[test]
    fn test_transform_partner_request_meta() {
        let messages = vec![serde_json::json!({"role": "user", "content": "Hello"})];
        let params = serde_json::json!({"temperature": 0.7});

        let result = transform_partner_request(&PartnerProvider::Meta, messages, params);

        assert!(result["instances"].is_array());
        assert!(result["instances"][0]["messages"].is_array());
        assert!(result["parameters"].is_object());
    }

    #[test]
    fn test_transform_partner_request_ai21() {
        let messages = vec![serde_json::json!({"role": "user", "content": "Hello"})];
        let params = serde_json::json!({"top_p": 0.9});

        let result = transform_partner_request(&PartnerProvider::AI21, messages, params);

        assert!(result["instances"].is_array());
        assert!(result["parameters"].is_object());
    }

    #[test]
    fn test_transform_partner_request_mistral() {
        let messages = vec![serde_json::json!({"role": "user", "content": "Hello"})];
        let params = serde_json::json!({});

        let result = transform_partner_request(&PartnerProvider::Mistral, messages, params);

        // Mistral uses the default format
        assert!(result["instances"].is_array());
    }

    #[test]
    fn test_transform_partner_request_cohere() {
        let messages = vec![serde_json::json!({"role": "user", "content": "Hello"})];
        let params = serde_json::json!({});

        let result = transform_partner_request(&PartnerProvider::Cohere, messages, params);

        // Cohere uses the default format
        assert!(result["instances"].is_array());
    }

    #[test]
    fn test_transform_partner_request_empty_messages() {
        let messages: Vec<Value> = vec![];
        let params = serde_json::json!({});

        let result = transform_partner_request(&PartnerProvider::Anthropic, messages, params);

        assert!(result["messages"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_transform_partner_request_multiple_messages() {
        let messages = vec![
            serde_json::json!({"role": "system", "content": "You are helpful"}),
            serde_json::json!({"role": "user", "content": "Hello"}),
            serde_json::json!({"role": "assistant", "content": "Hi there!"}),
            serde_json::json!({"role": "user", "content": "How are you?"}),
        ];
        let params = serde_json::json!({"max_tokens": 500});

        let result = transform_partner_request(&PartnerProvider::Anthropic, messages, params);

        assert_eq!(result["messages"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_partner_provider_debug() {
        let provider = PartnerProvider::Anthropic;
        assert!(format!("{:?}", provider).contains("Anthropic"));
    }

    #[test]
    fn test_partner_provider_clone() {
        let provider = PartnerProvider::Meta;
        let cloned = provider.clone();
        assert_eq!(provider, cloned);
    }
}
