# GitHub Copilot Setup Guide for Ninja Gekko

## Overview

This repository contains 3 AI-optimized milestone issues designed for autonomous implementation by GitHub Copilot, Claude Code, or other AI coding assistants. All timeline references have been removed in favor of phase-based, milestone-oriented completion criteria.

---

## 🤖 GitHub Copilot Context Prompts

Copy the appropriate prompt below into your Copilot chat **before** starting work on each milestone. These prompts provide essential context for November 2025 best practices and coding standards.

### MILESTONE 1: Exchange API Integration

**Copy this prompt to Copilot:**

```
You are implementing MILESTONE 1: Exchange API Integration for a production Rust trading system. Follow November 2025 standards: use Rust 2021 edition with async/await, tokio 1.0+, comprehensive error handling via thiserror, structured logging with tracing spans, no unsafe code, trait-based abstractions. Implement REST APIs and WebSocket streams for Coinbase Advanced Trade, Binance.US, and OANDA with HMAC-SHA256 auth. Each function needs rustdoc comments, all errors use Result<T, ExchangeError>, rate limiters enforce exchange policies, reconnection uses exponential backoff. Write integration tests with wiremock, unit tests for parsers, achieve >80% coverage. Target <100ms order placement latency. Reference existing traits in crates/exchange-connectors/src/lib.rs. Use reqwest 0.11 for HTTP, tokio-tungstenite 0.21 for WebSocket, serde for JSON. Code must pass cargo clippy --all-targets -- -D warnings and cargo fmt. Follow event-driven patterns: emit events to event bus, parse WebSocket to StreamMessage enums. Security: no hardcoded credentials, use env vars, validate all inputs, audit log all API calls. This enables real trading—precision and correctness are critical.
```

---

### MILESTONE 2: Arbitrage Engine Implementation

**Copy this prompt to Copilot:**

```
You are implementing MILESTONE 2: Arbitrage Detection & Execution Logic for a production Rust trading system. Follow November 2025 standards: Rust 2021 edition, tokio 1.0+ async, thiserror for errors, tracing for logging, no unsafe code. Implement volatility scanning with multi-timeframe analysis (1m/5m/15m windows), cross-exchange opportunity detection with confidence scoring (0-1 scale, 85%+ threshold), Kelly Criterion position sizing with constraints, simultaneous order execution coordination. Use Decimal type for all financial calculations, never f64. Every function needs rustdoc with examples. Risk management: enforce stop-loss (2%), circuit breakers (5 consecutive losses), daily loss limits ($5000), position exposure limits (15% per symbol). Event-driven: emit to event bus for all opportunities/executions. Performance: <100ms volatility scan, <50ms opportunity detection, <100ms execution. Testing: >80% coverage, unit tests for each component, integration tests for full arbitrage cycle, property-based tests with proptest for Kelly Criterion. Reference existing types in crates/arbitrage-engine/src/lib.rs and config/arbitrage.toml for parameters. Code must be production-grade: zero clippy warnings, formatted with cargo fmt, comprehensive error handling, audit logging. This is the core revenue feature—implement with precision and correctness.
```

---

###  MILESTONE 3: CI/CD Pipeline & Production Hardening

**Copy this prompt to Copilot:**

```
You are implementing MILESTONE 3: CI/CD Pipeline & Production Hardening for a Rust trading system. Follow November 2025 DevOps standards: GitHub Actions with caching, multi-stage Docker builds with distroless base, Kubernetes manifests with HPA/liveness/readiness probes, Prometheus metrics with Grafana dashboards, structured logging with tracing. Create GitHub Actions workflows: CI (fmt/clippy/test with Postgres/Redis services), security audit (cargo audit/deny), benchmarks (criterion with regression detection), Docker build/push to GHCR. Implement graceful shutdown: SIGTERM handler, cancel open orders, flush event queues, close connections within 30s timeout. Add health endpoints: /health/live (process alive), /health/ready (dependencies healthy). Prometheus metrics: counters for orders/fills, histograms for latency (p50/p95/p99), gauges for positions/balance/PnL. Activate circuit breaker in database layer: trip on 5 failures, half-open after timeout, track state. All async shutdown with tokio::select and proper signal handling. Docker: use rust:slim builder, debian:bookworm-slim runtime, non-root user, health checks. K8s: resource limits, rolling updates, config via ConfigMap, secrets via Secret. Monitoring: Grafana dashboards for trading/performance/system metrics, Prometheus alerts for failures/latency/loss limits. Testing: verify graceful shutdown, health endpoints return correct status, metrics export correctly. Code quality: clippy clean, formatted, comprehensive error handling, security-first (no secrets in logs). This enables production deployment—implement with reliability and observability as priorities.
```

---

## Quick Start for AI Agents

### Step 1: Choose Your Milestone

Start with **MILESTONE 1** (Exchange API Integration) as it's the foundation for everything else.

### Step 2: Copy Context Prompt to Copilot

Open GitHub Copilot chat in your IDE and paste the full context prompt for your chosen milestone.

### Step 3: Reference the Issue Template

Open the issue template file:
- MILESTONE 1: `.github/ISSUE_TEMPLATE/01-exchange-api-integration.md`
- MILESTONE 2: `.github/ISSUE_TEMPLATE/02-arbitrage-engine-implementation.md`
- MILESTONE 3: `.github/ISSUE_TEMPLATE/03-cicd-production-hardening.md`

### Step 4: Start Implementation

Ask Copilot to start implementing the first phase:

```
Implement Phase 1 of this milestone, starting with the first checklist item.
Reference the code examples provided and follow November 2025 best practices.
```

### Step 5: Iterate Through Phases

Work through each phase sequentially, checking off items as you complete them:
1. Implement the code
2. Run tests
3. Verify with provided commands
4. Check off the checklist item
5. Commit changes
6. Move to next item

### Step 6: Verification

At the end of each phase, run the verification commands from the issue template:

```bash
# Example for MILESTONE 1:
cargo test -p exchange-connectors
cargo clippy -p exchange-connectors --all-targets -- -D warnings
cargo fmt -p exchange-connectors -- --check
cargo tarpaulin -p exchange-connectors --out Html
```

### Step 7: Milestone Completion

The milestone is complete when:
✅ All checkboxes are checked
✅ All tests pass
✅ Clippy shows zero warnings
✅ Code coverage >80%
✅ Acceptance criteria met

---

## November 2025 Best Practices Summary

### Rust Standards
- **Edition**: Rust 2021
- **Async**: tokio 1.35+ with full features
- **Error Handling**: thiserror for custom errors, anyhow for application errors
- **Logging**: tracing with structured spans
- **Safety**: `#![forbid(unsafe_code)]`
- **Testing**: cargo-tarpaulin for coverage, criterion for benchmarks
- **Linting**: `cargo clippy --all-targets -- -D warnings`
- **Formatting**: `cargo fmt` (2021 edition style)

### Code Quality
- **Documentation**: Rustdoc comments with examples on all public items
- **Error Propagation**: Use `?` operator, never `unwrap()/expect()` in production
- **Types**: Use `Decimal` for money, never `f64`
- **Async Patterns**: tokio::spawn for tasks, tokio::select! for timeouts
- **Testing**: >80% coverage, unit + integration + property-based tests

### Security
- **Credentials**: Environment variables only, never hardcoded
- **Validation**: All inputs validated before use
- **Logging**: Audit all API calls, never log secrets
- **Dependencies**: Regular audits with cargo-audit and cargo-deny

### Performance
- **Order Placement**: <100ms latency
- **WebSocket Parsing**: <10ms per message
- **Opportunity Detection**: <50ms per scan
- **Memory**: Stable under continuous load

---

## Milestone Dependencies

```
MILESTONE 1 (Exchange APIs)
    └─> Required for: MILESTONE 2 (Arbitrage)
            └─> Required for: Production Trading

MILESTONE 3 (CI/CD)
    └─> Enables: Safe deployment of MILESTONE 1 & 2
```

**Recommended Order**:
1. MILESTONE 1 first (foundation)
2. MILESTONE 3 second (deployment automation)
3. MILESTONE 2 third (business logic with CI/CD safety net)

---

## AI Agent Tips

### For GitHub Copilot
1. Keep the context prompt active in chat
2. Reference code examples directly from issue templates
3. Ask for clarification if uncertain about patterns
4. Verify each phase before moving to next

### For Claude Code
1. Use the context prompt as system context
2. Leverage tool use for file operations
3. Run tests after each implementation step
4. Commit incrementally with descriptive messages

### For Cursor
1. Add context prompt to .cursorrules
2. Use Composer mode for multi-file changes
3. Reference existing code patterns
4. Validate against acceptance criteria

---

## Troubleshooting

### If Copilot suggests unsafe code
- Remind it: "No unsafe code allowed, use safe alternatives"
- Reference the context prompt again

### If tests fail
- Check error messages carefully
- Verify dependencies are correct versions
- Ensure database/Redis services are running for integration tests

### If clippy warnings appear
- Fix them immediately (blockers for PR merge)
- Run `cargo clippy --fix` for auto-fixes
- Manually fix remaining warnings

### If coverage is low
- Add unit tests for uncovered functions
- Add integration tests for workflows
- Add property-based tests for algorithms

---

## Progress Tracking

Use GitHub issues to track progress:

1. Create issue from template: `.github/ISSUE_TEMPLATE/01-exchange-api-integration.md`
2. Check off items as completed
3. Comment with progress updates
4. Link commits to issue
5. Close when all acceptance criteria met

---

## Completion Criteria

Each milestone is **COMPLETE** when:

✅ All phase checklists 100% checked
✅ `cargo test --all` passes
✅ `cargo clippy --all-targets -- -D warnings` shows zero warnings
✅ `cargo tarpaulin` shows >80% coverage
✅ All acceptance criteria validated
✅ Documentation builds without errors
✅ Production readiness checklist complete

**After all 3 milestones**: Repository is production-ready at ~95% completion.

---

## Getting Help

- **Copilot not sure?** Paste more context from issue template
- **Stuck on implementation?** Reference existing code in crates/
- **Tests failing?** Check verification commands in issue template
- **Pattern unclear?** See "Implementation Notes for AI Agents" section in each issue

---

## Repository Transformation

**Before Milestones**: 47.5% Complete
- ✅ Infrastructure (90%)
- 🟡 Exchanges (30%)
- 🟡 Arbitrage (35%)
- 🔴 CI/CD (0%)

**After Milestones**: 95.5% Complete
- ✅ Infrastructure (90%)
- ✅ Exchanges (95%)
- ✅ Arbitrage (95%)
- ✅ CI/CD (90%)

**Net Gain**: +48% → Production Ready

---

Last Updated: November 2025
AI-Optimized for: GitHub Copilot, Claude Code, Cursor, and similar AI coding assistants
