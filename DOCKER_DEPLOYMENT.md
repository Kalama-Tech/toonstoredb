# ToonStore Docker Guide

Complete guide for building, publishing, and using ToonStore Docker images.

---

## 🚀 Quick Start (Pull and Run)

### Pull from Docker Hub

```bash
# Pull latest version
docker pull toonstore/tstd:latest

# Or specific version
docker pull toonstore/tstd:0.1.0
```

### Run ToonStore

```bash
# Simple run
docker run -d \
  --name toonstore \
  -p 6379:6379 \
  -v toonstore_data:/data \
  toonstore/tstd:latest

# With environment variables
docker run -d \
  --name toonstore \
  -p 6379:6379 \
  -v toonstore_data:/data \
  -e RUST_LOG=info \
  toonstore/tstd:latest

# Custom configuration
docker run -d \
  --name toonstore \
  -p 6379:6379 \
  -v toonstore_data:/data \
  toonstore/tstd:latest \
  --bind 0.0.0.0:6379 \
  --data /data \
  --capacity 50000
```

### Test Connection

```bash
# Using redis-cli
docker exec -it toonstore tstd --health

# Or connect from host
redis-cli -h localhost -p 6379 PING
```

---

## 📦 Using Docker Compose

### Simple Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  toonstore:
    image: toonstore/tstd:latest
    container_name: toonstore
    ports:
      - "6379:6379"
    volumes:
      - toonstore_data:/data
    restart: unless-stopped

volumes:
  toonstore_data:
```

```bash
# Start
docker-compose up -d

# View logs
docker-compose logs -f toonstore

# Stop
docker-compose down
```

### With Redis CLI for Testing

```yaml
version: '3.8'

services:
  toonstore:
    image: toonstore/tstd:latest
    container_name: toonstore
    ports:
      - "6379:6379"
    volumes:
      - toonstore_data:/data
    restart: unless-stopped

  redis-cli:
    image: redis:alpine
    container_name: toonstore-cli
    command: sh -c "sleep infinity"
    depends_on:
      - toonstore
    network_mode: "service:toonstore"

volumes:
  toonstore_data:
```

```bash
# Start services
docker-compose up -d

# Connect with redis-cli
docker-compose exec redis-cli redis-cli -h localhost
```

---

## 🏗️ Building Your Own Image

### Prerequisites

- Docker installed
- Docker Hub account (for pushing)
- Rust toolchain (optional, Docker handles it)

### Build Locally

```bash
# Clone repository
git clone https://github.com/yourusername/toonstoredb.git
cd toonstoredb

# Build image
docker build -t toonstore/tstd:latest .

# Run locally built image
docker run -p 6379:6379 toonstore/tstd:latest
```

### Using Build Scripts

#### Linux/Mac

```bash
# Make script executable
chmod +x build-docker.sh

# Build and optionally push
./build-docker.sh
```

#### Windows (PowerShell)

```powershell
# Run build script
.\build-docker.ps1
```

The scripts will:
1. Extract version from `Cargo.toml`
2. Build Docker image
3. Tag with both `latest` and version number
4. Optionally push to Docker Hub

---

## 📤 Publishing to Docker Hub

### Manual Push

```bash
# Login to Docker Hub
docker login

# Build and tag
docker build -t toonstore/tstd:latest -t toonstore/tstd:0.1.0 .

# Push both tags
docker push toonstore/tstd:latest
docker push toonstore/tstd:0.1.0
```

### Automated with Script

```bash
# Linux/Mac
./build-docker.sh

# Windows
.\build-docker.ps1

# Follow prompts to push
```

---

## 🔧 Configuration Options

### Environment Variables

```bash
docker run -d \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=1 \
  toonstore/tstd:latest
```

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (error, warn, info, debug, trace) |
| `RUST_BACKTRACE` | `0` | Enable backtrace on panic (0 or 1) |

### Command Line Arguments

```bash
docker run toonstore/tstd:latest \
  --bind 0.0.0.0:6379 \
  --data /data \
  --capacity 100000
```

| Argument | Default | Description |
|----------|---------|-------------|
| `--bind` | `127.0.0.1:6379` | Bind address and port |
| `--data` | `./data` | Data directory |
| `--capacity` | `10000` | Cache capacity (items) |

---

## 💾 Data Persistence

### Using Volumes

```bash
# Create named volume
docker volume create toonstore_data

# Run with volume
docker run -d \
  -v toonstore_data:/data \
  toonstore/tstd:latest

# Inspect volume
docker volume inspect toonstore_data

# Backup volume
docker run --rm \
  -v toonstore_data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/toonstore-backup.tar.gz -C /data .

# Restore volume
docker run --rm \
  -v toonstore_data:/data \
  -v $(pwd):/backup \
  alpine tar xzf /backup/toonstore-backup.tar.gz -C /data
```

### Using Bind Mounts

```bash
# Create local directory
mkdir -p ./data

# Run with bind mount
docker run -d \
  -v $(pwd)/data:/data \
  toonstore/tstd:latest

# Data is now in ./data directory
ls -lh ./data/
```

---

## 🌐 Network Configuration

### Bridge Network (Default)

```bash
docker run -d -p 6379:6379 toonstore/tstd:latest
```

**Connect from host:** `redis-cli -h localhost -p 6379`

### Host Network

```bash
docker run -d --network host toonstore/tstd:latest
```

**Connect from host:** `redis-cli -h localhost -p 6379`

### Custom Network

```bash
# Create network
docker network create toonstore-net

# Run ToonStore
docker run -d \
  --name toonstore \
  --network toonstore-net \
  toonstore/tstd:latest

# Run your app
docker run -d \
  --name myapp \
  --network toonstore-net \
  -e DATABASE_URL=redis://toonstore:6379 \
  myapp:latest
```

**Connect from containers:** `redis://toonstore:6379`

---

## 🏥 Health Checks

### Built-in Health Check

```bash
# Check health
docker exec toonstore tstd --health

# View health status
docker inspect --format='{{.State.Health.Status}}' toonstore

# View health check logs
docker inspect --format='{{range .State.Health.Log}}{{.Output}}{{end}}' toonstore
```

### Health Check in Compose

```yaml
healthcheck:
  test: ["CMD", "tstd", "--health"]
  interval: 30s
  timeout: 3s
  retries: 3
  start_period: 5s
```

---

## 🚀 Production Deployment

### Docker Compose (Production)

```yaml
version: '3.8'

services:
  toonstore:
    image: toonstore/tstd:0.1.0  # Pin version
    container_name: toonstore
    restart: always
    ports:
      - "6379:6379"
    volumes:
      - toonstore_data:/data
    environment:
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "tstd", "--health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G

volumes:
  toonstore_data:
    driver: local
```

### Docker Swarm

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.yml toonstore

# View services
docker service ls

# Scale service
docker service scale toonstore_toonstore=3

# Remove stack
docker stack rm toonstore
```

### Kubernetes

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: toonstore
spec:
  replicas: 1
  selector:
    matchLabels:
      app: toonstore
  template:
    metadata:
      labels:
        app: toonstore
    spec:
      containers:
      - name: toonstore
        image: toonstore/tstd:0.1.0
        ports:
        - containerPort: 6379
        volumeMounts:
        - name: data
          mountPath: /data
        livenessProbe:
          exec:
            command:
            - tstd
            - --health
          initialDelaySeconds: 5
          periodSeconds: 30
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: toonstore-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: toonstore
spec:
  selector:
    app: toonstore
  ports:
  - port: 6379
    targetPort: 6379
  type: LoadBalancer
```

---

## 📊 Monitoring

### View Logs

```bash
# View logs
docker logs toonstore

# Follow logs
docker logs -f toonstore

# Last 100 lines
docker logs --tail 100 toonstore

# With timestamps
docker logs -t toonstore
```

### Container Stats

```bash
# Real-time stats
docker stats toonstore

# One-time stats
docker stats --no-stream toonstore
```

### Using redis-cli

```bash
# Connect
docker exec -it toonstore redis-cli -h localhost

# Get server info
INFO

# Monitor commands
MONITOR

# Get stats
DBSIZE
INFO stats
```

---

## 🔒 Security

### Non-Root User

ToonStore runs as user `toonstore` (UID 1000) by default.

### Read-Only Root Filesystem

```bash
docker run -d \
  --read-only \
  --tmpfs /tmp \
  -v toonstore_data:/data \
  toonstore/tstd:latest
```

### Resource Limits

```bash
docker run -d \
  --memory 2g \
  --memory-swap 2g \
  --cpus 2 \
  toonstore/tstd:latest
```

### No New Privileges

```bash
docker run -d \
  --security-opt=no-new-privileges \
  toonstore/tstd:latest
```

---

## 🐛 Troubleshooting

### Container Won't Start

```bash
# View logs
docker logs toonstore

# Check if port is in use
netstat -an | grep 6379

# Check permissions
docker exec toonstore ls -la /data
```

### Can't Connect

```bash
# Check if container is running
docker ps

# Check health
docker exec toonstore tstd --health

# Test from inside container
docker exec -it toonstore redis-cli -h localhost PING

# Check network
docker inspect toonstore | grep IPAddress
```

### Performance Issues

```bash
# Check resource usage
docker stats toonstore

# Increase cache capacity
docker run -d toonstore/tstd:latest --capacity 100000

# Check logs for errors
docker logs toonstore | grep ERROR
```

---

## 📚 Examples

### Node.js App with ToonStore

```yaml
# docker-compose.yml
version: '3.8'

services:
  toonstore:
    image: toonstore/tstd:latest
    volumes:
      - toonstore_data:/data

  app:
    build: .
    depends_on:
      - toonstore
    environment:
      - DATABASE_URL=redis://toonstore:6379
    ports:
      - "3000:3000"

volumes:
  toonstore_data:
```

```javascript
// app.js
const redis = require('redis');
const client = redis.createClient({
    url: process.env.DATABASE_URL
});

await client.connect();
await client.set('key', 'value');
console.log(await client.get('key'));
```

### Python App with ToonStore

```yaml
version: '3.8'

services:
  toonstore:
    image: toonstore/tstd:latest
    volumes:
      - toonstore_data:/data

  app:
    build: .
    depends_on:
      - toonstore
    environment:
      - TOONSTORE_URL=redis://toonstore:6379
    ports:
      - "8000:8000"

volumes:
  toonstore_data:
```

```python
# app.py
import redis
import os

client = redis.from_url(os.getenv('TOONSTORE_URL'))
client.set('key', 'value')
print(client.get('key'))
```

---

## 🔗 Related Documentation

- [Connection Guide](CONNECTION_GUIDE.md) - Network vs Embedded modes
- [Architecture](ARCHITECTURE.md) - System architecture
- [Connecting from Apps](docs/connecting-from-apps.md) - Node.js & Go examples

---

## 📞 Support

- **GitHub Issues:** https://github.com/yourusername/toonstoredb/issues
- **Documentation:** https://github.com/yourusername/toonstoredb
- **Docker Hub:** https://hub.docker.com/r/toonstore/tstd

---

## 📝 License

MIT License - See [LICENSE](LICENSE) file for details
