# litellm-rs

[![crates.io](https://img.shields.io/crates/v/litellm-rs.svg)](https://crates.io/crates/litellm-rs)
[![Documentation](https://docs.rs/litellm-rs/badge.svg)](https://docs.rs/litellm-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Build Status](https://github.com/majiayu000/litellm-rs/workflows/CI/badge.svg)](https://github.com/majiayu000/litellm-rs/actions)

A high-performance Rust library for unified LLM API access.

`litellm-rs` provides a simple, consistent interface to interact with multiple AI providers (OpenAI, Anthropic, Google, Azure, and more) through a single, unified API. Built with Rust's performance and safety guarantees, it simplifies multi-provider AI integration in production systems.

```rust
use litellm_rs::{completion, user_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Works with any supported provider
    let response = completion(
        "gpt-4",  // or "claude-3", "gemini-pro", etc.
        vec![user_message("Hello!")],
        None,
    ).await?;

    println!("{}", response.choices[0].message.content.as_ref().unwrap());
    Ok(())
}
```

## Key Features

- **Unified API** - Single interface for OpenAI, Anthropic, Google, Azure, and 100+ other providers
- **High Performance** - Built in Rust with async/await for maximum throughput
- **Production Ready** - Automatic retries, comprehensive error handling, and provider failover
- **Flexible Deployment** - Use as a Rust library or deploy as a standalone HTTP gateway
- **OpenAI Compatible** - Works with existing OpenAI client libraries and tools

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
litellm-rs = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

Or build from source:

```bash
git clone https://github.com/majiayu000/litellm-rs.git
cd litellm-rs
cargo build --release
```

## Usage

### As a Library

#### Basic Example

```rust
use litellm_rs::{completion, user_message, system_message, CompletionOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set your API key
    std::env::set_var("OPENAI_API_KEY", "your-openai-key");

    // Simple completion call
    let response = completion(
        "gpt-4",
        vec![user_message("Hello, how are you?")],
        None,
    ).await?;

    println!("Response: {}", response.choices[0].message.content.as_ref().unwrap());

    // With system message and options
    let response = completion(
        "gpt-4",
        vec![
            system_message("You are a helpful assistant."),
            user_message("Explain quantum computing"),
        ],
        Some(CompletionOptions {
            temperature: Some(0.7),
            max_tokens: Some(150),
            ..Default::default()
        }),
    ).await?;

    println!("AI: {}", response.choices[0].message.content.as_ref().unwrap());
    Ok(())
}
```

#### Using Multiple Providers

```rust
use litellm_rs::{completion, user_message, CompletionOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set API keys for different providers
    std::env::set_var("OPENAI_API_KEY", "your-openai-key");
    std::env::set_var("ANTHROPIC_API_KEY", "your-anthropic-key");
    std::env::set_var("GROQ_API_KEY", "your-groq-key");

    // Call OpenAI
    let openai_response = completion(
        "gpt-4",
        vec![user_message("Hello from OpenAI!")],
        None,
    ).await?;

    // Call Anthropic Claude
    let claude_response = completion(
        "anthropic/claude-3-sonnet-20240229",
        vec![user_message("Hello from Claude!")],
        None,
    ).await?;

    // Call Groq (with reasoning)
    let groq_response = completion(
        "groq/deepseek-r1-distill-llama-70b",
        vec![user_message("Solve this math problem: 2+2=?")],
        Some(CompletionOptions {
            extra_params: {
                let mut params = std::collections::HashMap::new();
                params.insert("reasoning_effort".to_string(), serde_json::json!("medium"));
                params
            },
            ..Default::default()
        }),
    ).await?;

    println!("OpenAI: {}", openai_response.choices[0].message.content.as_ref().unwrap());
    println!("Claude: {}", claude_response.choices[0].message.content.as_ref().unwrap());
    println!("Groq: {}", groq_response.choices[0].message.content.as_ref().unwrap());

    Ok(())
}
```

#### Custom Endpoints

```rust
use litellm_rs::{completion, user_message, CompletionOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Call any OpenAI-compatible API with custom endpoint
    let response = completion(
        "llama-3.1-70b",  // Model name
        vec![user_message("Hello from custom endpoint!")],
        Some(CompletionOptions {
            api_key: Some("your-custom-api-key".to_string()),
            api_base: Some("https://your-custom-endpoint.com/v1".to_string()),
            ..Default::default()
        }),
    ).await?;

    println!("Custom API: {}", response.choices[0].message.content.as_ref().unwrap());
    Ok(())
}
```

### As a Gateway Server

Start the server:

```bash
# Set your API keys
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"

# Start the proxy server
cargo run

# Server starts on http://localhost:8000
```

Make requests:

```bash
# OpenAI GPT-4
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello, how are you?"}]
  }'

# Anthropic Claude
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-sonnet",
    "messages": [{"role": "user", "content": "Hello, how are you?"}]
  }'
```

### Response (OpenAI Format)

```json
{
    "id": "chatcmpl-1214900a-6cdd-4148-b663-b5e2f642b4de",
    "created": 1751494488,
    "model": "claude-3-sonnet",
    "object": "chat.completion",
    "choices": [
        {
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "content": "Hello! I'm doing well, thank you for asking. How are you doing today?",
                "role": "assistant"
            }
        }
    ],
    "usage": {
        "completion_tokens": 17,
        "prompt_tokens": 12,
        "total_tokens": 29
    }
}
```

Call any model supported by a provider, with `model=<model_name>`. See [Supported Providers](#-supported-providers) for complete list.

## Streaming ([Docs](docs/))

LiteLLM-RS supports streaming the model response back, pass `stream=true` to get a streaming response.
Streaming is supported for all models (OpenAI, Anthropic, Google, Azure, Groq, etc.)

```bash
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Tell me a story"}],
    "stream": true
  }'
```

## Supported Providers

- **OpenAI** - GPT-4, GPT-3.5, DALL-E
- **Anthropic** - Claude 3 Opus, Sonnet, Haiku
- **Google** - Gemini Pro, Gemini Flash
- **Azure OpenAI** - Managed OpenAI deployments
- **Groq** - High-speed Llama inference
- **AWS Bedrock** - Claude, Llama, and more
- **And 95+ more providers...**

[View all providers →](docs/providers.md)

## Features

- **Unified Interface** - Single API for 100+ providers
- **OpenAI Compatible** - Drop-in replacement for OpenAI client
- **Streaming Support** - Real-time response streaming
- **Automatic Retries** - Built-in exponential backoff
- **Load Balancing** - Distribute requests across providers
- **Cost Tracking** - Monitor spending per request/user
- **Function Calling** - Tool use across all capable models
- **Vision Support** - Multimodal inputs for capable models
- **Custom Endpoints** - Connect to self-hosted models
- **Request Caching** - Reduce costs with intelligent caching
- **Rate Limiting** - Protect against quota exhaustion
- **Observability** - OpenTelemetry tracing and metrics

## Configuration

Create a `config/gateway.yaml` file:

```yaml
server:
  host: "0.0.0.0"
  port: 8000

providers:
  openai:
    api_key: "${OPENAI_API_KEY}"
  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
  google:
    api_key: "${GOOGLE_API_KEY}"

router:
  strategy: "round_robin"
  max_retries: 3
  timeout: 60
```

See [config/gateway.yaml.example](config/gateway.yaml.example) for a complete example.

## Documentation

- [API Reference](https://docs.rs/litellm-rs)
- [Configuration Guide](docs/configuration.md)
- [Provider Documentation](docs/providers.md)
- [Deployment Guide](deployment/README.md)
- [Examples](examples/README.md)

## Performance

| Metric | Value | Notes |
|--------|-------|-------|
| **Throughput** | 10,000+ req/s | On 8-core CPU |
| **Latency** | <10ms | Routing overhead |
| **Memory** | ~50MB | Base footprint |
| **Startup** | <100ms | Cold start time |


## Deployment

### Docker

```bash
docker run -p 8000:8000 \
  -e OPENAI_API_KEY="$OPENAI_API_KEY" \
  -e ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY" \
  litellm-rs:latest
```

### Kubernetes

```bash
kubectl apply -f deployment/kubernetes/
```

### Binary

```bash
# Download the latest release
curl -L https://github.com/majiayu000/litellm-rs/releases/latest/download/litellm-rs-linux-amd64 -o litellm-rs
chmod +x litellm-rs
./litellm-rs
```


## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

```bash
# Setup
git clone https://github.com/majiayu000/litellm-rs.git
cd litellm-rs

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## Roadmap

- [x] Core OpenAI-compatible API
- [x] 15+ provider integrations
- [x] Streaming support
- [x] Automatic retries and failover
- [ ] Response caching
- [ ] WebSocket support
- [ ] Plugin system
- [ ] Web dashboard

See [GitHub Issues](https://github.com/majiayu000/litellm-rs/issues) for detailed roadmap.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

Special thanks to the Rust community and all contributors to this project.

---

*Built with Rust for performance and reliability.*
