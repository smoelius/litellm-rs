# 🐳 Docker Deployment

## Quick Start

### 1. Production Deployment
```bash
cd deployment/docker
docker-compose up -d
```

### 2. Development Environment
```bash
cd deployment/docker
docker-compose -f docker-compose.dev.yml up -d
```

## 📁 Files

- **`Dockerfile`** - Multi-stage build for production
- **`docker-compose.yml`** - Production stack (Gateway + PostgreSQL + Redis)
- **`docker-compose.dev.yml`** - Development stack with debug tools

## 🔧 Configuration

### Environment Variables
```bash
# Copy and edit
cp ../../config/gateway.yaml.example ../../config/gateway.yaml
# Edit your API keys in config/gateway.yaml
```

### Custom Configuration
```bash
# Mount custom config
docker run -v ./config:/app/config rust-litellm-gateway
```

## 🚀 Build & Run

### Build Image
```bash
docker build -t rust-litellm-gateway -f Dockerfile ../..
```

### Run Container
```bash
docker run -p 8000:8000 \
  -v ./config:/app/config \
  rust-litellm-gateway
```

## 📊 Monitoring

Access services:
- **Gateway**: http://localhost:8000
- **Health Check**: http://localhost:8000/health
- **Metrics**: http://localhost:9090/metrics (if enabled)

## 🔍 Troubleshooting

### View Logs
```bash
docker-compose logs -f gateway
```

### Debug Container
```bash
docker exec -it gateway_container bash
```

### Reset Environment
```bash
docker-compose down -v
docker-compose up -d
```
