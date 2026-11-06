//! C-05 protocol integration: dc-core command/event handling.
//!
//! This module bridges dc-core's internal command processing with
//! dc-protocol's wire format for inter-process communication.

use dc_domain::ids::EntityId;
use dc_domain::work::{UnitAttemptCommand, ExpectedLifecycleBasis};
use dc_protocol::{Envelope, DecodedEnvelope, CommandOutcome, OutcomeEnvelope};
use serde::{Serialize, Deserialize};

use crate::error::CoreError;
use crate::command::CoreCommand;

/// Protocol adapter for dc-core commands
///
/// Converts between dc-protocol's wire format and dc-core's internal commands.
/// Note: dc-protocol currently only supports UnitAttemptCommand on the wire.
/// Root lifecycle and system commands are handled internally by dc-core.
#[derive(Debug, Clone)]
pub struct ProtocolAdapter {
    installation_id: EntityId,
    run_generation: dc_domain::ids::RunGeneration,
}

impl ProtocolAdapter {
    pub fn new(installation_id: EntityId, run_generation: dc_domain::ids::RunGeneration) -> Self {
        Self {
            installation_id,
            run_generation,
        }
    }

    /// Create a wire envelope from a unit attempt command
    pub fn create_envelope(
        &self,
        command_id: EntityId,
        expected: ExpectedLifecycleBasis,
        command: UnitAttemptCommand,
    ) -> Envelope {
        Envelope {
            command_id,
            installation_id: self.installation_id,
            expected,
            command,
        }
    }

    /// Process a decoded envelope into a core command
    pub fn process_envelope(&self, envelope: &DecodedEnvelope) -> Result<CoreCommand, CoreError> {
        let env = envelope.envelope();
        match &env.command {
            UnitAttemptCommand::Prepare { new_attempt_id: _ } => {
                Ok(CoreCommand::StartAttempt(env.expected.unit))
            }
            UnitAttemptCommand::Start {} => {
                // Start is handled internally, no core command needed
                Err(CoreError::Protocol("Start is internal".into()))
            }
            UnitAttemptCommand::ObserveSubmission { result_id: _ } => {
                // This would need result data from the envelope
                Err(CoreError::Protocol("ObserveSubmission requires result data".into()))
            }
            UnitAttemptCommand::RequestCancellation {} => {
                Err(CoreError::Protocol("Cancellation not directly mappable".into()))
            }
            UnitAttemptCommand::ConfirmCancellation {} => {
                Err(CoreError::Protocol("ConfirmCancellation not directly mappable".into()))
            }
            UnitAttemptCommand::ObserveLost { observation_id: _ } => {
                Err(CoreError::Protocol("ObserveLost not directly mappable".into()))
            }
        }
    }

    /// Create an outcome envelope from a core command result
    pub fn create_outcome(
        &self,
        command_id: EntityId,
        outcome: CommandOutcome,
    ) -> OutcomeEnvelope {
        OutcomeEnvelope {
            command_id,
            installation_id: self.installation_id,
            run_generation: self.run_generation,
            outcome,
        }
    }
}

/// Events emitted by dc-core for external consumers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreEvent {
    /// Work unit scheduled
    UnitScheduled { unit_id: EntityId, attempt_id: EntityId },
    /// Work unit completed
    UnitCompleted { unit_id: EntityId, result_id: EntityId },
    /// Verification completed
    VerificationCompleted { verification_id: EntityId, verdict: String },
    /// Objective completed
    ObjectiveCompleted { objective_id: EntityId },
    /// Error occurred
    Error { message: String, context: Option<String> },
}

/// Serialize a core event for transmission
pub fn serialize_event(event: &CoreEvent) -> Result<Vec<u8>, CoreError> {
    serde_json::to_vec(event).map_err(|e| CoreError::Serialization(e.to_string()))
}

/// Deserialize a core event from bytes
pub fn deserialize_event(data: &[u8]) -> Result<CoreEvent, CoreError> {
    serde_json::from_slice(data).map_err(|e| CoreError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_domain::ids::{EntityId, RunGeneration, RowVersion};
    use dc_domain::work::{ExpectedLifecycleBasis, UnitRef, WorkScope};
    use std::num::{NonZeroU32, NonZeroU64};

    fn test_id(n: u128) -> EntityId {
        EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | n)
            .expect("synthetic RFC 9562 UUIDv7")
    }

    fn test_scope() -> WorkScope {
        WorkScope {
            session_id: test_id(1),
            work_id: test_id(2),
            objective_id: test_id(3),
            objective_revision: NonZeroU32::new(1).unwrap(),
        }
    }

    #[test]
    fn protocol_adapter_creation() {
        let adapter = ProtocolAdapter::new(test_id(100), RunGeneration::new(1).unwrap());
        assert_eq!(adapter.installation_id, test_id(100));
    }

    #[test]
    fn event_serialization() {
        let event = CoreEvent::UnitScheduled {
            unit_id: test_id(201),
            attempt_id: test_id(202),
        };
        let data = serialize_event(&event).unwrap();
        let decoded: CoreEvent = deserialize_event(&data).unwrap();
        match decoded {
            CoreEvent::UnitScheduled { unit_id, attempt_id } => {
                assert_eq!(unit_id, test_id(201));
                assert_eq!(attempt_id, test_id(202));
            }
            _ => panic!("wrong event type"),
        }
    }
}
