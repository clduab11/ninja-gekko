---
name: CI/CD Pipeline & Production Hardening
about: Set up automation, testing, deployment, and production-grade reliability
title: "[MILESTONE 3] Implement CI/CD Pipeline & Production Hardening"
labels: infrastructure, devops, testing, production, ai-agent
assignees: ''
---

## 🤖 GitHub Copilot Context Prompt

**Copy this into your Copilot chat before starting implementation:**

```
You are implementing MILESTONE 3: CI/CD Pipeline & Production Hardening for a Rust trading system. Follow November 2025 DevOps standards: GitHub Actions with caching, multi-stage Docker builds with distroless base, Kubernetes manifests with HPA/liveness/readiness probes, Prometheus metrics with Grafana dashboards, structured logging with tracing. Create GitHub Actions workflows: CI (fmt/clippy/test with Postgres/Redis services), security audit (cargo audit/deny), benchmarks (criterion with regression detection), Docker build/push to GHCR. Implement graceful shutdown: SIGTERM handler, cancel open orders, flush event queues, close connections within 30s timeout. Add health endpoints: /health/live (process alive), /health/ready (dependencies healthy). Prometheus metrics: counters for orders/fills, histograms for latency (p50/p95/p99), gauges for positions/balance/PnL. Activate circuit breaker in database layer: trip on 5 failures, half-open after timeout, track state. All async shutdown with tokio::select and proper signal handling. Docker: use rust:slim builder, debian:bookworm-slim runtime, non-root user, health checks. K8s: resource limits, rolling updates, config via ConfigMap, secrets via Secret. Monitoring: Grafana dashboards for trading/performance/system metrics, Prometheus alerts for failures/latency/loss limits. Testing: verify graceful shutdown, health endpoints return correct status, metrics export correctly. Code quality: clippy clean, formatted, comprehensive error handling, security-first (no secrets in logs). This enables production deployment—implement with reliability and observability as priorities.
```

---


## Overview

**Milestone**: CI/CD & Production Readiness
**Priority**: HIGH - Enables safe iteration and deployment
**Implementation Scope**: Production deployment readiness - automation & reliability
**Dependencies**: Issues #1 and #2 (for complete test coverage)

## Problem Statement

The repository currently has no CI/CD automation, limited test coverage, and lacks production-grade reliability features. This creates risk when deploying to production and slows development velocity.

**Current State**:
- ❌ No GitHub Actions workflows
- ❌ No automated testing on commits/PRs
- ❌ No Docker containers or deployment automation
- ❌ No monitoring or observability setup
- ❌ Graceful shutdown not implemented
- ❌ Circuit breakers defined but not activated
- ❌ No health check endpoints
- ❌ No metrics collection

**Target State**: Full CI/CD pipeline with automated testing, Docker deployment, health checks, metrics, graceful shutdown, and production monitoring.

---

## Implementation Checklist

### Phase 1: GitHub Actions CI/CD

**Files to Create**: `.github/workflows/*.yml`

#### 1.1 Continuous Integration Workflow

- [ ] **Create `.github/workflows/ci.yml`**
  ```yaml
  name: Continuous Integration

  on:
    push:
      branches: [ main, develop ]
    pull_request:
      branches: [ main, develop ]

  env:
    CARGO_TERM_COLOR: always
    RUST_BACKTRACE: 1

  jobs:
    check:
      name: Check
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable

        - name: Cache cargo registry
          uses: actions/cache@v4
          with:
            path: |
              ~/.cargo/registry
              ~/.cargo/git
              target
            key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

        - name: Check compilation
          run: cargo check --all-features --workspace

    fmt:
      name: Rustfmt
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable
          with:
            components: rustfmt

        - name: Check formatting
          run: cargo fmt --all -- --check

    clippy:
      name: Clippy
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable
          with:
            components: clippy

        - name: Run clippy
          run: cargo clippy --all-targets --all-features -- -D warnings

    test:
      name: Test Suite
      runs-on: ubuntu-latest
      services:
        postgres:
          image: postgres:15
          env:
            POSTGRES_PASSWORD: postgres
            POSTGRES_DB: ninja_gekko_test
          options: >-
            --health-cmd pg_isready
            --health-interval 10s
            --health-timeout 5s
            --health-retries 5
          ports:
            - 5432:5432

        redis:
          image: redis:7
          options: >-
            --health-cmd "redis-cli ping"
            --health-interval 10s
            --health-timeout 5s
            --health-retries 5
          ports:
            - 6379:6379

      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable

        - name: Cache cargo
          uses: actions/cache@v4
          with:
            path: |
              ~/.cargo/registry
              ~/.cargo/git
              target
            key: ${{ runner.os }}-cargo-test-${{ hashFiles('**/Cargo.lock') }}

        - name: Run tests
          run: cargo test --all-features --workspace
          env:
            DATABASE_URL: postgres://postgres:postgres@localhost:5432/ninja_gekko_test
            REDIS_URL: redis://localhost:6379

        - name: Run doc tests
          run: cargo test --doc --workspace

    coverage:
      name: Code Coverage
      runs-on: ubuntu-latest
      services:
        postgres:
          image: postgres:15
          env:
            POSTGRES_PASSWORD: postgres
            POSTGRES_DB: ninja_gekko_test
          options: >-
            --health-cmd pg_isready
            --health-interval 10s
            --health-timeout 5s
            --health-retries 5
          ports:
            - 5432:5432

        redis:
          image: redis:7
          options: >-
            --health-cmd "redis-cli ping"
            --health-interval 10s
            --health-timeout 5s
            --health-retries 5
          ports:
            - 6379:6379

      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable

        - name: Install cargo-tarpaulin
          run: cargo install cargo-tarpaulin

        - name: Generate coverage
          run: cargo tarpaulin --all-features --workspace --out xml
          env:
            DATABASE_URL: postgres://postgres:postgres@localhost:5432/ninja_gekko_test
            REDIS_URL: redis://localhost:6379

        - name: Upload to codecov.io
          uses: codecov/codecov-action@v4
          with:
            files: ./cobertura.xml
            fail_ci_if_error: true
  ```

#### 1.2 Security Scanning Workflow

- [ ] **Create `.github/workflows/security.yml`**
  ```yaml
  name: Security Audit

  on:
    push:
      branches: [ main, develop ]
    pull_request:
      branches: [ main, develop ]
    schedule:
      - cron: '0 0 * * 0'  # Weekly on Sunday

  jobs:
    audit:
      name: Cargo Audit
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable

        - name: Install cargo-audit
          run: cargo install cargo-audit

        - name: Run security audit
          run: cargo audit

    deny:
      name: Cargo Deny
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable

        - name: Install cargo-deny
          run: cargo install cargo-deny

        - name: Check licenses
          run: cargo deny check licenses

        - name: Check advisories
          run: cargo deny check advisories

        - name: Check bans
          run: cargo deny check bans

    secret-scan:
      name: Secret Scanning
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
          with:
            fetch-depth: 0

        - name: TruffleHog OSS
          uses: trufflesecurity/trufflehog@main
          with:
            path: ./
            base: ${{ github.event.repository.default_branch }}
            head: HEAD
  ```

#### 1.3 Benchmark Workflow

- [ ] **Create `.github/workflows/benchmark.yml`**
  ```yaml
  name: Benchmarks

  on:
    push:
      branches: [ main ]
    pull_request:
      branches: [ main ]

  jobs:
    benchmark:
      name: Run Benchmarks
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable

        - name: Cache cargo
          uses: actions/cache@v4
          with:
            path: |
              ~/.cargo/registry
              ~/.cargo/git
              target
            key: ${{ runner.os }}-cargo-bench-${{ hashFiles('**/Cargo.lock') }}

        - name: Run benchmarks
          run: cargo bench --workspace -- --output-format bencher | tee output.txt

        - name: Store benchmark result
          uses: benchmark-action/github-action-benchmark@v1
          with:
            tool: 'cargo'
            output-file-path: output.txt
            github-token: ${{ secrets.GITHUB_TOKEN }}
            auto-push: true
            alert-threshold: '150%'
            comment-on-alert: true
            fail-on-alert: false
  ```

#### 1.4 Build & Release Workflow

- [ ] **Create `.github/workflows/build.yml`**
  ```yaml
  name: Build Release

  on:
    push:
      tags:
        - 'v*'

  jobs:
    build:
      name: Build Release Binary
      runs-on: ${{ matrix.os }}
      strategy:
        matrix:
          os: [ubuntu-latest, macos-latest]
          include:
            - os: ubuntu-latest
              target: x86_64-unknown-linux-gnu
            - os: macos-latest
              target: aarch64-apple-darwin

      steps:
        - uses: actions/checkout@v4

        - name: Install Rust toolchain
          uses: dtolnay/rust-toolchain@stable
          with:
            targets: ${{ matrix.target }}

        - name: Build release binary
          run: cargo build --release --target ${{ matrix.target }}

        - name: Upload artifact
          uses: actions/upload-artifact@v4
          with:
            name: ninja-gekko-${{ matrix.target }}
            path: target/${{ matrix.target }}/release/ninja-gekko
  ```

#### 1.5 Docker Build & Push Workflow

- [ ] **Create `.github/workflows/docker.yml`**
  ```yaml
  name: Docker Build & Push

  on:
    push:
      branches: [ main ]
      tags:
        - 'v*'
    pull_request:
      branches: [ main ]

  env:
    REGISTRY: ghcr.io
    IMAGE_NAME: ${{ github.repository }}

  jobs:
    build-and-push:
      runs-on: ubuntu-latest
      permissions:
        contents: read
        packages: write

      steps:
        - name: Checkout repository
          uses: actions/checkout@v4

        - name: Log in to Container Registry
          uses: docker/login-action@v3
          with:
            registry: ${{ env.REGISTRY }}
            username: ${{ github.actor }}
            password: ${{ secrets.GITHUB_TOKEN }}

        - name: Extract metadata
          id: meta
          uses: docker/metadata-action@v5
          with:
            images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
            tags: |
              type=ref,event=branch
              type=ref,event=pr
              type=semver,pattern={{version}}
              type=semver,pattern={{major}}.{{minor}}
              type=sha

        - name: Build and push Docker image
          uses: docker/build-push-action@v5
          with:
            context: .
            push: true
            tags: ${{ steps.meta.outputs.tags }}
            labels: ${{ steps.meta.outputs.labels }}
            cache-from: type=gha
            cache-to: type=gha,mode=max
  ```

---

### Phase 2: Docker & Containerization

**Files to Create**: `Dockerfile`, `docker-compose.yml`, `.dockerignore`

#### 2.1 Production Dockerfile

- [ ] **Create `Dockerfile`**
  ```dockerfile
  # Build stage
  FROM rust:1.91-slim as builder

  WORKDIR /app

  # Install build dependencies
  RUN apt-get update && apt-get install -y \
      pkg-config \
      libssl-dev \
      && rm -rf /var/lib/apt/lists/*

  # Copy manifests
  COPY Cargo.toml Cargo.lock ./
  COPY crates ./crates
  COPY core ./core
  COPY database ./database
  COPY api ./api
  COPY src ./src

  # Build for release
  RUN cargo build --release --bin ninja-gekko

  # Runtime stage
  FROM debian:bookworm-slim

  # Install runtime dependencies
  RUN apt-get update && apt-get install -y \
      ca-certificates \
      libssl3 \
      && rm -rf /var/lib/apt/lists/*

  # Create non-root user
  RUN useradd -m -u 1000 gekko && \
      mkdir -p /app/config /app/data && \
      chown -R gekko:gekko /app

  WORKDIR /app

  # Copy binary from builder
  COPY --from=builder /app/target/release/ninja-gekko /app/ninja-gekko

  # Copy configuration
  COPY config /app/config

  # Switch to non-root user
  USER gekko

  # Health check
  HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/ninja-gekko", "health"]

  # Expose ports
  EXPOSE 8787

  # Run the binary
  ENTRYPOINT ["/app/ninja-gekko"]
  CMD ["--config", "/app/config/arbitrage.toml"]
  ```

- [ ] **Create `.dockerignore`**
  ```
  target/
  .git/
  .github/
  node_modules/
  frontend/chat-ui/node_modules/
  *.log
  .env
  .env.local
  .DS_Store
  ```

#### 2.2 Docker Compose for Development

- [ ] **Create `docker-compose.yml`**
  ```yaml
  version: '3.8'

  services:
    trading-engine:
      build:
        context: .
        dockerfile: Dockerfile
      container_name: ninja-gekko-engine
      environment:
        - DATABASE_URL=postgres://postgres:postgres@postgres:5432/ninja_gekko
        - REDIS_URL=redis://redis:6379
        - RUST_LOG=info,ninja_gekko=debug
      env_file:
        - .env
      ports:
        - "8787:8787"
      depends_on:
        postgres:
          condition: service_healthy
        redis:
          condition: service_healthy
      restart: unless-stopped
      volumes:
        - ./config:/app/config:ro
        - gekko-data:/app/data
      networks:
        - gekko-network

    postgres:
      image: postgres:15-alpine
      container_name: ninja-gekko-postgres
      environment:
        - POSTGRES_USER=postgres
        - POSTGRES_PASSWORD=postgres
        - POSTGRES_DB=ninja_gekko
      ports:
        - "5432:5432"
      volumes:
        - postgres-data:/var/lib/postgresql/data
        - ./database/migrations:/docker-entrypoint-initdb.d:ro
      healthcheck:
        test: ["CMD-SHELL", "pg_isready -U postgres"]
        interval: 5s
        timeout: 5s
        retries: 5
      networks:
        - gekko-network

    redis:
      image: redis:7-alpine
      container_name: ninja-gekko-redis
      command: redis-server --appendonly yes
      ports:
        - "6379:6379"
      volumes:
        - redis-data:/data
      healthcheck:
        test: ["CMD", "redis-cli", "ping"]
        interval: 5s
        timeout: 3s
        retries: 5
      networks:
        - gekko-network

    prometheus:
      image: prom/prometheus:latest
      container_name: ninja-gekko-prometheus
      command:
        - '--config.file=/etc/prometheus/prometheus.yml'
        - '--storage.tsdb.path=/prometheus'
      ports:
        - "9090:9090"
      volumes:
        - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
        - prometheus-data:/prometheus
      networks:
        - gekko-network

    grafana:
      image: grafana/grafana:latest
      container_name: ninja-gekko-grafana
      environment:
        - GF_SECURITY_ADMIN_PASSWORD=admin
        - GF_USERS_ALLOW_SIGN_UP=false
      ports:
        - "3000:3000"
      volumes:
        - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
        - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
        - grafana-data:/var/lib/grafana
      depends_on:
        - prometheus
      networks:
        - gekko-network

  volumes:
    postgres-data:
    redis-data:
    prometheus-data:
    grafana-data:
    gekko-data:

  networks:
    gekko-network:
      driver: bridge
  ```

#### 2.3 Production Docker Compose

- [ ] **Create `docker-compose.prod.yml`**
  ```yaml
  version: '3.8'

  services:
    trading-engine:
      image: ghcr.io/your-org/ninja-gekko:latest
      container_name: ninja-gekko-engine-prod
      environment:
        - DATABASE_URL=${DATABASE_URL}
        - REDIS_URL=${REDIS_URL}
        - RUST_LOG=info
      env_file:
        - .env.production
      ports:
        - "8787:8787"
      restart: always
      deploy:
        resources:
          limits:
            cpus: '2'
            memory: 4G
          reservations:
            cpus: '1'
            memory: 2G
      healthcheck:
        test: ["CMD", "/app/ninja-gekko", "health"]
        interval: 30s
        timeout: 10s
        retries: 3
        start_period: 40s
      logging:
        driver: "json-file"
        options:
          max-size: "10m"
          max-file: "3"
      networks:
        - gekko-prod-network

  networks:
    gekko-prod-network:
      driver: bridge
  ```

---

### Phase 3: Production Hardening

#### 3.1 Graceful Shutdown

**Files to Modify**: `src/main.rs`

- [ ] **Implement signal handling**
  ```rust
  use tokio::signal;
  use std::sync::Arc;
  use tokio::sync::Notify;

  async fn shutdown_signal(shutdown_notify: Arc<Notify>) {
      let ctrl_c = async {
          signal::ctrl_c()
              .await
              .expect("failed to install Ctrl+C handler");
      };

      #[cfg(unix)]
      let terminate = async {
          signal::unix::signal(signal::unix::SignalKind::terminate())
              .expect("failed to install SIGTERM handler")
              .recv()
              .await;
      };

      #[cfg(not(unix))]
      let terminate = std::future::pending::<()>();

      tokio::select! {
          _ = ctrl_c => {
              tracing::info!("Received Ctrl+C signal");
          },
          _ = terminate => {
              tracing::info!("Received SIGTERM signal");
          },
      }

      shutdown_notify.notify_waiters();
  }

  #[tokio::main]
  async fn main() -> Result<()> {
      // ... initialization ...

      let shutdown_notify = Arc::new(Notify::new());

      // Spawn shutdown signal handler
      let shutdown_notify_clone = shutdown_notify.clone();
      tokio::spawn(async move {
          shutdown_signal(shutdown_notify_clone).await;
      });

      // Run main application
      let app_handle = tokio::spawn(async move {
          run_application(shutdown_notify.clone()).await
      });

      // Wait for shutdown signal
      shutdown_notify.notified().await;

      tracing::info!("Initiating graceful shutdown...");

      // Give application time to clean up (30 seconds max)
      tokio::select! {
          _ = app_handle => {
              tracing::info!("Application shutdown complete");
          },
          _ = tokio::time::sleep(Duration::from_secs(30)) => {
              tracing::warn!("Shutdown timeout exceeded, forcing exit");
          },
      }

      Ok(())
  }
  ```

- [ ] **Implement graceful shutdown in NinjaGekko**
  ```rust
  // In src/core.rs
  impl NinjaGekko {
      pub async fn shutdown(&mut self) -> Result<()> {
          tracing::info!("Starting graceful shutdown sequence");

          // Step 1: Stop accepting new opportunities
          self.pause_trading().await?;

          // Step 2: Cancel all open orders
          tracing::info!("Cancelling all open orders...");
          self.cancel_all_orders().await?;

          // Step 3: Wait for in-flight requests to complete
          tracing::info!("Waiting for in-flight requests (max 10s)...");
          tokio::time::sleep(Duration::from_secs(10)).await;

          // Step 4: Flush event queues
          tracing::info!("Flushing event queues...");
          self.event_bus.flush().await?;

          // Step 5: Close database connections
          tracing::info!("Closing database connections...");
          self.db.close().await?;

          // Step 6: Close exchange connections
          tracing::info!("Closing exchange connections...");
          for (name, connector) in &mut self.exchange_connectors {
              tracing::debug!("Closing connection to {}", name);
              connector.disconnect().await?;
          }

          tracing::info!("Graceful shutdown complete");
          Ok(())
      }

      async fn cancel_all_orders(&mut self) -> Result<()> {
          // Get all open orders from database
          let open_orders = self.db.get_open_orders().await?;

          for order in open_orders {
              match self.cancel_order(&order.exchange, &order.id).await {
                  Ok(_) => tracing::info!("Cancelled order {}", order.id),
                  Err(e) => tracing::error!("Failed to cancel order {}: {}", order.id, e),
              }
          }

          Ok(())
      }
  }
  ```

#### 3.2 Health Check Endpoints

**Files to Modify**: `src/web.rs`

- [ ] **Implement health check endpoints**
  ```rust
  use axum::{
      routing::get,
      Json,
      http::StatusCode,
  };
  use serde::Serialize;

  #[derive(Serialize)]
  pub struct HealthResponse {
      status: String,
      version: String,
      uptime_seconds: u64,
  }

  #[derive(Serialize)]
  pub struct ReadinessResponse {
      ready: bool,
      checks: HealthChecks,
  }

  #[derive(Serialize)]
  pub struct HealthChecks {
      database: bool,
      redis: bool,
      exchanges: ExchangeHealthStatus,
  }

  #[derive(Serialize)]
  pub struct ExchangeHealthStatus {
      coinbase: bool,
      binance_us: bool,
      oanda: bool,
  }

  async fn health_check(
      State(state): State<Arc<AppState>>,
  ) -> (StatusCode, Json<HealthResponse>) {
      let uptime = state.start_time.elapsed().as_secs();

      (StatusCode::OK, Json(HealthResponse {
          status: "ok".to_string(),
          version: env!("CARGO_PKG_VERSION").to_string(),
          uptime_seconds: uptime,
      }))
  }

  async fn readiness_check(
      State(state): State<Arc<AppState>>,
  ) -> (StatusCode, Json<ReadinessResponse>) {
      // Check database connectivity
      let db_healthy = state.db.health_check().await.is_ok();

      // Check Redis connectivity
      let redis_healthy = state.cache.health_check().await.is_ok();

      // Check exchange connectivity
      let exchanges = check_exchange_health(&state).await;

      let all_healthy = db_healthy && redis_healthy &&
          exchanges.coinbase && exchanges.binance_us && exchanges.oanda;

      let status_code = if all_healthy {
          StatusCode::OK
      } else {
          StatusCode::SERVICE_UNAVAILABLE
      };

      (status_code, Json(ReadinessResponse {
          ready: all_healthy,
          checks: HealthChecks {
              database: db_healthy,
              redis: redis_healthy,
              exchanges,
          },
      }))
  }

  async fn check_exchange_health(state: &AppState) -> ExchangeHealthStatus {
      let coinbase = state.exchange_manager
          .check_health("coinbase")
          .await
          .unwrap_or(false);

      let binance_us = state.exchange_manager
          .check_health("binance_us")
          .await
          .unwrap_or(false);

      let oanda = state.exchange_manager
          .check_health("oanda")
          .await
          .unwrap_or(false);

      ExchangeHealthStatus {
          coinbase,
          binance_us,
          oanda,
      }
  }

  // Add to router
  pub fn health_routes() -> Router {
      Router::new()
          .route("/health/live", get(health_check))
          .route("/health/ready", get(readiness_check))
  }
  ```

#### 3.3 Prometheus Metrics

**Files to Modify**: `src/web.rs`, create `src/metrics.rs`

- [ ] **Create `src/metrics.rs`**
  ```rust
  use prometheus::{
      IntCounter, IntCounterVec, Histogram, HistogramVec, IntGauge,
      register_int_counter, register_int_counter_vec,
      register_histogram, register_histogram_vec, register_int_gauge,
      Opts, HistogramOpts,
  };
  use lazy_static::lazy_static;

  lazy_static! {
      // Counters
      pub static ref ORDERS_PLACED_TOTAL: IntCounterVec = register_int_counter_vec!(
          "ninja_gekko_orders_placed_total",
          "Total number of orders placed",
          &["exchange", "side", "type"]
      ).unwrap();

      pub static ref ORDERS_FILLED_TOTAL: IntCounterVec = register_int_counter_vec!(
          "ninja_gekko_orders_filled_total",
          "Total number of orders filled",
          &["exchange", "side"]
      ).unwrap();

      pub static ref ORDERS_CANCELLED_TOTAL: IntCounterVec = register_int_counter_vec!(
          "ninja_gekko_orders_cancelled_total",
          "Total number of orders cancelled",
          &["exchange", "reason"]
      ).unwrap();

      pub static ref ARBITRAGE_OPPORTUNITIES_DETECTED: IntCounter = register_int_counter!(
          "ninja_gekko_arbitrage_opportunities_detected_total",
          "Total number of arbitrage opportunities detected"
      ).unwrap();

      pub static ref ARBITRAGE_OPPORTUNITIES_EXECUTED: IntCounter = register_int_counter!(
          "ninja_gekko_arbitrage_opportunities_executed_total",
          "Total number of arbitrage opportunities executed"
      ).unwrap();

      // Histograms
      pub static ref ORDER_EXECUTION_DURATION: HistogramVec = register_histogram_vec!(
          HistogramOpts::new(
              "ninja_gekko_order_execution_duration_seconds",
              "Order execution duration in seconds"
          ).buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
          &["exchange"]
      ).unwrap();

      pub static ref OPPORTUNITY_DETECTION_DURATION: Histogram = register_histogram!(
          HistogramOpts::new(
              "ninja_gekko_opportunity_detection_duration_seconds",
              "Opportunity detection duration in seconds"
          ).buckets(vec![0.01, 0.025, 0.05, 0.1, 0.25, 0.5])
      ).unwrap();

      pub static ref API_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
          HistogramOpts::new(
              "ninja_gekko_api_request_duration_seconds",
              "API request duration in seconds"
          ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
          &["endpoint", "method"]
      ).unwrap();

      // Gauges
      pub static ref ACTIVE_POSITIONS: IntGauge = register_int_gauge!(
          "ninja_gekko_active_positions",
          "Number of currently active positions"
      ).unwrap();

      pub static ref ACCOUNT_BALANCE_USD: IntGauge = register_int_gauge!(
          "ninja_gekko_account_balance_usd",
          "Total account balance in USD"
      ).unwrap();

      pub static ref DAILY_PNL_USD: IntGauge = register_int_gauge!(
          "ninja_gekko_daily_pnl_usd",
          "Daily profit/loss in USD"
      ).unwrap();

      pub static ref EXCHANGE_CONNECTED: IntCounterVec = register_int_counter_vec!(
          "ninja_gekko_exchange_connected",
          "Exchange connection status (1 = connected, 0 = disconnected)",
          &["exchange"]
      ).unwrap();
  }

  pub fn init_metrics() {
      // Initialize all metrics
      lazy_static::initialize(&ORDERS_PLACED_TOTAL);
      lazy_static::initialize(&ORDERS_FILLED_TOTAL);
      lazy_static::initialize(&ORDERS_CANCELLED_TOTAL);
      lazy_static::initialize(&ARBITRAGE_OPPORTUNITIES_DETECTED);
      lazy_static::initialize(&ARBITRAGE_OPPORTUNITIES_EXECUTED);
      lazy_static::initialize(&ORDER_EXECUTION_DURATION);
      lazy_static::initialize(&OPPORTUNITY_DETECTION_DURATION);
      lazy_static::initialize(&API_REQUEST_DURATION);
      lazy_static::initialize(&ACTIVE_POSITIONS);
      lazy_static::initialize(&ACCOUNT_BALANCE_USD);
      lazy_static::initialize(&DAILY_PNL_USD);
      lazy_static::initialize(&EXCHANGE_CONNECTED);
  }
  ```

- [ ] **Add metrics endpoint**
  ```rust
  // In src/web.rs
  use prometheus::{Encoder, TextEncoder};

  async fn metrics_handler() -> impl IntoResponse {
      let encoder = TextEncoder::new();
      let metric_families = prometheus::gather();

      let mut buffer = vec![];
      encoder.encode(&metric_families, &mut buffer).unwrap();

      Response::builder()
          .status(200)
          .header("Content-Type", encoder.format_type())
          .body(Body::from(buffer))
          .unwrap()
  }

  // Add to router
  .route("/metrics", get(metrics_handler))
  ```

- [ ] **Integrate metrics in execution engine**
  ```rust
  // In execution_engine.rs
  use crate::metrics::*;

  impl ExecutionEngine {
      pub async fn execute_arbitrage(...) -> Result<ExecutionResult> {
          let start = Instant::now();

          // ... execution logic ...

          // Record metrics
          ORDERS_PLACED_TOTAL
              .with_label_values(&[&opportunity.buy_exchange, "buy", "limit"])
              .inc();

          ORDERS_PLACED_TOTAL
              .with_label_values(&[&opportunity.sell_exchange, "sell", "limit"])
              .inc();

          ORDER_EXECUTION_DURATION
              .with_label_values(&[&opportunity.buy_exchange])
              .observe(start.elapsed().as_secs_f64());

          if result.is_ok() {
              ARBITRAGE_OPPORTUNITIES_EXECUTED.inc();
          }

          result
      }
  }
  ```

#### 3.4 Activate Circuit Breakers

**Files to Modify**: `database/src/connection.rs`

- [ ] **Activate circuit breaker logic**
  ```rust
  impl ConnectionPool {
      pub async fn execute_with_circuit_breaker<F, T>(
          &self,
          operation: F,
      ) -> Result<T>
      where
          F: FnOnce() -> BoxFuture<'static, Result<T>>,
      {
          // Check circuit breaker state
          let state = self.circuit_breaker.state().await;

          match state {
              CircuitState::Open => {
                  return Err(DatabaseError::CircuitBreakerOpen);
              }
              CircuitState::HalfOpen => {
                  tracing::debug!("Circuit breaker in half-open state, attempting operation");
              }
              CircuitState::Closed => {
                  // Normal operation
              }
          }

          // Execute operation with timeout and retry
          match self.execute_with_retry(operation).await {
              Ok(result) => {
                  self.circuit_breaker.record_success().await;
                  Ok(result)
              }
              Err(e) => {
                  self.circuit_breaker.record_failure().await;

                  // Check if circuit should open
                  if self.circuit_breaker.should_trip().await {
                      tracing::error!("Circuit breaker tripped due to failures");
                      self.circuit_breaker.trip().await;
                  }

                  Err(e)
              }
          }
      }
  }

  pub struct CircuitBreaker {
      state: Arc<Mutex<CircuitState>>,
      failure_count: Arc<AtomicU32>,
      failure_threshold: u32,
      timeout: Duration,
      last_failure_time: Arc<Mutex<Option<Instant>>>,
  }

  impl CircuitBreaker {
      async fn should_trip(&self) -> bool {
          self.failure_count.load(Ordering::Relaxed) >= self.failure_threshold
      }

      async fn trip(&self) {
          let mut state = self.state.lock().await;
          *state = CircuitState::Open;
          *self.last_failure_time.lock().await = Some(Instant::now());

          tracing::warn!("Circuit breaker OPEN - blocking all requests");

          // Schedule reset to half-open after timeout
          let state_clone = self.state.clone();
          let timeout = self.timeout;
          tokio::spawn(async move {
              tokio::time::sleep(timeout).await;
              let mut state = state_clone.lock().await;
              *state = CircuitState::HalfOpen;
              tracing::info!("Circuit breaker moved to HALF_OPEN state");
          });
      }

      async fn record_success(&self) {
          self.failure_count.store(0, Ordering::Relaxed);

          let mut state = self.state.lock().await;
          if matches!(*state, CircuitState::HalfOpen) {
              *state = CircuitState::Closed;
              tracing::info!("Circuit breaker CLOSED - normal operation resumed");
          }
      }

      async fn record_failure(&self) {
          self.failure_count.fetch_add(1, Ordering::Relaxed);
      }
  }
  ```

#### 3.5 Structured Logging Enhancement

**Files to Modify**: `src/main.rs`, various modules

- [ ] **Enhanced logging setup**
  ```rust
  use tracing_subscriber::{
      layer::SubscriberExt,
      util::SubscriberInitExt,
      EnvFilter,
  };

  fn init_logging() {
      let env_filter = EnvFilter::try_from_default_env()
          .unwrap_or_else(|_| EnvFilter::new("info,ninja_gekko=debug"));

      tracing_subscriber::registry()
          .with(env_filter)
          .with(tracing_subscriber::fmt::layer()
              .json()
              .with_current_span(true)
              .with_span_list(true)
              .with_target(true)
              .with_thread_ids(true)
              .with_file(true)
              .with_line_number(true))
          .init();
  }
  ```

- [ ] **Add request IDs for tracing**
  ```rust
  // Middleware for request ID
  pub async fn request_id_middleware(
      mut req: Request,
      next: Next,
  ) -> impl IntoResponse {
      let request_id = Uuid::new_v4().to_string();

      req.extensions_mut().insert(RequestId(request_id.clone()));

      let span = tracing::info_span!(
          "http_request",
          request_id = %request_id,
          method = %req.method(),
          uri = %req.uri(),
      );

      async move {
          next.run(req).await
      }.instrument(span).await
  }
  ```

---

### Phase 4: Monitoring & Observability

#### 4.1 Prometheus Configuration

- [ ] **Create `monitoring/prometheus.yml`**
  ```yaml
  global:
    scrape_interval: 15s
    evaluation_interval: 15s

  scrape_configs:
    - job_name: 'ninja-gekko'
      static_configs:
        - targets: ['trading-engine:8787']
      metrics_path: '/metrics'
  ```

#### 4.2 Grafana Dashboards

- [ ] **Create `monitoring/grafana/dashboards/ninja-gekko-dashboard.json`**
  - Trading metrics panel (orders, fills, P&L)
  - Performance metrics panel (latency, throughput)
  - System metrics panel (CPU, memory, network)
  - Error rates panel

- [ ] **Create `monitoring/grafana/datasources/prometheus.yml`**
  ```yaml
  apiVersion: 1

  datasources:
    - name: Prometheus
      type: prometheus
      access: proxy
      url: http://prometheus:9090
      isDefault: true
  ```

#### 4.3 Alerting Rules

- [ ] **Create `monitoring/alerts.yml`**
  ```yaml
  groups:
    - name: trading_alerts
      interval: 30s
      rules:
        - alert: HighOrderFailureRate
          expr: rate(ninja_gekko_orders_failed_total[5m]) > 0.1
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "High order failure rate detected"

        - alert: DailyLossLimitApproaching
          expr: ninja_gekko_daily_pnl_usd < -4000
          labels:
            severity: critical
          annotations:
            summary: "Daily loss limit approaching ($4000 of $5000)"

        - alert: ExchangeDisconnected
          expr: ninja_gekko_exchange_connected == 0
          for: 1m
          labels:
            severity: critical
          annotations:
            summary: "Exchange {{ $labels.exchange }} disconnected"

        - alert: HighLatency
          expr: histogram_quantile(0.95, rate(ninja_gekko_order_execution_duration_seconds_bucket[5m])) > 0.5
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "P95 order execution latency >500ms"
  ```

---

### Phase 5: Kubernetes Deployment

**Files to Create**: `k8s/*.yaml`

#### 5.1 Deployment Manifest

- [ ] **Create `k8s/deployment.yaml`**
  ```yaml
  apiVersion: apps/v1
  kind: Deployment
  metadata:
    name: ninja-gekko
    labels:
      app: ninja-gekko
  spec:
    replicas: 1
    selector:
      matchLabels:
        app: ninja-gekko
    template:
      metadata:
        labels:
          app: ninja-gekko
      spec:
        containers:
          - name: trading-engine
            image: ghcr.io/your-org/ninja-gekko:latest
            imagePullPolicy: Always
            ports:
              - containerPort: 8787
                name: http
            env:
              - name: DATABASE_URL
                valueFrom:
                  secretKeyRef:
                    name: ninja-gekko-secrets
                    key: database-url
              - name: REDIS_URL
                valueFrom:
                  secretKeyRef:
                    name: ninja-gekko-secrets
                    key: redis-url
              - name: RUST_LOG
                value: "info,ninja_gekko=debug"
            envFrom:
              - configMapRef:
                  name: ninja-gekko-config
            resources:
              requests:
                memory: "1Gi"
                cpu: "500m"
              limits:
                memory: "4Gi"
                cpu: "2000m"
            livenessProbe:
              httpGet:
                path: /health/live
                port: 8787
              initialDelaySeconds: 30
              periodSeconds: 10
              timeoutSeconds: 5
              failureThreshold: 3
            readinessProbe:
              httpGet:
                path: /health/ready
                port: 8787
              initialDelaySeconds: 10
              periodSeconds: 5
              timeoutSeconds: 3
              failureThreshold: 3
        imagePullSecrets:
          - name: ghcr-secret
  ```

#### 5.2 Service & ConfigMap

- [ ] **Create `k8s/service.yaml`**
  ```yaml
  apiVersion: v1
  kind: Service
  metadata:
    name: ninja-gekko
    labels:
      app: ninja-gekko
  spec:
    type: ClusterIP
    ports:
      - port: 8787
        targetPort: 8787
        protocol: TCP
        name: http
    selector:
      app: ninja-gekko
  ```

- [ ] **Create `k8s/configmap.yaml`**
  ```yaml
  apiVersion: v1
  kind: ConfigMap
  metadata:
    name: ninja-gekko-config
  data:
    GEKKO_MODE: "true"
    ALLOCATION_AGGRESSIVENESS: "0.9"
    MIN_PROFIT_PERCENTAGE: "0.05"
  ```

- [ ] **Create `k8s/secret.yaml` (template)**
  ```yaml
  apiVersion: v1
  kind: Secret
  metadata:
    name: ninja-gekko-secrets
  type: Opaque
  stringData:
    database-url: "postgres://user:pass@host:5432/db"
    redis-url: "redis://host:6379"
    coinbase-api-key: "your-key"
    coinbase-api-secret: "your-secret"
  ```

#### 5.3 HorizontalPodAutoscaler

- [ ] **Create `k8s/hpa.yaml`**
  ```yaml
  apiVersion: autoscaling/v2
  kind: HorizontalPodAutoscaler
  metadata:
    name: ninja-gekko-hpa
  spec:
    scaleTargetRef:
      apiVersion: apps/v1
      kind: Deployment
      name: ninja-gekko
    minReplicas: 1
    maxReplicas: 3
    metrics:
      - type: Resource
        resource:
          name: cpu
          target:
            type: Utilization
            averageUtilization: 70
      - type: Resource
        resource:
          name: memory
          target:
            type: Utilization
            averageUtilization: 80
  ```

---

## Acceptance Criteria

### CI/CD

- [ ] All GitHub Actions workflows pass on every commit
- [ ] Code coverage >80%
- [ ] No security vulnerabilities in dependencies
- [ ] Docker images build successfully and are pushed to registry
- [ ] Benchmarks run automatically and detect regressions

### Production Readiness

- [ ] Graceful shutdown completes within 30 seconds
- [ ] Health check endpoints return correct status
- [ ] Metrics endpoint exports Prometheus-compatible data
- [ ] Circuit breakers activate on database/exchange failures
- [ ] Structured logging includes request IDs and spans

### Deployment

- [ ] Docker Compose brings up full stack successfully
- [ ] Kubernetes manifests deploy without errors
- [ ] Health checks pass in deployed environment
- [ ] Grafana dashboards display real-time metrics
- [ ] Prometheus alerts trigger correctly

---

## Verification Commands

```bash
# Test GitHub Actions locally
act -j test

# Build Docker image
docker build -t ninja-gekko:test .

# Run Docker Compose
docker-compose up -d
docker-compose ps
docker-compose logs -f trading-engine

# Test health endpoints
curl http://localhost:8787/health/live
curl http://localhost:8787/health/ready
curl http://localhost:8787/metrics

# Deploy to Kubernetes
kubectl apply -f k8s/
kubectl get pods
kubectl logs -f deployment/ninja-gekko

# Test graceful shutdown
kubectl delete pod <pod-name>  # Should shutdown cleanly
```

---

## Related Issues

- Enables: Safe deployment of #1 and #2
- Blocks: Production launch

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Prometheus Documentation](https://prometheus.io/docs/)
