# Multi-stage build for LiteLLM-RS
FROM rust:1.87-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --features "postgres sqlite redis s3 metrics tracing websockets analytics"
RUN rm src/main.rs

# Copy source code
COPY src ./src
COPY config ./config

# Build the application
RUN cargo build --release --features "postgres sqlite redis s3 metrics tracing websockets analytics"

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/gateway /usr/local/bin/gateway
COPY --from=builder /app/target/release/google-gateway /usr/local/bin/google-gateway

# Copy configuration files
COPY --from=builder /app/config ./config

# Create data directory
RUN mkdir -p /app/data && chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose ports
EXPOSE 8000 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Default command
CMD ["gateway", "--config", "config/gateway.yaml"]

# Labels
LABEL org.opencontainers.image.title="LiteLLM-RS"
LABEL org.opencontainers.image.description="A high-performance AI Gateway written in Rust"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.authors="LiteLLM-RS Contributors"
LABEL org.opencontainers.image.url="https://github.com/majiayu000/litellm-rs"
LABEL org.opencontainers.image.source="https://github.com/majiayu000/litellm-rs"
LABEL org.opencontainers.image.licenses="MIT"
