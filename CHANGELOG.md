# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2024-12-18

### Fixed
- **docs.rs Build**: Fixed documentation build failure on docs.rs by excluding `vector-db` feature
  - Added `all-features = false` to `package.metadata.docs.rs` configuration
  - Explicitly listed features that work with docs.rs read-only filesystem
- **Internationalization**: Translated all Chinese comments and documentation to English
  - Cleaned 40+ files with hundreds of Chinese comments
  - Improved accessibility for international developers
  - Maintained technical accuracy in all translations

### Changed
- **Configuration**: Updated `Cargo.toml` metadata for better docs.rs compatibility
- **Documentation**: All code comments are now in English

## [0.1.1] - 2024-12-16

### Fixed
- **Security**: Excluded sensitive configuration file `config/gateway.yaml` from published package
- **Package**: Only include example configuration files (`.example`, `.template`) in published crate
- **Privacy**: Prevent accidental exposure of API keys and secrets in published package

## [0.1.0] - 2024-12-15

### Added
- Initial release of Rust LiteLLM Gateway
- High-performance AI Gateway with OpenAI-compatible APIs
- Intelligent routing and load balancing capabilities
- Support for multiple AI providers (OpenAI, Anthropic, Google, etc.)
- Enterprise features including authentication and monitoring
- Actix-web based web server with async/await support
- PostgreSQL and Redis integration for data persistence and caching
- Comprehensive configuration management via YAML
- Rate limiting and request throttling
- WebSocket support for real-time communication
- Prometheus metrics integration
- OpenTelemetry tracing support
- Vector database integration (Qdrant)
- S3-compatible object storage support
- JWT-based authentication system
- Docker and Kubernetes deployment configurations
- Comprehensive API documentation
- Integration tests and examples

### Features
- **Core Gateway**: OpenAI-compatible API endpoints
- **Multi-Provider Support**: Seamless integration with various AI providers
- **Load Balancing**: Intelligent request distribution
- **Caching**: Redis-based response caching
- **Monitoring**: Prometheus metrics and OpenTelemetry tracing
- **Authentication**: JWT-based security
- **Rate Limiting**: Configurable request throttling
- **WebSocket**: Real-time streaming support
- **Storage**: PostgreSQL for persistence, S3 for object storage
- **Vector DB**: Qdrant integration for embeddings
- **Deployment**: Docker, Kubernetes, and systemd configurations

[Unreleased]: https://github.com/majiayu000/litellm-rs/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/majiayu000/litellm-rs/compare/v0.1.1...v0.1.3
[0.1.1]: https://github.com/majiayu000/litellm-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/majiayu000/litellm-rs/releases/tag/v0.1.0
