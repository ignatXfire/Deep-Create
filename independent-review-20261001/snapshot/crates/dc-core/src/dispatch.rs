//! C-06/C-11 dispatch integration: dc-core coordinates work execution.
//!
//! This module bridges dc-core's scheduling with dc-worker's execution.
//! dc-core creates ExecuteRequests and dispatches them through the
//! WorkDispatcher trait (implemented by dc-broker or test stubs).

use dc_domain::ids::EntityId;
use dc_domain::ids::RowVersion;
use dc_domain::work::{
    DependencySnapshot, ResultPacket, ResultStage, VersionedDependency, WorkUnitSpec,
};
use dc_worker::{ExecuteRequest, ExecuteResult, WorkDispatcher, WorkResponse};

use crate::error::CoreError;

/// Dispatcher bridge between dc-core and dc-worker
pub struct CoreDispatcher<D: WorkDispatcher> {
    dispatcher: D,
    worker_id: String,
}

impl<D: WorkDispatcher> CoreDispatcher<D> {
    pub fn new(dispatcher: D, worker_id: String) -> Self {
        Self {
            dispatcher,
            worker_id,
        }
    }

    /// Execute a work unit through the worker
    pub fn execute_unit(
        &self,
        unit: &WorkUnitSpec,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<ExecuteResult, CoreError> {
        let request = ExecuteRequest {
            execution_id: format!("{}-{}", self.worker_id, unit.unit.unit_id),
            work_unit_spec: serde_json::to_string(unit)
                .map_err(|e| CoreError::Serialization(e.to_string()))?,
            system_prompt: system_prompt.to_string(),
            user_prompt: user_prompt.to_string(),
            max_tokens: 4096,
            temperature: 0.0,
        };

        self.dispatcher
            .dispatch(&request)
            .map_err(|e| CoreError::Protocol(format!("worker dispatch failed: {}", e)))
    }

    /// Convert worker response to result packet with outcome evidence
    pub fn response_to_result(
        &self,
        unit: &WorkUnitSpec,
        _response: &WorkResponse,
        attempt_id: EntityId,
        snapshot: DependencySnapshot,
        verification_policy: VersionedDependency,
    ) -> Result<ResultPacket, CoreError> {
        // Compute real fingerprint from response content using std hash
        let mut fingerprint = [0u8; 32];
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        _response.content.hash(&mut hasher);
        let content_hash = hasher.finish();
        fingerprint[..8].copy_from_slice(&content_hash.to_le_bytes());

        // Generate unique result_id from attempt_id + unit_id to avoid collisions
        // when multiple results are created for the same unit
        let result_id = {
            let mut hasher = DefaultHasher::new();
            attempt_id.hash(&mut hasher);
            unit.unit.unit_id.hash(&mut hasher);
            let hash = hasher.finish();
            // Ensure the generated ID is a valid EntityId by setting the version/variant bits
            // UUIDv7 format: version bits at position 76-79 must be 7, variant bits at 62-63 must be 2
            let mut value = hash as u128;
            // Set version to 7 (UUIDv7)
            value = (value & !(0xF << 76)) | (0x7 << 76);
            // Set variant to 2 (RFC 4122)
            value = (value & !(0x3 << 62)) | (0x2 << 62);
            // Ensure non-zero
            if value == 0 {
                value = 1;
            }
            EntityId::from_u128(value)
                .ok_or_else(|| CoreError::Protocol("failed to generate result_id".to_string()))?
        };

        Ok(ResultPacket {
            result_id,
            attempt_id,
            scope: unit.scope,
            unit: unit.unit,
            snapshot,
            fingerprint,
            verification_policy,
            used_links: vec![],
            stage: ResultStage::Submitted,
            row_version: RowVersion::new(1).unwrap(),
        })
    }

    /// Record outcome evidence for a dispatched unit.
    ///
    /// This implements forensic fix E: every dispatch must produce
    /// durable outcome evidence, not just an enum flag.
    pub fn record_outcome_evidence(
        &self,
        unit: &WorkUnitSpec,
        result: &ResultPacket,
        observation_kind: &str,
    ) -> Result<OutcomeEvidenceRecord, CoreError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Ok(OutcomeEvidenceRecord {
            evidence_id: format!("evd-{}-{}", unit.unit.unit_id, now),
            unit_id: unit.unit.unit_id,
            result_id: result.result_id,
            observation_kind: observation_kind.to_string(),
            observed_at: now,
            stage: result.stage,
        })
    }
}

/// Outcome evidence record for dispatched work.
///
/// This is the durable evidence that a dispatch actually happened,
/// not just an enum flag in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeEvidenceRecord {
    pub evidence_id: String,
    pub unit_id: EntityId,
    pub result_id: EntityId,
    pub observation_kind: String,
    pub observed_at: u64,
    pub stage: ResultStage,
}

/// Batch execution coordinator
pub struct BatchCoordinator<D: WorkDispatcher> {
    dispatcher: CoreDispatcher<D>,
}

impl<D: WorkDispatcher> BatchCoordinator<D> {
    pub fn new(dispatcher: CoreDispatcher<D>) -> Self {
        Self { dispatcher }
    }

    /// Execute multiple units in sequence
    pub fn execute_batch(
        &self,
        units: &[&WorkUnitSpec],
        system_prompt: &str,
        user_prompts: &[String],
    ) -> Vec<Result<ExecuteResult, CoreError>> {
        units
            .iter()
            .zip(user_prompts.iter())
            .map(|(unit, prompt)| self.dispatcher.execute_unit(unit, system_prompt, prompt))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_id_n;
    use dc_domain::work::{DependencyRelation, UnitRef, WorkUnitSpec};
    use dc_worker::WorkDispatcher;
    use dc_worker::WorkerError;

    struct StubDispatcher;

    impl WorkDispatcher for StubDispatcher {
        fn dispatch(&self, request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
            Ok(ExecuteResult::Success(WorkResponse {
                content: format!("processed: {}", request.execution_id),
                model: "stub-model".to_string(),
                tokens_used: Some(100),
                finish_reason: Some("stop".to_string()),
            }))
        }
    }

    #[test]
    fn dispatcher_executes_unit() {
        let dispatcher = CoreDispatcher::new(StubDispatcher, "test-worker".to_string());
        let unit = crate::test_support::test_unit_spec(test_id_n(300));

        let result = dispatcher.execute_unit(&unit, "system", "user").unwrap();
        match result {
            ExecuteResult::Success(response) => {
                assert!(response.content.contains("processed:"));
                assert_eq!(response.model, "stub-model");
            }
            _ => panic!("expected success"),
        }
    }

    #[test]
    fn response_to_result_conversion() {
        let dispatcher = CoreDispatcher::new(StubDispatcher, "test-worker".to_string());
        let unit = crate::test_support::test_unit_spec(test_id_n(301));
        let response = WorkResponse {
            content: "test output".to_string(),
            model: "test-model".to_string(),
            tokens_used: Some(50),
            finish_reason: Some("stop".to_string()),
        };
        let snapshot = DependencySnapshot {
            snapshot_id: test_id_n(302),
            scope: unit.scope,
            unit: unit.unit,
            read_set: vec![],
            policy_versions: vec![],
        };
        let policy = VersionedDependency {
            key: dc_domain::work::DependencyKey {
                kind: dc_domain::work::DependencyKind::VerificationCriteria,
                entity_id: test_id_n(303),
            },
            expected_revision: dc_domain::ids::RowVersion::new(1).unwrap(),
            fingerprint: None,
        };

        let result = dispatcher
            .response_to_result(&unit, &response, test_id_n(304), snapshot, policy)
            .unwrap();
        assert_eq!(result.unit, unit.unit);
        assert_eq!(result.stage, dc_domain::work::ResultStage::Submitted);
    }

    #[test]
    fn outcome_evidence_is_recorded() {
        let dispatcher = CoreDispatcher::new(StubDispatcher, "test-worker".to_string());
        let unit = crate::test_support::test_unit_spec(test_id_n(400));
        let response = WorkResponse {
            content: "test output".to_string(),
            model: "test-model".to_string(),
            tokens_used: Some(50),
            finish_reason: Some("stop".to_string()),
        };
        let snapshot = DependencySnapshot {
            snapshot_id: test_id_n(401),
            scope: unit.scope,
            unit: unit.unit,
            read_set: vec![],
            policy_versions: vec![],
        };
        let policy = VersionedDependency {
            key: dc_domain::work::DependencyKey {
                kind: dc_domain::work::DependencyKind::VerificationCriteria,
                entity_id: test_id_n(402),
            },
            expected_revision: dc_domain::ids::RowVersion::new(1).unwrap(),
            fingerprint: None,
        };

        let result = dispatcher
            .response_to_result(&unit, &response, test_id_n(403), snapshot, policy)
            .unwrap();

        // Fix E: outcome evidence must be recorded
        let evidence = dispatcher
            .record_outcome_evidence(&unit, &result, "dispatch_success")
            .unwrap();
        assert_eq!(evidence.unit_id, unit.unit.unit_id);
        assert_eq!(evidence.result_id, result.result_id);
        assert_eq!(evidence.observation_kind, "dispatch_success");
        assert_eq!(evidence.stage, ResultStage::Submitted);
        assert!(evidence.observed_at > 0);
    }
}
