#!/bin/bash

# Docker Startup Script for Rust LiteLLM Gateway
# This script is designed to run inside a Docker container

set -euo pipefail

# Default values
CONFIG_FILE="${CONFIG_FILE:-/app/config/gateway.yaml}"
LOG_LEVEL="${LOG_LEVEL:-info}"
ENVIRONMENT="${ENVIRONMENT:-production}"
HOST="${HOST:-0.0.0.0}"
PORT="${PORT:-8000}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to wait for dependencies
wait_for_dependencies() {
    print_info "Waiting for dependencies..."
    
    # Wait for database if DATABASE_URL is set
    if [ -n "${DATABASE_URL:-}" ]; then
        print_info "Waiting for database connection..."
        
        # Extract host and port from DATABASE_URL
        # This is a simplified parser for PostgreSQL URLs
        if [[ $DATABASE_URL =~ postgres://[^@]+@([^:/]+):?([0-9]+)?/ ]]; then
            local db_host="${BASH_REMATCH[1]}"
            local db_port="${BASH_REMATCH[2]:-5432}"
            
            print_info "Checking database connectivity: $db_host:$db_port"
            
            local max_attempts=30
            local attempt=1
            
            while [ $attempt -le $max_attempts ]; do
                if timeout 5 bash -c "</dev/tcp/$db_host/$db_port" 2>/dev/null; then
                    print_success "Database is available"
                    break
                fi
                
                print_info "Database not ready, attempt $attempt/$max_attempts..."
                sleep 2
                ((attempt++))
            done
            
            if [ $attempt -gt $max_attempts ]; then
                print_error "Database is not available after $max_attempts attempts"
                exit 1
            fi
        fi
    fi
    
    # Wait for Redis if REDIS_URL is set
    if [ -n "${REDIS_URL:-}" ]; then
        print_info "Waiting for Redis connection..."
        
        # Extract host and port from REDIS_URL
        if [[ $REDIS_URL =~ redis://[^@]*@?([^:/]+):?([0-9]+)?/?.*$ ]]; then
            local redis_host="${BASH_REMATCH[1]}"
            local redis_port="${BASH_REMATCH[2]:-6379}"
            
            print_info "Checking Redis connectivity: $redis_host:$redis_port"
            
            local max_attempts=30
            local attempt=1
            
            while [ $attempt -le $max_attempts ]; do
                if timeout 5 bash -c "</dev/tcp/$redis_host/$redis_port" 2>/dev/null; then
                    print_success "Redis is available"
                    break
                fi
                
                print_info "Redis not ready, attempt $attempt/$max_attempts..."
                sleep 2
                ((attempt++))
            done
            
            if [ $attempt -gt $max_attempts ]; then
                print_error "Redis is not available after $max_attempts attempts"
                exit 1
            fi
        fi
    fi
}

# Function to run database migrations
run_migrations() {
    if [ "${RUN_MIGRATIONS:-true}" = "true" ]; then
        print_info "Running database migrations..."
        
        if /app/rust-litellm-gateway database migrate --config "$CONFIG_FILE"; then
            print_success "Database migrations completed"
        else
            print_error "Database migrations failed"
            exit 1
        fi
    else
        print_info "Skipping database migrations (RUN_MIGRATIONS=false)"
    fi
}

# Function to validate configuration
validate_config() {
    print_info "Validating configuration..."
    
    if [ ! -f "$CONFIG_FILE" ]; then
        print_error "Configuration file not found: $CONFIG_FILE"
        exit 1
    fi
    
    if /app/rust-litellm-gateway validate-config --config "$CONFIG_FILE"; then
        print_success "Configuration is valid"
    else
        print_error "Configuration validation failed"
        exit 1
    fi
}

# Function to start the gateway
start_gateway() {
    print_info "Starting LiteLLM-RS Gateway..."
    print_info "Configuration: $CONFIG_FILE"
    print_info "Environment: $ENVIRONMENT"
    print_info "Log Level: $LOG_LEVEL"
    print_info "Host: $HOST"
    print_info "Port: $PORT"

    # Set environment variables
    export RUST_LOG="$LOG_LEVEL"
    export ENVIRONMENT="$ENVIRONMENT"

    # Start the gateway
    exec /app/gateway \
        --config "$CONFIG_FILE" \
        --host "$HOST" \
        --port "$PORT" \
        --log-level "$LOG_LEVEL"
}

# Function to handle shutdown signals
shutdown_handler() {
    print_info "Received shutdown signal, stopping gateway..."
    # The gateway should handle SIGTERM gracefully
    exit 0
}

# Function to show container information
show_info() {
    print_info "Rust LiteLLM Gateway Docker Container"
    print_info "====================================="
    print_info "Container ID: $(hostname)"
    print_info "User: $(whoami)"
    print_info "Working Directory: $(pwd)"
    print_info "Configuration File: $CONFIG_FILE"
    print_info "Environment: $ENVIRONMENT"
    print_info "Log Level: $LOG_LEVEL"
    print_info "Host: $HOST"
    print_info "Port: $PORT"
    
    if [ -n "${DATABASE_URL:-}" ]; then
        print_info "Database URL: [CONFIGURED]"
    else
        print_warning "Database URL: [NOT SET]"
    fi
    
    if [ -n "${REDIS_URL:-}" ]; then
        print_info "Redis URL: [CONFIGURED]"
    else
        print_warning "Redis URL: [NOT SET]"
    fi
    
    if [ -n "${JWT_SECRET:-}" ]; then
        print_info "JWT Secret: [CONFIGURED]"
    else
        print_warning "JWT Secret: [NOT SET]"
    fi
    
    print_info "====================================="
}

# Main execution function
main() {
    # Set up signal handlers
    trap shutdown_handler SIGTERM SIGINT
    
    # Show container information
    show_info
    
    # Wait for dependencies
    wait_for_dependencies
    
    # Validate configuration
    validate_config
    
    # Run migrations if enabled
    run_migrations
    
    # Start the gateway
    start_gateway
}

# Health check function (for Docker HEALTHCHECK)
health_check() {
    local health_url="http://${HOST}:${PORT}/health"
    
    if command -v curl >/dev/null 2>&1; then
        curl -f "$health_url" >/dev/null 2>&1
    elif command -v wget >/dev/null 2>&1; then
        wget -q --spider "$health_url" >/dev/null 2>&1
    else
        # Fallback to basic TCP check
        timeout 5 bash -c "</dev/tcp/$HOST/$PORT" 2>/dev/null
    fi
}

# Handle special commands
case "${1:-start}" in
    "start")
        main
        ;;
    "health")
        health_check
        exit $?
        ;;
    "validate")
        validate_config
        exit $?
        ;;
    "migrate")
        run_migrations
        exit $?
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Available commands: start, health, validate, migrate"
        exit 1
        ;;
esac
