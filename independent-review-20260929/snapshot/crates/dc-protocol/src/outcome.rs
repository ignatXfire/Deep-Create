//! C-02 wire contract for a command's outcome (its typed result).
//!
//! Decoding an outcome creates a *candidate*, never an authenticated receipt:
//! the sender's real identity, the durable commit confirmation and the
//! provenance of an observed external effect belong to C-03/C-05/C-12, not to
//! this byte-level parse. No sender identity, authority, dispatch or I/O is
//! created here.
//!
//! The three arms of [`CommandOutcome`] are mutually exclusive and wire-distinct
//! so a receiver can never conflate a definite refusal with non-confirmation, or
//! either of them with acceptance (document 19 §4: an unknown external outcome
//! is a separate type, not an ordinary error the caller may conveniently retry).

use dc_domain::ids::{EntityId, RunGeneration};
use serde::{Deserialize, Serialize};

use super::{MAX_ENVELOPE_BYTES, PROTOCOL_VERSION, ProtocolContext, ProtocolError, TextCounter};

/// Closed refusal reasons a well-formed, correctly decoded command can receive.
///
/// This is exactly the C-02 `validate_sender` refusal set (document 19 §C-02). It
/// carries no sender identity, role claim, secret, or caller-supplied reason
/// text: semantic transition refusals are C-05's separate, equally closed set.
/// `ReplayCapacityExhausted` is deliberately distinct from `ReplayConflict`:
/// one is backpressure on a bounded in-memory cache, the other is evidence of
/// a clashing body under a reused command identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DenialReason {
    WrongRole,
    WrongInstallation,
    StaleGeneration,
    ReplayConflict,
    ReplayCapacityExhausted,
}

/// A command's outcome, with denial and unknown-outcome as distinct arms.
///
/// - [`CommandOutcome::Denied`] is a definite refusal of a well-formed command.
/// - [`CommandOutcome::UnknownOutcome`] is a separate state: the dispatched
///   external action may have taken effect but its outcome is unconfirmed. It
///   must be reconciled through the referenced `intent_id`, never silently
///   retried and never treated as either success or denial.
/// - [`CommandOutcome::Accepted`] alone signals acceptance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    Accepted,
    Denied(DenialReason),
    /// The dispatched external action's effect is unconfirmed. `intent_id`
    /// names the exact intent a reconciliation step must resolve.
    UnknownOutcome {
        intent_id: EntityId,
    },
}

/// Candidate fields only; even a successfully decoded outcome conveys no trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeEnvelope {
    pub command_id: EntityId,
    pub installation_id: EntityId,
    pub run_generation: RunGeneration,
    pub outcome: CommandOutcome,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum WireOutcome {
    Accepted {},
    Denied { reason: DenialReason },
    UnknownOutcome { intent_id: EntityId },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireOutcomeEnvelope {
    schema_version: u32,
    command_id: EntityId,
    installation_id: EntityId,
    run_generation: TextCounter,
    outcome: WireOutcome,
}

impl WireOutcomeEnvelope {
    fn into_outcome(self) -> OutcomeEnvelope {
        OutcomeEnvelope {
            command_id: self.command_id,
            installation_id: self.installation_id,
            run_generation: self.run_generation.0,
            outcome: match self.outcome {
                WireOutcome::Accepted {} => CommandOutcome::Accepted,
                WireOutcome::Denied { reason } => CommandOutcome::Denied(reason),
                WireOutcome::UnknownOutcome { intent_id } => {
                    CommandOutcome::UnknownOutcome { intent_id }
                }
            },
        }
    }

    fn from_outcome(envelope: &OutcomeEnvelope) -> Self {
        Self {
            schema_version: PROTOCOL_VERSION,
            command_id: envelope.command_id,
            installation_id: envelope.installation_id,
            run_generation: TextCounter(envelope.run_generation),
            outcome: match &envelope.outcome {
                CommandOutcome::Accepted => WireOutcome::Accepted {},
                CommandOutcome::Denied(reason) => WireOutcome::Denied { reason: *reason },
                CommandOutcome::UnknownOutcome { intent_id } => WireOutcome::UnknownOutcome {
                    intent_id: *intent_id,
                },
            },
        }
    }
}

fn check_outcome(
    envelope: &OutcomeEnvelope,
    context: &ProtocolContext,
) -> Result<(), ProtocolError> {
    if envelope.installation_id != context.installation_id {
        return Err(ProtocolError::InstallationMismatch);
    }
    if envelope.run_generation != context.run_generation {
        return Err(ProtocolError::GenerationMismatch);
    }
    // The local part of the document-14 identity-reuse rule (the part visible in
    // this frame only): a command identity must not reuse an entity named in this
    // outcome, including the reconciled intent identity. C-03 owns global reuse.
    if envelope.command_id == envelope.installation_id
        || matches!(
            &envelope.outcome,
            CommandOutcome::UnknownOutcome { intent_id } if *intent_id == envelope.command_id
        )
    {
        return Err(ProtocolError::IdentifierReuse);
    }
    Ok(())
}

/// Direct typed parsing preserves duplicate-key rejection and unknown-field
/// denial at every level. This does not authenticate the sender or confirm a
/// commit: it only yields a candidate outcome from bounded bytes.
pub fn decode_outcome(
    bytes: &[u8],
    context: &ProtocolContext,
) -> Result<OutcomeEnvelope, ProtocolError> {
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::Oversize);
    }
    let wire: WireOutcomeEnvelope = serde_json::from_slice(bytes).map_err(super::decode_error)?;
    if wire.schema_version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion);
    }
    let envelope = wire.into_outcome();
    check_outcome(&envelope, context)?;
    Ok(envelope)
}

/// Inverse of [`decode_outcome`] with the same installation, generation,
/// identity-reuse and size checks. Encoding never grants authority.
pub fn encode_outcome(
    envelope: &OutcomeEnvelope,
    context: &ProtocolContext,
) -> Result<Vec<u8>, ProtocolError> {
    check_outcome(envelope, context)?;
    let wire = WireOutcomeEnvelope::from_outcome(envelope);
    let bytes = serde_json::to_vec(&wire).map_err(|_| ProtocolError::InvalidStructure)?;
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::Oversize);
    }
    Ok(bytes)
}
