//! OpenRouter Provider Completion Example
//!
//! OpenRouter provides unified access to 100+ models from various providers
//! Run with: OPENROUTER_API_KEY=xxx cargo run --example openrouter_completion

use litellm_rs::completion;
use litellm_rs::{system_message, user_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 OpenRouter Completion Example\n");
    println!("OpenRouter provides access to models from multiple providers through one API\n");

    let messages = vec![
        system_message("You are a helpful assistant."),
        user_message("Hello! Briefly introduce yourself and mention which model you are."),
    ];

    // OpenAI models via OpenRouter
    println!("📤 Testing OpenAI GPT-4 via OpenRouter...\n");

    match completion("openrouter/openai/gpt-5-mini", messages.clone(), None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ openai/gpt-5-mini Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Anthropic Claude via OpenRouter
    println!("📤 Testing Claude 3 Sonnet via OpenRouter...\n");

    match completion(
        "openrouter/anthropic/claude-sonnet-4",
        messages.clone(),
        None,
    )
    .await
    {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ Claude Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Google models via OpenRouter
    println!("📤 Testing Google Gemini Pro via OpenRouter...\n");

    match completion("openrouter/google/gemini-2.5-pro", messages.clone(), None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ Gemini Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Meta Llama via OpenRouter
    println!("📤 Testing Llama 3 70B via OpenRouter...\n");

    match completion(
        "openrouter/meta-llama/llama-3-70b-instruct",
        messages.clone(),
        None,
    )
    .await
    {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ Llama 3 Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Mixtral via OpenRouter (open-source alternative)
    println!("📤 Testing Mixtral 8x7B via OpenRouter...\n");

    match completion("openrouter/mistralai/mixtral-8x7b-instruct", messages, None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ Mixtral Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    println!("💡 OpenRouter Tips:");
    println!("   • Use format: openrouter/vendor/model");
    println!("   • Check available models at openrouter.ai/models");
    println!("   • Set custom routing preferences in your OpenRouter dashboard");
    println!("   • Some models may require additional permissions\n");

    Ok(())
}
