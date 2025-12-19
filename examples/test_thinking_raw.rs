//! Test to see raw API response from DeepSeek R1
//!
//! Run with: cargo run --example test_thinking_raw --all-features

use reqwest::Client;
use serde_json::{json, Value};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Raw API Response Test ===\n");

    let api_key = env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable must be set");

    let client = Client::new();

    let request_body = json!({
        "model": "deepseek/deepseek-r1",
        "messages": [
            {
                "role": "user",
                "content": "What is 2+2? Think step by step."
            }
        ],
        "max_tokens": 1000
    });

    println!("Sending request to OpenRouter...\n");
    println!("Request body: {}", serde_json::to_string_pretty(&request_body)?);

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://github.com/litellm-rs")
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();
    println!("\nResponse Status: {}", status);

    let response_text = response.text().await?;
    println!("\n=== RAW RESPONSE ===");
    println!("{}", response_text);

    // Parse and pretty print
    if let Ok(json_value) = serde_json::from_str::<Value>(&response_text) {
        println!("\n=== PARSED RESPONSE ===");
        println!("{}", serde_json::to_string_pretty(&json_value)?);

        // Check for reasoning_content
        if let Some(choices) = json_value.get("choices").and_then(|c| c.as_array()) {
            for (i, choice) in choices.iter().enumerate() {
                println!("\n=== Choice {} ===", i);
                if let Some(message) = choice.get("message") {
                    println!("Message keys: {:?}", message.as_object().map(|o| o.keys().collect::<Vec<_>>()));

                    // Check specific fields
                    if let Some(content) = message.get("content") {
                        println!("content: {} chars", content.as_str().map(|s| s.len()).unwrap_or(0));
                    }
                    if let Some(reasoning) = message.get("reasoning") {
                        println!("reasoning: {:?}", reasoning);
                    }
                    if let Some(reasoning_content) = message.get("reasoning_content") {
                        println!("reasoning_content: {} chars", reasoning_content.as_str().map(|s| s.len()).unwrap_or(0));
                        println!("reasoning_content preview: {:?}", reasoning_content.as_str().map(|s| &s[..s.len().min(200)]));
                    }
                }
            }
        }

        // Check usage
        if let Some(usage) = json_value.get("usage") {
            println!("\n=== Usage ===");
            println!("{}", serde_json::to_string_pretty(usage)?);
        }
    }

    Ok(())
}
