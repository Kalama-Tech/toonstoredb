# ToonStore Docker Image
# Multi-stage build for minimal image size

# Stage 1: Builder
FROM rust:1.83-slim AS builder

WORKDIR /build

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build release binary
RUN cargo build --release --bin tstd

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -m -u 1000 toonstore

# Copy binary from builder
COPY --from=builder /build/target/release/tstd /usr/local/bin/tstd

# Set up data directory
RUN mkdir -p /data && chown toonstore:toonstore /data
VOLUME /data

# Switch to non-root user
USER toonstore

# Expose RESP port
EXPOSE 6379

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD tstd --health || exit 1

# Run server
ENTRYPOINT ["tstd"]
CMD ["--bind", "0.0.0.0:6379", "--data", "/data"]
