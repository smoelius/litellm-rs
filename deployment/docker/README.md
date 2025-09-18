# LiteLLM-RS Docker Deployment

Docker files for building and running LiteLLM-RS.

## 🚀 Quick Start

```bash
# Build the image
./deployment/docker/build.sh

# Run the container
docker run -p 8000:8000 litellm-rs:latest
```

## 🛠️ Build Options

```bash
# Custom tag
./deployment/docker/build.sh -t v1.0.0

# Custom image name
./deployment/docker/build.sh -n my-litellm

# Using environment variables
IMAGE_TAG=dev ./deployment/docker/build.sh
```

## 🔧 Configuration

### Environment Variables

```bash
docker run -p 8000:8000 \
  -e OPENAI_API_KEY=sk-... \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  -e DATABASE_URL=postgresql://... \
  -e REDIS_URL=redis://... \
  litellm-rs:latest
```

### Volume Mounts

```bash
# Mount configuration
docker run -p 8000:8000 \
  -v $(pwd)/config:/app/config \
  litellm-rs:latest

# Mount data directory
docker run -p 8000:8000 \
  -v litellm_data:/app/data \
  litellm-rs:latest
```

## 📁 Files

- `Dockerfile` - Multi-stage build definition
- `.dockerignore` - Build context exclusions
- `build.sh` - Automated build script
- `README.md` - This documentation

## 🐳 Docker Compose Example

```yaml
version: '3.8'
services:
  litellm:
    image: litellm-rs:latest
    ports:
      - "8000:8000"
      - "9090:9090"
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - DATABASE_URL=postgresql://user:pass@postgres:5432/litellm
    volumes:
      - ./config:/app/config
    depends_on:
      - postgres

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=litellm
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
```

## 🔍 Health Check

```bash
# Check container health
curl http://localhost:8000/health

# View metrics
curl http://localhost:9090/metrics
```