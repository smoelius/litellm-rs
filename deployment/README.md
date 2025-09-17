# 🚀 Deployment Guide

This directory contains all deployment-related files for the Rust LiteLLM Gateway.

## 📁 Directory Structure

```
deployment/
├── 📁 docker/              # Docker deployment
│   ├── Dockerfile          # Main Docker image
│   ├── docker-compose.yml  # Production compose
│   └── docker-compose.dev.yml # Development compose
├── 📁 kubernetes/          # Kubernetes manifests
│   └── (K8s YAML files)
├── 📁 systemd/             # System service files
│   └── rust-litellm-gateway.service
├── 📁 scripts/             # Deployment scripts
│   ├── start.sh            # Quick start script
│   ├── setup.sh            # Environment setup
│   ├── docker-start.sh     # Docker startup
│   └── init-*.sql          # Database initialization
└── 📁 configs/             # Deployment configurations
    └── monitoring/         # Monitoring configs
```

## 🚀 Quick Deployment Options

### 1. Local Development
```bash
# Quick start
./deployment/scripts/start.sh

# Or manually
cargo run
```

### 2. Docker
```bash
# Build and run
cd deployment/docker
docker-compose up -d
```

### 3. Production (systemd)
```bash
# Install service
sudo cp deployment/systemd/rust-litellm-gateway.service /etc/systemd/system/
sudo systemctl enable rust-litellm-gateway
sudo systemctl start rust-litellm-gateway
```

### 4. Kubernetes
```bash
# Deploy to K8s
kubectl apply -f deployment/kubernetes/
```

## 📋 Prerequisites

- **Rust 1.85+** for local builds
- **Docker & Docker Compose** for containerized deployment
- **PostgreSQL & Redis** for data storage
- **Kubernetes** for cluster deployment

## 🔧 Configuration

1. **Edit main config**: `config/gateway.yaml`
2. **Set environment variables** as needed
3. **Choose deployment method** from above

## 📚 Detailed Guides

- [Docker Deployment](docker/README.md)
- [Kubernetes Deployment](kubernetes/README.md)
- [Production Setup](scripts/README.md)

## 🆘 Troubleshooting

- Check logs: `journalctl -u rust-litellm-gateway -f`
- Verify config: `./deployment/scripts/start.sh`
- Test API: `curl http://localhost:8000/health`
