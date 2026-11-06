# Multi-Store Consistency Analysis

## Problem Statement

DeepCreate currently has two components that persist `authority.anchor` records:

1. **dc-core** (`crates/dc-core/src/persistence.rs:292`) — stores anchors under arbitrary `fence_scope` values
2. **dc-watchdog** (`crates/dc-watchdog/src/anchor_store.rs:44`) — stores anchors under the fixed scope `watchdog.authority.anchor`

Both write to **separate SQLite files** (dc-core uses its own store path, dc-watchdog uses another). This creates a **split-brain risk**: if dc-core and dc-watchdog disagree on the current epoch, the system may admit stale work or reject valid recovery.

## Current State

| Component | Scope | Store Path | Anti-Rollback |
|-----------|-------|-----------|---------------|
| dc-core | arbitrary (`"core"`, etc.) | configurable | yes (via `record_authority_anchor`) |
| dc-watchdog | `watchdog.authority.anchor` | configurable | yes (via `persist_anchor`) |

**Gap**: No cross-store validation. If dc-core writes epoch=10 and dc-watchdog writes epoch=5, both succeed independently.

## Solution Options

### Option A: Single Shared Store (Recommended)
Both dc-core and dc-watchdog open the **same SQLite file** and use the same `authority.anchor` table with a well-known `entity_id` (the installation ID) and `fence_scope`.

**Pros:**
- SQLite serializes writes via WAL
- Anti-rollback enforced by the same `Stage3Backend` implementation
- No distributed consensus needed

**Cons:**
- Requires careful initialization order (only one component should create tables)
- Both components must agree on the store path

### Option B: Cross-Store Validation
dc-watchdog reads dc-core's store before writing, and vice versa.

**Pros:**
- Stores can be separate files

**Cons:**
- Race conditions between read and write
- Complex failure modes (what if one store is corrupted?)

## Recommended Implementation

1. **Unify on a single store path** (e.g., `%APPDATA%/DeepCreate/deepcreate.db`)
2. **dc-core initializes the store** (creates tables, WAL mode, etc.)
3. **dc-watchdog opens the same file** with `SQLiteStore::new(path)`
4. **Both use the same `entity_id`** (installation ID) and scope `authority.anchor`
5. **Anti-rollback is enforced by whoever writes last** — the invariant is monotonic epoch, not component identity

## Verification

The existing tests already prove the anti-rollback invariant:
- `persistence_authority_anchor_round_trip` (dc-core)
- `anchor_anti_rollback_refused` (dc-watchdog)

What is missing: an integration test where **both** dc-core and dc-watchdog write to the same store and verify the epoch is monotonic across both.

## Action Items

- [ ] Add `DC_STORE_PATH` environment variable convention
- [ ] dc-core: expose `store_path()` getter
- [ ] dc-watchdog: accept store path instead of creating its own
- [ ] Integration test: shared store between dc-core and dc-watchdog
- [ ] Document the single-store invariant in architecture docs

## Status

**Not implemented.** This is a production hardening item, not a correctness bug. The system works correctly if dc-core and dc-watchdog are configured to use the same store path (which is the intended deployment).