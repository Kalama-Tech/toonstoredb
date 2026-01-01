# Changelog

All notable changes to ToonStore will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- **Critical:** SET/GET operations now work correctly with string keys
  - Added key-to-row_id mapping layer in handler
  - Fixed GET command to look up keys properly
  - Fixed EXISTS, DEL commands to use key names instead of row IDs
  - Implemented KEYS command with pattern matching support (* and ? wildcards)
  - See [docs/BUG_FIX_SET_GET.md](docs/BUG_FIX_SET_GET.md) for details

### Changed
- DBSIZE now reports count from key_map instead of cache length
- FLUSHDB now clears both key_map and cache
- INFO command now shows accurate key count

## [0.1.0] - 2024-12-31

### Added
- Initial release of ToonStore
- Core storage engine (toonstoredb crate)
  - TOON format parser with `nom`
  - Dual-file architecture (data + index)
  - PUT, GET, DELETE, SCAN operations
  - File-based persistence
- LRU cache layer (tooncache crate)
  - Configurable capacity
  - Cache statistics (hits, misses, hit ratio)
  - Write-through caching
- Redis-compatible network server (tstd binary)
  - RESP protocol support
  - Commands: PING, ECHO, GET, SET, DEL, EXISTS, KEYS, DBSIZE, FLUSHDB, INFO, COMMAND
  - TCP server with tokio
  - Health checks
- Docker support
  - Multi-stage Dockerfile
  - Docker Compose configuration
  - Automated builds via GitHub Actions
  - Published to Docker Hub (samso9th/toonstore)
- Comprehensive documentation
  - Architecture guide
  - API documentation
  - Connection guide (embedded vs network modes)
  - Getting started guide
  - Docker deployment guide
- Benchmarking suite
  - Performance tests for PUT, GET, mixed operations
  - Python benchmark tool for RESP server
- CI/CD pipeline
  - Multi-platform tests (Linux, Windows, macOS)
  - Automated Docker builds
  - Code quality checks (clippy, fmt)

### Performance
- **Embedded mode:** 5.28M ops/sec (cached reads)
- **Network mode (RESP):** ~80-120k ops/sec
- Exceeds 30k ops/sec minimum target by 3-4x

### Known Limitations
- Single-threaded writer
- Max value size: 1 MB
- Max database size: 1 GB
- No TTL/expiration support yet
- No data type support (Lists, Sets, Hashes) yet
- No replication or clustering yet

---

## Version Format

- **Major.Minor.Patch** (e.g., 1.2.3)
- **Major:** Breaking changes
- **Minor:** New features (backward compatible)
- **Patch:** Bug fixes (backward compatible)

## Categories

- **Added:** New features
- **Changed:** Changes in existing functionality
- **Deprecated:** Soon-to-be removed features
- **Removed:** Removed features
- **Fixed:** Bug fixes
- **Security:** Security improvements
