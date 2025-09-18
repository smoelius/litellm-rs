# LiteLLM-RS Documentation

A high-performance AI Gateway written in Rust that provides unified access to 100+ AI providers through OpenAI-compatible APIs.

## 📚 Documentation Structure

### Architecture & Design
- [System Overview](./architecture/system-overview.md) - Complete system architecture and design patterns
- [Error System](./architecture/error-system.md) - Unified error handling architecture and patterns
- [Provider Implementation](./architecture/provider-implementation.md) - Guide for implementing individual providers
- [Architecture Improvements](./architecture/improvements.md) - Historical improvements and optimizations

### Implementation Guides
- [Getting Started](./guides/getting-started.md) - Quick start guide and basic usage
- [Configuration](./guides/configuration.md) - Configuration management and environment setup
- [Deployment](./guides/deployment.md) - Production deployment strategies
- [Testing](./guides/testing.md) - Testing strategies and best practices

### Provider Documentation
- [Provider Overview](./providers/README.md) - Supported providers and capabilities
- [DeepSeek](./providers/deepseek.md) - DeepSeek V3.1 integration guide
- [OpenAI](./providers/openai.md) - OpenAI and compatible providers
- [Anthropic](./providers/anthropic.md) - Claude models integration
- [Adding Providers](./providers/adding-new-provider.md) - Step-by-step provider implementation

### Examples & Tutorials
- [Basic Examples](./examples/basic-usage.md) - Simple completion examples
- [Advanced Features](./examples/advanced-features.md) - Streaming, function calling, etc.
- [Integration Examples](./examples/integrations.md) - Web frameworks and service integrations

## 🚀 Quick Start

```rust
use litellm_rs::{completion, user_message, system_message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = completion(
        "gpt-4",
        vec![
            system_message("You are a helpful assistant."),
            user_message("Hello, how are you?"),
        ],
        None,
    ).await?;
    
    println!("Response: {}", response.choices[0].message.content);
    Ok(())
}
```

## 🏗️ Architecture Highlights

- **High Performance**: Built with Rust and Tokio for maximum throughput (10,000+ req/s)
- **OpenAI Compatible**: Drop-in replacement for OpenAI API
- **100+ Providers**: Unified interface to all major AI providers
- **Intelligent Routing**: Smart load balancing and failover
- **Enterprise Ready**: Authentication, monitoring, cost tracking
- **Type Safety**: Compile-time guarantees and zero-cost abstractions

## 📖 Key Concepts

### Provider System
LiteLLM-RS uses a trait-based provider system that ensures consistency across all AI providers while allowing for provider-specific optimizations.

### Routing Engine
Sophisticated routing with multiple strategies:
- Round Robin
- Least Latency
- Cost Optimized
- Health-Based
- Custom Weighted

### Unified Error Handling
All provider-specific errors are mapped to a unified error system for consistent error handling across the entire system.

## 🛠️ Development

### Prerequisites
- Rust 1.70+
- PostgreSQL (optional)
- Redis (optional)

### Essential Commands
```bash
# Development
make dev              # Start development server
cargo test --all-features  # Run tests
cargo clippy --all-features  # Lint code

# Production
make build            # Build release binary
make docker           # Build Docker image
```

## 🤝 Contributing

1. Read the [Provider Implementation Guide](./architecture/provider-implementation.md)
2. Check existing [issues](https://github.com/your-org/litellm-rs/issues)
3. Follow the [development setup](./guides/getting-started.md#development-setup)
4. Submit PRs with tests and documentation

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.