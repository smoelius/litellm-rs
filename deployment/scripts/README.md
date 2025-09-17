# 📜 Deployment Scripts

## Available Scripts

### 🚀 `start.sh`
Quick start script for local development.

```bash
./deployment/scripts/start.sh
```

**What it does:**
- Checks for `config/gateway.yaml`
- Compiles and runs the gateway
- Shows helpful error messages

### 🔧 `setup.sh`
Environment setup script.

```bash
./deployment/scripts/setup.sh
```

**What it does:**
- Installs system dependencies
- Sets up database
- Creates configuration files

### 🐳 `docker-start.sh`
Docker container startup script.

```bash
./deployment/scripts/docker-start.sh
```

**What it does:**
- Runs inside Docker container
- Handles environment setup
- Starts the gateway service

### 🗄️ Database Scripts

#### `init-db.sql`
Production database initialization.

```bash
psql -f deployment/scripts/init-db.sql
```

#### `init-dev-db.sql`
Development database setup.

```bash
psql -f deployment/scripts/init-dev-db.sql
```

### 🧪 `test_api.sh`
API testing script.

```bash
./deployment/scripts/test_api.sh
```

**What it does:**
- Tests all API endpoints
- Validates responses
- Reports results

## 🔧 Script Configuration

### Environment Variables
```bash
# Set these before running scripts
export GATEWAY_HOST=0.0.0.0
export GATEWAY_PORT=8000
export DATABASE_URL=postgresql://...
```

### Custom Paths
```bash
# Override default paths
export CONFIG_PATH=custom/config.yaml
export LOG_PATH=custom/logs/
```

## 📋 Usage Examples

### Development Workflow
```bash
# 1. Setup environment
./deployment/scripts/setup.sh

# 2. Start gateway
./deployment/scripts/start.sh

# 3. Test API
./deployment/scripts/test_api.sh
```

### Production Deployment
```bash
# 1. Setup production environment
sudo ./deployment/scripts/setup.sh --production

# 2. Install systemd service
sudo cp ../systemd/litellm-rs.service /etc/systemd/system/
sudo systemctl enable litellm-rs
sudo systemctl start litellm-rs
```
