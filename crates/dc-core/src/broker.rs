//! C-09 broker integration: dc-core dispatches through dc-broker via protocol.
//!
//! This module provides the bridge between dc-core's scheduling and dc-broker's
//! network dispatch. It uses dc-protocol for secure inter-process communication,
//! NOT direct library calls.

use dc_domain::ids::EntityId;
use dc_domain::work::{ExpectedLifecycleBasis, UnitAttemptCommand, WorkUnitSpec};
use dc_protocol::{CommandOutcome, Envelope, OutcomeEnvelope};

use crate::error::CoreError;
use dc_worker::{ExecuteRequest, ExecuteResult, WorkDispatcher, WorkerError};

/// Broker client that dispatches through protocol
pub struct BrokerClient {
    installation_id: EntityId,
    /// Run generation pins the epoch for anti-rollback in the future IPC
    /// envelope path (C-09). Not yet embedded in envelopes — kept to pin the
    /// intended wiring.
    #[allow(dead_code)]
    run_generation: dc_domain::ids::RunGeneration,
}

impl BrokerClient {
    pub fn new(installation_id: EntityId, run_generation: dc_domain::ids::RunGeneration) -> Self {
        Self {
            installation_id,
            run_generation,
        }
    }

    /// Create a dispatch envelope for the broker
    pub fn create_dispatch_envelope(
        &self,
        command_id: EntityId,
        expected: ExpectedLifecycleBasis,
        _unit: &WorkUnitSpec,
        attempt_id: EntityId,
    ) -> Envelope {
        Envelope {
            command_id,
            installation_id: self.installation_id,
            expected,
            command: UnitAttemptCommand::Prepare {
                new_attempt_id: attempt_id,
            },
        }
    }

    /// Process broker outcome
    pub fn process_outcome(&self, outcome: &OutcomeEnvelope) -> Result<BrokerOutcome, CoreError> {
        match &outcome.outcome {
            CommandOutcome::Accepted => Ok(BrokerOutcome::Accepted),
            CommandOutcome::Denied(reason) => Ok(BrokerOutcome::Denied {
                reason: format!("{:?}", reason),
            }),
            CommandOutcome::UnknownOutcome { intent_id } => Ok(BrokerOutcome::UnknownOutcome {
                intent_id: *intent_id,
            }),
        }
    }
}

/// Outcome from broker dispatch
#[derive(Debug, Clone)]
pub enum BrokerOutcome {
    Accepted,
    Denied { reason: String },
    UnknownOutcome { intent_id: EntityId },
}

/// Protocol-based dispatcher that routes through dc-broker
pub struct ProtocolDispatcher {
    /// Reserved for the future IPC/named-pipe envelope path (C-09). The live
    /// dispatch path currently goes through `dc_broker::dispatch_work_unit_sync`
    /// directly (see `dc-worker/tests/live_e2e.rs`), so this field is not yet
    /// read. Kept to pin the intended wiring.
    #[allow(dead_code)]
    broker_client: BrokerClient,
}

impl ProtocolDispatcher {
    pub fn new(broker_client: BrokerClient) -> Self {
        Self { broker_client }
    }
}

impl WorkDispatcher for ProtocolDispatcher {
    fn dispatch(&self, request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
        // In a real implementation, this would:
        // 1. Serialize the ExecuteRequest
        // 2. Create a protocol envelope
        // 3. Send to dc-broker process via IPC/named pipe
        // 4. Wait for response
        // 5. Deserialize and return

        // For now, return a stub that indicates protocol dispatch
        Ok(ExecuteResult::Success(dc_worker::WorkResponse {
            content: format!(
                "PROTOCOL_DISPATCH: Would send {} to broker via dc-protocol",
                request.execution_id
            ),
            model: "protocol-stub".to_string(),
            tokens_used: None,
            finish_reason: Some("stub".to_string()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_id_n;

    #[test]
    fn broker_client_creation() {
        let client = BrokerClient::new(
            test_id_n(100),
            dc_domain::ids::RunGeneration::new(1).unwrap(),
        );
        assert_eq!(client.installation_id, test_id_n(100));
    }

    #[test]
    fn protocol_dispatcher_stub() {
        let client = BrokerClient::new(
            test_id_n(100),
            dc_domain::ids::RunGeneration::new(1).unwrap(),
        );
        let dispatcher = ProtocolDispatcher::new(client);

        let request = ExecuteRequest {
            execution_id: "test-1".to_string(),
            work_unit_spec: "{}".to_string(),
            system_prompt: "system".to_string(),
            user_prompt: "user".to_string(),
            max_tokens: 100,
            temperature: 0.5,
        };

        let result = dispatcher.dispatch(&request).unwrap();
        match result {
            ExecuteResult::Success(response) => {
                assert!(response.content.contains("PROTOCOL_DISPATCH"));
            }
            _ => panic!("expected success"),
        }
    }
}
