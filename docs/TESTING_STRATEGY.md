# Ninja Gekko Testing Strategy

## Philosophy

Testing is a first-class requirement in Ninja Gekko. Following the TDD mandate from `AGENTS.md`, no feature or fix ships without corresponding test coverage. This document outlines our comprehensive testing strategy aligned with production requirements.

## Core Principles

1. **Test-Driven Development (TDD)**: Write tests before implementation
2. **High Coverage**: Target 80%+ code coverage for critical paths
3. **Performance Validation**: Guard latency budgets with benchmark assertions
4. **Security First**: Comprehensive security testing for all external interfaces
5. **Isolation**: Tests should be independent and reproducible

## Testing Pyramid

```
         /\
        /  \        E2E Tests (10%)
       /----\       - Trading flow tests
      /      \      - Autonomous mode tests
     /--------\     Integration Tests (30%)
    /          \    - Exchange connector tests
   /------------\   - Multi-component tests
  /              \  Unit Tests (60%)
 /----------------\ - Module-level tests
                    - Function-level tests
```

## Test Categories

### 1. Unit Tests

**Purpose**: Test individual functions and modules in isolation

**Location**: `*/tests/` directories within each crate

**Coverage Requirements**: 90% for new code

**Guidelines**:
- Test one thing per test
- Use descriptive test names
- Mock external dependencies
- Test edge cases and error conditions

**Example**:
```rust
#[test]
fn test_signature_generation() {
    let secret = "test-secret";
    let message = "test-message";
    let signature = utils::hmac_sha256_signature(secret, message);

    assert!(!signature.is_empty());
    assert!(base64::decode(&signature).is_ok());
}
```

### 2. Integration Tests

**Purpose**: Test interactions between components

**Location**: `crates/*/tests/` for crate-level, `tests/` for cross-crate

**Coverage Requirements**: Cover all major component interactions

**Key Areas**:
- Exchange connector operations
- Database interactions
- Event bus message passing
- API endpoint validation

**Example**:
```rust
#[tokio::test]
async fn test_exchange_connector_lifecycle() {
    let mut connector = BinanceUsConnector::new();

    connector.connect().await.unwrap();
    assert!(connector.is_connected().await);

    connector.disconnect().await.unwrap();
    assert!(!connector.is_connected().await);
}
```

### 3. End-to-End Tests

**Purpose**: Validate complete user flows and system behavior

**Location**: `tests/e2e/`

**Key Scenarios**:
- Complete trading flow (data → signal → order → fill)
- Multi-exchange arbitrage execution
- Autonomous mode operations
- Risk management enforcement

### 4. Property-Based Tests

**Purpose**: Test invariants with generated inputs

**Tools**: `proptest` or `quickcheck`

**Applications**:
- Risk calculation functions
- Portfolio math
- Order validation
- Data normalization

**Example**:
```rust
proptest! {
    #[test]
    fn test_position_sizing_invariants(
        portfolio_value in 1000.0..1000000.0,
        risk_percentage in 0.01..0.1
    ) {
        let position_size = calculate_position_size(portfolio_value, risk_percentage);
        prop_assert!(position_size > 0.0);
        prop_assert!(position_size <= portfolio_value * risk_percentage);
    }
}
```

### 5. Benchmark Tests

**Purpose**: Validate performance requirements

**Location**: `benches/`

**Tools**: `criterion`

**Performance Targets** (from AGENTS.md):
- Order path: < 1ms
- Market data handling: < 100μs
- Strategy evaluation: < 10μs
- Memory footprint: < 500MB for 10 strategies × 50 pairs

**Regression Policy**: Fail build if regression > 10%

**Example**:
```rust
fn benchmark_order_placement(c: &mut Criterion) {
    c.bench_function("order_placement", |b| {
        b.iter(|| {
            black_box(create_and_validate_order());
        })
    });
}
```

### 6. Security Tests

**Purpose**: Validate security controls

**Location**: `api/tests/integration_security.rs`

**Key Tests**:
- Authentication bypass attempts
- SQL injection prevention
- XSS protection
- Input validation
- Rate limiting effectiveness
- Credential handling

## Exchange Connector Testing

### Test Infrastructure

Each exchange connector requires:

1. **Connection tests**: Connect/disconnect lifecycle
2. **Authentication tests**: Signature generation, token handling
3. **Streaming tests**: WebSocket message handling
4. **Error handling**: Rate limits, timeouts, malformed responses
5. **Mock fixtures**: Recorded API responses for reproducibility

### Mock Strategy

```rust
// Use recorded fixtures for API responses
#[test]
fn test_parse_ticker_message() {
    let fixture = include_str!("fixtures/binance_ticker.json");
    let result = parse_ticker(fixture);
    assert!(result.is_ok());
}
```

### Network Tests

Network-dependent tests should be:
- Marked with `#[ignore]` attribute
- Clearly documented with requirements
- Run separately in CI with proper credentials

```rust
#[tokio::test]
#[ignore = "requires network connection to Binance.US"]
async fn test_live_market_stream() {
    // ...
}
```

## Backtesting Requirements

### Overview

The backtesting framework must support:
- Historical data replay
- Strategy evaluation
- Performance metrics calculation
- Slippage simulation
- Commission modeling

### Test Requirements

1. **Data Integrity**: Validate historical data loading
2. **Determinism**: Same inputs produce same outputs
3. **Performance**: Complete backtest within reasonable time
4. **Accuracy**: Match expected results for known strategies

## CI/CD Integration

### Test Execution

```yaml
# GitHub Actions workflow
test:
  runs-on: ubuntu-latest
  steps:
    - name: Run unit tests
      run: cargo test --all

    - name: Run integration tests
      run: cargo test --all --features integration

    - name: Run benchmarks
      run: cargo bench --all
```

### Coverage Reporting

Use `cargo-tarpaulin` or `cargo-llvm-cov` for coverage:

```bash
cargo tarpaulin --out Html --output-dir coverage/
```

### Benchmark Baselines

Store benchmark results in `benches/baselines/` and compare against them:

```bash
cargo bench -- --baseline previous
```

## Test Data Management

### Fixtures

Store test fixtures in `tests/fixtures/`:
- JSON response samples
- WebSocket message sequences
- Order book snapshots

### Sensitive Data

- Never commit real credentials
- Use environment variables for test accounts
- Sanitize all logged output

## Debugging Tests

### Verbose Output

```bash
cargo test -- --nocapture
```

### Single Test

```bash
cargo test test_name -- --exact
```

### With Logs

```bash
RUST_LOG=debug cargo test test_name
```

## Performance Testing Guidelines

### Latency Measurements

```rust
let start = Instant::now();
// Operation under test
let elapsed = start.elapsed();

assert!(
    elapsed < Duration::from_millis(1),
    "Operation took {:?}, exceeds 1ms target",
    elapsed
);
```

### Memory Profiling

Use `valgrind` or `heaptrack`:

```bash
valgrind --tool=massif target/debug/ninja-gekko
```

### Load Testing

For concurrent operation tests:

```rust
#[tokio::test]
async fn test_concurrent_load() {
    let mut handles = vec![];

    for _ in 0..1000 {
        let handle = tokio::spawn(async {
            // Concurrent operation
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
```

## Test Maintenance

### Review Schedule

- Weekly: Review failing tests
- Monthly: Update test coverage reports
- Quarterly: Audit test effectiveness

### Deprecation

When removing tests:
1. Document the reason
2. Ensure replacement coverage exists
3. Update coverage targets if needed

## References

- `AGENTS.md` - Testing Doctrine section
- `api/tests/integration_security.rs` - Security test patterns
- `crates/event-bus/benches/dispatcher.rs` - Benchmark examples

---

*This testing strategy ensures Ninja Gekko meets its quality, performance, and security requirements. All contributors must follow these guidelines.*
