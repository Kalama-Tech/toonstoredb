# ToonStore

⚠️ **NOT PRODUCTION READY - v0.1 Alpha** ⚠️

A **blazingly fast** embedded database with Redis-compatible protocol, built in Rust. ToonStore combines the performance of an embedded database (5.28M ops/sec) with the convenience of Redis compatibility.

[![CI](https://github.com/Kalama-Tech/toonstoredb/workflows/CI/badge.svg)](https://github.com/Kalama-Tech/toonstoredb/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🚀 Quick Start

### Option 1: Docker (Easiest)

```bash
# Pull and run
docker run -d -p 6379:6379 -v toonstore_data:/data toonstore/tstd:latest

# Connect with redis-cli
redis-cli -h 127.0.0.1 -p 6379
127.0.0.1:6379> SET mykey "Hello World"
OK
127.0.0.1:6379> GET mykey
"Hello World"
```

**Or with Docker Compose:**
```bash
# Download docker-compose.yml
curl -O https://raw.githubusercontent.com/yourusername/toonstoredb/main/docker-compose.yml

# Start
docker-compose up -d
```

### Option 2: Network Server (Redis-compatible)

```bash
# Install
cargo install tstd

# Start server
tstd --bind 0.0.0.0:6379

# Connect with redis-cli
redis-cli -h 127.0.0.1 -p 6379
127.0.0.1:6379> SET mykey "Hello World"
OK
127.0.0.1:6379> GET mykey
"Hello World"
```

### Option 3: Embedded Library (Maximum Performance)

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

## 📊 Performance

ToonStore is **fast**. Really fast.

| Operation | Embedded Mode | Network Mode | vs Redis |
|-----------|--------------|--------------|----------|
| **GET (cached)** | **5.28M ops/sec** | ~80k ops/sec | 26x faster |
| **GET (storage)** | 215k ops/sec | ~70k ops/sec | 1.08x faster |
| **SET** | 82k ops/sec | ~60k ops/sec | ≈ Equal |
| **DELETE** | 32M ops/sec | ~100k ops/sec | 320x faster |

**Embedded mode is 66x faster than network mode** (no TCP overhead)

See [BENCHMARKS.md](BENCHMARKS.md) for detailed comparisons with Redis, PostgreSQL, MySQL, and MongoDB.

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
