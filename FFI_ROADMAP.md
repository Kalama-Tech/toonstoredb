# ToonStore FFI & Language Bindings Roadmap

**Status**: Planning Phase  
**Target**: Week 4 + Post-v0.1

## Overview

ToonStore will provide native bindings for multiple languages through a C-compatible FFI layer. The core library is written in Rust and compiles to a static library (`.a`) or dynamic library (`.so`/`.dll`/`.dylib`).

## Architecture

```
┌─────────────────────────────────────────┐
│     Application Layer                   │
│  (Python, Node.js, Go, C, Ruby, etc.)  │
├─────────────────────────────────────────┤
│     Language-Specific Bindings          │
│  (PyO3, Neon, cgo, FFI, Helix)         │
├─────────────────────────────────────────┤
│     ToonStore C API (FFI)               │
│  (Exported C functions)                 │
├─────────────────────────────────────────┤
│     ToonStore Core (Rust)               │
│  (toonstoredb + tooncache)             │
└─────────────────────────────────────────┘
```

## Language Support Matrix

### Week 4 (v0.1 Launch)

| Language | Priority | Status | Package Manager | Target |
|----------|----------|--------|-----------------|--------|
| **Python** | 🔴 P0 | ⏳ Week 4 | PyPI (`pip`) | `pip install toonstore` |
| **Node.js** | 🟡 P1 | ⏳ Week 4 | npm | `npm install toonstore` |

### Post-v0.1 (Week 5+)

| Language | Priority | Status | Package Manager | Target |
|----------|----------|--------|-----------------|--------|
| C | 🟡 P1 | 📝 Planned | Manual | Header files |
| Go | 🟡 P1 | 📝 Planned | Go modules | `go get` |
| Ruby | 🟢 P2 | 📝 Planned | RubyGems | `gem install` |
| Rust | ✅ Native | ✅ Done | Crates.io | `cargo add` |
| C++ | 🟢 P2 | 📝 Planned | Manual | Header wrapper |
| Java/JVM | 🔵 P3 | 📋 Backlog | Maven/Gradle | JNI |
| C# / .NET | 🔵 P3 | 📋 Backlog | NuGet | P/Invoke |
| Zig | 🔵 P3 | 📋 Backlog | Manual | C import |

## 1. Python Bindings (`toonstore-py`)

### Week 4 Deliverable

**Package**: `toonstore` on PyPI  
**Technology**: PyO3 (Rust ↔ Python)  
**Target**: v0.1.0

### Installation
```bash
pip install toonstore
```

### API Design
```python
from toonstore import ToonStore

# Open database
db = ToonStore.open("/path/to/db")

# Basic operations
row_id = db.put(b"users[1]{id,name}: 1,Alice")
data = db.get(row_id)
db.delete(row_id)

# Iteration
for row_id, data in db.scan():
    print(f"{row_id}: {data}")

# Context manager
with ToonStore.open("/path/to/db") as db:
    db.put(b"data")

# Close
db.close()
```

### Features
- ✅ Open/close database
- ✅ PUT, GET, DELETE, SCAN
- ✅ Iterator support (`__iter__`)
- ✅ Context manager (`__enter__`, `__exit__`)
- ✅ Error handling (Python exceptions)
- ✅ Type hints (`.pyi` stubs)

### Build & Distribution
```toml
# crates/toonstore-py/Cargo.toml
[package]
name = "toonstore-py"
version = "0.1.0"

[dependencies]
pyo3 = { workspace = true }
toonstoredb = { path = "../toonstoredb" }

[lib]
crate-type = ["cdylib"]
```

**Build**: `maturin build --release`  
**Publish**: `maturin publish`

### Platforms
- ✅ Windows (x86_64)
- ✅ macOS (x86_64, ARM64)
- ✅ Linux (x86_64, ARM64)

Wheels pre-built in CI for all platforms.

---

## 2. Node.js Bindings (`toonstore-node`)

### Week 4 Stretch Goal / Week 5

**Package**: `toonstore` on npm  
**Technology**: Neon (Rust ↔ Node.js)  
**Target**: v0.1.0

### Installation
```bash
npm install toonstore
# or
yarn add toonstore
```

### API Design
```javascript
const { ToonStore } = require('toonstore');

// Open database
const db = ToonStore.open('/path/to/db');

// Basic operations
const rowId = db.put(Buffer.from('users[1]{id,name}: 1,Alice'));
const data = db.get(rowId);
db.delete(rowId);

// Iteration
for (const [rowId, data] of db.scan()) {
    console.log(`${rowId}: ${data}`);
}

// Async API
await db.putAsync(data);
const result = await db.getAsync(rowId);

// Close
db.close();
```

### Features
- ✅ Sync API (PUT, GET, DELETE, SCAN)
- ✅ Async API (Promises)
- ✅ Iterator support
- ✅ TypeScript definitions (`.d.ts`)
- ✅ Error handling (JS exceptions)
- ✅ Stream support (Node.js Readable)

### Build & Distribution
```json
// package.json
{
  "name": "toonstore",
  "version": "0.1.0",
  "main": "index.node",
  "scripts": {
    "build": "neon build --release",
    "install": "neon build --release"
  },
  "neon": {
    "type": "library",
    "org": "toonstore"
  }
}
```

**Build**: `npm run build`  
**Publish**: `npm publish`

### Platforms
- ✅ Windows (x86_64)
- ✅ macOS (x86_64, ARM64)
- ✅ Linux (x86_64, ARM64)

Pre-built binaries for all Node.js LTS versions (16, 18, 20).

---

## 3. C API (FFI Layer)

### Core C Header (`toonstore.h`)

```c
#ifndef TOONSTORE_H
#define TOONSTORE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle
typedef struct ToonStore ToonStore;

// Open/close
ToonStore* toonstore_open(const char* path);
void toonstore_close(ToonStore* db);

// Basic operations
int64_t toonstore_put(ToonStore* db, const uint8_t* data, size_t len);
int toonstore_get(ToonStore* db, int64_t row_id, uint8_t** data, size_t* len);
int toonstore_delete(ToonStore* db, int64_t row_id);

// Scan iterator
typedef struct ToonStoreIter ToonStoreIter;
ToonStoreIter* toonstore_scan(ToonStore* db);
int toonstore_iter_next(ToonStoreIter* iter, int64_t* row_id, uint8_t** data, size_t* len);
void toonstore_iter_free(ToonStoreIter* iter);

// Error handling
const char* toonstore_last_error();
void toonstore_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif // TOONSTORE_H
```

### Rust FFI Implementation
```rust
// crates/toonstore-ffi/src/lib.rs

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use toonstoredb::ToonStore;

#[no_mangle]
pub extern "C" fn toonstore_open(path: *const c_char) -> *mut ToonStore {
    let path = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    match ToonStore::open(path) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn toonstore_close(db: *mut ToonStore) {
    if !db.is_null() {
        unsafe { Box::from_raw(db) };
    }
}

// ... more functions
```

---

## 4. Go Bindings

### Week 5+

**Package**: `github.com/toonstore/toonstore-go`  
**Technology**: cgo

### API Design
```go
package main

import "github.com/toonstore/toonstore-go"

func main() {
    // Open database
    db, err := toonstore.Open("/path/to/db")
    if err != nil {
        panic(err)
    }
    defer db.Close()

    // Basic operations
    rowID, _ := db.Put([]byte("users[1]{id,name}: 1,Alice"))
    data, _ := db.Get(rowID)
    db.Delete(rowID)

    // Iteration
    iter := db.Scan()
    for iter.Next() {
        rowID, data := iter.Value()
        fmt.Printf("%d: %s\n", rowID, data)
    }
}
```

---

## 5. Ruby Bindings

### Post-v0.1

**Gem**: `toonstore`  
**Technology**: Helix (Rust ↔ Ruby)

### API Design
```ruby
require 'toonstore'

# Open database
db = ToonStore.open('/path/to/db')

# Basic operations
row_id = db.put('users[1]{id,name}: 1,Alice')
data = db.get(row_id)
db.delete(row_id)

# Iteration
db.scan do |row_id, data|
  puts "#{row_id}: #{data}"
end

# Block syntax
ToonStore.open('/path/to/db') do |db|
  db.put('data')
end
```

---

## Performance Targets

### Python (PyO3)
- **FFI Overhead**: <10 µs per call
- **Throughput**: 30k-50k ops/sec
- **GIL**: Release during I/O operations

### Node.js (Neon)
- **FFI Overhead**: <15 µs per call
- **Throughput**: 25k-40k ops/sec
- **Async**: Non-blocking I/O

### C (Direct FFI)
- **FFI Overhead**: <5 µs per call
- **Throughput**: 80k-90k ops/sec (near-native)

### Go (cgo)
- **FFI Overhead**: <20 µs per call
- **Throughput**: 20k-30k ops/sec
- **Note**: cgo overhead

---

## Build Matrix (CI)

### Platforms × Languages

| Platform | Python | Node.js | C Header | Go |
|----------|--------|---------|----------|-----|
| Windows x64 | ✅ | ✅ | ✅ | ✅ |
| macOS x64 | ✅ | ✅ | ✅ | ✅ |
| macOS ARM64 | ✅ | ✅ | ✅ | ✅ |
| Linux x64 | ✅ | ✅ | ✅ | ✅ |
| Linux ARM64 | ✅ | ✅ | ✅ | ✅ |

### Python Wheel Matrix
- Python 3.8, 3.9, 3.10, 3.11, 3.12
- manylinux2014, macOS 11+, Windows Server 2019+

### Node.js Binary Matrix
- Node.js 16, 18, 20 (LTS)
- Electron support

---

## Week 4 Task Breakdown

### Python Bindings (Priority: P0)
- [ ] Setup PyO3 project structure (1h)
- [ ] Implement core FFI wrappers (3h)
- [ ] Add error handling (1h)
- [ ] Write Python tests (2h)
- [ ] Create type stubs (`.pyi`) (1h)
- [ ] Setup maturin build (1h)
- [ ] CI: Build wheels for all platforms (2h)
- [ ] Write documentation (2h)
- [ ] Publish to TestPyPI (0.5h)
- [ ] Publish to PyPI (0.5h)

**Total**: ~14 hours

### Node.js Bindings (Priority: P1 - Stretch)
- [ ] Setup Neon project structure (1h)
- [ ] Implement sync API (2h)
- [ ] Implement async API (2h)
- [ ] Add TypeScript definitions (1h)
- [ ] Write tests (Jest) (2h)
- [ ] Setup prebuild (1h)
- [ ] CI: Build binaries for all platforms (2h)
- [ ] Write documentation (2h)
- [ ] Publish to npm (0.5h)

**Total**: ~13.5 hours

### C Header (Priority: P1)
- [ ] Write `toonstore.h` (1h)
- [ ] Implement FFI exports (2h)
- [ ] Write C example (1h)
- [ ] Test with gcc/clang (1h)

**Total**: ~5 hours

---

## Distribution Strategy

### Python
1. **PyPI**: `pip install toonstore`
2. **Pre-built wheels**: All platforms
3. **Source distribution**: For custom builds
4. **Documentation**: ReadTheDocs

### Node.js
1. **npm**: `npm install toonstore`
2. **Pre-built binaries**: All platforms
3. **Fallback**: Build from source if no binary
4. **Documentation**: JSDoc → website

### Go
1. **Go modules**: `go get github.com/toonstore/toonstore-go`
2. **Requires**: libtoonstoredb.a
3. **Documentation**: pkg.go.dev

---

## Testing Strategy

### Integration Tests
```bash
# Test all bindings
./scripts/test-all-bindings.sh

# Output:
# ✓ Python bindings (pytest)
# ✓ Node.js bindings (jest)
# ✓ C API (unity)
# ✓ Go bindings (go test)
```

### Performance Tests
```bash
# Benchmark each language
./scripts/bench-all-bindings.sh

# Expected results:
# Python:  30-50k ops/sec
# Node.js: 25-40k ops/sec
# C:       80-90k ops/sec
# Go:      20-30k ops/sec
```

---

## Documentation

### Per-Language Docs
- **Python**: `docs/python/`
- **Node.js**: `docs/nodejs/`
- **C**: `docs/c/`
- **Go**: `docs/go/`

### Quickstart Examples
Each language gets:
- Installation guide
- Basic usage example
- API reference
- Performance tips
- Troubleshooting

---

## Release Checklist

### Week 4 (Python + npm)
- [ ] Python package builds on all platforms
- [ ] npm package builds on all platforms
- [ ] Tests pass for both
- [ ] Documentation published
- [ ] Packages published to test registries
- [ ] Final publish to PyPI + npm

---

## Post-v0.1 Roadmap

### v0.2 (Weeks 5-8)
- [ ] Go bindings
- [ ] Ruby bindings
- [ ] C++ wrapper

### v0.3 (Weeks 9-12)
- [ ] Java/JVM (JNI)
- [ ] C# / .NET (P/Invoke)
- [ ] Zig bindings

### v1.0 (3-6 months)
- [ ] Stable FFI ABI
- [ ] All languages supported
- [ ] Production-ready

---

## Questions?

Open an issue: https://github.com/yourusername/toonstore/issues

**Status**: This document is a living roadmap and will be updated as implementation progresses.
