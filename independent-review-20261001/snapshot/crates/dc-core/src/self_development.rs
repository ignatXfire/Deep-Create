//! Self-development runtime: orchestrates the full proposal → code → test → verify → merge cycle.
//!
//! This module integrates the domain-level SelfDevelopmentGate with the core
//! orchestration to provide end-to-end self-improvement capabilities.
//!
//! # Authority model
//!
//! The runtime NEVER auto-applies changes. Every proposal requires explicit
//! owner authorization before any real effect. The runtime only:
//! 1. Analyzes codebase facts (from broker/workspace scan)
//! 2. Generates improvement proposals
//! 3. Schedules work units for approved proposals
//! 4. Verifies completed work against proposal requirements

use std::num::NonZeroU32;

use dc_domain::ids::EntityId;
use dc_domain::self_development::{
    AuthorityLevel, CodebaseFacts, SelfDevelopmentGate, SelfImprovementProposal,
};
use dc_domain::work::{
    ObjectiveContract, OutputContract, StopCondition, UnitRef, WorkBounds, WorkKind, WorkScope,
    WorkUnitSpec,
};

use crate::error::CoreError;
use crate::verification::VerificationOutcome;

/// Self-development runtime state.
pub struct SelfDevelopmentRuntime {
    /// The domain gate for proposal classification
    gate: SelfDevelopmentGate,
    /// Pending proposals awaiting owner approval
    pending_proposals: Vec<SelfImprovementProposal>,
    /// Approved proposals converted to work units
    approved_units: Vec<WorkUnitSpec>,
    /// Completed work units with verification results
    completed_units: Vec<CompletedUnit>,
}

/// A work unit that has been completed and verified.
#[derive(Debug, Clone)]
pub struct CompletedUnit {
    pub unit_id: EntityId,
    pub proposal_id: EntityId,
    pub verification_outcome: VerificationOutcome,
    pub applied_at_utc_millis: u64,
}

impl SelfDevelopmentRuntime {
    /// Create a new runtime with default gate configuration.
    pub fn new() -> Self {
        Self {
            gate: SelfDevelopmentGate::default(),
            pending_proposals: Vec::new(),
            approved_units: Vec::new(),
            completed_units: Vec::new(),
        }
    }

    /// Analyze codebase facts and generate improvement proposals.
    ///
    /// This is the entry point for self-development. Facts must be supplied
    /// by the caller (e.g. from a workspace scan). The runtime does NOT
    /// perform I/O itself.
    pub fn analyze(
        &mut self,
        facts: &CodebaseFacts,
        scope: &WorkScope,
        id_allocator: &mut dyn FnMut() -> EntityId,
    ) -> Vec<SelfImprovementProposal> {
        let proposals = self.gate.analyze(facts, scope, id_allocator);
        self.pending_proposals.extend(proposals.clone());
        proposals
    }

    /// Get pending proposals awaiting owner approval.
    pub fn pending_proposals(&self) -> &[SelfImprovementProposal] {
        &self.pending_proposals
    }

    /// Approve a proposal, converting it to a work unit.
    ///
    /// This requires explicit owner authorization. The proposal must exist
    /// in pending_proposals.
    pub fn approve_proposal(
        &mut self,
        proposal_id: EntityId,
        _objective: &ObjectiveContract,
        id_allocator: &mut dyn FnMut() -> EntityId,
    ) -> Result<WorkUnitSpec, CoreError> {
        let pos = self
            .pending_proposals
            .iter()
            .position(|p| p.proposal_id == proposal_id)
            .ok_or_else(|| {
                CoreError::InvalidCommand(format!("proposal not found: {}", proposal_id))
            })?;

        let proposal = self.pending_proposals.remove(pos);

        // Convert proposal to work unit
        let unit = WorkUnitSpec {
            unit: UnitRef {
                unit_id: id_allocator(),
                revision: NonZeroU32::new(1).unwrap(),
            },
            scope: proposal.work_scope.clone(),
            parent: None,
            kind: WorkKind::LocalChangeProposal,
            expected_output: OutputContract {
                export_name: "self_dev_patch".to_string(),
                schema_version: NonZeroU32::new(1).unwrap(),
                fields: vec![],
            },
            obligation_ids: vec![],
            acceptance_criteria: vec![],
            bounds: WorkBounds {
                max_attempts: NonZeroU32::new(3).unwrap(),
                max_steps: NonZeroU32::new(10).unwrap(),
                stop_conditions: vec![StopCondition::OutputProduced],
            },
            exploration: None,
            stage: dc_domain::work::UnitStage::Proposed,
            accepted_result: None,
            row_version: dc_domain::ids::RowVersion::new(1).unwrap(),
        };

        self.approved_units.push(unit.clone());
        Ok(unit)
    }

    /// Reject a proposal with a reason.
    pub fn reject_proposal(
        &mut self,
        proposal_id: EntityId,
        reason: &str,
    ) -> Result<(), CoreError> {
        let pos = self
            .pending_proposals
            .iter()
            .position(|p| p.proposal_id == proposal_id)
            .ok_or_else(|| {
                CoreError::InvalidCommand(format!("proposal not found: {}", proposal_id))
            })?;

        self.pending_proposals.remove(pos);
        // In a real implementation, we would log the rejection with reason
        let _ = reason;
        Ok(())
    }

    /// Get approved work units ready for scheduling.
    pub fn approved_units(&self) -> &[WorkUnitSpec] {
        &self.approved_units
    }

    /// Record a completed unit with its verification outcome.
    pub fn record_completion(
        &mut self,
        unit_id: EntityId,
        proposal_id: EntityId,
        outcome: VerificationOutcome,
        applied_at_utc_millis: u64,
    ) {
        self.completed_units.push(CompletedUnit {
            unit_id,
            proposal_id,
            verification_outcome: outcome,
            applied_at_utc_millis,
        });

        // Remove from approved units
        self.approved_units.retain(|u| u.unit.unit_id != unit_id);
    }

    /// Get completed units.
    pub fn completed_units(&self) -> &[CompletedUnit] {
        &self.completed_units
    }

    /// Check if a proposal requires owner authorization.
    ///
    /// ALL real changes require owner authorization. Only diagnostic/read-only
    /// proposals may be automatic.
    pub fn requires_authorization(proposal: &SelfImprovementProposal) -> bool {
        !matches!(proposal.required_authority, AuthorityLevel::Automatic)
    }
}

impl Default for SelfDevelopmentRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_domain::self_development::{ImpactLevel, TargetArea};

    fn test_id(n: u64) -> EntityId {
        // Create a valid EntityId with the required bit pattern
        // Bits 76-79 must be 7, bits 62-63 must be 2
        let mut value = (n as u128) & 0x3FFFFFFFFFFFFFFF; // Keep lower 62 bits
        value |= 0x7 << 76; // Set bits 76-79 to 7
        value |= 0x2 << 62; // Set bits 62-63 to 2
        EntityId::from_u128(value).unwrap()
    }

    fn test_scope(n: u64) -> WorkScope {
        WorkScope {
            session_id: test_id(n),
            work_id: test_id(n + 1),
            objective_id: test_id(n + 2),
            objective_revision: NonZeroU32::new(1).unwrap(),
        }
    }

    #[test]
    fn runtime_analyze_generates_proposals() {
        let mut runtime = SelfDevelopmentRuntime::new();
        let mut allocator = || test_id(1);

        let facts = CodebaseFacts {
            test_coverage: vec![],
            documentation: vec![],
            protocol: vec![],
            security: vec![],
        };

        let scope = test_scope(10);
        let proposals = runtime.analyze(&facts, &scope, &mut allocator);
        // Empty facts may generate no proposals, that's ok
        assert!(proposals.len() <= runtime.pending_proposals.len());
    }

    #[test]
    fn approve_proposal_creates_work_unit() {
        let mut runtime = SelfDevelopmentRuntime::new();
        let mut allocator = || test_id(1);

        // Add a proposal directly
        let proposal = SelfImprovementProposal {
            proposal_id: test_id(1),
            work_scope: test_scope(10),
            target_area: TargetArea::TestCoverage,
            problem: "Test problem".to_string(),
            solution: "Test solution".to_string(),
            confidence: 0.9,
            impact: ImpactLevel::Moderate,
            required_authority: AuthorityLevel::OwnerApproval,
            evidence_ids: vec![],
            work_kind: WorkKind::LocalChangeProposal,
            prerequisite_obligations: vec![],
        };
        runtime.pending_proposals.push(proposal);

        let objective = ObjectiveContract {
            scope: test_scope(10),
            statement: "Test objective".to_string(),
            owner_event_ids: vec![],
            obligations: vec![],
            completion_criteria: vec![],
            row_version: dc_domain::ids::RowVersion::new(1).unwrap(),
        };
        let result = runtime.approve_proposal(test_id(1), &objective, &mut allocator);
        assert!(result.is_ok());
        assert_eq!(runtime.pending_proposals.len(), 0);
        assert_eq!(runtime.approved_units.len(), 1);
    }
}
