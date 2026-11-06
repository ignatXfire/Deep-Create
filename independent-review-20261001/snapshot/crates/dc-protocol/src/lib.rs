//! C-02 bounded codec for existing C-01 unit-attempt commands and command outcomes.
//! Decoding creates a candidate, never AuthenticatedPeer or ValidatedCommand.
//! C-12 must authenticate the real sender; C-03 must durably deduplicate commands.
//! No sender identity, authority, dispatch, replay cache or I/O is created here.
//!
//! There is intentionally NO `DeepCreateCommand`-style orchestration enum here:
//! agent orchestration proposals are domain work units (C-01) and owner/channel
//! decisions (C-14), not wire commands. Payload strings (`approved_by: "Owner"`,
//! self-declared `ReportVerification`) never grant authority; only an
//! authenticated channel identity (C-12) plus the closed role table
//! ([`authority::RoleTable`]) may produce a [`authority::ValidatedCommand`].

mod auth;
mod authority;
mod outcome;

pub use auth::{
    AuthContext, AuthError, AuthenticatedEnvelope, ProofVerifier, SenderProof,
    authenticate_envelope, decode_authenticated, encode_authenticated, validate_authenticated,
};
pub use authority::{
    AuthenticatedPeer, PeerRole, ReplayCache, ReplayCheck, RoleTable, SenderValidation,
    ValidateSenderError, ValidatedCommand, validate_sender,
};
pub use outcome::{CommandOutcome, DenialReason, OutcomeEnvelope, decode_outcome, encode_outcome};

use dc_domain::ids::{EntityId, RunGeneration};
use dc_domain::work::{ExpectedLifecycleBasis, UnitAttemptCommand, UnitRef, WorkScope};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use thiserror::Error;

pub const PROTOCOL_VERSION: u32 = 1;
/// Version-one structural frame ceiling, NOT a permission or spending limit.
pub const MAX_ENVELOPE_BYTES: usize = 65_536;

/// Candidate fields only; even a successfully decoded value conveys no trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub command_id: EntityId,
    pub installation_id: EntityId,
    pub expected: ExpectedLifecycleBasis,
    pub command: UnitAttemptCommand,
}

/// A decoded envelope inseparably bound to the exact wire bytes it came from.
///
/// Audit remediation (2026-09-17): previously `validate_sender` accepted the
/// parsed [`Envelope`] and its `canonical_bytes` as two independent arguments,
/// so a caller could pair envelope A with bytes B and poison the replay
/// fingerprint under A's `command_id`. The only way to obtain a
/// `DecodedEnvelope` is [`decode_envelope`] (or [`encode_envelope`] followed
/// by [`decode_envelope`] of those bytes — the codec is deterministic, so the
/// round-trip preserves the binding). There is no public constructor from
/// detached parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedEnvelope {
    envelope: Envelope,
    original_bytes: Vec<u8>,
}

impl DecodedEnvelope {
    /// The parsed candidate envelope. Still conveys no sender trust.
    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    /// The exact wire bytes that decoded to [`Self::envelope`]. This is the
    /// only byte string the replay layer may fingerprint for this envelope.
    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }

    /// Split into parts without losing the binding: the returned tuple is
    /// still the matched pair produced by one decode. Consumed by
    /// [`authority::validate_sender`].
    pub(crate) fn into_parts(self) -> (Envelope, Vec<u8>) {
        (self.envelope, self.original_bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolContext {
    pub installation_id: EntityId,
    pub run_generation: RunGeneration,
}

/// Safe for incident reporting: never stores raw bytes, field names or errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ProtocolError {
    #[error("envelope exceeds the structural frame ceiling")]
    Oversize,
    #[error("unsupported protocol schema version")]
    UnsupportedVersion,
    #[error("malformed or truncated envelope")]
    Malformed,
    #[error("duplicate field in envelope")]
    DuplicateField,
    #[error("unknown field in envelope")]
    UnknownField,
    #[error("unknown command in envelope")]
    UnknownCommand,
    #[error("invalid envelope structure")]
    InvalidStructure,
    #[error("attempt id and row version must be paired")]
    MismatchedAttemptBasis,
    #[error("envelope targets another installation")]
    InstallationMismatch,
    #[error("envelope belongs to another run generation")]
    GenerationMismatch,
    #[error("command identifier reuses an entity named in this envelope")]
    IdentifierReuse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
struct TextCounter(NonZeroU64);

impl TryFrom<String> for TextCounter {
    type Error = ProtocolError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty()
            || value.len() > 20
            || value.starts_with('0')
            || !value.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(ProtocolError::InvalidStructure);
        }
        value
            .parse::<u64>()
            .ok()
            .and_then(NonZeroU64::new)
            .map(Self)
            .ok_or(ProtocolError::InvalidStructure)
    }
}

impl From<TextCounter> for String {
    fn from(value: TextCounter) -> Self {
        value.0.to_string()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireExpected {
    scope: WorkScope,
    unit: UnitRef,
    unit_row_version: TextCounter,
    attempt_id: Option<EntityId>,
    attempt_row_version: Option<TextCounter>,
    run_generation: TextCounter,
}

impl WireExpected {
    fn into_domain(self) -> Result<ExpectedLifecycleBasis, ProtocolError> {
        if self.attempt_id.is_some() != self.attempt_row_version.is_some() {
            return Err(ProtocolError::MismatchedAttemptBasis);
        }
        Ok(ExpectedLifecycleBasis {
            scope: self.scope,
            unit: self.unit,
            unit_row_version: self.unit_row_version.0,
            attempt_id: self.attempt_id,
            attempt_row_version: self.attempt_row_version.map(|v| v.0),
            run_generation: self.run_generation.0,
        })
    }
    fn from_domain(value: &ExpectedLifecycleBasis) -> Result<Self, ProtocolError> {
        if value.attempt_id.is_some() != value.attempt_row_version.is_some() {
            return Err(ProtocolError::MismatchedAttemptBasis);
        }
        Ok(Self {
            scope: value.scope,
            unit: value.unit,
            unit_row_version: TextCounter(value.unit_row_version),
            attempt_id: value.attempt_id,
            attempt_row_version: value.attempt_row_version.map(TextCounter),
            run_generation: TextCounter(value.run_generation),
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireEnvelope {
    schema_version: u32,
    command_id: EntityId,
    installation_id: EntityId,
    expected: WireExpected,
    command: UnitAttemptCommand,
}

fn check_context(envelope: &Envelope, context: &ProtocolContext) -> Result<(), ProtocolError> {
    if envelope.installation_id != context.installation_id {
        return Err(ProtocolError::InstallationMismatch);
    }
    if envelope.expected.run_generation != context.run_generation {
        return Err(ProtocolError::GenerationMismatch);
    }
    // Document 14 forbids identity reuse. This rejects collisions visible in
    // this frame only; C-03 still owns global identity and replay checks.
    let command_reference = match envelope.command {
        UnitAttemptCommand::Prepare { new_attempt_id } => Some(new_attempt_id),
        UnitAttemptCommand::ObserveSubmission { result_id } => Some(result_id),
        UnitAttemptCommand::ObserveLost { observation_id } => Some(observation_id),
        UnitAttemptCommand::Start {}
        | UnitAttemptCommand::RequestCancellation {}
        | UnitAttemptCommand::ConfirmCancellation {} => None,
    };
    if [
        Some(envelope.installation_id),
        Some(envelope.expected.scope.session_id),
        Some(envelope.expected.scope.work_id),
        Some(envelope.expected.scope.objective_id),
        Some(envelope.expected.unit.unit_id),
        envelope.expected.attempt_id,
        command_reference,
    ]
    .into_iter()
    .flatten()
    .any(|id| id == envelope.command_id)
    {
        return Err(ProtocolError::IdentifierReuse);
    }
    Ok(())
}

fn decode_error(error: serde_json::Error) -> ProtocolError {
    if error.is_syntax() || error.is_eof() {
        return ProtocolError::Malformed;
    }
    // Classification is tested against the locked serde version. Never expose
    // the original message: it may contain a sensitive attacker-supplied key.
    let message = error.to_string();
    if message.starts_with("duplicate field") {
        ProtocolError::DuplicateField
    } else if message.starts_with("unknown field") {
        ProtocolError::UnknownField
    } else if message.starts_with("unknown variant") {
        ProtocolError::UnknownCommand
    } else {
        ProtocolError::InvalidStructure
    }
}

/// Direct typed parsing preserves duplicate-key rejection in nested forms.
/// This does not validate sender role, command permission or durable replay.
///
/// Returns a [`DecodedEnvelope`]: the parsed candidate inseparably bound to
/// the exact wire bytes it came from. Downstream authority and replay checks
/// consume that pair, never a detached envelope plus separately supplied bytes.
pub fn decode_envelope(
    bytes: &[u8],
    context: &ProtocolContext,
) -> Result<DecodedEnvelope, ProtocolError> {
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::Oversize);
    }
    let wire: WireEnvelope = serde_json::from_slice(bytes).map_err(decode_error)?;
    if wire.schema_version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion);
    }
    let envelope = Envelope {
        command_id: wire.command_id,
        installation_id: wire.installation_id,
        expected: wire.expected.into_domain()?,
        command: wire.command,
    };
    check_context(&envelope, context)?;
    Ok(DecodedEnvelope {
        envelope,
        original_bytes: bytes.to_vec(),
    })
}

pub fn encode_envelope(
    envelope: &Envelope,
    context: &ProtocolContext,
) -> Result<Vec<u8>, ProtocolError> {
    check_context(envelope, context)?;
    let wire = WireEnvelope {
        schema_version: PROTOCOL_VERSION,
        command_id: envelope.command_id,
        installation_id: envelope.installation_id,
        expected: WireExpected::from_domain(&envelope.expected)?,
        command: envelope.command.clone(),
    };
    let bytes = serde_json::to_vec(&wire).map_err(|_| ProtocolError::InvalidStructure)?;
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::Oversize);
    }
    Ok(bytes)
}
