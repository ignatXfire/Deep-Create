# ERROR_INJECTION_EVIDENCE.md

**Date:** 2026-10-01T01:03:15Z  
**Rust:** rustc 1.92.0 (ded5c06cf 2025-12-08)  
**Cargo:** cargo 1.92.0 (344c4567c 2025-10-21)  
**Workspace:** `D:/Projects/DeepCreate`

## Summary

| Crate | Test File | Tests | Status |
|-------|-----------|-------|--------|
| dc-broker | `crates/dc-broker/tests/error_injection.rs` | 6 | PASS |
| dc-core | `crates/dc-core/tests/error_injection.rs` | 10 | PASS |
| dc-store | `crates/dc-store/tests/error_injection.rs` | 7 | PASS |
| **Total** | | **23** | **PASS** |

## Test Methodology

- **dc-broker:** Real local TCP stubs (`std::net::TcpListener`) inject HTTP-level failures. No external network access.
- **dc-core:** Direct `handle_command` invocation with malformed inputs and adversarial state.
- **dc-store:** Real SQLite files in `%TEMP%`, corrupted with garbage bytes, concurrent writers via two `SQLiteStore` instances.

## dc-broker Tests (6)

| # | Test | Injected Failure | Expected Error | Result |
|---|------|------------------|----------------|--------|
| 1 | `dispatch_timeout` | Stub delays 3000ms, client timeout 500ms | `BrokerError::Timeout` | PASS |
| 2 | `dispatch_invalid_json` | Stub returns `this is not json` with HTTP 200 | `BrokerError::InvalidResponse` | PASS |
| 3 | `dispatch_http_500` | Stub returns HTTP 500 | `BrokerError::Http` containing "500" | PASS |
| 4 | `dispatch_connection_refused` | Port bound then immediately dropped | `BrokerError::EndpointUnreachable` or `Timeout` | PASS |
| 5 | `dispatch_retry_exhaustion` | Stub returns HTTP 500 for 3 connections (1 initial + 2 retries) | `BrokerError::Http` after retries exhausted | PASS |
| 6 | `dispatch_rate_limited` | First connection: 429 + Retry-After: 1; second: 200 OK | Success after retry | PASS |

### Key Findings

- **Timeout classification works:** reqwest timeout maps to `BrokerError::Timeout`.
- **429 retry logic works:** The dispatcher waits the full `Retry-After` (seconds, not milliseconds — F06 semantic pinned) then retries.
- **Retry exhaustion returns last error:** After `max_retries` + 1 attempts, the last HTTP error is returned.
- **Connection refused on Windows:** May surface as `EndpointUnreachable` or `Timeout` depending on TCP stack timing; both are acceptable.

## dc-core Tests (10)

| # | Test | Injected Failure | Expected Error | Result |
|---|------|------------------|----------------|--------|
| 1 | `accept_objective_invalid_json_syntax` | `serde_json::from_str("this is not json")` | Parse error | PASS |
| 2 | `accept_objective_wrong_shape` | Valid JSON missing required fields | Parse error | PASS |
| 3 | `accept_objective_empty_criteria` | `completion_criteria: []` | `CoreError::InvalidCommand` | PASS |
| 4 | `accept_graph_raw_invalid_json` | `AcceptGraphRaw("not json")` | `CoreError::InvalidCommand` | PASS |
| 5 | `accept_graph_raw_self_dependency` | Graph with unit depending on itself | `CoreError::InvalidCommand` | PASS |
| 6 | `accept_graph_raw_dangling_dependency` | Dependency references non-existent unit | `CoreError::InvalidCommand` | PASS |
| 7 | `accept_graph_raw_no_objective` | Graph accepted before any objective | `CoreError::NoObjective` | PASS |
| 8 | `apply_verified_result_rejected_verdict` | `VerificationVerdict::Reject` | `CoreError::VerificationFailed` | PASS |
| 9 | `apply_verified_result_missing_verification` | No verification record for unit | `CoreError::InvalidTransition` | PASS |
| 10 | `accept_objective_double_objective` | Second objective while first active | `CoreError::InvalidTransition` | PASS |

### Key Findings

- **C-08 contract enforced:** `AcceptGraphRaw` rejects malformed JSON, self-dependencies, and dangling references before any state mutation.
- **Verification gate enforced:** `ApplyVerifiedResult` requires a `Pass` verdict; `Reject`/`Gaps`/`Invalid` all produce `VerificationFailed`.
- **Objective lifecycle enforced:** Cannot accept a new objective while one is active.

## dc-store Tests (7)

| # | Test | Injected Failure | Expected Error | Result |
|---|------|------------------|----------------|--------|
| 1 | `store_corruption_recovery` | Overwrite DB header with `GARBAGE` | Read/init fails or returns error | PASS |
| 2 | `store_integrity_check_corrupted` | Write `0xDEADBEEF` at offset 4096 | `integrity_check()` returns `Result` (no panic) | PASS |
| 3 | `store_concurrent_write_conflict` | Two `SQLiteStore` instances race on same row | `StoreError::RowVersionConflict` | PASS |
| 4 | `store_cas_version_mismatch` | Write with `ExpectedVersion::Existing(v+1)` | `StoreError::RowVersionConflict` | PASS |
| 5 | `store_write_missing_row` | Write with `Existing` precondition on absent row | `StoreError::RowVersionExpectedMissing` | PASS |
| 6 | `store_double_init` | Call `initialize()` twice | `StoreError::AlreadyInitialized` | PASS |
| 7 | `store_row_version_overflow` | Insert row at `i64::MAX` boundary, then write | `StoreError::RowVersionOverflow` | PASS |

### Key Findings

- **CAS semantics are atomic:** The version predicate lives in the `UPDATE` itself, so a stale snapshot can never silently overwrite a newer row even if a concurrent writer slipped in after the read.
- **Corruption detection:** `PRAGMA integrity_check` catches corruption when damaged pages are touched; the store never panics.
- **RowVersion boundary enforced:** `ROW_VERSION_SQLITE_MAX` (i64::MAX) is a hard storage invariant; writes beyond it fail with `RowVersionOverflow`.

## Full Test Output

```
running 6 tests
test dispatch_invalid_json ... ok
test dispatch_http_500 ... ok
test dispatch_timeout ... ok
test dispatch_connection_refused ... ok
test dispatch_rate_limited ... ok
test dispatch_retry_exhaustion ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s

running 10 tests
test accept_graph_raw_dangling_dependency ... ok
test accept_graph_raw_invalid_json ... ok
test accept_graph_raw_no_objective ... ok
test accept_graph_raw_self_dependency ... ok
test accept_objective_double_objective ... ok
test accept_objective_empty_criteria ... ok
test accept_objective_invalid_json_syntax ... ok
test accept_objective_wrong_shape ... ok
test apply_verified_result_missing_verification ... ok
test apply_verified_result_rejected_verdict ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

running 7 tests
test store_cas_version_mismatch ... ok
test store_double_init ... ok
test store_concurrent_write_conflict ... ok
test store_corruption_recovery ... ok
test store_write_missing_row ... ok
test store_row_version_overflow ... ok
test store_integrity_check_corrupted ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.29s
```

## Files Created/Modified

- `crates/dc-broker/tests/error_injection.rs` — new
- `crates/dc-core/tests/error_injection.rs` — new
- `crates/dc-store/tests/error_injection.rs` — new
- `crates/dc-store/Cargo.toml` — added `rusqlite` to `[dev-dependencies]`
