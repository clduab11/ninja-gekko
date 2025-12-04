# Build stage
FROM rust:1.80-bookworm AS builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml ./core/
COPY database/Cargo.toml ./database/
COPY api/Cargo.toml ./api/
COPY crates/exchange-connectors/Cargo.toml ./crates/exchange-connectors/
COPY crates/arbitrage-engine/Cargo.toml ./crates/arbitrage-engine/
COPY crates/trading-core/Cargo.toml ./crates/trading-core/
COPY crates/neural-engine/Cargo.toml ./crates/neural-engine/
COPY crates/swarm-intelligence/Cargo.toml ./crates/swarm-intelligence/
COPY crates/mcp-client/Cargo.toml ./crates/mcp-client/
COPY crates/event-bus/Cargo.toml ./crates/event-bus/
COPY crates/data-pipeline/Cargo.toml ./crates/data-pipeline/
COPY crates/strategy-engine/Cargo.toml ./crates/strategy-engine/

# Create dummy source files to compile dependencies
RUN mkdir -p src core/src database/src api/src \
    crates/exchange-connectors/src \
    crates/arbitrage-engine/src \
    crates/trading-core/src \
    crates/neural-engine/src \
    crates/swarm-intelligence/src \
    crates/mcp-client/src \
    crates/event-bus/src \
    crates/data-pipeline/src \
    crates/strategy-engine/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "pub fn lib() {}" > src/lib.rs \
    && for d in core database api crates/*/; do echo "pub fn lib() {}" > "${d}src/lib.rs"; done

# Disable sqlx compile-time checks
ENV SQLX_OFFLINE=true

# Build dependencies only
RUN cargo build --release 2>/dev/null || true

# Remove dummy source files
RUN rm -rf src core/src database/src api/src crates/*/src

# Copy actual source code
COPY . .

# Build the release binary
RUN cargo build --release --bin ninja-gekko

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash ninja-gekko

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/ninja-gekko /app/ninja-gekko

# Copy configuration files
COPY config/arbitrage.toml /etc/ninja-gekko/arbitrage.toml

# Create log directory
RUN mkdir -p /var/log/ninja-gekko && chown -R ninja-gekko:ninja-gekko /var/log/ninja-gekko

# Set ownership
RUN chown -R ninja-gekko:ninja-gekko /app

# Switch to non-root user
USER ninja-gekko

# Expose ports
EXPOSE 8080 8787 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info,ninja_gekko=debug
ENV RUST_BACKTRACE=1

# Run the binary
ENTRYPOINT ["/app/ninja-gekko"]
CMD ["--mode", "swarm", "--config", "/etc/ninja-gekko/arbitrage.toml"]
