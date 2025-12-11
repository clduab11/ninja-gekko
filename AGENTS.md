# Ninja Gekko Agent Operations Manual

## Mission Profile
- Uphold the existing modular Rust architecture and extend it without regressions.
- Deliver production-grade, low-latency trading capabilities across OANDA, Binance.US, and Kraken via Tenno-MCP orchestration.
- Treat performance, safety, and observability as first-class requirements in every change.

## Core Principles
1. Performance first, profit close second: target <1 ms order path, <100 microseconds market data handling, <10 microseconds basic strategy evaluation, <500 MB footprint for 10 strategies across 50 pairs.
2. Safety and correctness: forbid unsafe code; rely on ownership, Send/Sync correctness, and comprehensive error handling.
3. Strict modularity: Data handlers, strategy runners, portfolio, execution, risk, and exchange connectors communicate only through the event system.
4. TDD mandate: no feature or fix without unit/integration/bench coverage.
5. Preservation doctrine: audit before altering; keep proven code; extend rather than replace.

## Repository Atlas (Do Not Break These Contracts)
- `src/core.rs` and `core/src/*`: High-level bot orchestration, rich order/portfolio types, order manager, smart routing. Reuse these types instead of redefining.
- `crates/exchange-connectors`: Unified exchange trait, per-exchange adapters (Binance US, Kraken, OANDA). Follow their trait signatures and error semantics.
- `crates/arbitrage-engine`: Volatility scanning, opportunity detection, allocation, execution scaffolding. Integrate rather than reimplement.
- `database/src/*`: PostgreSQL, Redis, Supabase integration with advanced connection pooling. Extend configs and migrations here.
- `src/mcp.rs` plus `mcp_admin`: Tenno-MCP bridge and admin actions. Augment connection logic instead of replacing placeholder scaffolding wholesale.

## Workflow Protocol
1. Recon: read relevant modules, note strengths, confirm existing tests/examples.
2. Plan: outline tasks (minimum two steps) via CLI plan tool before coding when work is non-trivial.
3. Design: respect existing abstractions; propose additive traits/structs if gaps exist.
4. Implement: write minimal, composable Rust following module boundaries.
5. Validate: run targeted tests/benches; add new ones if gaps exist.
6. Report: summarize impact, reference code paths, list follow-up actions.

## Event-Driven Execution Blueprint
- Bus primitives: prefer `crossbeam::channel::{Sender, Receiver}` or `crossbeam::queue::SegQueue` for lock-free hops; expose typed envelopes (`MarketEvent`, `SignalEvent`, `OrderEvent`, `ExecutionEvent`, `RiskEvent`).
- Dispatcher: central async task owning senders, wiring ingestion -> strategy -> portfolio -> execution -> risk loops; leverage `tokio::select!` for multiplexing.
- Backpressure: size bounded channels; drop or buffer per module policy; record metrics.
- Serialization: use zero-copy serde/bincode or postcard on the wire, with const generic payload sizes to avoid heap churn.

### Event Lifecycle Checklist
[ ] Ingestion converts raw exchange payloads into normalized structs without cloning large buffers.
[ ] Normalizer stamps monotonic sequence numbers and timestamps.
[ ] Strategy stage consumes immutable refs, emits `SignalEvent` via sender.
[ ] Portfolio stage performs VaR/risk checks before forwarding `OrderEvent`.
[ ] Execution stage reuses exchange connectors, emits order updates to bus.
[ ] Risk guardian can halt execution within 100 ms by signaling stop event.

## Data Pipeline Guardrails
- Data flow: WebSocket ingestion task -> normalizer -> distributor fan-out (`crossbeam::channel::Sender<Event>` clones).
- Level 2 order book: maintain price ladders with delta compression; use `AHashMap` or fixed arrays for hot paths.
- SIMD indicators: prefer `wide`, `packed_simd_2`, or `std::simd` gated by const generics; benchmark with Criterion.
- Persistence: stream normalized candles/ticks into PostgreSQL partitions (time-series schema) and Redis caches for hot reads.

## Strategy Isolation
- Strategies implement trait `StrategyExecutor<const N: usize>` for compile-time sizing.
- Host them in WASM sandbox via `wasmtime` or `wasmer` with resource limits; communicate through serialized events.
- Provide host functions for market snapshots, order submission, logging.

## Risk Management Protocol
- Real-time portfolio metrics from `core::types::Portfolio` and `core::types::Position`.
- Implement VaR (historical/parametric) with rolling windows stored in Redis.
- Dynamic position sizing using Kelly fraction or volatility scaling; integrate with allocation manager.
- Audit log every decision (orders, halts, reallocations) to PostgreSQL partitioned tables.
- Global kill switch must propagate through event bus and stop execution tasks within 100 ms.

## Exchange Integration Standards
- Reuse `ExchangeConnector` trait; implement streaming via WebSocket-first clients using `tokio_tungstenite`.
- Handle REST fallbacks with exponential backoff and governor rate limiting.
- Maintain unified authentication flow (key/secret/passphrase) with encrypted storage (AES-GCM, Argon2 derived).
- Provide depth snapshot + incremental diff support for each exchange.

## Database and Cache Requirements
- PostgreSQL: use `sqlx` query macros; enforce time-series partitioning (daily/weekly) via migrations.
- Redis: connection pooling through `redis::Client::get_multiplexed_tokio_connection`; cache hot metrics, order books, VaR states.
- Connection manager (`database/src/connection.rs`) already supports circuit breakers—configure rather than rewrite.

## Observability and Benchmarking
- Instrument every async boundary with `tracing` spans; export to Prometheus via `tracing-opentelemetry` + `axum` metrics endpoint.
- Benchmarks: use `criterion` for hot loops (event dispatch, indicator calc); store baselines under `benches/`.
- Log structure: JSON logs with unique correlation IDs per event cycle.

## Security and Credential Handling
- Never embed credentials in code; rely on environment-backed secrets + encrypted vault (AES-GCM key derived with Argon2id).
- MCP administrative actions must be logged and require explicit capability checks.
- Validate all external input with schema checks; sanitize file operations executed via MCP admin.

## Testing Doctrine
- Unit tests per module (`core`, `exchange-connectors`, etc.).
- Integration tests covering multi-exchange scenarios, strategy lifecycles, VaR halts.
- Property tests for risk/portfolio math (`proptest` or `quickcheck`).
- WASM strategies: conformance tests ensuring deterministic execution under sandbox constraints.
- Benchmark assertions to guard latency budgets (fail build if regressions >10%).

## Deliverable Template
Every change must include:
1. Analysis of touched modules with references (path:line).
2. Explicit list of preserved components and why they remain.
3. New functionality description and integration points.
4. Test and benchmark evidence (commands + results).
5. Follow-up tasks or risks.

## Quick Command Reference
- Audit: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all`, `cargo bench`.
- Database: `sqlx migrate run`, `sqlx migrate add`, `psql -f database/migrations/...`.
- Observability: run `cargo run -p api -- --metrics-endpoint` then scrape `/metrics`.
- WASM build: `cargo build --target wasm32-wasi -p strategy-*`.

## Decision Matrix
When considering modifications:
- Replace only if existing code cannot meet latency/safety targets with reasonable tuning.
- Augment by layering new traits/structs or feature flags that coexist with incumbents.
- Document migration paths and deprecation timelines inside module docs if changes are disruptive.

Keep this manual open during every session. Deviating from these guardrails requires explicit written justification and a rollback plan.
