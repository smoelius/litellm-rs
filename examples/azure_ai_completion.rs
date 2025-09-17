//! Azure AI Provider Completion Example
//!
//! Azure AI provides access to OpenAI models through Azure infrastructure
//! Run with: AZURE_AI_API_KEY=xxx AZURE_AI_API_BASE=xxx cargo run --example azure_ai_completion
//!
//! Before running, ensure you have:
//! 1. An Azure AI resource created
//! 2. Model deployments configured (e.g., gpt-4, gpt-3.5-turbo)
//! 3. API key and endpoint URL

use litellm_rs::{completion, CompletionOptions};
use litellm_rs::{system_message, user_message, assistant_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔷 Azure AI Completion Example\n");
    println!("Azure AI provides enterprise-grade OpenAI models with additional security and compliance\n");

    // Check environment variables
    if std::env::var("AZURE_AI_API_KEY").is_err() || std::env::var("AZURE_AI_API_BASE").is_err() {
        println!("⚠️  Please set environment variables:");
        println!("   export AZURE_AI_API_KEY='your-api-key'");
        println!("   export AZURE_AI_API_BASE='https://your-resource.cognitiveservices.azure.com'");
        println!("\n   Or for Azure OpenAI Service:");
        println!("   export AZURE_AI_API_BASE='https://your-resource.openai.azure.com'");
        return Ok(());
    }

    // Simple conversation
    println!("📤 Testing GPT-4o on Azure AI...\n");
    
    let messages = vec![
        system_message("You are a helpful AI assistant running on Azure infrastructure."),
        user_message("Hello! What makes Azure AI special compared to direct OpenAI API?"),
    ];

    match completion("azure_ai/gpt-4o", messages.clone(), None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ GPT-4o Response: {:?}\n", content);
            }
            if let Some(usage) = response.usage {
                println!("📊 Tokens - Input: {}, Output: {}, Total: {}\n", 
                    usage.prompt_tokens, 
                    usage.completion_tokens, 
                    usage.total_tokens);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Testing with parameters
    println!("📤 Testing with custom parameters (temperature, max_tokens)...\n");
    
    let params = CompletionOptions {
        temperature: Some(0.7),
        max_tokens: Some(150),
        top_p: Some(0.9),
        ..Default::default()
    };

    let creative_messages = vec![
        system_message("You are a creative storyteller."),
        user_message("Write a haiku about cloud computing."),
    ];

    match completion("azure_ai/gpt-4o", creative_messages, Some(params)).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ Creative Response:\n{:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Multi-turn conversation
    println!("📤 Testing multi-turn conversation...\n");
    
    let conversation = vec![
        system_message("You are an Azure expert. Be concise but informative."),
        user_message("What is Azure AI Foundry?"),
        assistant_message("Azure AI Foundry is Microsoft's comprehensive platform for building, deploying, and managing AI applications. It provides unified access to various AI models, tools for MLOps, and enterprise-grade security features."),
        user_message("Can I use it with open-source models?"),
    ];

    match completion("azure_ai/gpt-4o", conversation, None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ Conversation Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Error: {}\n", e),
    }

    // Testing different models (if deployed)
    println!("📤 Testing GPT-3.5-Turbo on Azure AI...\n");
    
    let fast_messages = vec![
        user_message("Explain quantum computing in one sentence."),
    ];

    match completion("azure_ai/gpt-35-turbo", fast_messages, None).await {
        Ok(response) => {
            if let Some(ref content) = response.choices[0].message.content {
                println!("✅ GPT-3.5-Turbo Response: {:?}\n", content);
            }
        }
        Err(e) => println!("❌ Note: {}\n   (This model might not be deployed in your Azure resource)\n", e),
    }

    // Testing with embedding models
    println!("📤 Testing Text Embedding Model...\n");
    
    // Note: This would use the embedding endpoint, not chat completion
    println!("ℹ️  Text embeddings require using the embedding-specific functions\n");

    // Testing with DALL-E (if available)
    println!("📤 Image Generation with DALL-E 3...\n");
    
    println!("ℹ️  Image generation requires using the image-specific functions\n");

    println!("💡 Azure AI Tips:");
    println!("   • Use format: azure_ai/your-deployment-name");
    println!("   • Models must be deployed in your Azure resource first");
    println!("   • Check Azure Portal for available deployments");
    println!("   • Monitor costs in Azure Cost Management");
    println!("   • Use managed identities for production scenarios");
    println!("   • Enable content filtering for safety");
    println!("\n🔗 Supported Azure AI Models:");
    println!("   • GPT-4o, GPT-4, GPT-4-Turbo");
    println!("   • GPT-3.5-Turbo");
    println!("   • Text-Embedding-3-Large, Text-Embedding-3-Small");
    println!("   • DALL-E 3 (for image generation)");
    println!("   • Whisper (for speech-to-text)");
    println!("   • Custom fine-tuned models\n");

    Ok(())
}