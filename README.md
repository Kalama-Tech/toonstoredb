# ToonStore

⚠️ **NOT PRODUCTION READY** ⚠️

A high-performance embedded database using TOON (Token-Oriented Object Notation) format with built-in caching and Redis-compatible protocol.

## Status: Week 1 - Storage Engine

Currently implementing the core storage layer (`toonstoredb`).

## Features (Planned)

- 🗄️ **toonstoredb**: Embedded TOON-format database with memory-mapped storage
- ⚡ **tooncache**: Built-in LRU cache for hot data
- 🔌 **RESP Protocol**: Redis-compatible server (GET, SET, DEL, PING)
- 🐍 **Python Bindings**: `pip install toonstore`

## Quick Start

**Coming Soon - Week 1 in Progress**

```bash
# Install (not yet available)
cargo install toonstore

# Or build from source
git clone https://github.com/Kalama-Tech/toonstoredb
cd toonstore
cargo build --release
```

## Architecture

```
┌─────────────────────────────────────────┐
│         Application Layer               │
├─────────────────────────────────────────┤
│  tooncache (Week 2)                     │
│  - LRU eviction                         │
│  - Hot data in memory                   │
├─────────────────────────────────────────┤
│  toonstoredb (Week 1)                   │
│  - TOON format parser                   │
│  - Memory-mapped storage                │
│  - Single-threaded writer               │
│  - Unlimited readers                    │
└─────────────────────────────────────────┘
```

## Constraints (v0.1)

- **Max value size**: 1 MB
- **Max database size**: 1 GB
- **Crash safety**: fsync on close() only
- **Concurrency**: Single writer, unlimited readers
- **No WAL**: Append-only storage

## Roadmap

- [x] Week 1: Storage engine
- [ ] Week 2: Cache layer
- [ ] Week 3: RESP server + benchmarks
- [ ] Week 4: Python bindings + docs

## License

MIT License - See LICENSE file for details

## Contributing

Project is in active development. Contributions welcome after v0.1 ships (Week 4).

---

**Timeline**: 4 weeks (120 productive hours)  
**Target**: 50k ops/sec @ 50% cache hit, 1kB values  
**Kill Switch**: <30k ops/sec at Week 2 end → cut RESP server
