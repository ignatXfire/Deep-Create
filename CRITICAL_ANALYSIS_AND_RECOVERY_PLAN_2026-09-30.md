# Deep Create — Critical Analysis & Recovery Plan
**Date:** 2026-09-30
**Analyst:** Hermes Agent (self-critical, evidence-based)
**HEAD:** b194df4 (main), f80fb9d (candidate)

---

## Executive Summary

**The independent review (2026-09-29) is CORRECT.** My previous reports were optimistic. The real readiness is **~25-35%**, not 65-75%.

**Critical finding:** The system has working Rust code with 428 passing tests, BUT the **user-facing path is fundamentally broken** at multiple independent points. The tests pass because they test isolated components, not the integrated flow.

**Root cause of past failures:** I treated symptoms (fixing test counts, architecture checks) instead of verifying the **actual end-to-end user path**. I was a function executing tasks, not an engineer verifying outcomes.

---

## Verified Facts (Live Evidence)

| Check | Command | Result |
|-------|---------|--------|
| Tests | `cargo test --workspace --offline` | ✅ 428 passed, 0 failed |
| Architecture | `node tools/check-architecture.mjs` | ✅ `ok=true` |
| Compilation | `cargo check --workspace` | ✅ 0 errors |
| Git state | `git status` | ⚠️ Untracked audit files only |

---

## Critical Defects (Confirmed Live)

### F01 · P1 · UI ↔ Rust ID Mismatch (BLOCKS ALL USER FLOWS)

**Evidence:**
- `ui/src/application-info.ts:170-179` generates UUIDs with dashes: `00000000-0000-7000-8000-000000000001`
- `crates/dc-domain/src/ids.rs:36-45` accepts ONLY 32 lowercase hex digits WITHOUT dashes
- `EntityId::try_from` rejects dashed format: `value.len() != 32 || !value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))`

**Impact:** `submit_objective`, `plan_objective_graph`, `accept_graph` ALL fail on deserialization.

**Additional mismatch:** Tauri IPC argument naming:
- UI calls: `invoke("submit_objective", { objective_json: ... })`
- Rust expects: `fn submit_objective(objective_json: String)` — Tauri v2 uses camelCase by default
- Actual parameter name received: `objectiveJson` vs expected `objective_json`

**Fix required:**
1. UI must send 32-hex-char IDs (no dashes)
2. Either add `#[tauri::command(rename_all = "snake_case")]` or change UI to camelCase

### F02 · P1 · Persistence Panic on Startup (BLOCKS PRODUCTION)

**Evidence:**
- `crates/dc-desktop/src/main.rs:309`: `EntityId::from_u128(1).unwrap()`
- `crates/dc-domain/src/ids.rs:22-25`: `from_u128` checks UUIDv7 version bits — `(value >> 76) & 0xf != 7` returns `None` for `1`
- Result: `.unwrap()` panics

**Impact:** Setting `DC_STORE_PATH` env var crashes the app on startup.

**Fix required:** Use a valid UUIDv7 constant for singleton state ID, or make persistence optional with graceful degradation.

### F03 · P1 · Fake Verification (ACCEPTS GARBAGE)

**Evidence:**
- `crates/dc-desktop/src/executor.rs:142-153`: Creates `ResultPacket` with:
  - `result_id: EntityId::from_u128(i + 1000).unwrap()` — INVALID (not UUIDv7)
  - `attempt_id: EntityId::from_u128(i + 2000).unwrap()` — INVALID
  - `snapshot_id: EntityId::from_u128(i + 3000).unwrap()` — INVALID
  - `fingerprint: [0u8; 32]` — ZEROS
- `crates/dc-core/src/verification.rs:57-130`: `run_machine_verification` checks ONLY `stage == Submitted`, creates empty `findings`, empty `criterion_ids`, `QualificationStatus::Qualified`

**Impact:** Any result is "verified" and "accepted" without checking content.

**Fix required:**
1. Real fingerprint computation (hash of content)
2. Real verification criteria from `unit.acceptance_criteria`
3. Actual content validation

### F04 · P1 · No Real Task Sent to Model

**Evidence:**
- `crates/dc-desktop/src/executor.rs:110-111`: `format!("{:?}", u.expected_output)` — sends Debug format of OutputContract struct
- `crates/dc-core/src/planner.rs:54-59`: Creates single Research unit with `root_result: None`
- `dc-broker/src/dispatch.rs:121-128`: Sends only system_prompt + user_prompt to model

**Impact:** Model receives garbage prompt like `OutputContract { export_name: "test", schema_version: 1, fields: [] }`

### F05 · P1 · Retry Storm (12 attempts instead of 3)

**Evidence:**
- `crates/dc-desktop/src/executor.rs:59-91`: 3 attempts with backoff
- `crates/dc-broker/src/dispatch.rs:138`: `0..=max_retries` where default is 3
- Combined: up to 12 HTTP POSTs per unit

### F06 · P2 · Retry-After Interpreted as Milliseconds

**Evidence:**
- `crates/dc-broker/src/dispatch.rs:175-185`: `Duration::from_millis(retry_after)`
- RFC 9110: Retry-After is in SECONDS

### F07 · P1 · AppContainer Not Actually Applied

**Evidence:**
- `dc-windows/src/job_os.rs:99-109`: Creates profile, attaches existing PID
- `appcontainer.rs:149-163`: `is_process_appcontainer` returns `Ok(false)` without checking token
- No `CreateProcess` with AppContainer token

### F09 · P1 · Event History Can Be Overwritten

**Evidence:**
- `dc-store/src/sqlite.rs:794,900,956`: `INSERT OR REPLACE` for events/evidence/checkpoints
- `dc-core/src/persistence.rs:239-245`: Event ID = `format!("ev-{}-{}", entity_id, now)` — collision overwrites

### F10 · P1/P2 · Unknown DB Schema Version Accepted

**Evidence:**
- `sqlite.rs:62-171`: `INSERT OR IGNORE schema_version=2`, no validation of existing version

### F12 · P1 · Tests Make Real Network Calls

**Evidence:**
- `dc-broker/tests/dispatch.rs:19-35`: `dispatch_stub_success` sends real HTTP to localhost
- `live_dispatch_against_local_omniroute` hits real gateway

---

## What Actually Works

| Component | Status | Evidence |
|-----------|--------|----------|
| Domain types | ✅ | Serialize-only, compile_fail tests |
| Store backend | ✅ | SQLite with WAL, FK, CAS |
| Protocol codec | ✅ | Bounded frames, replay cache |
| Architecture checker | ✅ | Clean boundaries |
| Test suite | ✅ | 428 tests pass (isolated) |

---

## Recovery Plan (Priority Order)

### Phase 1: Fix the Vertical Slice (User Can Actually Use)

**Goal:** One objective → plan → accept → execute → real verification → persist

1. **Fix F01 (ID mismatch)**
   - Change UI to send 32-hex IDs
   - Add `rename_all = "snake_case"` to all Tauri commands
   - Test: submit → plan → accept works

2. **Fix F02 (persistence panic)**
   - Replace `EntityId::from_u128(1)` with valid UUIDv7
   - Test: `DC_STORE_PATH=/tmp/test.db` starts without panic

3. **Fix F03 (fake verification)**
   - Generate real fingerprints (SHA-256 of content)
   - Pass real acceptance criteria to verification
   - Return `Indeterminate` when criteria missing
   - Test: empty content rejected, valid content accepted

4. **Fix F04 (real task to model)**
   - Extract `statement` from ObjectiveContract
   - Send actual task description to model
   - Test: model receives meaningful prompt

### Phase 2: Harden the Core

5. **Fix F05/F06 (retry logic)**
   - Single retry owner (broker only)
   - Correct Retry-After parsing (seconds)
   - Distinguish proven-failure vs unknown

6. **Fix F09/F10 (store integrity)**
   - INSERT-only for events (no REPLACE)
   - Schema version validation
   - Test: duplicate event ID rejected

### Phase 3: Isolation & Production

7. **Fix F07 (AppContainer)**
   - Real process creation with AppContainer token
   - Verify token contains AppContainer SID

8. **Fix F12 (test isolation)**
   - Mock HTTP server for tests
   - Opt-in for live tests

9. **Crash recovery test**
   - Kill process mid-execution
   - Verify state restored correctly

---

## Timeline Estimate

| Phase | Effort | Confidence |
|-------|--------|------------|
| Phase 1 | 4-6 hours | High — clear defects, clear fixes |
| Phase 2 | 3-4 hours | Medium — need to understand broker retry contract |
| Phase 3 | 6-8 hours | Lower — Windows APIs, test infrastructure |

**Total to "user can run one objective":** ~8-12 hours
**Total to "production ready":** ~20-30 hours

---

## What I Will NOT Do

- Claim readiness without running the actual user flow
- Trust test counts as proof of correctness
- Rewrite working code that passes tests
- Skip verification of each fix with live evidence

---

## Next Actions (Immediate)

1. Fix F01 (ID mismatch) — UI + Tauri commands
2. Fix F02 (persistence panic) — valid EntityId
3. Write integration test: submit → plan → accept → execute
4. Fix F03 (fake verification) — real criteria checking

**I will report after each fix with:**
- Files changed
- Test evidence
- Git commit hash
- Remaining gaps
