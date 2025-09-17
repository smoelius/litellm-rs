# Rust LiteLLM Gateway Makefile
# Provides convenient commands for development and deployment

.PHONY: help build test clean dev prod docker docs lint format check install deps start

# Default target
help: ## Show this help message
	@echo "Rust LiteLLM Gateway - Available Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Examples:"
	@echo "  make dev          # Start development environment"
	@echo "  make test         # Run all tests"
	@echo "  make docker       # Build Docker image"
	@echo "  make prod         # Build production release"

# =============================================================================
# DEVELOPMENT
# =============================================================================

start: ## Quick start (auto-loads config/gateway.yaml)
	@echo "🚀 Starting Rust LiteLLM Gateway..."
	cargo run

dev: deps dev-services ## Start development environment
	@echo "Starting development server..."
	RUST_LOG=debug cargo run --bin gateway -- --config config/dev.yaml

dev-services: ## Start development services (PostgreSQL, Redis, etc.)
	@echo "Starting development services..."
	docker-compose -f docker-compose.dev.yml up -d postgres-dev redis-dev
	@echo "Waiting for services to be ready..."
	@sleep 5

dev-stop: ## Stop development services
	@echo "Stopping development services..."
	docker-compose -f docker-compose.dev.yml down

dev-logs: ## Show development services logs
	docker-compose -f docker-compose.dev.yml logs -f

# =============================================================================
# BUILDING
# =============================================================================

build: ## Build debug version
	cargo build --all-features

build-release: ## Build optimized release version
	cargo build --release --all-features

prod: build-release ## Build production release (alias for build-release)

install: build-release ## Install binaries to system
	cargo install --path . --force

# =============================================================================
# TESTING
# =============================================================================

test: ## Run all tests
	cargo test --all-features

test-unit: ## Run unit tests only
	cargo test --lib --all-features

test-integration: ## Run integration tests only
	cargo test --test integration_tests --all-features

test-coverage: ## Generate test coverage report
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo "Coverage report generated: lcov.info"

bench: ## Run benchmarks
	cargo bench --all-features

# =============================================================================
# CODE QUALITY
# =============================================================================

lint: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

format: ## Format code with rustfmt
	cargo fmt --all

format-check: ## Check code formatting
	cargo fmt --all -- --check

check: ## Run cargo check
	cargo check --all-features

audit: ## Run security audit
	cargo audit

fix: ## Auto-fix code issues
	cargo clippy --all-targets --all-features --fix --allow-dirty
	cargo fmt --all

# =============================================================================
# DOCUMENTATION
# =============================================================================

docs: ## Generate and open documentation
	cargo doc --all-features --open

docs-build: ## Build documentation without opening
	cargo doc --all-features --no-deps

# =============================================================================
# DOCKER
# =============================================================================

docker: ## Build Docker image
	docker build -t litellm-rs:latest .

docker-dev: ## Build development Docker image
	docker build -t litellm-rs:dev --target builder .

docker-run: ## Run Docker container
	docker run -p 8000:8000 -v ./config:/app/config litellm-rs:latest

docker-compose: ## Start full stack with docker-compose
	docker-compose up -d

docker-compose-dev: ## Start development stack
	docker-compose -f docker-compose.dev.yml up -d

docker-compose-down: ## Stop docker-compose stack
	docker-compose down

docker-clean: ## Clean Docker images and containers
	docker system prune -f
	docker image prune -f

# =============================================================================
# DATABASE
# =============================================================================

db-migrate: ## Run database migrations
	cargo run --bin gateway -- --config config/dev.yaml --migrate

db-reset: ## Reset development database
	docker-compose -f docker-compose.dev.yml down postgres-dev
	docker volume rm litellm_postgres_dev_data || true
	docker-compose -f docker-compose.dev.yml up -d postgres-dev
	@sleep 5
	$(MAKE) db-migrate

db-backup: ## Backup development database
	@mkdir -p backups
	docker exec litellm-postgres-dev pg_dump -U gateway_dev gateway_dev > backups/dev_backup_$(shell date +%Y%m%d_%H%M%S).sql

# =============================================================================
# DEPENDENCIES
# =============================================================================

deps: ## Install development dependencies
	@echo "Installing Rust toolchain components..."
	rustup component add rustfmt clippy llvm-tools-preview
	@echo "Installing cargo tools..."
	cargo install cargo-audit cargo-llvm-cov || true
	@echo "Dependencies installed!"

deps-update: ## Update dependencies
	cargo update

deps-outdated: ## Check for outdated dependencies
	cargo outdated || echo "Install cargo-outdated: cargo install cargo-outdated"

# =============================================================================
# DEPLOYMENT
# =============================================================================

deploy-staging: ## Deploy to staging environment
	@echo "Deploying to staging..."
	# Add your staging deployment commands here

deploy-prod: ## Deploy to production environment
	@echo "Deploying to production..."
	# Add your production deployment commands here

k8s-apply: ## Apply Kubernetes manifests
	kubectl apply -f deploy/kubernetes/

k8s-delete: ## Delete Kubernetes resources
	kubectl delete -f deploy/kubernetes/

# =============================================================================
# UTILITIES
# =============================================================================

clean: ## Clean build artifacts
	cargo clean
	docker system prune -f

clean-all: clean ## Clean everything including Docker volumes
	docker-compose down -v
	docker-compose -f docker-compose.dev.yml down -v

version: ## Show version information
	@echo "Rust version: $(shell rustc --version)"
	@echo "Cargo version: $(shell cargo --version)"
	@echo "Gateway version: $(shell cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "litellm-rs") | .version')"

health: ## Check gateway health
	@curl -s http://localhost:8000/health | jq . || echo "Gateway not running or jq not installed"

logs: ## Show gateway logs (if running in Docker)
	docker-compose logs -f gateway

# =============================================================================
# RELEASE
# =============================================================================

release-check: format-check lint test ## Run all checks before release
	@echo "All checks passed! Ready for release."

release-build: ## Build release artifacts for all platforms
	@echo "Building release artifacts..."
	cargo build --release --target x86_64-unknown-linux-gnu --all-features
	cargo build --release --target x86_64-unknown-linux-musl --all-features
	cargo build --release --target x86_64-pc-windows-msvc --all-features
	cargo build --release --target x86_64-apple-darwin --all-features

# =============================================================================
# MONITORING
# =============================================================================

metrics: ## Show Prometheus metrics
	@curl -s http://localhost:9090/metrics | head -20

grafana: ## Open Grafana dashboard
	@open http://localhost:3000 || echo "Grafana not running or 'open' command not available"

jaeger: ## Open Jaeger UI
	@open http://localhost:16686 || echo "Jaeger not running or 'open' command not available"

# =============================================================================
# CONFIGURATION
# =============================================================================

config-validate: ## Validate configuration
	cargo run --bin gateway -- --config config/dev.yaml --validate

config-example: ## Copy example configurations
	cp config/gateway.yaml.example config/gateway.yaml
	cp .env.example .env
	@echo "Example configurations copied. Please edit them with your values."

# =============================================================================
# VARIABLES
# =============================================================================

# Detect OS for platform-specific commands
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    OPEN_CMD = xdg-open
endif
ifeq ($(UNAME_S),Darwin)
    OPEN_CMD = open
endif

# Default values
RUST_LOG ?= info
DATABASE_URL ?= postgresql://gateway_dev:dev_password@localhost:5433/gateway_dev
REDIS_URL ?= redis://localhost:6380
