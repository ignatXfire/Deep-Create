//! C-05: Command handling — the single entry point for owner commands.
//!
//! Every state change goes through `handle_command`. No other path may
//! modify core state. This is the bottleneck that enforces the
//! "single writer" rule from the technical constitution.

use dc_domain::ids::EntityId;
use dc_domain::work::{GraphCandidate, ObjectiveContract, UnitRef, UnitStage};
use dc_domain::work::{ResultPacket, VerificationRecord};

use crate::CoreState;
use crate::error::CoreError;

/// Commands that the owner can issue to the system
#[derive(Debug, Clone)]
pub enum CoreCommand {
    /// Accept a new objective (starts planning)
    AcceptObjective(ObjectiveContract),
    /// Accept a work graph (commits to execution plan)
    AcceptGraph(GraphCandidate),
    /// Accept a work graph from raw JSON (C-08: core deserializes)
    AcceptGraphRaw(String),
    /// Start a work unit attempt
    StartAttempt(UnitRef),
    /// Record a result from a worker
    RecordResult { unit: UnitRef, result: ResultPacket },
    /// Record verification of a result
    RecordVerification(VerificationRecord),
    /// Apply a verified result to state
    ApplyVerifiedResult(UnitRef),
    /// Propose a setting change
    ProposeSettingChange {
        key: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
    /// Accept a setting change (owner confirmation)
    AcceptSettingChange { proposal_id: EntityId },
    /// Accept completion of the objective
    AcceptCompletion,
    /// Reject completion (objective not met)
    RejectCompletion { reason: String },
}

/// Outcome of command processing
#[derive(Debug, Clone)]
pub enum CommandOutcome {
    /// Command accepted, state updated
    Accepted {
        /// Description of what changed
        summary: String,
        /// New state version
        new_version: u64,
    },
    /// Command rejected (validation failed)
    Rejected {
        /// Why rejected
        reason: String,
        /// Suggested correction
        suggestion: Option<String>,
    },
    /// Command requires owner confirmation (native dialog)
    RequiresConfirmation {
        /// What to confirm
        prompt: String,
        /// Serialized command for confirmation
        command_hash: String,
    },
    /// Command queued for later (dependencies not ready)
    Queued {
        /// Position in queue
        position: usize,
        /// Estimated wait
        estimated_wait_ms: Option<u64>,
    },
}

/// A planned state transition — can be atomically committed
#[derive(Debug, Clone)]
pub enum TransitionPlan {
    AcceptObjective {
        objective: ObjectiveContract,
    },
    AcceptGraph {
        graph: GraphCandidate,
    },
    StartAttempt {
        unit: UnitRef,
    },
    RecordResult {
        unit: UnitRef,
        result: ResultPacket,
    },
    RecordVerification {
        record: VerificationRecord,
    },
    ApplyVerifiedResult {
        unit: UnitRef,
    },
    ProposeSettingChange {
        key: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
    AcceptSettingChange {
        proposal_id: EntityId,
    },
    AcceptCompletion,
    RejectCompletion {
        reason: String,
    },
}

/// Pure transition decision — no I/O, no state mutation.
///
/// Given current state and a validated command, decide what transition
/// (if any) should occur. Returns a plan that can be atomically committed.
pub fn decide_transition(
    state: &CoreState,
    command: &CoreCommand,
) -> Result<TransitionPlan, CoreError> {
    match command {
        CoreCommand::AcceptObjective(objective) => {
            // Validate objective has required criteria
            if objective.completion_criteria.is_empty() {
                return Err(CoreError::InvalidCommand(
                    "objective must have at least one completion criterion".into(),
                ));
            }

            // Check no active objective (must complete or abandon first)
            if state.objective.is_some() {
                return Err(CoreError::InvalidTransition(
                    "cannot accept new objective while one is active".into(),
                ));
            }

            Ok(TransitionPlan::AcceptObjective {
                objective: objective.clone(),
            })
        }

        CoreCommand::AcceptGraph(graph) => {
            // Must have objective first
            if state.objective.is_none() {
                return Err(CoreError::NoObjective);
            }

            // Validate graph structure
            validate_graph(graph)?;

            Ok(TransitionPlan::AcceptGraph {
                graph: graph.clone(),
            })
        }

        CoreCommand::AcceptGraphRaw(json) => {
            // C-08: Core-only parsing of raw JSON. We parse into individual
            // domain types (all Deserialize) and reconstruct GraphCandidate
            // in the core — never deserializing GraphCandidate itself.
            let mut dto: serde_json::Value = serde_json::from_str(&json)
                .map_err(|e| CoreError::InvalidCommand(format!("invalid JSON: {}", e)))?;

            // Helper: extract and deserialize a field
            fn take<T: serde::de::DeserializeOwned>(
                v: &mut serde_json::Value,
                key: &str,
            ) -> Result<T, CoreError> {
                let raw = v
                    .get_mut(key)
                    .ok_or_else(|| CoreError::InvalidCommand(format!("missing field: {}", key)))?
                    .take();
                serde_json::from_value(raw)
                    .map_err(|e| CoreError::InvalidCommand(format!("invalid {}: {}", key, e)))
            }

            // Reconstruct all fields from the JSON
            let expected_graph_version = take(&mut dto, "expected_graph_version")?;
            let current_graph_version = take(&mut dto, "current_graph_version")?;
            let roots = take(&mut dto, "roots")?;
            let objectives = take(&mut dto, "objectives")?;
            let units = take(&mut dto, "units")?;
            let dependencies = take(&mut dto, "dependencies")?;
            let links = take(&mut dto, "links")?;
            // publications: Serialize-only (C-08), must be empty at this stage
            let publications = Vec::new();
            // publication_states: Serialize-only (C-08), must be empty at this stage
            let publication_states = Vec::new();
            let publication_bindings = take(&mut dto, "publication_bindings")?;
            // owner_acceptances: Serialize-only (C-08), must be empty at this stage
            let owner_acceptances = Vec::new();

            // Reconstruct GraphCandidate in the core (C-08: only core builds it)
            let graph = GraphCandidate {
                expected_graph_version,
                current_graph_version,
                roots,
                objectives,
                units,
                dependencies,
                links,
                publications,
                publication_states,
                publication_bindings,
                owner_acceptances,
            };

            // Validate the reconstructed graph
            validate_graph(&graph)?;

            // Must have objective first
            if state.objective.is_none() {
                return Err(CoreError::NoObjective);
            }

            Ok(TransitionPlan::AcceptGraph { graph })
        }

        CoreCommand::StartAttempt(unit_ref) => {
            let graph = state
                .graph
                .as_ref()
                .ok_or(CoreError::InvalidTransition("no graph accepted".into()))?;

            // Find unit
            let unit = graph
                .units
                .iter()
                .find(|u| u.unit == *unit_ref)
                .ok_or_else(|| CoreError::UnitNotFound(format!("{:?}", unit_ref)))?;

            // Check dependencies satisfied
            for dep in &graph.dependencies {
                if dep.consumer == *unit_ref {
                    let dep_unit = graph
                        .units
                        .iter()
                        .find(|u| u.unit == dep.producer)
                        .ok_or_else(|| CoreError::UnitNotFound(format!("{:?}", dep.producer)))?;

                    if dep_unit.stage != UnitStage::Accepted {
                        return Err(CoreError::DependencyNotSatisfied(format!(
                            "dependency {:?} not accepted",
                            dep.producer
                        )));
                    }
                }
            }

            // Check unit is in startable state
            if unit.stage != UnitStage::Ready && unit.stage != UnitStage::Proposed {
                return Err(CoreError::InvalidTransition(format!(
                    "unit {:?} not in startable state: {:?}",
                    unit_ref, unit.stage
                )));
            }

            Ok(TransitionPlan::StartAttempt { unit: *unit_ref })
        }

        CoreCommand::RecordResult { unit, result } => {
            let graph = state
                .graph
                .as_ref()
                .ok_or(CoreError::InvalidTransition("no graph accepted".into()))?;

            let unit_spec = graph
                .units
                .iter()
                .find(|u| u.unit == *unit)
                .ok_or_else(|| CoreError::UnitNotFound(format!("{:?}", unit)))?;

            // Check unit is in progress
            if unit_spec.stage != UnitStage::Running
                && unit_spec.stage != UnitStage::ResultSubmitted
            {
                return Err(CoreError::InvalidTransition(format!(
                    "unit {:?} not in progress: {:?}",
                    unit, unit_spec.stage
                )));
            }

            Ok(TransitionPlan::RecordResult {
                unit: *unit,
                result: result.clone(),
            })
        }

        CoreCommand::RecordVerification(record) => {
            // Verify the record references a real unit
            let graph = state
                .graph
                .as_ref()
                .ok_or(CoreError::InvalidTransition("no graph accepted".into()))?;

            if !graph.units.iter().any(|u| u.unit == record.unit) {
                return Err(CoreError::InvalidCommand(format!(
                    "verification for unknown unit: {:?}",
                    record.unit
                )));
            }

            Ok(TransitionPlan::RecordVerification {
                record: record.clone(),
            })
        }

        CoreCommand::ApplyVerifiedResult(unit_ref) => {
            // Check verification exists
            let verification = state
                .verifications
                .iter()
                .find(|v| v.unit == *unit_ref)
                .ok_or_else(|| {
                    CoreError::InvalidTransition(format!(
                        "no verification for unit: {:?}",
                        unit_ref
                    ))
                })?;

            // Check verification passed
            if verification.verdict != dc_domain::work::VerificationVerdict::Pass {
                return Err(CoreError::VerificationFailed(format!(
                    "verification for {:?} did not pass",
                    unit_ref
                )));
            }

            Ok(TransitionPlan::ApplyVerifiedResult { unit: *unit_ref })
        }

        CoreCommand::ProposeSettingChange {
            key,
            old_value,
            new_value,
        } => Ok(TransitionPlan::ProposeSettingChange {
            key: key.clone(),
            old_value: old_value.clone(),
            new_value: new_value.clone(),
        }),

        CoreCommand::AcceptSettingChange { proposal_id } => {
            Ok(TransitionPlan::AcceptSettingChange {
                proposal_id: *proposal_id,
            })
        }

        CoreCommand::AcceptCompletion => {
            let graph = state.graph.as_ref().ok_or(CoreError::NoObjective)?;

            let incomplete: Vec<_> = graph
                .units
                .iter()
                .filter(|u| u.stage != UnitStage::Accepted)
                .map(|u| u.unit)
                .collect();

            if !incomplete.is_empty() {
                return Err(CoreError::InvalidTransition(format!(
                    "{} units not completed",
                    incomplete.len()
                )));
            }

            Ok(TransitionPlan::AcceptCompletion)
        }

        CoreCommand::RejectCompletion { reason } => Ok(TransitionPlan::RejectCompletion {
            reason: reason.clone(),
        }),
    }
}

/// Handle a validated owner command — the single entry point.
///
/// After the transition is applied, the state is persisted to dc-store
/// if a persistence backend is configured.
pub fn handle_command(
    state: &mut CoreState,
    command: CoreCommand,
) -> Result<CommandOutcome, CoreError> {
    let command_kind = command_kind_name(&command);
    let plan = match decide_transition(state, &command) {
        Ok(plan) => plan,
        Err(e) => {
            crate::metrics::Metrics::global().inc_errors();
            tracing::warn!(
                command = command_kind,
                error = %e,
                "command rejected"
            );
            return Err(e);
        }
    };

    let outcome = match &plan {
        TransitionPlan::AcceptObjective { objective } => {
            state.objective = Some(objective.clone());
            crate::metrics::Metrics::global().inc_objectives();
            tracing::info!(
                event = "objective_submitted",
                statement = %objective.statement,
                criteria_count = objective.completion_criteria.len(),
                "objective accepted"
            );
            Ok(CommandOutcome::Accepted {
                summary: format!("objective accepted: {}", objective.statement),
                new_version: 1,
            })
        }

        TransitionPlan::AcceptGraph { graph } => {
            state.graph = Some(graph.clone());
            crate::metrics::Metrics::global().inc_graphs_accepted();
            tracing::info!(
                event = "graph_accepted",
                units = graph.units.len(),
                dependencies = graph.dependencies.len(),
                graph_version = graph.current_graph_version,
                "graph accepted"
            );
            Ok(CommandOutcome::Accepted {
                summary: format!("graph accepted with {} units", graph.units.len()),
                new_version: 2,
            })
        }

        TransitionPlan::StartAttempt { unit } => {
            if let Some(graph) = &mut state.graph {
                if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == *unit) {
                    unit_spec.stage = UnitStage::Running;
                }
            }
            crate::metrics::Metrics::global().inc_units_dispatched();
            tracing::info!(
                event = "unit_dispatched",
                unit_id = ?unit.unit_id,
                "unit attempt started"
            );
            Ok(CommandOutcome::Accepted {
                summary: format!("attempt started for unit {:?}", unit),
                new_version: 3,
            })
        }

        TransitionPlan::RecordResult { unit, result: _ } => {
            if let Some(graph) = &mut state.graph {
                if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == *unit) {
                    unit_spec.stage = UnitStage::ResultSubmitted;
                }
            }
            Ok(CommandOutcome::Accepted {
                summary: format!("result recorded for unit {:?}", unit),
                new_version: 4,
            })
        }

        TransitionPlan::RecordVerification { record } => {
            let passed =
                record.verdict == dc_domain::work::VerificationVerdict::Pass;
            if passed {
                crate::metrics::Metrics::global().inc_verifications_passed();
            } else {
                crate::metrics::Metrics::global().inc_verifications_failed();
            }
            tracing::info!(
                event = "verification_completed",
                unit_id = ?record.unit,
                verdict = ?record.verdict,
                "verification completed"
            );
            state.verifications.push(record.clone());
            Ok(CommandOutcome::Accepted {
                summary: format!("verification recorded for unit {:?}", record.unit),
                new_version: 5,
            })
        }

        TransitionPlan::ApplyVerifiedResult { unit } => {
            if let Some(graph) = &mut state.graph {
                if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == *unit) {
                    unit_spec.stage = UnitStage::Accepted;
                }
            }
            crate::metrics::Metrics::global().inc_units_completed();
            tracing::info!(
                event = "unit_completed",
                unit_id = ?unit.unit_id,
                "verified result applied, unit accepted"
            );
            Ok(CommandOutcome::Accepted {
                summary: format!("result applied for unit {:?}", unit),
                new_version: 6,
            })
        }

        TransitionPlan::ProposeSettingChange { key, .. } => {
            Ok(CommandOutcome::RequiresConfirmation {
                prompt: format!("confirm setting change: {}", key),
                command_hash: format!("{:?}", command),
            })
        }

        TransitionPlan::AcceptSettingChange { proposal_id } => Ok(CommandOutcome::Accepted {
            summary: format!("setting change {} accepted", proposal_id),
            new_version: 7,
        }),

        TransitionPlan::AcceptCompletion => Ok(CommandOutcome::Accepted {
            summary: "objective completed".into(),
            new_version: 8,
        }),

        TransitionPlan::RejectCompletion { reason } => Ok(CommandOutcome::Accepted {
            summary: format!("completion rejected: {}", reason),
            new_version: 9,
        }),
    };

    outcome
}

/// Static name for each command variant — used for structured logging.
fn command_kind_name(command: &CoreCommand) -> &'static str {
    match command {
        CoreCommand::AcceptObjective(_) => "accept_objective",
        CoreCommand::AcceptGraph(_) => "accept_graph",
        CoreCommand::AcceptGraphRaw(_) => "accept_graph_raw",
        CoreCommand::StartAttempt(_) => "start_attempt",
        CoreCommand::RecordResult { .. } => "record_result",
        CoreCommand::RecordVerification(_) => "record_verification",
        CoreCommand::ApplyVerifiedResult(_) => "apply_verified_result",
        CoreCommand::ProposeSettingChange { .. } => "propose_setting_change",
        CoreCommand::AcceptSettingChange { .. } => "accept_setting_change",
        CoreCommand::AcceptCompletion => "accept_completion",
        CoreCommand::RejectCompletion { .. } => "reject_completion",
    }
}

/// Handle command with persistence — writes state to dc-store after each command.
pub fn handle_command_with_persistence(
    state: &mut CoreState,
    command: CoreCommand,
    persistence: &mut crate::persistence::CorePersistence<impl dc_store::StoreBackend>,
    state_id: dc_domain::ids::EntityId,
) -> Result<CommandOutcome, CoreError> {
    let outcome = handle_command(state, command)?;

    // Persist state after successful command
    let persisted = crate::persistence::PersistedCoreState {
        current_objective: state.objective.as_ref().map(|o| {
            // Generate a deterministic ID from objective statement
            dc_domain::ids::EntityId::from_u128(
                o.statement
                    .bytes()
                    .fold(0u128, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u128)),
            )
            .unwrap_or_else(|| dc_domain::ids::EntityId::from_u128(0).unwrap())
        }),
        current_graph_version: state
            .graph
            .as_ref()
            .map(|g| g.current_graph_version.get())
            .unwrap_or(0),
        active_units: state
            .graph
            .as_ref()
            .map(|g| g.units.iter().map(|u| u.unit.unit_id).collect())
            .unwrap_or_default(),
        completed_units: state
            .graph
            .as_ref()
            .map(|g| {
                g.units
                    .iter()
                    .filter(|u| u.stage == dc_domain::work::UnitStage::Accepted)
                    .map(|u| u.unit.unit_id)
                    .collect()
            })
            .unwrap_or_default(),
        // Serialize objective for crash recovery
        objective_json: state
            .objective
            .as_ref()
            .and_then(|o| serde_json::to_string(o).ok()),
        // Serialize full graph for crash recovery (C-08: only core deserializes)
        graph_json: state
            .graph
            .as_ref()
            .and_then(|g| serde_json::to_string(g).ok()),
    };

    let expected = dc_store::ExpectedVersion::Any; // TODO: Use CAS for safety
    persistence
        .persist_state(state_id, &persisted, expected)
        .map_err(|e| CoreError::Persistence(e.to_string()))?;

    Ok(outcome)
}

/// Validate work graph structure
pub(crate) fn validate_graph(graph: &GraphCandidate) -> Result<(), CoreError> {
    // Check all dependencies reference existing units
    for dep in &graph.dependencies {
        let consumer_exists = graph.units.iter().any(|u| u.unit == dep.consumer);
        let producer_exists = graph.units.iter().any(|u| u.unit == dep.producer);

        if !consumer_exists {
            return Err(CoreError::InvalidCommand(format!(
                "dependency consumer {:?} not found",
                dep.consumer
            )));
        }
        if !producer_exists {
            return Err(CoreError::InvalidCommand(format!(
                "dependency producer {:?} not found",
                dep.producer
            )));
        }
    }

    // Check for cycles (simplified — real impl would do topological sort)
    // For now, just check no unit depends on itself
    for dep in &graph.dependencies {
        if dep.consumer == dep.producer {
            return Err(CoreError::InvalidCommand(format!(
                "unit {:?} depends on itself",
                dep.consumer
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_id_n;
    use dc_domain::ids::{EntityId, RowVersion};
    use dc_domain::work::{
        DependencyRelation, ExpectedLifecycleBasis, WorkDependency, WorkUnitSpec,
    };
    use dc_domain::work::{
        FieldSchema, OutputContract, StopCondition, WorkBounds, WorkKind, WorkScope,
    };
    use std::num::NonZeroU32;

    fn test_scope() -> WorkScope {
        WorkScope {
            session_id: test_id_n(1),
            work_id: test_id_n(2),
            objective_id: test_id_n(3),
            objective_revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
        }
    }

    fn test_objective() -> ObjectiveContract {
        ObjectiveContract {
            scope: test_scope(),
            statement: "Test objective".into(),
            owner_event_ids: vec![],
            obligations: vec![],
            completion_criteria: vec![test_id_n(4)],
            row_version: RowVersion::new(1).unwrap(),
        }
    }

    fn test_unit(unit_id: EntityId) -> WorkUnitSpec {
        WorkUnitSpec {
            unit: UnitRef {
                unit_id,
                revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
            },
            scope: test_scope(),
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

    #[test]
    fn accept_objective_success() {
        let mut state = CoreState::empty();
        let cmd = CoreCommand::AcceptObjective(test_objective());

        let outcome = handle_command(&mut state, cmd).unwrap();

        match outcome {
            CommandOutcome::Accepted { summary, .. } => {
                assert!(summary.contains("objective accepted"));
            }
            _ => panic!("expected Accepted"),
        }

        assert!(state.objective.is_some());
    }

    #[test]
    fn accept_objective_rejects_empty_criteria() {
        let mut state = CoreState::empty();
        let mut obj = test_objective();
        obj.completion_criteria.clear();

        let cmd = CoreCommand::AcceptObjective(obj);
        let result = handle_command(&mut state, cmd);

        assert!(result.is_err());
    }

    #[test]
    fn metrics_increment_on_command_lifecycle() {
        use std::sync::atomic::Ordering;
        let m = crate::metrics::Metrics::global();
        let obj_before = m.objectives_processed.load(Ordering::Relaxed);
        let err_before = m.errors_total.load(Ordering::Relaxed);

        // Successful objective increments objectives_processed
        let mut state = CoreState::empty();
        handle_command(&mut state, CoreCommand::AcceptObjective(test_objective())).unwrap();
        assert_eq!(
            m.objectives_processed.load(Ordering::Relaxed),
            obj_before + 1
        );

        // Rejected command increments errors_total
        let mut bad = test_objective();
        bad.completion_criteria.clear();
        let _ = handle_command(&mut CoreState::empty(), CoreCommand::AcceptObjective(bad));
        assert_eq!(m.errors_total.load(Ordering::Relaxed), err_before + 1);
    }

    #[test]
    fn tracing_events_emitted_on_objective_accept() {
        // Capture tracing output into a buffer via a custom MakeWriter.
        use std::io::Write;
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::fmt::MakeWriter;

        #[derive(Clone, Default)]
        struct BufWriter(Arc<Mutex<Vec<u8>>>);
        impl Write for BufWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        impl<'a> MakeWriter<'a> for BufWriter {
            type Writer = BufWriter;
            fn make_writer(&'a self) -> Self::Writer {
                self.clone()
            }
        }

        let buf = BufWriter::default();
        let target = buf.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(buf)
            .with_ansi(false)
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);

        let mut state = CoreState::empty();
        handle_command(&mut state, CoreCommand::AcceptObjective(test_objective())).unwrap();

        drop(_guard);
        let out = String::from_utf8(target.0.lock().unwrap().clone()).unwrap();
        assert!(
            out.contains("objective_submitted"),
            "expected objective_submitted event in logs, got: {}",
            out
        );
    }

    #[test]
    fn accept_graph_requires_objective() {
        let mut state = CoreState::empty();
        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![],
            dependencies: vec![],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let cmd = CoreCommand::AcceptGraph(graph);
        let result = handle_command(&mut state, cmd);

        assert!(matches!(result, Err(CoreError::NoObjective)));
    }

    #[test]
    fn start_attempt_requires_dependencies() {
        let mut state = CoreState::empty();
        state.objective = Some(test_objective());

        let unit1_id = test_id_n(10);
        let unit2_id = test_id_n(11);

        let mut unit2 = test_unit(unit2_id);
        unit2.stage = UnitStage::Proposed;

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![test_unit(unit1_id), unit2],
            dependencies: vec![WorkDependency {
                consumer: UnitRef {
                    unit_id: unit2_id,
                    revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                },
                producer: UnitRef {
                    unit_id: unit1_id,
                    revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                },
                relation: dc_domain::work::DependencyRelation::MustFinish,
            }],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        state.graph = Some(graph);

        let unit2_ref = UnitRef {
            unit_id: unit2_id,
            revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
        };

        let cmd = CoreCommand::StartAttempt(unit2_ref);
        let result = handle_command(&mut state, cmd);

        // Should fail because dependency not completed
        assert!(result.is_err());
    }

    #[test]
    fn accept_completion_requires_all_units_completed() {
        let mut state = CoreState::empty();
        state.objective = Some(test_objective());

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![test_unit(test_id_n(20))],
            dependencies: vec![],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        state.graph = Some(graph);

        let cmd = CoreCommand::AcceptCompletion;
        let result = handle_command(&mut state, cmd);

        assert!(result.is_err());
    }
}
