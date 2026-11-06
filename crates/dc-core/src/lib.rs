//! Deep Create core controller (C-05/C-06/C-07/C-08).
//!
//! The single writer of the main database. Accepts owner commands, plans
//! transitions, schedules work units, compiles context, and verifies results.
//!
//! # Architecture
//!
//! - **C-05 Command**: `handle_command` — validates and applies owner commands
//! - **C-06 Scheduler**: `select_ready_units` — picks units ready for execution
//! - **C-07 Context**: `compile_prompt_bundle` — builds model prompts
//! - **C-08 Verification**: `run_machine_verification` — verifies results
//!
//! # Dependency rules
//!
//! - Depends on: `dc-domain`, `dc-protocol`, `dc-store`
//! - Does NOT depend on: `dc-broker`, `dc-worker` (they are downstream)
//! - Is depended on by: `dc-desktop` (UI), future `dc-hermes` (bridge)

pub mod broker;
pub mod command;
pub mod context;
pub mod dispatch;
pub mod error;
pub mod metrics;
pub mod persistence;
pub mod planner;
pub mod protocol;
pub mod scheduler;
pub mod self_development;
pub mod verification;

pub use broker::{BrokerClient, BrokerOutcome, ProtocolDispatcher};
pub use command::{CommandOutcome, CoreCommand, handle_command};
pub use context::{ContextPlan, PromptBundle, compile_prompt_bundle};
pub use dispatch::{BatchCoordinator, CoreDispatcher};
pub use error::CoreError;
pub use persistence::{CorePersistence, PersistedCoreState, PersistenceError};
pub use protocol::{CoreEvent, ProtocolAdapter, deserialize_event, serialize_event};
pub use scheduler::{ReadyUnits, SchedulingContext, select_ready_units, topological_levels};
pub use verification::{CompletionCandidate, VerificationOutcome, run_machine_verification};

use dc_domain::permission::CapabilityLease;
use dc_domain::resource::LedgerPosition;
use dc_domain::work::{GraphCandidate, ObjectiveContract, UnitRef, UnitStage, WorkUnitSpec};

/// Core controller state — the single source of truth for work progression.
///
/// This is NOT the database. This is the in-memory planning state that
/// gets committed to dc-store atomically via `commit_transition`.
#[derive(Debug, Clone)]
pub struct CoreState {
    /// Current objective under execution
    pub objective: Option<ObjectiveContract>,
    /// Work graph with units and dependencies
    pub graph: Option<GraphCandidate>,
    /// Active capability leases
    pub leases: Vec<CapabilityLease>,
    /// Resource ledger positions
    pub resource_positions: Vec<LedgerPosition>,
    /// Verification records for completed units
    pub verifications: Vec<dc_domain::work::VerificationRecord>,
}

impl CoreState {
    /// Create empty state (no objective, no work)
    pub fn empty() -> Self {
        Self {
            objective: None,
            graph: None,
            leases: Vec::new(),
            resource_positions: Vec::new(),
            verifications: Vec::new(),
        }
    }

    /// Check if state has active work
    pub fn has_active_work(&self) -> bool {
        self.graph.as_ref().is_some_and(|g| !g.units.is_empty())
    }

    /// Find a unit by reference
    pub fn find_unit(&self, unit_ref: UnitRef) -> Option<&WorkUnitSpec> {
        self.graph
            .as_ref()?
            .units
            .iter()
            .find(|u| u.unit == unit_ref)
    }

    /// Check if a unit is completed
    pub fn is_unit_completed(&self, unit_ref: UnitRef) -> bool {
        self.find_unit(unit_ref)
            .is_some_and(|u| u.stage == UnitStage::Accepted)
    }
}

impl Default for CoreState {
    fn default() -> Self {
        Self::empty()
    }
}

/// Shared test helpers — valid UUIDv7 EntityIds and a structurally valid
/// `CapabilityLease`. Domain deliberately offers no lease constructor, so
/// tests build one directly (all fields are `pub`).
#[cfg(test)]
pub(crate) mod test_support {
    use dc_domain::ids::{AuthorityEpoch, EntityId, RowVersion, RunGeneration};
    use dc_domain::permission::{
        ActionKind, CanonicalResource, CapabilityLease, LeaseExpiry, LeaseScope, LeaseStatus,
    };
    use dc_domain::resource::ExplicitLimit;
    use std::collections::BTreeSet;

    /// Build a valid UUIDv7 EntityId. `n` occupies the low random bits,
    /// so every distinct `n` yields a distinct ID.
    ///
    /// Layout: timestamp(48) | version(4)=7 | rand_a(12) | variant(2)=2 | rand_b(62)
    pub(crate) fn test_id_n(n: u128) -> EntityId {
        let timestamp = 0x123456789abc_u128;
        let rand_a = 0x123_u128;
        let rand_b = (0x123456789abcdef_u128 + n) & ((1u128 << 62) - 1);
        let value = (timestamp << 80) | (7 << 76) | (rand_a << 64) | (2 << 62) | rand_b;
        EntityId::from_u128(value).expect("constructed UUIDv7 must be valid")
    }

    /// Minimal active lease for scheduling/context tests.
    pub(crate) fn active_lease() -> CapabilityLease {
        CapabilityLease {
            lease_id: test_id_n(900),
            owner_event_id: test_id_n(901),
            installation_id: test_id_n(902),
            scope: LeaseScope {
                work_id: test_id_n(903),
                unit_id: None,
                action_kind: ActionKind::CallModelConnection,
                resource: CanonicalResource::ModelConnection {
                    connection_id: test_id_n(905),
                    revision: RowVersion::new(1).unwrap(),
                },
                recipient: None,
                data_classes: BTreeSet::from([dc_domain::permission::DataClass::Public]),
                max_calls: ExplicitLimit::Finite(100),
                max_bytes: ExplicitLimit::Finite(1_000_000),
                cost_limits: Default::default(),
            },
            not_before_utc_millis: 0,
            expiry: LeaseExpiry::Never,
            parent_lease_id: None,
            delegation_depth: 0,
            max_delegation_depth: 3,
            authority_epoch: AuthorityEpoch::new(1).unwrap(),
            run_generation: RunGeneration::new(1).unwrap(),
            support_fences: vec![],
            confirmed_conditions_hash: [0u8; 32],
            issued_dependency_snapshot_id: test_id_n(904),
            lease_version: RowVersion::new(1).unwrap(),
            status: LeaseStatus::Active,
        }
    }

    /// Minimal WorkUnitSpec for persistence tests.
    pub(crate) fn test_unit_spec(unit_id: EntityId) -> dc_domain::work::WorkUnitSpec {
        use dc_domain::ids::UnitRevision;
        use dc_domain::work::{
            OutputContract, StopCondition, UnitRef, UnitStage, WorkBounds, WorkKind, WorkScope,
        };
        use std::num::NonZeroU32;

        dc_domain::work::WorkUnitSpec {
            unit: UnitRef {
                unit_id,
                revision: UnitRevision::new(1).unwrap(),
            },
            scope: WorkScope {
                session_id: test_id_n(800),
                work_id: test_id_n(801),
                objective_id: test_id_n(802),
                objective_revision: UnitRevision::new(1).unwrap(),
            },
            parent: None,
            kind: WorkKind::Research,
            expected_output: OutputContract {
                export_name: "test".into(),
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
            stage: UnitStage::Proposed,
            accepted_result: None,
            row_version: RowVersion::new(1).unwrap(),
        }
    }
}
