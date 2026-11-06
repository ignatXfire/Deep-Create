//! C-02 sender-authority boundary.
//!
//! After [`crate::decode_envelope`] has produced a candidate
//! [`crate::DecodedEnvelope`] and C-12 has produced an [`AuthenticatedPeer`]
//! for the channel that carried it, this module decides whether *this* sender
//! may propose *this* command on *this* installation and run generation. The
//! success output is a [`SenderValidation`]: only its `Fresh` arm carries a
//! [`ValidatedCommand`] that the downstream state machine may execute, and
//! constructing one outside this module is impossible by visibility — the
//! fields are private and the only constructor is [`validate_sender`].
//!
//! Closed authority model (document 19 §C-02):
//! - the role table is a closed, exhaustive [`PeerRole`] × command-kind matrix,
//!   not a string-keyed map and not loaded from configuration text;
//! - the refusal set is closed: [`ValidateSenderError`] — `WrongRole`,
//!   `WrongInstallation`, `StaleGeneration`, `ReplayConflict`,
//!   `ReplayCapacityExhausted`;
//! - replay of the same `command_id` with byte-identical wire body is a
//!   [`SenderValidation::Duplicate`]: never re-executed, routed to the durable
//!   C-03 outcome lookup (idempotent redelivery);
//! - replay of the same `command_id` with a *different* body is a hard
//!   `ReplayConflict`, never silently coerced.
//!
//! Non-goals (still owned elsewhere):
//! - durable replay storage: the [`ReplayCache`] here is in-memory only; C-03
//!   owns the durable ledger. Process restart MUST NOT treat this cache as
//!   evidence of prior acceptance.
//! - cryptography and channel identity: C-12.
//! - transition validity and current state: C-05.

use dc_domain::ids::EntityId;
use dc_domain::work::UnitAttemptCommand;
use std::collections::HashMap;
use thiserror::Error;

use crate::{DecodedEnvelope, Envelope, ProtocolContext};

/// Closed set of peer roles a channel can be authenticated as.
///
/// One variant per process kind in the C-02 trust topology. The set is
/// intentionally NOT `#[non_exhaustive]`: adding a new kind of peer is a
/// protocol change and must produce a compile error at every match site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeerRole {
    /// C-05/C-06 managing process: drives the attempt lifecycle.
    Controller,
    /// Sandboxed executor process: reports its own observations back.
    Executor,
    /// C-09 broker: relays confirmed outcomes, never initiates lifecycle.
    Broker,
    /// Watchdog: only reports process loss.
    Watchdog,
}

/// Closed command-kind tag, one per [`UnitAttemptCommand`] variant.
///
/// Used as the column index in the authority matrix. Carrying the full
/// `UnitAttemptCommand` here would force every entry to repeat entity data;
/// the kind alone is what authority is keyed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
    Prepare,
    Start,
    ObserveSubmission,
    RequestCancellation,
    ConfirmCancellation,
    ObserveLost,
}

impl CommandKind {
    fn of(command: &UnitAttemptCommand) -> Self {
        match command {
            UnitAttemptCommand::Prepare { .. } => Self::Prepare,
            UnitAttemptCommand::Start { .. } => Self::Start,
            UnitAttemptCommand::ObserveSubmission { .. } => Self::ObserveSubmission,
            UnitAttemptCommand::RequestCancellation { .. } => Self::RequestCancellation,
            UnitAttemptCommand::ConfirmCancellation { .. } => Self::ConfirmCancellation,
            UnitAttemptCommand::ObserveLost { .. } => Self::ObserveLost,
        }
    }
}

/// Closed refusal set for [`validate_sender`].
///
/// The variant names and count are part of the protocol contract
/// (document 19 §C-02). Do not collapse them into a generic "denied" —
/// downstream audit and safe-incident reporting distinguish them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ValidateSenderError {
    /// The authenticated role may not propose this command kind.
    #[error("sender role may not propose this command")]
    WrongRole,
    /// The envelope targets a different installation than the local context.
    #[error("envelope targets another installation")]
    WrongInstallation,
    /// The envelope belongs to an older or newer run generation.
    #[error("envelope belongs to another run generation")]
    StaleGeneration,
    /// A different body was seen before under the same command identifier.
    #[error("command identifier was seen with a different body")]
    ReplayConflict,
    /// The bounded in-memory replay cache is full; the command cannot be
    /// admitted without silently evicting a prior fingerprint.
    ///
    /// This is a capacity/backpressure signal, NOT a security conflict: the
    /// safe-incident pipeline must record it differently from
    /// [`Self::ReplayConflict`]. The caller's backstop is the durable C-03
    /// dedup ledger — retry after the durable check, never bypass it.
    #[error("replay cache capacity exhausted")]
    ReplayCapacityExhausted,
}

/// Authenticated channel identity handed to C-02 by C-12.
///
/// The role is *attested by the authenticator* (C-12); it is NOT a string
/// claim inside the wire message. Construction outside this crate is
/// impossible: the fields are private and the only constructor is
/// [`AuthenticatedPeer::new`], which is `pub(crate)`. Tests in this crate
/// build peers via [`AuthenticatedPeer::for_test`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticatedPeer {
    role: PeerRole,
    peer_id: EntityId,
}

impl AuthenticatedPeer {
    /// Production constructor for the crate's own authentication layer
    /// (`auth` module). The proof-verification result (an attested role plus
    /// peer identity) becomes a channel identity only through here.
    ///
    /// Unused until C-12 wires channel attestation to a `PeerRole`; kept
    /// `pub(crate)` so no external code can mint a peer.
    #[allow(dead_code)]
    pub(crate) fn new(role: PeerRole, peer_id: EntityId) -> Self {
        Self { role, peer_id }
    }

    /// Test-only constructor for downstream crates. Not compiled in
    /// production builds; integration tests use their own fixture.
    ///
    /// C-12 will get a `pub(crate)` constructor when the authenticated
    /// channel layer lands. Until then, no production caller exists.
    #[cfg(any(test, feature = "test-peer-constructor"))]
    pub fn for_test(role: PeerRole, peer_id: EntityId) -> Self {
        Self { role, peer_id }
    }

    pub fn role(&self) -> PeerRole {
        self.role
    }

    pub fn peer_id(&self) -> EntityId {
        self.peer_id
    }
}

/// The only executable output of [`validate_sender`].
///
/// Carries exactly the fields the downstream state machine may rely on:
/// - the candidate envelope (already shape-checked by `decode_envelope`);
/// - the authenticated role (already authority-checked against the kind);
/// - the exact wire bytes that decoded to this envelope, used as the replay
///   fingerprint. The binding between envelope and bytes is established by
///   [`crate::decode_envelope`] and is unrepresentable to fake.
#[derive(Debug, Clone)]
pub struct ValidatedCommand {
    envelope: Envelope,
    role: PeerRole,
    canonical_bytes: Vec<u8>,
}

impl ValidatedCommand {
    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    pub fn role(&self) -> PeerRole {
        self.role
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

/// The C-02 authority decision handed to the consumer.
///
/// Audit remediation (2026-09-17): previously `validate_sender` returned the
/// same `ValidatedCommand` for a first delivery and a byte-identical
/// redelivery, so the consumer could not tell "new command, execute it" from
/// "duplicate, do NOT re-execute". The two cases are now distinct types:
///
/// - [`SenderValidation::Fresh`] — first sight of this `command_id`. This is
///   the ONLY arm the C-05 state machine may execute.
/// - [`SenderValidation::Duplicate`] — byte-identical redelivery of an
///   already-admitted `command_id`. The consumer MUST NOT execute; it routes
///   to the durable C-03 outcome ledger and returns the previously recorded
///   outcome for this command identity.
#[derive(Debug, Clone)]
pub enum SenderValidation {
    /// First admission of this command identity; safe to execute once.
    Fresh(ValidatedCommand),
    /// Byte-identical redelivery; look up the recorded outcome, never execute.
    Duplicate {
        /// The duplicated command identity, for the C-03 outcome lookup.
        command_id: EntityId,
    },
}

/// Closed authority matrix.
///
/// Constructed at compile time; runtime mutation is not representable. The
/// default table below encodes the contract from document 19 §C-02:
///
/// | Command              | Controller | Executor | Broker | Watchdog |
/// |----------------------|------------|----------|--------|----------|
/// | Prepare              | ✓          |          |        |          |
/// | Start                | ✓          |          |        |          |
/// | ObserveSubmission    |            | ✓        |        |          |
/// | RequestCancellation  | ✓          |          |        |          |
/// | ConfirmCancellation  | ✓          |          | ✓      |          |
/// | ObserveLost          |            |          | ✓      | ✓        |
///
/// Broker may relay `ConfirmCancellation` and `ObserveLost` because those
/// observations originate outside the managing process; Broker may NOT
/// initiate lifecycle transitions. Watchdog is loss-only.
#[derive(Debug, Clone, Copy)]
pub struct RoleTable {
    /// [role][kind] → allowed. Fixed-size array; no string lookup, no config.
    allowed: [[bool; 6]; 4],
}

impl RoleTable {
    /// The closed production table. There is no other constructor for
    /// production code; integration tests that need an altered table must
    /// construct it through [`RoleTable::for_test_override`] under the
    /// `test-peer-constructor` feature.
    pub const fn production() -> Self {
        use CommandKind::*;
        use PeerRole::*;
        let mut allowed = [[false; 6]; 4];
        // Controller
        allowed[Controller as usize][Prepare as usize] = true;
        allowed[Controller as usize][Start as usize] = true;
        allowed[Controller as usize][RequestCancellation as usize] = true;
        allowed[Controller as usize][ConfirmCancellation as usize] = true;
        // Executor
        allowed[Executor as usize][ObserveSubmission as usize] = true;
        // Broker
        allowed[Broker as usize][ConfirmCancellation as usize] = true;
        allowed[Broker as usize][ObserveLost as usize] = true;
        // Watchdog
        allowed[Watchdog as usize][ObserveLost as usize] = true;
        Self { allowed }
    }

    /// Test-only override. Not compiled in production.
    #[cfg(any(test, feature = "test-peer-constructor"))]
    pub fn for_test_override(allowed: [[bool; 6]; 4]) -> Self {
        Self { allowed }
    }

    fn is_allowed(&self, role: PeerRole, kind: CommandKind) -> bool {
        self.allowed[role as usize][kind as usize]
    }
}

/// In-memory replay cache. NOT durable; C-03 owns the durable ledger.
///
/// Semantics (document 19 §C-02):
/// - first sight of `command_id` → record the wire bytes, return
///   [`ReplayCheck::FirstSeen`]; the caller proceeds to the state machine.
/// - same `command_id`, byte-identical wire body → [`ReplayCheck::Duplicate`];
///   caller MUST NOT re-execute. The previously recorded outcome (if any) is
///   returned by the caller via the durable C-03 ledger, not by this cache:
///   the cache does not own outcomes.
/// - same `command_id`, different body → [`ReplayCheck::Conflict`]; caller
///   maps to [`ValidateSenderError::ReplayConflict`].
/// - cache at capacity with an unknown `command_id` →
///   [`ReplayCheck::CapacityExhausted`]; caller maps to
///   [`ValidateSenderError::ReplayCapacityExhausted`]. This is deliberately
///   NOT `Conflict`: exhaustion is a backpressure event, not evidence that a
///   clashing body exists.
///
/// Capacity is bounded and the cache never silently evicts. Losing an entry
/// would misclassify a redelivery as `FirstSeen`; refusing admission keeps the
/// recorded fingerprints authoritative, and the durable C-03 ledger is the
/// backstop for anything this cache cannot hold. Process restart MUST NOT
/// treat this cache as evidence of prior acceptance — restart state comes
/// from C-03 alone.
#[derive(Debug)]
pub struct ReplayCache {
    entries: HashMap<EntityId, Vec<u8>>,
    capacity: usize,
}

/// Outcome of a [`ReplayCache::check_and_record`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayCheck {
    FirstSeen,
    Duplicate,
    Conflict,
    /// Cache is at capacity and the command id is unknown. Fail-closed:
    /// admission is refused rather than evicting a recorded fingerprint.
    CapacityExhausted,
}

impl ReplayCache {
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0, "replay cache capacity must be non-zero");
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    /// Check `command_id` against `canonical_bytes`. On `FirstSeen`, record.
    /// On `Conflict`, do NOT overwrite the recorded body: the first body seen
    /// under an identifier stays authoritative for future comparisons.
    pub fn check_and_record(
        &mut self,
        command_id: EntityId,
        canonical_bytes: &[u8],
    ) -> ReplayCheck {
        // Read first (immutable borrow ends before any mutable one begins).
        if let Some(existing) = self.entries.get(&command_id) {
            return if existing.as_slice() == canonical_bytes {
                ReplayCheck::Duplicate
            } else {
                ReplayCheck::Conflict
            };
        }
        if self.entries.len() >= self.capacity {
            // Fail closed with a DISTINCT outcome: refuse to silently evict,
            // and do not masquerade exhaustion as a body conflict.
            return ReplayCheck::CapacityExhausted;
        }
        self.entries.insert(command_id, canonical_bytes.to_vec());
        ReplayCheck::FirstSeen
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_full(&self) -> bool {
        self.entries.len() >= self.capacity
    }
}

/// The C-02 authority decision.
///
/// Inputs:
/// - `decoded` — the [`DecodedEnvelope`] produced by
///   [`crate::decode_envelope`]: parsed candidate inseparably bound to the
///   exact wire bytes it came from. There is no way to hand this function an
///   envelope paired with foreign bytes; that class of replay-fingerprint
///   poisoning is unrepresentable.
/// - `peer` — authenticated channel identity from C-12;
/// - `context` — local installation and run generation;
/// - `table` — closed role table, normally `RoleTable::production()`;
/// - `replay` — in-memory replay cache (durable owner is C-03).
///
/// Order of checks matters and is fixed:
///   1. installation match (cheap, independent of role);
///   2. run-generation match (cheap, independent of role);
///   3. role permits this command kind (closed table);
///   4. replay check (mutates the cache only on first sight).
///
/// Replay runs LAST so a command that fails 1–3 never consumes a cache slot
/// and never poisons replay comparison for a later well-formed redelivery.
///
/// Returns [`SenderValidation::Fresh`] ONLY on first admission — the sole
/// case the state machine may execute. A byte-identical redelivery returns
/// [`SenderValidation::Duplicate`] and MUST be answered from the durable C-03
/// outcome ledger, never re-executed.
pub fn validate_sender(
    decoded: DecodedEnvelope,
    peer: &AuthenticatedPeer,
    context: &ProtocolContext,
    table: &RoleTable,
    replay: &mut ReplayCache,
) -> Result<SenderValidation, ValidateSenderError> {
    let (envelope, canonical_bytes) = decoded.into_parts();
    if envelope.installation_id != context.installation_id {
        return Err(ValidateSenderError::WrongInstallation);
    }
    if envelope.expected.run_generation != context.run_generation {
        return Err(ValidateSenderError::StaleGeneration);
    }
    let kind = CommandKind::of(&envelope.command);
    if !table.is_allowed(peer.role, kind) {
        return Err(ValidateSenderError::WrongRole);
    }
    match replay.check_and_record(envelope.command_id, &canonical_bytes) {
        ReplayCheck::FirstSeen => Ok(SenderValidation::Fresh(ValidatedCommand {
            envelope,
            role: peer.role,
            canonical_bytes,
        })),
        ReplayCheck::Duplicate => Ok(SenderValidation::Duplicate {
            command_id: envelope.command_id,
        }),
        ReplayCheck::Conflict => Err(ValidateSenderError::ReplayConflict),
        ReplayCheck::CapacityExhausted => Err(ValidateSenderError::ReplayCapacityExhausted),
    }
}
