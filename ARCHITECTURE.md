# ToonStore Architecture

## 📐 System Architecture

ToonStore has a **layered architecture** with 3 distinct components:

```
┌─────────────────────────────────────────────────────────────┐
│                    CLIENT APPLICATIONS                       │
│  (Python, Node.js, Go, Redis-cli, etc.)                     │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ TCP (RESP Protocol)
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                         TSTD                                 │
│              Network Server (Port 6379)                      │
│              Connection: redis://host:port                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Direct Function Calls
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      TOONCACHE                               │
│          LRU Cache Layer (In-Memory Cache)                   │
│          Connection: file://path?capacity=N                  │
│          API: ToonCache::new("./data", 10000)                │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Direct Function Calls
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    TOONSTOREDB                               │
│            Storage Engine (Memory-Mapped Files)              │
│            Connection: file://path                           │
│            API: ToonStore::open("./data")                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │  DISK STORAGE │
                    │   (./data/)   │
                    └───────────────┘
```

---

## 🔌 Connection Strings for Each Layer

### 1️⃣ **TSTD** (Network Server)

**Type**: Network server with RESP protocol  
**Connection String**: `redis://host:port`  
**Default**: `redis://127.0.0.1:6379`

```python
# Python
import redis
client = redis.from_url('redis://127.0.0.1:6379')
```

```javascript
// Node.js
const redis = require('redis');
const client = redis.createClient({ url: 'redis://127.0.0.1:6379' });
```

```bash
# CLI
redis-cli -h 127.0.0.1 -p 6379
```

**Use Case**: Remote access, multi-language support, Redis compatibility

---

### 2️⃣ **TOONCACHE** (Cache Layer)

**Type**: Embedded Rust library  
**Connection String**: `file://path?capacity=N` (conceptual)  
**API**: `ToonCache::new(path, capacity)`  
**Default**: `./data` with capacity `10000`

```rust
use tooncache::ToonCache;

// Direct embedded access (66x faster than network)
let cache = ToonCache::new("./data", 10000)?;

// Store data
let id = cache.put(b"Hello, World!")?;

// Retrieve data
let data = cache.get(id)?;
```

**Use Case**: Maximum performance, Rust applications, 5.28M ops/sec cached reads

---

### 3️⃣ **TOONSTOREDB** (Storage Engine)

**Type**: Embedded Rust library  
**Connection String**: `file://path` (conceptual)  
**API**: `ToonStore::open(path)`  
**Default**: `./data`

```rust
use toonstoredb::ToonStore;

// Direct database access (no cache)
let store = ToonStore::open("./data")?;

// Store data
let id = store.put(b"Raw data")?;

// Retrieve data
let data = store.get(id)?;
```

**Use Case**: Low-level access, custom caching strategies, raw storage operations

---

## 📊 Performance Comparison

| Layer | Type | Connection | Performance | Use Case |
|-------|------|-----------|-------------|----------|
| **tstd** | Network | `redis://host:port` | ~80k ops/sec | Remote access, multi-language |
| **tooncache** | Embedded | `file://path?capacity=N` | 5.28M ops/sec (cached) | Maximum performance, Rust apps |
| **toonstoredb** | Embedded | `file://path` | 215k ops/sec (storage) | Low-level access, custom caching |

**Key Insight**: Embedded mode is **66x faster** than network mode (no TCP overhead)

---

## 🏗️ Layer Responsibilities

### TSTD (Network Server)
- **Purpose**: Expose ToonStore over TCP using RESP protocol
- **Features**:
  - Redis-compatible protocol
  - Multi-client support
  - Connection pooling
  - Health checks
- **Dependencies**: Uses `tooncache` internally
- **Connection**: Network socket (TCP)

### TOONCACHE (Cache Layer)
- **Purpose**: LRU cache for hot data
- **Features**:
  - In-memory cache (HashMap + LRU list)
  - Automatic eviction
  - Hit/miss statistics
  - 5.28M ops/sec for cached reads
- **Dependencies**: Uses `toonstoredb` for storage
- **Connection**: Direct Rust function calls

### TOONSTOREDB (Storage Engine)
- **Purpose**: Persistent storage using TOON format
- **Features**:
  - Memory-mapped I/O
  - Token-oriented storage
  - Efficient serialization
  - 215k ops/sec for storage reads
- **Dependencies**: None (core storage layer)
- **Connection**: Direct file system access

---

## 🎯 Which Layer to Use?

### Use **TSTD** (Network Mode) when:
✅ Accessing from non-Rust languages (Python, Node.js, Go, etc.)  
✅ Need multi-client support  
✅ Deploying as a microservice  
✅ Using existing Redis client libraries  
✅ Remote access required  

**Connection**: `redis://127.0.0.1:6379`

### Use **TOONCACHE** (Embedded Mode) when:
✅ Building Rust applications  
✅ Maximum performance needed (5.28M ops/sec)  
✅ No network overhead tolerated  
✅ Single-process access sufficient  
✅ Want automatic LRU caching  

**Connection**: `ToonCache::new("./data", 10000)`

### Use **TOONSTOREDB** (Low-Level) when:
✅ Building custom caching strategies  
✅ Need low-level storage control  
✅ Implementing custom data structures  
✅ Bypassing cache layer  
✅ Advanced use cases  

**Connection**: `ToonStore::open("./data")`

---

## 🚀 Deployment Modes

### Mode 1: Network Server (Multi-Language)
```bash
# Start server
tstd --bind 0.0.0.0:6379 --data ./data --capacity 10000

# Connect from any language
python app.py   # Uses redis://localhost:6379
node app.js     # Uses redis://localhost:6379
go run main.go  # Uses redis://localhost:6379
```

### Mode 2: Embedded Library (Rust Only)
```rust
// Directly embed in Rust application
use tooncache::ToonCache;

#[tokio::main]
async fn main() -> Result<()> {
    let cache = ToonCache::new("./data", 10000)?;
    
    // Use cache directly (66x faster)
    let id = cache.put(b"data")?;
    let data = cache.get(id)?;
    
    Ok(())
}
```

---

## 🔍 Connection String Examples

### Network Mode (TSTD)

| Language | Connection String |
|----------|-------------------|
| Python | `redis.from_url('redis://127.0.0.1:6379')` |
| Node.js | `redis.createClient({ url: 'redis://127.0.0.1:6379' })` |
| Go | `redis.NewClient(&redis.Options{Addr: "127.0.0.1:6379"})` |
| Java | `new Jedis("127.0.0.1", 6379)` |
| Ruby | `Redis.new(url: "redis://127.0.0.1:6379")` |
| CLI | `redis-cli -h 127.0.0.1 -p 6379` |

### Embedded Mode (ToonCache/ToonStore)

| Layer | Rust API | Conceptual URI |
|-------|----------|----------------|
| ToonCache | `ToonCache::new("./data", 10000)?` | `file://./data?capacity=10000` |
| ToonStore | `ToonStore::open("./data")?` | `file://./data` |

---

## 📁 Data Directory Structure

```
./data/
├── data.toon       # Main data file (TOON format)
├── index.toon      # Index file (optional)
└── metadata.toon   # Metadata (optional)
```

**All three layers** (`tstd`, `tooncache`, `toonstoredb`) use the **same data directory**.

---

## 🔄 Data Flow Example

### Write Operation
```
Client → TSTD → ToonCache → ToonStore → Disk
 (TCP)   (RESP)  (Cache)     (Storage)   (File)
```

### Read Operation (Cached)
```
Client → TSTD → ToonCache → Return
 (TCP)   (RESP)  (Hit! 5.28M ops/sec)
```

### Read Operation (Uncached)
```
Client → TSTD → ToonCache → ToonStore → Disk → Return
 (TCP)   (RESP)  (Miss)      (Storage)   (File)
```

---

## 🎓 Summary

| Component | Type | Connection | Speed | Usage |
|-----------|------|------------|-------|-------|
| **tstd** | Network Server | `redis://host:port` | ~80k ops/sec | Multi-language, remote access |
| **tooncache** | Embedded Library | `file://path?capacity=N` | 5.28M ops/sec | Rust apps, max performance |
| **toonstoredb** | Embedded Library | `file://path` | 215k ops/sec | Low-level, custom caching |

**Key Takeaway**:
- **Network Mode**: One connection string (`redis://host:port`)
- **Embedded Mode**: Two layers, same data directory
  - `ToonCache::new("./data", capacity)` - Cache + Storage
  - `ToonStore::open("./data")` - Storage only

Both embedded layers use **file paths**, not network connection strings, because they're **libraries**, not servers!

---

## 📚 See Also

- [CONNECTION_GUIDE.md](CONNECTION_GUIDE.md) - Detailed connection guide
- [docs/getting-started.md](docs/getting-started.md) - Quick start tutorial
- [docs/api-rust.md](docs/api-rust.md) - Rust API reference
- [BENCHMARKS.md](BENCHMARKS.md) - Performance benchmarks
