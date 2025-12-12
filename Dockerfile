# Multi-stage Dockerfile with GPU support for Ninja Gekko
# Supports CPU-only and CUDA builds via BUILD_TARGET arg

# ============================================
# Stage: builder-base (Common Rust builder)
# ============================================
FROM rust:1.83-bookworm AS builder-base

WORKDIR /app

# Install common build dependencies
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

# ============================================
# Stage: builder-cuda (CUDA-enabled builder)
# ============================================
FROM nvidia/cuda:12.2.0-devel-ubuntu22.04 AS builder-cuda

WORKDIR /app

# Install Rust and build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:${PATH}"
ENV SQLX_OFFLINE=true

# Copy everything from builder-base or copy manifests again
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

# Create dummy source files
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

# Build dependencies with CUDA features
ARG FEATURES="cuda"
RUN cargo build --release --features "${FEATURES}"

# Remove dummy files
RUN rm -rf src core/src database/src api/src crates/*/src

# Copy actual source
COPY . .

# Update mtimes and build with CUDA features
RUN find . -name "*.rs" -exec touch {} + \
    && cargo build --release --features "${FEATURES}" --bin ninja-gekko

# ============================================
# Stage: builder-cpu (CPU-only builder)
# ============================================
FROM builder-base AS builder-cpu

# Build dependencies (CPU-only, no features)
RUN cargo build --release

# Remove dummy source files
RUN rm -rf src core/src database/src api/src crates/*/src

# Copy actual source code
COPY . .

# Update mtimes to ensure Cargo rebuilds
RUN find . -name "*.rs" -exec touch {} +

# Build the release binary (CPU-only)
RUN cargo build --release --bin ninja-gekko

# ============================================
# Stage: runtime (Conditional runtime)
# ============================================
ARG BUILD_TARGET=cpu

# CPU runtime
FROM debian:bookworm-slim AS runtime-cpu
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# CUDA runtime
FROM nvidia/cuda:12.2.0-runtime-ubuntu22.04 AS runtime-cuda
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Select runtime based on BUILD_TARGET
FROM runtime-${BUILD_TARGET} AS runtime

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash ninja-gekko

WORKDIR /app

# Copy binary from appropriate builder
ARG BUILD_TARGET=cpu
COPY --from=builder-${BUILD_TARGET} /app/target/release/ninja-gekko /app/ninja-gekko

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
