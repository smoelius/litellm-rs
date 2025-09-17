#!/bin/bash

# Rust LiteLLM Gateway Setup Script
# This script sets up the development environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check system requirements
check_requirements() {
    log_info "Checking system requirements..."
    
    # Check Rust
    if ! command_exists rustc; then
        log_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    local rust_version=$(rustc --version | cut -d' ' -f2)
    log_success "Rust $rust_version found"
    
    # Check Cargo
    if ! command_exists cargo; then
        log_error "Cargo is not installed"
        exit 1
    fi
    
    # Check Docker
    if ! command_exists docker; then
        log_warning "Docker is not installed. Some features may not work."
    else
        log_success "Docker found"
    fi
    
    # Check Docker Compose
    if ! command_exists docker-compose; then
        log_warning "Docker Compose is not installed. Development services may not work."
    else
        log_success "Docker Compose found"
    fi
    
    # Check PostgreSQL client (optional)
    if command_exists psql; then
        log_success "PostgreSQL client found"
    else
        log_warning "PostgreSQL client not found. Database operations may be limited."
    fi
    
    # Check Redis client (optional)
    if command_exists redis-cli; then
        log_success "Redis client found"
    else
        log_warning "Redis client not found. Cache operations may be limited."
    fi
}

# Install Rust components
install_rust_components() {
    log_info "Installing Rust components..."
    
    rustup component add rustfmt clippy llvm-tools-preview
    log_success "Rust components installed"
}

# Install cargo tools
install_cargo_tools() {
    log_info "Installing cargo tools..."
    
    # List of useful cargo tools
    local tools=(
        "cargo-audit"
        "cargo-llvm-cov"
        "cargo-outdated"
        "cargo-edit"
        "cargo-watch"
    )
    
    for tool in "${tools[@]}"; do
        if ! command_exists "$tool"; then
            log_info "Installing $tool..."
            cargo install "$tool" || log_warning "Failed to install $tool"
        else
            log_success "$tool already installed"
        fi
    done
}

# Setup configuration files
setup_config() {
    log_info "Setting up configuration files..."
    
    # Copy example configurations if they don't exist
    if [ ! -f ".env" ]; then
        cp .env.example .env
        log_success "Created .env from example"
        log_warning "Please edit .env with your actual values"
    else
        log_info ".env already exists"
    fi
    
    if [ ! -f "config/gateway.yaml" ]; then
        cp config/gateway.yaml.example config/gateway.yaml
        log_success "Created config/gateway.yaml from example"
        log_warning "Please edit config/gateway.yaml with your settings"
    else
        log_info "config/gateway.yaml already exists"
    fi
    
    # Copy VSCode settings if they don't exist
    if [ ! -f ".vscode/settings.json" ]; then
        mkdir -p .vscode
        cp .vscode/settings.json.example .vscode/settings.json
        log_success "Created VSCode settings"
    fi
    
    if [ ! -f ".vscode/launch.json" ]; then
        cp .vscode/launch.json.example .vscode/launch.json
        log_success "Created VSCode launch configuration"
    fi
    
    if [ ! -f ".vscode/tasks.json" ]; then
        cp .vscode/tasks.json.example .vscode/tasks.json
        log_success "Created VSCode tasks"
    fi
}

# Build the project
build_project() {
    log_info "Building the project..."
    
    cargo build --all-features
    log_success "Project built successfully"
}

# Setup development services
setup_dev_services() {
    log_info "Setting up development services..."
    
    if command_exists docker-compose; then
        # Start development services
        docker-compose -f docker-compose.dev.yml up -d postgres-dev redis-dev
        
        # Wait for services to be ready
        log_info "Waiting for services to be ready..."
        sleep 10
        
        # Check if services are running
        if docker-compose -f docker-compose.dev.yml ps | grep -q "Up"; then
            log_success "Development services started"
        else
            log_warning "Some development services may not be running properly"
        fi
    else
        log_warning "Docker Compose not available. Skipping development services setup."
    fi
}

# Run tests
run_tests() {
    log_info "Running tests..."
    
    if cargo test --all-features; then
        log_success "All tests passed"
    else
        log_warning "Some tests failed. This is normal for a fresh setup without proper configuration."
    fi
}

# Main setup function
main() {
    echo "🚀 Rust LiteLLM Gateway Setup"
    echo "=============================="
    echo ""
    
    check_requirements
    echo ""
    
    install_rust_components
    echo ""
    
    install_cargo_tools
    echo ""
    
    setup_config
    echo ""
    
    build_project
    echo ""
    
    setup_dev_services
    echo ""
    
    run_tests
    echo ""
    
    log_success "Setup completed!"
    echo ""
    echo "Next steps:"
    echo "1. Edit .env with your API keys and configuration"
    echo "2. Edit config/gateway.yaml with your settings"
    echo "3. Run 'make dev' to start the development server"
    echo "4. Visit http://localhost:8000/health to verify it's working"
    echo ""
    echo "For more information, see docs/quickstart.md"
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --no-services  Skip setting up development services"
        echo "  --no-tests     Skip running tests"
        echo ""
        exit 0
        ;;
    --no-services)
        SKIP_SERVICES=1
        ;;
    --no-tests)
        SKIP_TESTS=1
        ;;
esac

# Run main function
main
