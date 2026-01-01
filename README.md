# ToonStore

⚠️ **NOT PRODUCTION READY - v0.1 Alpha** ⚠️

**A blazingly fast embedded database with Redis-compatible protocol, built in Rust.**

ToonStore is a high-performance key-value store that gives you the **speed of embedded databases** (5.28M ops/sec) with the **convenience of Redis compatibility**. Use it as an embedded library for maximum performance, or run it as a Redis-compatible server accessible from any language.

[![CI/CD](https://github.com/Kalama-Tech/toonstoredb/workflows/CI/CD%20Pipeline/badge.svg)](https://github.com/Kalama-Tech/toonstoredb/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker Pulls](https://img.shields.io/docker/pulls/samso9th/toonstore)](https://hub.docker.com/r/samso9th/toonstore)

---

## 🎯 What is ToonStore?

ToonStore is a **persistent key-value database** designed for applications that need:
- 🚀 **Extreme performance** - 5.28M ops/sec for cached reads, 66x faster than network databases
- 💾 **Data persistence** - All data stored on disk and survives restarts
- 🔌 **Redis compatibility** - Works with existing Redis clients (Node.js, Python, Go, etc.)
- 📦 **Embedded mode** - Use directly in Rust applications for maximum speed
- 🌐 **Network mode** - Run as a server, connect from any language

---

## ⚡ Why ToonStore?

### The Problem
- **Redis** is fast but volatile (RAM-only by default) and complex to persist data
- **PostgreSQL/MySQL** are reliable but slower for key-value workloads
- **RocksDB/LevelDB** are fast but lack network access and Redis compatibility

### The Solution: ToonStore
ToonStore combines the best of all worlds:

| Feature | ToonStore | Redis | PostgreSQL | RocksDB |
|---------|-----------|-------|------------|---------|
| **Speed** | 5.28M ops/sec (embedded) | ~80k ops/sec | ~65k ops/sec | ~100k ops/sec |
| **Persistent** | ✅ Yes | ❌ Optional | ✅ Yes | ✅ Yes |
| **Redis Protocol** | ✅ Yes | ✅ Yes | ❌ No | ❌ No |
| **Embedded Mode** | ✅ Yes | ❌ No | ❌ No | ✅ Yes |
| **Network Mode** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No |
| **Multi-language** | ✅ Yes | ✅ Yes | ✅ Yes | ⚠️ Limited |

---

## 🎁 Key Benefits

### 1. **Blazingly Fast**
- **5.28M operations/second** in embedded mode (cached reads)
- **215k ops/sec** for storage operations (66x faster than network)
- **32M deletions/second** (320x faster than Redis)

### 2. **Data Persistence**
- All data stored on disk using efficient TOON format
- Survives restarts and crashes
- Memory-mapped I/O for fast disk access

### 3. **Redis Compatible**
- Use any Redis client library (50+ languages supported)
- Familiar commands: `GET`, `SET`, `DEL`, `EXISTS`, `KEYS`
- Drop-in replacement for Redis in many use cases

### 4. **Dual Mode Operation**

**Network Mode:**
```javascript
// Connect from Node.js, Python, Go, etc.
const redis = require('redis');
const client = redis.createClient({ url: 'redis://localhost:6379' });
await client.set('key', 'value');
```

**Embedded Mode:**
```rust
// Direct Rust integration (66x faster!)
let cache = ToonCache::new("./data", 10000)?;
let id = cache.put(b"data")?;
let data = cache.get(id)?;
```

### 5. **Built-in LRU Cache**
- Automatic caching of hot data in RAM
- 10,000 item default capacity (configurable)
- No manual cache management needed

### 6. **Easy Deployment**
- Single binary, no dependencies
- Docker images available
- Works on Linux, Windows, macOS
- Cross-platform (amd64 and arm64)

---

## 📊 Performance Comparison

ToonStore is designed for speed:

| Operation | ToonStore (Embedded) | ToonStore (Network) | Redis | PostgreSQL |
|-----------|---------------------|---------------------|-------|------------|
| **GET (cached)** | **5.28M ops/sec** | ~80k ops/sec | ~80k ops/sec | ~65k ops/sec |
| **GET (storage)** | 215k ops/sec | ~70k ops/sec | ~65k ops/sec | ~65k ops/sec |
| **SET** | 82k ops/sec | ~60k ops/sec | ~60k ops/sec | ~55k ops/sec |
| **DELETE** | 32M ops/sec | ~100k ops/sec | ~100k ops/sec | ~70k ops/sec |

**Key Insight:** Embedded mode is 66x faster than network mode (no TCP overhead)

See [BENCHMARKS.md](BENCHMARKS.md) for detailed benchmarks and methodology.

---

## 🚀 Quick Start

### Option 1: Docker (Easiest - Recommended)

```bash
# Pull from Docker Hub
docker pull samso9th/toonstore:latest

# Or pull from GitHub Container Registry
docker pull ghcr.io/kalama-tech/toonstoredb:latest

# Run ToonStore
docker run -d \
  --name toonstore \
  -p 6379:6379 \
  -v toonstore_data:/data \
  samso9th/toonstore:latest

# Test connection
redis-cli -h 127.0.0.1 -p 6379 PING
# Output: PONG

# Use it
redis-cli -h 127.0.0.1 -p 6379
127.0.0.1:6379> SET mykey "Hello World"
OK
127.0.0.1:6379> GET mykey
"Hello World"
```

**With Docker Compose:**
```bash
# Download docker-compose.yml
curl -O https://raw.githubusercontent.com/Kalama-Tech/toonstoredb/main/docker-compose.yml

# Start
docker-compose up -d

# Stop
docker-compose down
```

### Option 2: Pre-built Binary (Coming Soon)

```bash
# Linux/macOS
curl -L https://github.com/Kalama-Tech/toonstoredb/releases/latest/download/tstd -o tstd
chmod +x tstd
./tstd --bind 0.0.0.0:6379

# Windows
# Download from: https://github.com/Kalama-Tech/toonstoredb/releases
```

### Option 3: Build from Source

```bash
# Clone repository
git clone https://github.com/Kalama-Tech/toonstoredb.git
cd toonstoredb

# Build (requires Rust 1.70+)
cargo build --release

# Run
./target/release/tstd --bind 0.0.0.0:6379
```

### Option 4: Embedded Library (Rust Only)

```toml
# Cargo.toml
[dependencies]
tooncache = { git = "https://github.com/Kalama-Tech/toonstoredb" }
```

```rust
use tooncache::ToonCache;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open database
    let cache = ToonCache::new("./data", 10000)?;
    
    // Store data
    let id = cache.put(b"Hello, World!")?;
    
    // Retrieve data
    let data = cache.get(id)?;
    println!("Retrieved: {:?}", String::from_utf8(data)?);
    
    Ok(())
}
```

---

## 🔌 Connect from Your Application

ToonStore is Redis-compatible, so you can use any Redis client library:

### Node.js
```javascript
const redis = require('redis');
const client = redis.createClient({ url: 'redis://localhost:6379' });

await client.connect();
await client.set('user:1', 'John Doe');
const user = await client.get('user:1');
console.log(user); // "John Doe"
```

### Python
```python
import redis

client = redis.from_url('redis://localhost:6379')
client.set('user:1', 'John Doe')
user = client.get('user:1')
print(user)  # b'John Doe'
```

### Go
```go
import "github.com/redis/go-redis/v9"

client := redis.NewClient(&redis.Options{
    Addr: "localhost:6379",
})

client.Set(ctx, "user:1", "John Doe", 0)
user, _ := client.Get(ctx, "user:1").Result()
fmt.Println(user)  // "John Doe"
```

See [docs/connecting-from-apps.md](docs/connecting-from-apps.md) for more examples.

---

## ✨ Features

### Core Features (v0.1)
- ✅ **Dual Mode**: Network (Redis-compatible) or Embedded (5.28M ops/sec)
- ✅ **LRU Cache**: Automatic caching with 5.28M ops/sec cached reads
- ✅ **RESP Protocol**: Works with any Redis client library
- ✅ **TOON Format**: Efficient token-oriented storage format
- ✅ **Memory-Mapped I/O**: Fast disk access with OS-level caching
- ✅ **Cross-Platform**: Linux, Windows, macOS
- ✅ **Docker Ready**: Official images on Docker Hub

### Commands Supported (v0.1)
```
PING, ECHO          - Connection testing
GET, SET, DEL       - Core operations  
EXISTS, KEYS        - Key inspection
DBSIZE, FLUSHDB     - Database management
INFO                - Server statistics
```

---

## 📚 Documentation

### Getting Started
- **[Quick Start Guide](docs/getting-started.md)** - Your first ToonStore app
- **[Installation](docs/installation.md)** - All installation methods
- **[Docker Deployment](DOCKER_DEPLOYMENT.md)** - Complete Docker guide
- **[Connection Guide](CONNECTION_GUIDE.md)** - Network vs Embedded mode
- **[Architecture](ARCHITECTURE.md)** - 3-layer architecture & connection strings

### Usage Guides
- **[Rust API Reference](docs/api-rust.md)** - Embedded library usage
- **[RESP Server Guide](docs/resp-server.md)** - Network server setup
- **[Configuration](docs/configuration.md)** - Server & cache tuning

### Advanced
- **[TOON Format](docs/toon-format.md)** - Storage format specification
- **[TOON Format](docs/toon-format.md)** - Storage format specification
- **[Performance Tuning](docs/performance.md)** - Optimization guide
- **[Benchmarks](BENCHMARKS.md)** - Detailed performance data

### Deployment
- **[Docker Guide](DOCKER_SETUP_GUIDE.md)** - Container deployment
- **[Production Checklist](docs/production.md)** - Before going live
- **[Monitoring](docs/monitoring.md)** - Health checks & metrics

---

## 🏗️ Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                      Application                                │
└─────────────────┬──────────────────────┬───────────────────────┘
                  │                      │
        Network Mode (tstd)    Embedded Mode (library)
        redis://host:port      ToonCache::new()
        ~70k ops/sec           5.28M ops/sec
                  │                      │
                  └──────────┬───────────┘
                             ↓
              ┌──────────────────────────────────┐
              │  tooncache (LRU Cache)           │
              │  - 5.28M ops/sec (cached reads)  │
              │  - Configurable capacity         │
              └──────────────┬───────────────────┘
                             ↓
              ┌──────────────────────────────────┐
              │  toonstoredb (Storage Engine)    │
              │  - 215k ops/sec (storage reads)  │
              │  - TOON format parser            │
              │  - Memory-mapped files           │
              └──────────────────────────────────┘
```

---

## 🎯 Use Cases

### ✅ Ideal For:
- 🚀 **High-performance caching** (5.28M ops/sec!)
- 📦 **Embedded databases** in Rust applications
- 🔄 **Redis replacement** with better performance
- 💾 **Key-value storage** with persistence
- ⚡ **In-process caching** with disk backup

### ❌ Not Suitable For (v0.1):
- 🔐 ACID transactions
- 🔗 Complex queries / JOINs
- 🌐 Multi-node clustering
- 🔒 Strong consistency guarantees

---

## 📦 Installation

### From Source (Recommended for v0.1)

```bash
# Clone repository
git clone https://github.com/Kalama-Tech/toonstoredb
cd toonstoredb

# Build release
cargo build --release

# Run server
./target/release/tstd --bind 0.0.0.0:6379
```

### Using Cargo (Coming Soon)

```bash
# Install server binary
cargo install tstd

# Add to your Rust project
cargo add tooncache
```

### Docker

```bash
# Pull image
docker pull ghcr.io/yourusername/toonstore:latest

# Run server
docker run -d \
  -p 6379:6379 \
  -v $(pwd)/data:/data \
  ghcr.io/yourusername/toonstore:latest
```

See [DOCKER_SETUP_GUIDE.md](DOCKER_SETUP_GUIDE.md) for complete Docker setup.

---

## 🔌 Language Support

### Network Mode (Any Language via Redis Protocol)

**Python**
```python
import redis
client = redis.from_url('redis://localhost:6379')
client.set('key', 'value')
```

**Node.js**
```javascript
const Redis = require('ioredis');
const client = new Redis('redis://localhost:6379');
await client.set('key', 'value');
```

**Go**
```go
import "github.com/redis/go-redis/v9"
client := redis.NewClient(&redis.Options{Addr: "localhost:6379"})
client.Set(ctx, "key", "value", 0)
```

### Embedded Mode (Direct Library)

**Rust**
```rust
use tooncache::ToonCache;
let cache = ToonCache::new("./data", 10000)?;
```

**Python** (Coming Week 4)
```python
import toonstore
db = toonstore.ToonCache("./data", capacity=10000)
```

---

## 🛠️ Configuration

### Server Options

```bash
tstd \
  --bind 0.0.0.0:6379 \       # Bind address
  --data ./data \              # Data directory
  --capacity 10000             # Cache capacity
```

### Environment Variables

```bash
RUST_LOG=info    # Logging level (info, debug, trace)
```

### Embedded Configuration

```rust
let cache = ToonCache::new(
    "./data",    // Data directory
    10000        // Cache capacity
)?;
```

---

## 📈 Monitoring

### Check Server Status

```bash
# Connect with redis-cli
redis-cli -h 127.0.0.1 -p 6379

# Get statistics
127.0.0.1:6379> INFO
# Server
toonstore_version:0.1.0

# Stats
total_keys:1000
cache_size:850
cache_capacity:10000
cache_hits:95000
cache_misses:5000
cache_hit_ratio:0.95

# Check database size
127.0.0.1:6379> DBSIZE
(integer) 1000
```

### Health Check

```bash
# Docker health check
tstd --health

# Or via TCP
redis-cli PING
PONG
```

---

## 🐛 Troubleshooting

### Server won't start

```bash
# Check if port is already in use
netstat -an | grep 6379

# Try different port
tstd --bind 127.0.0.1:6380
```

### Connection refused

```bash
# Check server is running
ps aux | grep tstd

# Check firewall
sudo ufw allow 6379/tcp
```

### Low performance

```bash
# Increase cache capacity
tstd --capacity 50000

# Check cache hit ratio
redis-cli INFO | grep cache_hit_ratio

# Use embedded mode for maximum performance
```

See [docs/troubleshooting.md](docs/troubleshooting.md) for more solutions.

---

## 🗺️ Roadmap

### v0.1 (Current - Weeks 1-3) ✅
- [x] Storage engine (toonstoredb)
- [x] LRU cache (tooncache)
- [x] RESP server (tstd)
- [x] Basic benchmarks
- [x] Docker support

### v0.2 (Week 4) 🚧
- [ ] Python bindings (PyO3)
- [ ] npm package (Neon)
- [ ] Complete documentation
- [ ] PyPI + npm publish

### v0.3 (Future)
- [ ] WAL for durability
- [ ] Transactions
- [ ] Replication
- [ ] More RESP commands
- [ ] Clustering support

---

## 🤝 Contributing

ToonStore is in active development. We welcome contributions!

### Development Setup

```bash
# Clone
git clone https://github.com/Kalama-Tech/toonstoredb
cd toonstoredb

# Build
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Guidelines

- Write tests for new features
- Run `cargo fmt` and `cargo clippy`
- Update documentation
- Follow existing code style

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## 📜 License

ToonStore is licensed under the [MIT License](LICENSE).

---

## 🙏 Acknowledgments

- Built with [Rust](https://rust-lang.org)
- TOON format inspired by [toondb](https://github.com/ameyakhot/toondb)
- RESP protocol compatible with [Redis](https://redis.io)

---

## 📞 Support & Community

- **Issues**: [GitHub Issues](https://github.com/Kalama-Tech/toonstoredb/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Kalama-Tech/toonstoredb/discussions)
- **Documentation**: [docs/](docs/)

---

## ⚡ Quick Links

- [Getting Started](docs/getting-started.md)
- [API Documentation](docs/api-rust.md)
- [Benchmarks](BENCHMARKS.md)
- [Connection Guide](CONNECTION_GUIDE.md)
- [Docker Setup](DOCKER_SETUP_GUIDE.md)
- [Architecture](docs/architecture.md)

---

**Built with ❤️ in Rust** | **Performance: 5.28M ops/sec** | **License: MIT**
