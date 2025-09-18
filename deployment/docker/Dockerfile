# Multi-stage build for LiteLLM-RS
FROM rust:1.87-slim AS builder

# Install system dependencies including cross-compilation tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    git \
    cmake \
    build-essential \
    clang \
    llvm \
    musl-tools \
    gcc-arm-linux-gnueabihf \
    libc6-dev-armhf-cross \
    && rm -rf /var/lib/apt/lists/*

# Set environment variables for cross-compilation and AWS LC
ENV CMAKE=cmake
ENV CC=clang
ENV CXX=clang++
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

# Add Rust targets for cross-compilation
RUN rustup target add armv7-unknown-linux-gnueabihf

# Create app user
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy dependency files and build configuration
COPY Cargo.toml Cargo.lock build.rs ./

# Create dummy source files to cache dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "#![allow(dead_code)] pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > src/bin/google_gateway.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --features "postgres sqlite redis s3 metrics tracing websockets analytics"
RUN rm -rf src/*

# Copy source code and necessary files
COPY src ./src
COPY config ./config
COPY .git ./.git

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
