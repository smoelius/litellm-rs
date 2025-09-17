# DeepSeek Provider

DeepSeek is an AI company that provides powerful language models with competitive performance and pricing. LiteLLM-RS supports DeepSeek V3.1 models through their official API.

## 🚀 Models Available

### DeepSeek V3.1 Models

| Model | Context | Max Output | Pricing (per 1M tokens) | Use Case |
|-------|---------|------------|-------------------------|----------|
| **deepseek-chat** | 128K | 8K | $0.56 input, $1.68 output | General chat, coding, analysis |
| **deepseek-reasoner** | 128K | 8K | $0.56 input, $1.68 output | Advanced reasoning, complex problem solving |

### Model Capabilities

| Feature | deepseek-chat | deepseek-reasoner |
|---------|---------------|-------------------|
| **Chat Completion** | ✅ | ✅ |
| **Streaming** | ✅ | ✅ |
| **Function Calling** | ✅ | ✅ |
| **System Messages** | ✅ | ✅ |
| **Reasoning Mode** | ❌ | ✅ |
| **Vision Support** | ❌ | ❌ |
| **Embeddings** | ❌ | ❌ |

## ⚙️ Setup & Configuration

### Environment Variables
```bash
export DEEPSEEK_API_KEY=your_deepseek_api_key_here
```

### YAML Configuration
```yaml
providers:
  deepseek:
    api_key: "${DEEPSEEK_API_KEY}"
    api_base: "https://api.deepseek.com"
    timeout_seconds: 30
    max_retries: 3
    extra_params:
      reasoning_effort: "medium"  # For deepseek-reasoner
```

### Programmatic Configuration
```rust
use litellm_rs::core::providers::deepseek::{DeepSeekProvider, DeepSeekConfig};

let config = DeepSeekConfig {
    api_key: Some("your_api_key".to_string()),
    api_base: "https://api.deepseek.com".to_string(),
    timeout_seconds: 30,
    max_retries: 3,
    ..Default::default()
};

let provider = DeepSeekProvider::new(config).await?;
```

## 🔨 Usage Examples

### Basic Chat Completion
```rust
use litellm_rs::{completion, user_message, system_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = completion(
        "deepseek-chat",
        vec![
            system_message("You are a helpful programming assistant."),
            user_message("Explain the difference between stack and heap memory in Rust."),
        ],
        None,
    ).await?;

    println!("DeepSeek: {}", response.choices[0].message.content);
    Ok(())
}
```

### Advanced Reasoning with DeepSeek Reasoner
```rust
use litellm_rs::{completion, user_message, system_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = completion(
        "deepseek-reasoner",
        vec![
            system_message("You are an expert problem solver. Show your reasoning process."),
            user_message("A farmer has chickens and rabbits. In total, there are 35 heads and 94 legs. How many chickens and how many rabbits are there?"),
        ],
        None,
    ).await?;

    println!("DeepSeek Reasoner: {}", response.choices[0].message.content);
    Ok(())
}
```

### Streaming Responses
```rust
use litellm_rs::core::providers::deepseek::{DeepSeekProvider, DeepSeekConfig};
use litellm_rs::core::types::requests::ChatRequest;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DeepSeekConfig::from_env();
    let provider = DeepSeekProvider::new(config).await?;
    
    let request = ChatRequest::new("deepseek-chat")
        .add_user_message("Tell me a short story about AI and creativity")
        .with_stream(true);
    
    let mut stream = provider.chat_completion_stream(request, Default::default()).await?;
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                for choice in chunk.choices {
                    if let Some(content) = choice.delta.content {
                        print!("{}", content);
                    }
                }
            }
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }
    
    Ok(())
}
```

### Function Calling
```rust
use litellm_rs::{completion, user_message, CompletionOptions};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tools = vec![json!({
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get weather information for a city",
            "parameters": {
                "type": "object",
                "properties": {
                    "city": {"type": "string", "description": "The city name"},
                    "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                },
                "required": ["city"]
            }
        }
    })];

    let options = CompletionOptions::default()
        .with_tools(tools)
        .with_tool_choice("auto");

    let response = completion(
        "deepseek-chat",
        vec![user_message("What's the weather like in San Francisco?")],
        Some(options),
    ).await?;

    println!("Response: {:?}", response.choices[0]);
    Ok(())
}
```

### Multi-turn Conversation
```rust
use litellm_rs::{completion, user_message, system_message, assistant_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conversation = vec![
        system_message("You are a helpful coding mentor."),
        user_message("I'm learning Rust and struggling with lifetimes."),
        assistant_message("Lifetimes are indeed challenging! They ensure memory safety by tracking how long references are valid. What specific issue are you encountering?"),
        user_message("I keep getting 'borrowed value does not live long enough' errors when returning references from functions."),
    ];

    let response = completion("deepseek-chat", conversation, None).await?;
    
    println!("DeepSeek: {}", response.choices[0].message.content);
    Ok(())
}
```

## 🎛️ Advanced Configuration

### Custom Parameters
```rust
use litellm_rs::core::providers::deepseek::DeepSeekConfig;
use std::collections::HashMap;

let mut extra_params = HashMap::new();
extra_params.insert("reasoning_effort".to_string(), json!("high"));
extra_params.insert("temperature".to_string(), json!(0.7));

let config = DeepSeekConfig {
    extra_params,
    ..Default::default()
};
```

### Request Options
```rust
use litellm_rs::{completion, CompletionOptions};

let options = CompletionOptions::default()
    .with_temperature(0.8)
    .with_max_tokens(1000)
    .with_top_p(0.9)
    .with_stop_sequences(vec!["###".to_string()]);

let response = completion("deepseek-chat", messages, Some(options)).await?;
```

### Custom Headers
```rust
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("X-Custom-Header".to_string(), "value".to_string());

let config = DeepSeekConfig {
    headers,
    ..Default::default()
};
```

## 📊 Performance & Pricing

### Performance Characteristics
- **Latency**: ~900ms average response time
- **Throughput**: Up to 500 requests/minute (API limits)
- **Context Window**: 128K tokens (both models)
- **Max Output**: 8K tokens

### Cost Comparison
```rust
use litellm_rs::core::providers::base::calculate_cost;

// Example usage costs
let input_tokens = 1000;
let output_tokens = 500;
let cost = calculate_cost("deepseek-chat", input_tokens, output_tokens);
println!("Cost: ${:.4}", cost); // ~$0.0014
```

### Cost vs Competitors (per 1M tokens)
- **DeepSeek**: $0.56 input, $1.68 output
- **GPT-3.5-Turbo**: $0.50 input, $1.50 output
- **GPT-4**: $30.00 input, $60.00 output
- **Claude Sonnet**: $3.00 input, $15.00 output

## 🔍 Model Comparison

### deepseek-chat vs deepseek-reasoner

| Aspect | deepseek-chat | deepseek-reasoner |
|--------|---------------|-------------------|
| **Best For** | General chat, coding, analysis | Complex reasoning, math, logic |
| **Speed** | Faster responses | Slower (more thinking time) |
| **Reasoning** | Good reasoning | Exceptional reasoning |
| **Cost** | Same pricing | Same pricing |
| **Use Cases** | Code generation, Q&A, creative writing | Problem solving, research, analysis |

### When to Use Each Model

**Use `deepseek-chat` for:**
- Code generation and explanation
- General conversation
- Content creation
- Quick analysis tasks
- Most everyday AI tasks

**Use `deepseek-reasoner` for:**
- Complex mathematical problems
- Multi-step reasoning tasks
- Research and analysis
- Logic puzzles
- Scientific problem solving

## 🛠️ Error Handling

### Common Error Types
```rust
use litellm_rs::core::providers::deepseek::DeepSeekError;

match completion("deepseek-chat", messages, None).await {
    Ok(response) => println!("Success: {}", response.choices[0].message.content),
    Err(e) => match e.downcast_ref::<DeepSeekError>() {
        Some(DeepSeekError::Authentication(_)) => {
            println!("Check your DEEPSEEK_API_KEY environment variable");
        }
        Some(DeepSeekError::RateLimit(_)) => {
            println!("Rate limit exceeded, please wait");
        }
        Some(DeepSeekError::InvalidRequest(msg)) => {
            println!("Invalid request: {}", msg);
        }
        _ => println!("Unknown error: {}", e),
    }
}
```

### Retry Configuration
```rust
let config = DeepSeekConfig {
    max_retries: 5,
    timeout_seconds: 60,
    ..Default::default()
};
```

## 🧪 Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deepseek_chat() {
        let response = completion(
            "deepseek-chat",
            vec![user_message("Hello, world!")],
            None,
        ).await;
        
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_deepseek_reasoner() {
        let response = completion(
            "deepseek-reasoner",
            vec![user_message("What is 2+2?")],
            None,
        ).await;
        
        assert!(response.is_ok());
    }
}
```

### Integration Testing
```bash
# Set API key for integration tests
export DEEPSEEK_API_KEY=your_key_here

# Run integration tests
cargo test --all-features deepseek_integration -- --ignored
```

## 📋 Best Practices

### 1. **Model Selection**
- Use `deepseek-chat` for most general tasks
- Reserve `deepseek-reasoner` for complex reasoning
- Consider cost vs. quality trade-offs

### 2. **Error Handling**
- Always handle rate limiting gracefully
- Implement retry logic for transient failures
- Check API key validity at startup

### 3. **Performance Optimization**
- Reuse HTTP connections when possible
- Implement request batching for multiple queries
- Use streaming for long responses

### 4. **Cost Management**
- Monitor token usage in production
- Set max_tokens to control costs
- Use temperature settings appropriate for your use case

## 🔗 Additional Resources

- [DeepSeek Official API Documentation](https://api.deepseek.com/docs)
- [DeepSeek Model Cards](https://deepseek.com/models)
- [Provider Implementation Guide](../architecture/provider-implementation.md)
- [Example Code Repository](../../examples/)