# Changelog

All notable changes to ToonStore will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Security Features**
  - Multi-user authentication system with BCrypt password hashing (cost 12)
  - Role-based access control (RBAC) with 3 permission levels: Admin, ReadWrite, ReadOnly
  - User management commands: USER CREATE, USER DELETE, USER LIST, USER SETPASS, USER WHOAMI
  - Per-command authorization based on user roles
  - Persistent user storage in users.json file
  - Session state tracking for authenticated connections
- **Security Hardening**
  - Buffer size limits: 512MB for bulk strings, 1M for array elements (prevents DoS attacks)
  - Connection limiting: 10,000 concurrent connections maximum (prevents connection flooding)
  - Path traversal prevention in RESTORE command with canonicalization
  - Input validation for all RESP protocol operations
- **Auto-Setup Scripts**
  - `start_toonstore.py` - Python script for one-command deployment with auto-generated credentials
  - `start_toonstore.ps1` - PowerShell script for Windows deployment
  - Automatic user creation with secure 24-character passwords
  - Connection string generation with format: `toonstore://username:password@host:port`
  - JSON export of connection info to `data/connection_info.json`
- **Documentation**
  - `COMPREHENSIVE_SECURITY_GUIDE.md` - 32KB complete security and authentication guide
  - `docs/UBUNTU_DEPLOYMENT.md` - 16KB Ubuntu/Linux production deployment guide
  - `docs/AUTHENTICATION.md` - Authentication setup guide
  - `docs/SECURITY.md` - Security best practices
  - `SECURITY_FIXES_CHANGELOG.md` - Technical security changelog
  - `SECURITY_REPORT.txt` - Visual security assessment
  - `SECURITY_TESTING.md` - Security testing guide
  - `QUICK_START.md` - Quick start guide with auto-setup
  - `AUTO_SETUP_README.md` - Auto-setup documentation
  - `FINAL_DELIVERY.md` - Complete delivery summary
  - `DOCKER_REPO_UPDATE.md` - Docker repository migration guide
- **Testing & Tools**
  - `test_vulnerabilities.py` - Comprehensive security vulnerability scanner
  - Automated security testing for 8 vulnerability types
  - `update-docker-repo.ps1` - Script to update Docker repository references
  - `deploy-ubuntu.sh` - Automated Ubuntu deployment script (documented in guide)
- **Production Features**
  - Systemd service configuration with security hardening
  - Docker Compose setup with health checks
  - Firewall configuration examples (UFW/iptables)
  - Monitoring and backup scripts
  - Health check and backup automation
  - Cron job examples for maintenance

### Changed
- **Docker Repository Migration**
  - Changed Docker Hub repository from `samso9th/toonstore` to `toonstore/toonstoredb`
  - Updated all build scripts (build-docker.ps1, build-docker.sh)
  - Updated docker-compose.yml
  - Updated GitHub Actions workflow (.github/workflows/docker.yml)
  - Updated README.md badge and examples
  - Updated all documentation references (~25+ files)
- **Authentication System**
  - Replaced single-password auth with multi-user system
  - AUTH command now accepts username and password: `AUTH username password`
  - Default admin account created on first start (admin/admin - must be changed)
  - Session-based authentication tracking
  - Role-based command filtering
- **Connection Strings**
  - Standardized format: `toonstore://username:password@host:port`
  - Port now included in all connection string examples
  - Auto-generated connection strings in setup scripts
- **Security Defaults**
  - Authentication now required by default in multi-user mode
  - Localhost-only binding (127.0.0.1) as secure default
  - Restrictive file permissions recommended (700 for data, 600 for users.json)
  - Dedicated non-root user (toonstore) for systemd service

### Fixed
- **Critical:** SET/GET operations now work correctly with string keys
  - Added key-to-row_id mapping layer in handler
  - Fixed GET command to look up keys properly
  - Fixed EXISTS, DEL commands to use key names instead of row IDs
  - Implemented KEYS command with pattern matching support (* and ? wildcards)
  - See [docs/BUG_FIX_SET_GET.md](docs/BUG_FIX_SET_GET.md) for details
- **Security Vulnerabilities (All Fixed)**
  - **HIGH:** Buffer DoS via large bulk string (512MB limit enforced)
  - **HIGH:** Buffer DoS via array bomb (1M element limit enforced)
  - **MEDIUM:** Path traversal in RESTORE command (validation + canonicalization)
  - **MEDIUM:** Connection flood attack (10K connection limit)
  - Fixed compiler warnings in user management module
- **DBSIZE** now reports count from key_map instead of cache length
- **FLUSHDB** now clears both key_map and cache
- **INFO** command now shows accurate key count

### Security
- **Security Rating: HIGH** ✅
  - 0 HIGH priority vulnerabilities
  - 0 MEDIUM priority vulnerabilities
  - 0 compiler warnings
- **Not Vulnerable** to MongoDB Mongoled (CVE-2025-14847)
  - No compression support in RESP protocol
  - Proper buffer validation before allocation
  - Safe memory handling
- **Compliance**
  - OWASP Top 10 considerations
  - CWE Top 25 mitigations
  - NIST Cybersecurity Framework alignment
  - PCI DSS considerations (encryption, access control, logging)
- **Audit Trail**
  - Comprehensive security event logging
  - Authentication success/failure tracking
  - Permission denial logging
  - Path traversal attempt logging
  - Connection limit events

### Deployment
- **Ubuntu/Linux Production Ready**
  - One-command deployment with systemd service
  - Automated user creation and password generation
  - Security hardening with systemd options
  - Firewall configuration guides
  - Monitoring and backup automation
- **Docker Deployment**
  - Updated to `toonstore/toonstoredb:latest`
  - Multi-architecture support (amd64, arm64)
  - Health checks integrated
  - Persistent volume configuration
  - Auto-generated connection strings

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
  - Published to Docker Hub
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
- **Deployment:** Deployment and operations changes
