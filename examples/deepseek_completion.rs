//! DeepSeek Provider Completion Example
//!
//! DeepSeek provides advanced reasoning capabilities with V3.1 models
//! Run with: DEEPSEEK_API_KEY=xxx cargo run --example deepseek_completion

use litellm_rs::completion;
use litellm_rs::{system_message, user_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 DeepSeek Completion Example\n");
    println!("DeepSeek V3.1 offers both standard chat and advanced reasoning modes\n");

    let messages = vec![
        system_message("You are a helpful assistant."),
        user_message("Hello! Briefly introduce yourself and mention which model you are."),
    ];

    // DeepSeek Chat Mode (faster, good for general tasks)
    println!("📤 Testing DeepSeek Chat Mode...\n");

    match completion("deepseek/deepseek-chat", messages.clone(), None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ DeepSeek Chat Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // DeepSeek Reasoner Mode (thinking mode, better for complex reasoning)
    println!("📤 Testing DeepSeek Reasoner Mode...\n");

    let reasoning_messages = vec![
        system_message("You are a helpful assistant capable of deep reasoning."),
        user_message("Solve this step by step: If a train travels 60 mph for 2 hours, then 80 mph for 1.5 hours, what's the total distance?"),
    ];

    match completion("deepseek/deepseek-reasoner", reasoning_messages, None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ DeepSeek Reasoner Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Example with coding task
    println!("📤 Testing DeepSeek with Coding Task...\n");

    let coding_messages = vec![
        system_message("You are an expert programmer."),
        user_message("Write a simple function in Rust that calculates the factorial of a number."),
    ];

    match completion("deepseek/deepseek-chat", coding_messages, None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ DeepSeek Coding Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Complex reasoning example
    println!("📤 Testing DeepSeek Reasoner with Complex Problem...\n");

    let complex_messages = vec![
        system_message("You are a helpful assistant that thinks step by step."),
        user_message("Explain the philosophical implications of artificial intelligence achieving human-level reasoning capabilities."),
    ];

    match completion("deepseek/deepseek-reasoner", complex_messages, None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ DeepSeek Complex Reasoning Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }


    Ok(())
}