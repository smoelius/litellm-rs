#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-litellm-rs}"
IMAGE_TAG="${IMAGE_TAG:-latest}"

# Functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

cleanup() {
    if [ -f "$PROJECT_ROOT/.dockerignore" ]; then
        log_info "Cleaning up temporary .dockerignore"
        rm "$PROJECT_ROOT/.dockerignore"
    fi
}

# Trap cleanup on exit
trap cleanup EXIT

main() {
    log_info "Building LiteLLM-RS Docker image..."
    log_info "Project root: $PROJECT_ROOT"
    log_info "Image: $IMAGE_NAME:$IMAGE_TAG"

    # Change to project root
    cd "$PROJECT_ROOT"

    # Copy dockerignore to project root
    log_info "Setting up .dockerignore"
    cp "$SCRIPT_DIR/.dockerignore" .dockerignore

    # Build Docker image
    log_info "Starting Docker build..."
    if docker build \
        -f "$SCRIPT_DIR/Dockerfile" \
        -t "$IMAGE_NAME:$IMAGE_TAG" \
        --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --build-arg VCS_REF="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')" \
        .; then
        log_success "Docker build completed successfully!"
        log_success "Image: $IMAGE_NAME:$IMAGE_TAG"
    else
        log_error "Docker build failed!"
        exit 1
    fi

    # Show image info
    log_info "Image details:"
    docker images "$IMAGE_NAME:$IMAGE_TAG" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
}

# Help function
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Build LiteLLM-RS Docker image"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -t, --tag      Set image tag (default: latest)"
    echo "  -n, --name     Set image name (default: litellm-rs)"
    echo ""
    echo "Environment variables:"
    echo "  IMAGE_NAME     Docker image name (default: litellm-rs)"
    echo "  IMAGE_TAG      Docker image tag (default: latest)"
    echo ""
    echo "Examples:"
    echo "  $0                          # Build litellm-rs:latest"
    echo "  $0 -t v1.0.0               # Build litellm-rs:v1.0.0"
    echo "  IMAGE_NAME=myapp $0        # Build myapp:latest"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -t|--tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        -n|--name)
            IMAGE_NAME="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    log_error "Docker is not installed or not in PATH"
    exit 1
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    log_error "Docker daemon is not running"
    exit 1
fi

# Run main function
main