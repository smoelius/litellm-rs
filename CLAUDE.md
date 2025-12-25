# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Development Commands
- **Start development**: `make dev` or `cargo run` (auto-loads config/gateway.yaml)
- **Build**: `cargo build --all-features` 
- **Test**: `cargo test --all-features`
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Format**: `cargo fmt --all`
- **Quick start**: `make start` (fastest way to start the gateway)

### Testing Commands
- **All tests**: `make test`
- **Unit tests only**: `make test-unit` 
- **Integration tests**: `make test-integration`
- **Test coverage**: `make test-coverage`
- **Single test**: `cargo test <test_name> --all-features`

### Development Services
- **Start dev services**: `make dev-services` (starts PostgreSQL, Redis)
- **Stop dev services**: `make dev-stop`
- **Database migration**: `make db-migrate`
- **Reset database**: `make db-reset`

## Architecture Overview

This is a **high-performance AI Gateway** written in Rust that provides OpenAI-compatible APIs with intelligent routing across 100+ AI providers. It's a Rust implementation of the Python LiteLLM library, designed for production environments requiring maximum throughput and minimal latency.

### Core Components

**Gateway Architecture**: Modular, trait-based design with dependency injection
- `src/core/` - Central orchestrator and business logic
- `src/server/` - Actix-web HTTP server with middleware pipeline
- `src/auth/` - Multi-layered authentication (JWT, API keys, RBAC)
- `src/core/providers/` - Pluggable provider system (OpenAI, Anthropic, Azure, Google, etc.)
- `src/core/router/` - Intelligent routing with multiple strategies
- `src/core/mcp/` - MCP Gateway for external tool integration (90 tests)
- `src/core/a2a/` - A2A Protocol for agent-to-agent communication (48 tests)
- `src/storage/` - Multi-backend storage (PostgreSQL, Redis, S3, Vector DB)
- `src/monitoring/` - Observability (Prometheus, tracing, health checks)

### Key Design Patterns
- **Async-first**: All I/O is non-blocking using Tokio
- **Trait-based abstractions**: Pluggable components via traits
- **Error handling**: Comprehensive error types with context preservation
- **Configuration**: Type-safe config models with Default implementations
- **No backward compatibility**: Break old formats freely - prioritize clean architecture over legacy support

### Provider Integration
- **Unified Provider trait**: Common interface for all AI providers
- **Format conversion**: Automatic translation between OpenAI and provider-specific APIs
- **Health monitoring**: Per-provider health checks and failover
- **Cost calculation**: Built-in token counting and cost estimation

### Request Flow
1. HTTP Request → Authentication → Authorization → Router → Provider → Response
2. Middleware pipeline handles auth, logging, metrics, and transformations
3. Intelligent routing selects optimal provider based on health, latency, cost

## Configuration

- **Main config**: `config/gateway.yaml` (auto-loaded by default)
- **Example config**: `config/gateway.yaml.example`
- **Environment variables**: Override config values with `${ENV_VAR}` syntax
- **Config validation**: `make config-validate`

## Important Files

- `src/main.rs` - Application entry point
- `src/lib.rs` - Library entry point with core Gateway struct and Python LiteLLM compatible exports
- `Cargo.toml` - Dependencies and features (use `--all-features` for development)
- `Makefile` - All development commands and workflows
- `config/gateway.yaml` - Main configuration file

## Binaries

- `gateway` (default) - Main gateway server
- `google-gateway` - Specialized Google API gateway

## Features

The codebase uses Cargo features extensively:
- **Storage**: `postgres`, `sqlite`, `redis`, `s3`
- **Monitoring**: `metrics`, `tracing` 
- **Advanced**: `vector-db`, `websockets`, `analytics`, `enterprise`
- **Development**: Use `--all-features` flag for full functionality

## Database & Storage

- **Primary DB**: PostgreSQL with Sea-ORM migrations
- **Cache**: Redis for high-speed operations
- **File storage**: S3-compatible object storage
- **Vector DB**: Optional Qdrant integration for semantic caching

## Testing Architecture

- Unit tests in each module (`#[cfg(test)]`)
- Test files use inline tests within source files
- Postman collections for API testing (`tests/*.postman_collection.json`)
- Mock implementations for external services

## Common Development Patterns

1. **Adding new providers**: Implement the `Provider` trait in `src/core/providers/`
2. **New API endpoints**: Add routes in `src/server/routes/`
3. **Authentication**: Extend auth modules in `src/auth/`
4. **Configuration**: Update models in `src/config/models/`
5. **Monitoring**: Add metrics in respective modules
6. **MCP servers**: Add server configs in `src/core/mcp/config.rs`
7. **A2A agents**: Add agent configs in `src/core/a2a/config.rs`

## Protocol Gateways

### MCP Gateway (`src/core/mcp/`)
Model Context Protocol for connecting LLMs to external tools:
- `config.rs` - Server configuration, authentication (Bearer, API Key, OAuth 2.0)
- `transport.rs` - HTTP, SSE, WebSocket, stdio transports
- `protocol.rs` - JSON-RPC 2.0 implementation
- `tools.rs` - Tool definitions and invocation
- `server.rs` - Individual server connection management
- `gateway.rs` - Main gateway aggregating servers
- `permissions.rs` - Fine-grained access control

### A2A Protocol (`src/core/a2a/`)
Agent-to-Agent communication with multi-provider support:
- `config.rs` - Agent configuration, provider types
- `message.rs` - JSON-RPC 2.0 message format, task states
- `provider.rs` - Provider adapters (LangGraph, Vertex AI, Azure, Bedrock, Pydantic AI)
- `registry.rs` - Agent discovery and health monitoring
- `gateway.rs` - Main gateway for agent management

## Docker & Deployment

- **Docker build**: `make docker`
- **Development stack**: `make docker-compose-dev`
- **Production**: `make docker-compose`
- **Kubernetes**: `make k8s-apply`

## Performance Characteristics

- **Throughput**: 10,000+ requests/second
- **Latency**: <10ms routing overhead
- **Memory**: ~50MB base footprint
- **Architecture**: Fully async, connection pooling, zero-copy where possible

## Python LiteLLM Compatibility

This Rust implementation maintains API compatibility with the original Python LiteLLM:
- Core completion API exposed via `src/core/completion.rs`
- Helper functions: `completion()`, `user_message()`, `system_message()`, `assistant_message()`
- Unified interface for 100+ providers with automatic routing

## Known Issues & Solutions

### docs.rs Build Issue
The `vector-db` feature (which includes `qdrant-client`) fails to build on docs.rs due to its read-only filesystem. The qdrant-client build script attempts to write files during compilation.

**Solution**:
- In `Cargo.toml`, the `[package.metadata.docs.rs]` section explicitly:
  - Sets `all-features = false` to prevent docs.rs from using `--all-features`
  - Lists specific features excluding `vector-db`
  - This allows documentation to build successfully on docs.rs

**Testing docs.rs compatibility locally**:
```bash
env DOCS_RS=1 cargo doc --no-deps --features "postgres sqlite redis s3 metrics tracing websockets analytics"
```