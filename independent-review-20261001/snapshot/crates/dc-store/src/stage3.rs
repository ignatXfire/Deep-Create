//! Stage 3 canonical schema: event journal, resource ledger, evidence archive.
//!
//! Design principles (production event-sourcing, see STAGE3_SCHEMA.md):
//! - **Append-only**: rows are INSERT-only; corrections are compensating entries.
//! - **Total order per stream**, not global: `(stream_id, sequence)` UNIQUE is
//!   the optimistic-concurrency control for the journal.
//! - **Immutability with content addressing**: evidence artifacts are keyed by
//!   their SHA-256 digest — the same bytes stored twice collapse to one row.
//! - **Projections are downstream**: these tables are the source of truth;
//!   derived views (summaries, dashboards) read from them and never write back.
//!
//! These tables coexist with — and do not replace — the older `events`,
//! `resources`, `evidence` tables used by `Stage3Backend`. The new tables are
//! prefixed `stage3_` and follow the stricter canonical rules.

use dc_domain::ids::EntityId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::StoreError;

/// Compute the canonical SHA-256 hex digest of a payload.
///
/// The journal stores this alongside every event so tampering is detectable
/// without storing payload twice; the evidence archive uses it as the primary
/// key (content-addressed lookup).
pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    let digest = h.finalize();
    let mut out = String::with_capacity(64);
    for b in digest.iter() {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// (1) Event journal — append-only state transitions
// ─────────────────────────────────────────────────────────────────────────────

/// One immutable state-transition event.
///
/// - `sequence` is monotonically increasing per `stream_id`; the
///   `UNIQUE(stream_id, sequence)` constraint in SQLite IS the optimistic
///   concurrency control — two writers racing for the same next sequence
///   produce exactly one winner.
/// - `actor` records who/what produced the transition (user id, worker id,
///   system component). Required — anonymous transitions are forbidden.
/// - `payload_hash` is the SHA-256 of `payload`, denormalized so integrity
///   can be verified at read time without recomputing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEvent {
    pub event_id: String,
    /// Stream identifier — typically the aggregate/entity id the event
    /// belongs to. Scoped ordering is per-stream only.
    pub stream_id: String,
    /// Per-stream sequence number, starting at 1. Gap-free for a given
    /// stream under single-writer; the DB rejects duplicate (stream, seq).
    pub sequence: u64,
    pub event_type: String,
    /// Who performed the transition: user, worker, or system component id.
    pub actor: String,
    pub payload: Vec<u8>,
    /// SHA-256 hex of `payload`. Must match `sha256_hex(payload)`.
    pub payload_hash: String,
    /// Logical timestamp (caller-supplied; the store has no clock).
    pub created_at: u64,
}

impl JournalEvent {
    /// Build a new event, computing `payload_hash` from `payload`.
    pub fn new(
        event_id: String,
        stream_id: String,
        sequence: u64,
        event_type: String,
        actor: String,
        payload: Vec<u8>,
        created_at: u64,
    ) -> Self {
        let payload_hash = sha256_hex(&payload);
        Self {
            event_id,
            stream_id,
            sequence,
            event_type,
            actor,
            payload,
            payload_hash,
            created_at,
        }
    }

    /// Verify the stored hash matches the payload. Returns Err on mismatch.
    pub fn verify_hash(&self) -> Result<(), StoreError> {
        let actual = sha256_hex(&self.payload);
        if actual != self.payload_hash {
            return Err(StoreError::IntegrityFailure(format!(
                "event {} payload hash mismatch: stored {}, computed {}",
                self.event_id, self.payload_hash, actual
            )));
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// (2) Resource ledger — token / time / cost per work unit (append-only)
// ─────────────────────────────────────────────────────────────────────────────

/// The unit of measure for a ledger entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    /// LLM or API tokens consumed.
    Tokens,
    /// Wall-clock or CPU time in milliseconds.
    TimeMs,
    /// Monetary cost in micro-units of the account currency
    /// (e.g. micro-USD: 1 USD = 1_000_000).
    CostMicros,
}

impl ResourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceKind::Tokens => "tokens",
            ResourceKind::TimeMs => "time_ms",
            ResourceKind::CostMicros => "cost_micros",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "tokens" => Some(ResourceKind::Tokens),
            "time_ms" => Some(ResourceKind::TimeMs),
            "cost_micros" => Some(ResourceKind::CostMicros),
            _ => None,
        }
    }
}

/// One append-only resource-consumption entry against a work unit.
///
/// Aggregation (sum per work unit, per entity, per day) is a projection —
/// readers compute it; the ledger itself stays immutable. Corrections are
/// posted as new entries with negative `quantity` referencing the original
/// `entry_id` in `metadata`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub entry_id: String,
    /// The work unit this consumption is attributed to (task / step / run id).
    pub work_unit_id: String,
    /// The entity that owns the work unit.
    pub entity_id: EntityId,
    pub kind: ResourceKind,
    /// Amount consumed in the unit given by `kind`. Positive for consumption,
    /// negative only for compensating corrections.
    pub quantity: i64,
    /// Actor that consumed the resource (worker id, model id, ...).
    pub actor: String,
    /// Optional opaque metadata (e.g. `{"corrects": "<entry_id>"}`).
    pub metadata: Option<Vec<u8>>,
    pub created_at: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// (3) Evidence archive — immutable artifacts, content-addressed by SHA-256
// ─────────────────────────────────────────────────────────────────────────────

/// One immutable artifact in the evidence archive.
///
/// The primary key is the SHA-256 of `content` — identical bytes uploaded
/// twice collapse to one row (content-addressed lookup). `content` is stored
/// in-line as a BLOB; larger deployments can move bytes to external storage
/// and keep only the hash + locator, but the lookup key stays the hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceArtifact {
    /// SHA-256 hex of `content`. This is the lookup key.
    pub content_hash: String,
    /// Optional entity this artifact is bound to (for indexing/listing).
    pub entity_id: Option<EntityId>,
    /// MIME-ish type tag, e.g. "text/log", "image/png", "application/json".
    pub artifact_type: String,
    /// Who deposited the artifact.
    pub deposited_by: String,
    pub content: Vec<u8>,
    pub created_at: u64,
}

impl EvidenceArtifact {
    /// Build a new artifact, computing `content_hash` from `content`.
    pub fn new(
        entity_id: Option<EntityId>,
        artifact_type: String,
        deposited_by: String,
        content: Vec<u8>,
        created_at: u64,
    ) -> Self {
        let content_hash = sha256_hex(&content);
        Self {
            content_hash,
            entity_id,
            artifact_type,
            deposited_by,
            content,
            created_at,
        }
    }

    /// Verify integrity: stored hash must equal SHA-256 of content.
    pub fn verify(&self) -> Result<(), StoreError> {
        let actual = sha256_hex(&self.content);
        if actual != self.content_hash {
            return Err(StoreError::IntegrityFailure(format!(
                "evidence hash mismatch: stored {}, computed {}",
                self.content_hash, actual
            )));
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Store API extension
// ─────────────────────────────────────────────────────────────────────────────

/// Stage-3 canonical store extension. Separate from `Stage3Backend` (the
/// legacy typed tables) so the two can evolve independently. Only production
/// backends (SQLite) implement this.
pub trait Stage3CanonicalBackend: crate::StoreBackend {
    // ── Event journal ────────────────────────────────────────────────────
    /// Append an event. INSERT-only: duplicate `event_id` OR duplicate
    /// `(stream_id, sequence)` is an error, never a silent overwrite.
    fn journal_append(&mut self, ev: &JournalEvent) -> Result<(), StoreError>;
    /// Read events of one stream in sequence order.
    fn journal_read_stream(&self, stream_id: &str) -> Result<Vec<JournalEvent>, StoreError>;
    /// The highest sequence currently stored for a stream (None if empty).
    /// Callers use this to pick the next sequence under OCC.
    fn journal_head_sequence(&self, stream_id: &str) -> Result<Option<u64>, StoreError>;

    // ── Resource ledger ─────────────────────────────────────────────────
    /// Append a ledger entry. INSERT-only.
    fn ledger_append(&mut self, e: &LedgerEntry) -> Result<(), StoreError>;
    /// All entries for a work unit, oldest first.
    fn ledger_for_work_unit(&self, work_unit_id: &str) -> Result<Vec<LedgerEntry>, StoreError>;
    /// Sum of `quantity` per kind for a work unit (projection computed on read).
    fn ledger_totals_for_work_unit(
        &self,
        work_unit_id: &str,
    ) -> Result<Vec<(ResourceKind, i64)>, StoreError>;

    // ── Evidence archive ────────────────────────────────────────────────
    /// Deposit an artifact. Idempotent on `content_hash`: storing the same
    /// bytes twice succeeds and keeps one row. Returns the hash.
    fn evidence_put(&mut self, a: &EvidenceArtifact) -> Result<String, StoreError>;
    /// Content-addressed lookup by SHA-256 hash.
    fn evidence_get(&self, content_hash: &str) -> Result<Option<EvidenceArtifact>, StoreError>;
    /// List artifacts bound to an entity, oldest first.
    fn evidence_list_by_entity(
        &self,
        entity_id: &EntityId,
    ) -> Result<Vec<EvidenceArtifact>, StoreError>;
}
