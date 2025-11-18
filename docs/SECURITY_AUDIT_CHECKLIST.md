# Ninja Gekko Security Audit Checklist

## Overview

This checklist provides comprehensive security audit guidelines for Ninja Gekko. Use this document for pre-production audits, routine security reviews, and compliance verification.

**Reference Documents**:
- `AGENTS.md` - Security and Credential Handling section
- `api/tests/integration_security.rs` - Security test patterns
- `docs/overview.md` - System architecture

---

## 1. Exchange Authentication

### 1.1 Binance.US (HMAC-SHA256)

- [ ] Signature generation uses constant-time comparison
- [ ] Timestamp synchronized with exchange (NTP)
- [ ] API secret never logged or exposed in errors
- [ ] Request payload properly encoded before signing
- [ ] Signature included in correct header (`X-MBX-SIGNATURE`)

### 1.2 Coinbase (HMAC-SHA256 + Passphrase)

- [ ] All four CB-ACCESS headers present
- [ ] Message format: `timestamp + method + requestPath + body`
- [ ] Passphrase stored with same security as API secret
- [ ] Signature is base64 encoded
- [ ] Headers sent over HTTPS only

### 1.3 OANDA (Bearer Token)

- [ ] Token stored securely (not in plain text)
- [ ] Token refresh before expiration
- [ ] Invalid token triggers re-authentication
- [ ] Bearer prefix included in Authorization header
- [ ] Account ID validated before operations

### 1.4 General Authentication

- [ ] No credentials hardcoded in source
- [ ] Credentials loaded from environment variables
- [ ] Test/sandbox credentials separate from production
- [ ] Credential rotation procedure documented
- [ ] Failed authentication attempts logged and monitored

---

## 2. API Key Storage

### 2.1 Encryption at Rest

- [ ] API keys encrypted using AES-GCM (256-bit)
- [ ] Encryption key derived using Argon2id
- [ ] Salt unique per stored credential
- [ ] IV/nonce never reused
- [ ] Encrypted data authenticated (AEAD)

### 2.2 Key Derivation

- [ ] Argon2id parameters meet security requirements:
  - Memory: ≥ 64 MB
  - Iterations: ≥ 3
  - Parallelism: ≥ 1
- [ ] Master key not stored, derived from secret
- [ ] Key derivation inputs from secure source

### 2.3 Secure Storage

- [ ] Credentials stored in dedicated secrets table
- [ ] Database column encryption if available
- [ ] File permissions restricted (600/400)
- [ ] No credentials in version control
- [ ] Backup encryption maintained

---

## 3. Database Security

### 3.1 Connection Security

- [ ] TLS required for database connections
- [ ] Certificate validation enabled
- [ ] Connection string credentials not logged
- [ ] Connection pooling configured securely
- [ ] Idle connections timeout appropriately

### 3.2 Query Safety

- [ ] All queries use parameterized statements
- [ ] No string concatenation in SQL
- [ ] User input validated before queries
- [ ] Query logging sanitizes sensitive data
- [ ] ORM prevents injection (sqlx compile-time checks)

### 3.3 Access Control

- [ ] Database user has minimal required permissions
- [ ] Separate users for read/write operations
- [ ] Admin access restricted and audited
- [ ] Schema migrations reviewed for security
- [ ] Sensitive columns marked appropriately

### 3.4 Data Protection

- [ ] PII identified and protected
- [ ] Sensitive data encrypted in database
- [ ] Audit tables for critical operations
- [ ] Data retention policy implemented
- [ ] Backup encryption verified

---

## 4. Network Security

### 4.1 TLS Configuration

- [ ] TLS 1.2+ required for all connections
- [ ] Strong cipher suites only
- [ ] Certificate chain validation enabled
- [ ] HSTS headers configured
- [ ] Certificate expiration monitored

### 4.2 WebSocket Security

- [ ] WSS (TLS) required for all WebSocket connections
- [ ] Origin validation for WebSocket upgrade
- [ ] Message size limits enforced
- [ ] Connection timeout configured
- [ ] Reconnection backoff implemented

### 4.3 API Security

- [ ] HTTPS enforced for all endpoints
- [ ] CORS properly configured
- [ ] Security headers present:
  - Content-Security-Policy
  - X-Content-Type-Options
  - X-Frame-Options
  - X-XSS-Protection
- [ ] Request size limits enforced

### 4.4 Rate Limiting

- [ ] Rate limiting per IP/user implemented
- [ ] Different limits for different endpoints
- [ ] Rate limit headers in responses
- [ ] Distributed rate limiting (Redis)
- [ ] Graceful handling of limit exceeded

### 4.5 DDoS Protection

- [ ] Request rate monitoring in place
- [ ] Automatic blocking of abusive IPs
- [ ] CDN/WAF integration considered
- [ ] Graceful degradation under load
- [ ] Alerting for attack detection

---

## 5. Input Validation

### 5.1 SQL Injection Prevention

Reference: `api/src/validation.rs`

- [ ] All inputs validated before database use
- [ ] Parameterized queries throughout
- [ ] Input length limits enforced
- [ ] Special characters escaped appropriately
- [ ] Error messages don't reveal schema

### 5.2 XSS Prevention

- [ ] Output encoding for web responses
- [ ] Content-Type headers set correctly
- [ ] User input sanitized before display
- [ ] CSP headers configured
- [ ] No inline JavaScript from user input

### 5.3 Path Traversal Prevention

- [ ] File paths validated against whitelist
- [ ] No user input in file paths directly
- [ ] Canonical path comparison
- [ ] Restricted file system access
- [ ] MCP file operations validated

### 5.4 Command Injection Prevention

- [ ] No shell command execution with user input
- [ ] If required, use allowlist of commands
- [ ] Arguments properly escaped
- [ ] Environment variables sanitized
- [ ] Process execution privileges minimized

### 5.5 General Validation

Reference: `api/src/env_validation.rs`

- [ ] Type validation for all inputs
- [ ] Range validation for numeric inputs
- [ ] Format validation for structured data
- [ ] Required fields enforced
- [ ] Unknown fields rejected or ignored safely

---

## 6. Rate Limiting

Reference: `crates/exchange-connectors/src/lib.rs`

### 6.1 Implementation

- [ ] Governor-based rate limiting operational
- [ ] Per-endpoint limits configured
- [ ] Per-user/IP limits enforced
- [ ] Exchange-specific limits respected
- [ ] Burst handling configured

### 6.2 Configuration

- [ ] Limits match exchange requirements:
  - Binance.US: 1200 req/min
  - Coinbase: 10 req/sec (private)
  - OANDA: 120 req/sec
- [ ] Limits adjustable without code changes
- [ ] Different limits for different operations

### 6.3 Monitoring

- [ ] Rate limit hits logged
- [ ] Metrics exposed for monitoring
- [ ] Alerting on excessive rate limiting
- [ ] Client feedback on remaining quota

---

## 7. Authorization

### 7.1 Access Control

- [ ] Role-based access control implemented
- [ ] Principle of least privilege enforced
- [ ] Admin actions require elevated permissions
- [ ] Resource ownership validated
- [ ] Action authorization logged

### 7.2 MCP Admin Security

Reference: `AGENTS.md` - MCP admin actions

- [ ] All admin actions require capability checks
- [ ] Admin actions logged with actor identity
- [ ] Sensitive operations require confirmation
- [ ] Rate limiting on admin endpoints
- [ ] Admin sessions timeout appropriately

### 7.3 API Authorization

- [ ] JWT validation for authenticated endpoints
- [ ] Token expiration enforced
- [ ] Refresh token rotation implemented
- [ ] Invalid tokens rejected immediately
- [ ] Authorization errors logged

---

## 8. Logging & Audit

### 8.1 Credential Sanitization

- [ ] API keys never logged
- [ ] Passwords never logged
- [ ] Tokens never logged
- [ ] Sensitive headers redacted
- [ ] Request bodies sanitized

### 8.2 Audit Trail

- [ ] All authentication attempts logged
- [ ] Authorization decisions logged
- [ ] Trading operations logged
- [ ] Configuration changes logged
- [ ] Admin actions logged with actor

### 8.3 Log Security

- [ ] Logs stored securely
- [ ] Log access restricted
- [ ] Log integrity verified
- [ ] Log retention policy defined
- [ ] Logs not accessible via web

### 8.4 Monitoring

- [ ] Security events generate alerts
- [ ] Failed auth attempts monitored
- [ ] Unusual activity patterns detected
- [ ] Rate limit violations tracked
- [ ] Error rates monitored

---

## 9. Compliance

### 9.1 Financial Regulations

- [ ] Trading records retained as required
- [ ] Audit trail meets regulatory requirements
- [ ] Risk controls documented
- [ ] Reporting capabilities available
- [ ] Data residency requirements met

### 9.2 Data Protection

- [ ] PII handling compliant with GDPR
- [ ] Data subject rights implementable
- [ ] Data processing documented
- [ ] Third-party sharing disclosed
- [ ] Breach notification process defined

### 9.3 Security Standards

- [ ] OWASP Top 10 addressed
- [ ] CWE/SANS Top 25 considered
- [ ] Industry best practices followed
- [ ] Security training for developers
- [ ] Third-party dependencies audited

---

## 10. Vulnerability Management

### 10.1 Dependency Scanning

- [ ] `cargo-audit` run regularly
- [ ] `cargo-deny` configured
- [ ] Dependabot or similar enabled
- [ ] Critical vulnerabilities patched within 24h
- [ ] High vulnerabilities patched within 1 week

### 10.2 Code Analysis

- [ ] Clippy security lints enabled
- [ ] SAST tools integrated in CI
- [ ] Code review includes security check
- [ ] Security patterns documented
- [ ] Unsafe code blocks justified and reviewed

### 10.3 Penetration Testing

- [ ] Regular penetration testing scheduled
- [ ] Findings tracked to resolution
- [ ] Retest after remediation
- [ ] Scope includes all external interfaces
- [ ] Internal interfaces tested

---

## 11. Emergency Procedures

### 11.1 Incident Response

- [ ] Incident response plan documented
- [ ] Contact information current
- [ ] Escalation procedures defined
- [ ] Communication templates ready
- [ ] Post-incident review process

### 11.2 Kill Switch

Reference: `AGENTS.md` - Risk Management Protocol

- [ ] Global kill switch functional
- [ ] Propagates within 100ms
- [ ] Cancels all active orders
- [ ] Stops all trading algorithms
- [ ] Triggers administrator notification

### 11.3 Credential Compromise

- [ ] Rotation procedure documented
- [ ] Can revoke all sessions
- [ ] Can disable compromised accounts
- [ ] Audit of compromised credential usage
- [ ] Communication to affected parties

---

## 12. Pre-Production Checklist

Before deploying to production, verify:

### Critical Items

- [ ] All authentication mechanisms tested
- [ ] Encryption properly configured
- [ ] Rate limiting operational
- [ ] Audit logging enabled
- [ ] Kill switch tested
- [ ] No debug/test credentials
- [ ] TLS certificates valid
- [ ] Monitoring and alerting configured

### Final Review

- [ ] Security audit completed
- [ ] All critical/high findings resolved
- [ ] Medium findings have remediation plan
- [ ] Security tests passing
- [ ] Documentation updated
- [ ] Incident response plan reviewed

---

## Audit Sign-off

| Auditor | Date | Scope | Result |
|---------|------|-------|--------|
| | | | |

---

*This checklist should be reviewed and updated quarterly or when significant changes occur to the system.*
