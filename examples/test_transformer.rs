//! Test transformer directly with DeepSeek response format
//!
//! Run with: cargo run --example test_transformer --all-features

use litellm_rs::core::providers::openai::models::{OpenAIChatResponse, OpenAIChoice, OpenAIMessage, OpenAIUsage};
use litellm_rs::core::providers::openai::transformer::OpenAIResponseTransformer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Transformer Test ===\n");

    // Create a mock response similar to what DeepSeek R1 returns
    let mock_response = r#"{
        "id": "test-id",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "deepseek-r1",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "The answer is 42.",
                    "reasoning": "Let me think step by step... First, I need to consider..."
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 50,
            "total_tokens": 60
        }
    }"#;

    println!("--- Input JSON ---");
    println!("{}", mock_response);

    // Parse as OpenAIChatResponse
    let openai_response: OpenAIChatResponse = serde_json::from_str(mock_response)?;
    println!("\n--- Parsed OpenAIChatResponse ---");
    println!("ID: {}", openai_response.id);
    println!("Model: {}", openai_response.model);
    if let Some(choice) = openai_response.choices.first() {
        println!("Message role: {}", choice.message.role);
        println!("Message content: {:?}", choice.message.content);
        println!("Message reasoning: {:?}", choice.message.reasoning);
        println!("Message reasoning_content: {:?}", choice.message.reasoning_content);
    }

    // Transform to ChatResponse
    println!("\n--- Transforming to ChatResponse ---");
    let chat_response = OpenAIResponseTransformer::transform(openai_response)?;

    println!("Response ID: {}", chat_response.id);
    println!("Response Model: {}", chat_response.model);

    if let Some(choice) = chat_response.choices.first() {
        println!("\nMessage role: {:?}", choice.message.role);
        println!("Message content: {:?}", choice.message.content);
        println!("Message thinking: {:?}", choice.message.thinking);
    }

    // Serialize back to JSON to see what we get
    println!("\n--- Serialized ChatResponse ---");
    let output_json = serde_json::to_string_pretty(&chat_response)?;
    println!("{}", output_json);

    Ok(())
}
