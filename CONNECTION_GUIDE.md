# ToonStore Connection Guide

ToonStore provides **two connection modes** for different use cases:

## 🌐 Mode 1: Network Server (RESP Protocol)

**Use when**: You need network access, Redis compatibility, or multi-process communication

### Starting the Server

```bash
# Start server (default: 127.0.0.1:6379)
tstd

# Custom address
tstd --bind 0.0.0.0:6379

# Custom data directory and cache
tstd --data /path/to/data --capacity 50000
```

### Connection String Format

```
redis://[host]:[port]
```

**Examples**:
```
redis://127.0.0.1:6379        # Local server
redis://localhost:6379        # Same as above
redis://192.168.1.100:6379    # Remote server
redis://myserver.com:6379     # DNS hostname
```

### Connecting from Different Languages

#### Redis-CLI
```bash
redis-cli -h 127.0.0.1 -p 6379

# Test connection
127.0.0.1:6379> PING
PONG
```

#### Python (redis-py)
```python
import redis

# Connect
client = redis.Redis(
    host='127.0.0.1',
    port=6379,
    decode_responses=True
)

# Or use connection string
client = redis.from_url('redis://127.0.0.1:6379')

# Test
print(client.ping())  # True

# Use
client.set('key', 'value')
value = client.get('key')
```

#### Node.js (ioredis)
```javascript
const Redis = require('ioredis');

// Connect
const client = new Redis({
  host: '127.0.0.1',
  port: 6379
});

// Or use connection string
const client = new Redis('redis://127.0.0.1:6379');

// Test
await client.ping(); // 'PONG'

// Use
await client.set('key', 'value');
const value = await client.get('key');
```

#### Go (go-redis)
```go
import "github.com/redis/go-redis/v9"

// Connect
client := redis.NewClient(&redis.Options{
    Addr: "127.0.0.1:6379",
})

// Or parse connection string
opts, _ := redis.ParseURL("redis://127.0.0.1:6379")
client := redis.NewClient(opts)

// Test
pong, _ := client.Ping(ctx).Result()
fmt.Println(pong) // "PONG"

// Use
client.Set(ctx, "key", "value", 0)
val, _ := client.Get(ctx, "key").Result()
```

#### Rust (redis-rs)
```rust
use redis::Commands;

// Connect
let client = redis::Client::open("redis://127.0.0.1:6379")?;
let mut con = client.get_connection()?;

// Test
let pong: String = redis::cmd("PING").query(&mut con)?;
println!("{}", pong); // "PONG"

// Use
con.set("key", "value")?;
let value: String = con.get("key")?;
```

### Performance (Network Mode)

```
Operation       Throughput       Latency
────────────────────────────────────────
GET (cached)    ~80k ops/sec     ~12.5 µs
SET             ~60k ops/sec     ~16.7 µs
Mixed 50/50     ~70k ops/sec     ~14.3 µs
```

**Network overhead**: ~60 µs round-trip on localhost

---

## 📚 Mode 2: Embedded Library (Direct)

**Use when**: You need maximum performance, no network overhead, single-process

### Connection (Rust)

#### Using ToonCache (Recommended)
```rust
use tooncache::ToonCache;

// "Connect" (open database)
let cache = ToonCache::new("./data", 10000)?;

// Use
let row_id = cache.put(b"my data")?;
let data = cache.get(row_id)?;
cache.delete(row_id)?;

// Scan
for result in cache.scan() {
    let (id, data) = result?;
    println!("{}: {:?}", id, data);
}

// Stats
let stats = cache.stats();
println!("Hit ratio: {:.2}%", stats.hit_ratio() * 100.0);
```

#### Using ToonStore (Storage Only)
```rust
use toonstoredb::ToonStore;

// "Connect" (open database)
let db = ToonStore::open("./data")?;

// Use
let row_id = db.put(b"my data")?;
let data = db.get(row_id)?;
db.delete(row_id)?;

// Close (syncs to disk)
db.close()?;
```

### "Connection String" Format

In embedded mode, there's no network connection string. Instead, you specify:

```rust
// Format: ToonCache::new(data_dir, cache_capacity)
ToonCache::new("./data", 10000)?

// Or with path
ToonCache::new("/var/lib/toonstore", 50000)?
```

### Performance (Embedded Mode)

```
Operation       Throughput       Latency
────────────────────────────────────────
GET (cached)    5.28M ops/sec    189 ns
GET (storage)   215k ops/sec     4.6 µs
PUT             82k ops/sec      12.2 µs
DELETE          32M ops/sec      31 ns
Mixed 50/50     121k ops/sec     8.3 µs
```

**No network overhead**: Direct memory access

---

## 🔄 Comparison: Network vs Embedded

| Feature | Network (RESP) | Embedded (Library) |
|---------|----------------|-------------------|
| **Performance** | ~70k ops/sec | ~5.28M ops/sec (cached) |
| **Latency** | ~14 µs + network | 189 ns (cached) |
| **Network Overhead** | Yes (~60 µs) | None |
| **Multi-process** | ✅ Yes | ❌ No (single process only) |
| **Language Support** | Any (via RESP) | Rust (+ FFI bindings) |
| **Redis Compatibility** | ✅ Yes | ❌ No |
| **Use Case** | Microservices, remote access | High-performance local storage |
| **Connection String** | `redis://host:port` | `ToonCache::new(path, cap)` |

---

## 📊 Architecture Diagram

```
┌────────────────────────────────────────────────────────────────┐
│                      Application                                │
└─────────────────┬──────────────────────┬───────────────────────┘
                  │                      │
                  │ Network Mode         │ Embedded Mode
                  │ (RESP Protocol)      │ (Direct Library)
                  ↓                      ↓
┌─────────────────────────────┐  ┌──────────────────────────────┐
│  tstd (RESP Server)         │  │  ToonCache Library           │
│  - TCP: 127.0.0.1:6379      │  │  - In-process                │
│  - Redis-compatible         │  │  - No network                │
│  - ~70k ops/sec             │  │  - 5.28M ops/sec (cached)    │
└─────────────┬───────────────┘  └──────────────┬───────────────┘
              │                                  │
              └──────────────┬───────────────────┘
                             ↓
              ┌──────────────────────────────────┐
              │  tooncache (LRU Cache)           │
              │  - 5.28M ops/sec (cached reads)  │
              └──────────────┬───────────────────┘
                             ↓
              ┌──────────────────────────────────┐
              │  toonstoredb (Storage Engine)    │
              │  - 215k ops/sec (storage reads)  │
              │  - Data dir: ./data              │
              └──────────────────────────────────┘
```

---

## 🎯 Which Mode Should I Use?

### Use **Network Mode (tstd)** if:
- ✅ You need to access ToonStore from multiple processes
- ✅ You want Redis compatibility (existing tools/libraries)
- ✅ You need remote access over TCP
- ✅ Your app is in a language other than Rust
- ✅ You're building microservices
- ✅ You can tolerate ~60 µs network overhead

### Use **Embedded Mode (library)** if:
- ✅ You need maximum performance (5.28M ops/sec)
- ✅ Your app is in Rust (or has FFI bindings)
- ✅ You only need single-process access
- ✅ You want to avoid network overhead
- ✅ You're building a performance-critical system
- ✅ You need sub-microsecond latency

---

## 🔗 Connection Examples by Use Case

### Web API Server (Network Mode)
```bash
# Start ToonStore server
tstd --bind 0.0.0.0:6379

# Connect from Python Flask app
import redis
cache = redis.from_url('redis://toonstore-server:6379')
```

### High-Performance Service (Embedded Mode)
```rust
// Direct integration in Rust
use tooncache::ToonCache;

let cache = ToonCache::new("./data", 100000)?;
// 5.28M ops/sec, no network!
```

### Microservices (Network Mode)
```yaml
# docker-compose.yml
services:
  toonstore:
    image: ghcr.io/yourusername/toonstore:latest
    ports:
      - "6379:6379"
  
  app:
    environment:
      - REDIS_URL=redis://toonstore:6379
```

### Embedded Database (Library Mode)
```rust
// Single binary, maximum performance
fn main() {
    let db = ToonCache::new("/var/lib/myapp", 50000)?;
    // Use db throughout your app
}
```

---

## 📝 Quick Reference

### Network Mode (tstd)
```bash
# Start
tstd --bind 127.0.0.1:6379 --data ./data --capacity 10000

# Connection string
redis://127.0.0.1:6379

# Performance
~70k ops/sec over TCP
```

### Embedded Mode (library)
```rust
// Rust code
let cache = ToonCache::new("./data", 10000)?;

// "Connection string" equivalent
data_dir: "./data"
capacity: 10000

// Performance
5.28M ops/sec (cached), 215k ops/sec (storage)
```

---

## 🚀 Getting Started

### 1. Install ToonStore
```bash
cargo install toonstore  # CLI tool
```

### 2. Start Server (Network Mode)
```bash
tstd
# Server starts on redis://127.0.0.1:6379
```

### 3. Connect and Test
```bash
redis-cli -p 6379
127.0.0.1:6379> PING
PONG
```

---

## 📚 Additional Resources

- [README.md](README.md) - Project overview
- [BENCHMARKS.md](BENCHMARKS.md) - Performance comparison
- [DOCKER_SETUP_GUIDE.md](DOCKER_SETUP_GUIDE.md) - Docker deployment
- [FFI_ROADMAP.md](FFI_ROADMAP.md) - Language bindings (Week 4)

---

**Summary**: 
- **Network Mode**: `redis://host:port` (Redis-compatible, ~70k ops/sec)
- **Embedded Mode**: `ToonCache::new(path, capacity)` (Direct, 5.28M ops/sec)

Choose based on your performance needs and deployment architecture!
