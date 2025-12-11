# Build stage
FROM rust:1.83-bookworm AS builder

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
COPY crates/mcp-server-trade/Cargo.toml ./crates/mcp-server-trade/

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
    && echo "pub fn lib() {}" > core/src/lib.rs \
    && echo "pub fn lib() {}" > database/src/lib.rs \
    && echo "pub fn lib() {}" > api/src/lib.rs \
    && echo "pub fn lib() {}" > crates/exchange-connectors/src/lib.rs \
    && echo "pub fn lib() {}" > crates/arbitrage-engine/src/lib.rs \
    && echo "pub fn lib() {}" > crates/trading-core/src/lib.rs \
    && echo "pub fn lib() {}" > crates/neural-engine/src/lib.rs \
    && echo "pub fn lib() {}" > crates/swarm-intelligence/src/lib.rs \
    && echo "pub fn lib() {}" > crates/mcp-client/src/lib.rs \
    && echo "pub fn lib() {}" > crates/event-bus/src/lib.rs \
    && echo "pub fn lib() {}" > crates/data-pipeline/src/lib.rs \
    && echo "pub fn lib() {}" > crates/strategy-engine/src/lib.rs \
    && mkdir -p crates/mcp-server-trade/src && echo "pub fn lib() {}" > crates/mcp-server-trade/src/lib.rs \
    && mkdir -p crates/event-bus/benches && echo "fn main() {}" > crates/event-bus/benches/dispatcher.rs \
    && mkdir -p crates/data-pipeline/benches && echo "fn main() {}" > crates/data-pipeline/benches/normalizer.rs \
    && mkdir -p crates/strategy-engine/benches && echo "fn main() {}" > crates/strategy-engine/benches/strategy_eval.rs

# Disable sqlx compile-time checks
ENV SQLX_OFFLINE=true

# Build dependencies only
RUN cargo build --release

# Remove dummy source files
RUN rm -rf src core/src database/src api/src crates/*/src

# Copy actual source code
COPY . .

# Update mtimes to ensure Cargo rebuilds (Critical for Docker caching behavior)
RUN find . -name "*.rs" -exec touch {} +

# Build the release binary
RUN cargo build --release --bin ninja-gekko

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash ninja-gekko

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/ninja-gekko /app/ninja-gekko

# Copy configuration files
COPY config/arbitrage.toml /etc/ninja-gekko/arbitrage.toml

# Copy migrations
COPY database/migrations /app/database/migrations

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
    CMD curl -f http://localhost:8787/health || exit 1

# Set environment variables
ENV RUST_LOG=info,ninja_gekko=debug
ENV RUST_BACKTRACE=1

# Run the binary
ENTRYPOINT ["/app/ninja-gekko"]
CMD ["--mode", "swarm", "--config", "/etc/ninja-gekko/arbitrage.toml"]
