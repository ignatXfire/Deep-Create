# Stage 3 Canonical Schema — dc-store

**Schema version: 3** (bumped from 2; existing tables unchanged, new `stage3_*` tables added).

## Design principles

Borrowed from production event-sourcing practice (see References):

1. **Append-only everywhere.** Rows in `stage3_journal`, `stage3_ledger`, `stage3_evidence` are INSERT-only. Corrections are new *compensating* entries, never UPDATE or DELETE.
2. **Per-stream ordering, no global clock.** The journal guarantees total order *within* a stream via `(stream_id, sequence)`; across streams, `created_at` is informational only.
3. **Optimistic concurrency control via UNIQUE.** `UNIQUE(stream_id, sequence)` on the journal *is* the OCC mechanism — concurrent writers race for the next sequence, the loser gets a constraint violation and retries.
4. **Projections separate from the log.** `stage3_ledger` stores entries; totals are computed at read time (`GROUP BY`), never materialized.
5. **Schema evolution at read, never rewrite.** Payloads are opaque bytes; upcasting is the reader's job. Stored events are never rewritten.
6. **Content-addressed evidence.** SHA-256 hash is the primary key of the evidence archive — immutable, self-verifying, naturally deduplicated.

## Tables

### `stage3_journal` — append-only event journal

| Column | Type | Constraint | Purpose |
|---|---|---|---|
| `event_id` | TEXT | PRIMARY KEY | Globally unique event ID |
| `stream_id` | TEXT | NOT NULL | Aggregate/stream this event belongs to |
| `sequence` | INTEGER | NOT NULL, `> 0` | Position within the stream (1-based) |
| `event_type` | TEXT | NOT NULL | Event kind (e.g. `goal_created`) |
| `actor` | TEXT | NOT NULL | Who/what caused the transition |
| `payload` | BLOB | NOT NULL | Opaque event data (JSON/Protobuf/…) |
| `payload_hash` | TEXT | NOT NULL, 64 hex chars | SHA-256 of `payload` — verified on write |
| `created_at` | INTEGER | NOT NULL | Milliseconds since Unix epoch |

Constraints:
- `UNIQUE (stream_id, sequence)` — the OCC mechanism. Two writers cannot claim the same position.
- `CHECK (payload_hash = lower(hex(sha256(payload))))` — enforced by application code on write (`JournalEvent::new` computes; `verify_hash()` validates before INSERT).

Indexes:
- `(stream_id, sequence)` — replay a stream in order.
- `(event_type)` — filter projections by kind.
- `(created_at)` — time-range queries.

### `stage3_ledger` — resource usage ledger

| Column | Type | Constraint | Purpose |
|---|---|---|---|
| `entry_id` | TEXT | PRIMARY KEY | Unique ledger entry ID |
| `work_unit_id` | TEXT | NOT NULL | The work unit (task/job) being charged |
| `entity_id` | TEXT | NOT NULL, FK → entities.id | Parent entity |
| `kind` | TEXT | NOT NULL, CHECK in enum | `tokens` / `time_ms` / `cost_micros` |
| `quantity` | INTEGER | NOT NULL, `>= 0` | Amount in the unit named by `kind` |
| `actor` | TEXT | NOT NULL | Worker/component that consumed the resource |
| `metadata` | TEXT | nullable | Optional JSON metadata |
| `created_at` | INTEGER | NOT NULL | Milliseconds since Unix epoch |

Indexes:
- `(work_unit_id)` — all charges for a work unit.
- `(entity_id)` — all charges under an entity.

**Totals are projections**, computed at read:
```sql
SELECT kind, SUM(quantity) FROM stage3_ledger WHERE work_unit_id = ?1 GROUP BY kind
```

### `stage3_evidence` — immutable evidence archive

| Column | Type | Constraint | Purpose |
|---|---|---|---|
| `content_hash` | TEXT | PRIMARY KEY, 64 hex chars | SHA-256 of `content` — the address |
| `entity_id` | TEXT | nullable, FK → entities.id | Optional binding to an entity |
| `artifact_type` | TEXT | NOT NULL | MIME-ish type (`text/log`, `image/png`, …) |
| `verifier_id` | TEXT | NOT NULL | Who produced/attested the artifact |
| `content` | BLOB | NOT NULL | Raw bytes |
| `created_at` | INTEGER | NOT NULL | Milliseconds since Unix epoch |

Properties:
- **Content-addressed**: `content_hash` is the PK. Two inserts of identical bytes are idempotent (same key, same row).
- **Self-verifying**: readers can recompute `SHA-256(content)` and compare with the key; `EvidenceArtifact::verify()` does this in code.
- **Immutable**: no UPDATE path exists in the API.

Indexes:
- `(entity_id)` — list evidence bound to an entity.

## Rust API

```rust
use dc_store::stage3::{
    JournalEvent, LedgerEntry, ResourceKind, EvidenceArtifact,
    Stage3CanonicalBackend, sha256_hex,
};

// Event journal
let ev = JournalEvent::new(id, stream_id, seq, event_type, actor, payload, created_at);
store.journal_append(&ev)?;                            // OCC-enforced, hash-verified
let stream = store.journal_read_stream("stream-A")?;   // replay in order
let head = store.journal_head_sequence("stream-A")?;   // next = head + 1

// Resource ledger
let entry = LedgerEntry { /* tokens, time_ms, cost_micros */ };
store.ledger_append(&entry)?;
let entries = store.ledger_for_work_unit("wu-1")?;
let totals = store.ledger_totals_for_work_unit("wu-1")?; // projection

// Evidence archive
let artifact = EvidenceArtifact::new(entity, type, verifier, content, created_at);
let hash = store.evidence_put(&artifact)?;   // idempotent on content
let found = store.evidence_get(&hash)?;      // content-addressed lookup
let by_entity = store.evidence_list_by_entity(&entity)?;
```

## Coexistence with Stage-2 tables

The existing `events` / `resources` / `evidence` tables (accessed via `Stage3Backend`) are **untouched**. The new `stage3_*` tables live under a separate trait `Stage3CanonicalBackend` so both can evolve independently. Migration path: new writes go to canonical tables; readers may consult legacy tables until a backfill is done.

## References

- Sujeet Jaiswal, *Event Sourcing Deep Dive* — https://sujeet.pro/articles/event-sourcing-deep-dive
- Tanh Nguyen, *Core Banking Architecture, Part 3: Event Sourcing & CQRS* — https://tanhdev.com/series/core-banking-architecture/part-3-event-sourcing-cqrs
