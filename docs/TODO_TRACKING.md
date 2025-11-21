# Ninja Gekko TODO Tracking

## Overview

This document tracks the 22 TODO items identified across the codebase, organized by category and priority. These items represent stubbed functionality that needs to be implemented for production readiness.

**Reference**: `PLAN.md` for milestone-based implementation schedule

---

## Summary

| Category | Count | Priority |
|----------|-------|----------|
| Authentication | 7 | High |
| Trading Operations | 7 | High |
| Infrastructure | 3 | Medium |
| Autonomous Modes | 3 | Medium |
| Exchange Connectors | 1 | High |
| Backtesting | 1 | Medium |
| **Total** | **22** | |

---

## Authentication TODOs (7)

### High Priority

#### AUTH-001: Database Authentication
**Location**: `api/src/auth_validation.rs`
**Status**: Stubbed - returns mock validation
**Phase**: Phase 3 (Weeks 8-9)

**Current Implementation**:
```rust
// TODO: Replace with database lookup
pub async fn validate_credentials(email: &str, password: &str) -> Result<User, AuthError> {
    // Mock implementation
}
```

**Requirements**:
- Integrate with user database
- Use Argon2id for password verification
- Return proper user object with permissions

---

#### AUTH-002: Refresh Token Storage
**Location**: `api/src/handlers/auth_utils.rs`
**Status**: Stubbed
**Phase**: Phase 3 (Weeks 8-9)

**Requirements**:
- Implement HMAC-SHA256 token signing
- Store tokens encrypted with AES-GCM
- Implement token rotation mechanism
- Set appropriate expiration times

---

#### AUTH-003: JWT Token Validation
**Location**: `api/src/middleware/`
**Status**: Basic implementation
**Phase**: Phase 3 (Weeks 8-9)

**Requirements**:
- Validate all JWT claims
- Check expiration and issued-at times
- Verify audience and issuer
- Handle token refresh flow

---

#### AUTH-004: API Key Authentication
**Location**: `api/src/auth/`
**Status**: Not implemented
**Phase**: Phase 3 (Weeks 8-9)

**Requirements**:
- Generate secure API keys
- Hash keys for storage
- Implement key rotation
- Rate limiting per key

---

#### AUTH-005: User Database Schema
**Location**: `database/migrations/`
**Status**: Not created
**Phase**: Phase 3 (Weeks 8-9)

**Requirements**:
- User accounts table
- Refresh token storage
- API key management
- Audit log table

---

#### AUTH-006: Credential Encryption
**Location**: `api/src/crypto/`
**Status**: Partially implemented
**Phase**: Phase 3 (Weeks 8-9)

**Requirements**:
- AES-GCM encryption for API keys
- Argon2id key derivation
- Secure random generation
- Key rotation support

---

#### AUTH-007: Session Management
**Location**: `api/src/handlers/`
**Status**: Basic implementation
**Phase**: Phase 3 (Weeks 8-9)

**Requirements**:
- Session timeout handling
- Concurrent session limits
- Session invalidation
- Activity tracking

---

## Trading Operations TODOs (7)

### High Priority

#### TRADE-001: Binance.us Trading Operations
**Location**: `crates/exchange-connectors/src/binance_us.rs`
**Status**: Returns error for all trading ops
**Phase**: Phase 2 (Weeks 5-7)

**Current Implementation**:
```rust
async fn place_order(&self, ...) -> ExchangeResult<ExchangeOrder> {
    Err(ExchangeError::InvalidRequest(
        "Trading not implemented for Binance.us connector".to_string(),
    ))
}
```

**Requirements**:
- Implement `place_order` with HMAC-SHA256 auth
- Implement `cancel_order` with validation
- Implement `get_order` with status tracking
- Handle rate limiting

---

#### TRADE-002: OANDA Trading Operations
**Location**: `crates/exchange-connectors/src/oanda.rs`
**Status**: Stubbed
**Phase**: Phase 2 (Weeks 5-7)

**Requirements**:
- Implement `place_order` for forex
- Implement `cancel_order` with position tracking
- Implement `get_order` with fill details
- Bearer token refresh logic

---

#### TRADE-003: Coinbase Trading Operations
**Location**: `crates/exchange-connectors/src/coinbase.rs`
**Status**: Stubbed
**Phase**: Phase 2 (Weeks 5-7)

**Requirements**:
- Implement `place_order` with CB-ACCESS headers
- Implement `cancel_order`
- Implement `get_order` via Advanced Trade API
- Handle passphrase in authentication

---

#### TRADE-004: Trade Execution Engine
**Location**: `api/src/handlers/trades.rs`
**Status**: Mock responses
**Phase**: Phase 4 (Weeks 10-12)

**Requirements**:
- Integrate with `core::order_manager`
- Order validation pipeline
- Trade persistence to database
- Statistics calculation

---

#### TRADE-005: Order Manager Integration
**Location**: `core/src/order_manager.rs`
**Status**: Partially implemented
**Phase**: Phase 4 (Weeks 10-12)

**Requirements**:
- Smart order routing
- Order state machine
- Partial fill handling
- Timeout management

---

#### TRADE-006: Fund Transfer Operations
**Location**: `crates/exchange-connectors/src/lib.rs`
**Status**: Returns error
**Phase**: Phase 2 (Weeks 5-7)

**Requirements**:
- Implement for supported exchanges
- Transfer status tracking
- Error handling for insufficient balance
- Priority-based transfers

---

#### TRADE-007: Batch Order Operations
**Location**: `api/src/handlers/trades.rs`
**Status**: Not implemented
**Phase**: Phase 4 (Weeks 10-12)

**Requirements**:
- Bulk order placement
- Batch cancellation
- Atomic operations where possible
- Error handling for partial failures

---

## Infrastructure TODOs (3)

### Medium Priority

#### INFRA-001: Redis Rate Limiting Integration
**Location**: `api/src/middleware/`
**Status**: Local rate limiting only
**Phase**: Phase 4 (Weeks 10-12)

**Requirements**:
- Distributed rate limiting via Redis
- Sliding window algorithm
- Per-user and per-IP limits
- Graceful degradation

---

#### INFRA-002: Connection Pooling Optimization
**Location**: `database/src/connection.rs`
**Status**: Basic pooling
**Phase**: Phase 4 (Weeks 10-12)

**Requirements**:
- Optimize pool sizes
- Health checks
- Connection recycling
- Metrics collection

---

#### INFRA-003: Migration Rollback Support
**Location**: `database/migrations/`
**Status**: No rollback column
**Phase**: Phase 7 (Weeks 18-19)

**Requirements**:
- Add `rolled_back_at` column
- Rollback procedures
- Backup before migration
- Verification queries

---

## Autonomous Modes TODOs (3)

### Medium Priority

#### MODE-001: Stealth Mode Implementation
**Location**: `src/core.rs`
**Status**: Placeholder
**Phase**: Phase 5 (Weeks 13-15)

**Requirements**:
- Fragmented order execution
- Randomized timing
- Volume-weighted distribution
- Market impact estimation

---

#### MODE-002: Precision Mode Implementation
**Location**: `src/core.rs`
**Status**: Placeholder
**Phase**: Phase 5 (Weeks 13-15)

**Requirements**:
- Microsecond timing optimization
- Neural network integration
- High-frequency data processing
- Latency monitoring

---

#### MODE-003: Swarm Mode Implementation
**Location**: `src/core.rs`
**Status**: Placeholder
**Phase**: Phase 5 (Weeks 13-15)

**Requirements**:
- Distributed coordination
- Collaborative decision making
- Consensus mechanism
- Load balancing

---

## Exchange Connectors TODO (1)

### High Priority

#### CONN-001: Private Order Stream
**Location**: `crates/exchange-connectors/src/binance_us.rs`
**Status**: Returns empty channel
**Phase**: Phase 2 (Weeks 5-7)

**Current Implementation**:
```rust
async fn start_order_stream(&self) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
    let (_tx, rx) = mpsc::unbounded_channel();
    warn!("Binance.us private order stream requires authenticated keys; not yet implemented");
    Ok(rx)
}
```

**Requirements**:
- Authenticated WebSocket connection
- User data stream
- Order update handling
- Balance change notifications

---

## Backtesting TODO (1)

### Medium Priority

#### BACK-001: Backtesting Framework
**Location**: `crates/strategy-engine/`
**Status**: Not implemented
**Phase**: Phase 7 (Weeks 18-19)

**Requirements**:
- Historical data replay
- Strategy evaluation
- Performance metrics
- Slippage simulation
- Commission modeling

---

## Implementation Strategy

### Milestone Approach

Use GitHub milestones (created via issue templates) to track progress:

1. **Phase 2 Milestone**: Exchange Connector Completion
   - TRADE-001, TRADE-002, TRADE-003
   - TRADE-006
   - CONN-001

2. **Phase 3 Milestone**: Authentication & Authorization
   - AUTH-001 through AUTH-007

3. **Phase 4 Milestone**: Trading Engine Integration
   - TRADE-004, TRADE-005, TRADE-007
   - INFRA-001, INFRA-002

4. **Phase 5 Milestone**: Autonomous Modes
   - MODE-001, MODE-002, MODE-003

5. **Phase 7 Milestone**: Production Readiness
   - INFRA-003
   - BACK-001

### Prioritization

**Immediate** (Block other work):
- AUTH-001 (database auth)
- TRADE-001, TRADE-002, TRADE-003 (trading ops)

**Short-term** (Next milestone):
- AUTH-002 through AUTH-007
- CONN-001

**Medium-term** (Following milestone):
- TRADE-004, TRADE-005, TRADE-007
- INFRA-001, INFRA-002

**Long-term** (Before production):
- MODE-001, MODE-002, MODE-003
- INFRA-003
- BACK-001

---

## Tracking Updates

| Date | Item | Status | Notes |
|------|------|--------|-------|
| 2025-11 | Initial tracking | Created | 22 items identified |

---

*Update this document as TODOs are completed. Use issue templates for detailed tracking.*
