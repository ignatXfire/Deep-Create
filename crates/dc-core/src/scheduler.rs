//! C-06: Scheduling — select which work units are ready for execution.

use dc_domain::permission::CapabilityLease;
use dc_domain::work::{
    FinalReviewEvidence, GraphCandidate, UnitRef, UnitStage, VerificationRecord,
    VerificationVerdict,
};

use crate::error::CoreError;
use crate::verification::{
    AutoApproveDecision, auto_approve_final_review, auto_approve_final_review_with_reason,
};

/// Context for scheduling decisions
#[derive(Debug, Clone)]
pub struct SchedulingContext {
    /// Active capability leases
    pub leases: Vec<CapabilityLease>,
    /// Maximum concurrent attempts
    pub max_concurrent: usize,
    /// Current running attempts
    pub running_attempts: usize,
}

/// Set of units ready for execution
#[derive(Debug, Clone)]
pub struct ReadyUnits {
    /// Units that can start immediately
    pub ready: Vec<UnitRef>,
    /// Units blocked by dependencies
    pub blocked_by_deps: Vec<(UnitRef, Vec<UnitRef>)>, // (unit, missing_deps)
    /// Units blocked by concurrency limit
    pub blocked_by_concurrency: Vec<UnitRef>,
}

impl ReadyUnits {
    /// Check if any units are ready
    pub fn has_ready(&self) -> bool {
        !self.ready.is_empty()
    }

    /// Total units considered
    pub fn total(&self) -> usize {
        self.ready.len() + self.blocked_by_deps.len() + self.blocked_by_concurrency.len()
    }
}

/// Select units ready for execution
///
/// A unit is ready when:
/// 1. All dependencies are satisfied (accepted)
/// 2. Concurrency limit not reached
/// 3. Unit is in Ready or Proposed stage
pub fn select_ready_units(
    graph: &GraphCandidate,
    context: &SchedulingContext,
) -> Result<ReadyUnits, CoreError> {
    let mut ready = Vec::new();
    let mut blocked_by_deps = Vec::new();
    let mut blocked_by_concurrency = Vec::new();

    // Check concurrency limit
    let slots_available = context
        .max_concurrent
        .saturating_sub(context.running_attempts);
    if slots_available == 0 {
        // All pending units blocked by concurrency
        for unit in &graph.units {
            if unit.stage == UnitStage::Proposed || unit.stage == UnitStage::Ready {
                blocked_by_concurrency.push(unit.unit);
            }
        }
        return Ok(ReadyUnits {
            ready,
            blocked_by_deps,
            blocked_by_concurrency,
        });
    }

    for unit in &graph.units {
        // Skip non-pending units
        if unit.stage != UnitStage::Proposed && unit.stage != UnitStage::Ready {
            continue;
        }

        // Check dependencies
        let mut missing_deps = Vec::new();
        for dep in &graph.dependencies {
            if dep.consumer == unit.unit {
                let dep_unit = graph
                    .units
                    .iter()
                    .find(|u| u.unit == dep.producer)
                    .ok_or_else(|| CoreError::UnitNotFound(format!("{:?}", dep.producer)))?;

                if dep_unit.stage != UnitStage::Accepted {
                    missing_deps.push(dep.producer);
                }
            }
        }

        if !missing_deps.is_empty() {
            blocked_by_deps.push((unit.unit, missing_deps));
            continue;
        }

        // Check lease (simplified)
        let has_lease = context
            .leases
            .iter()
            .any(|l| l.status == dc_domain::permission::LeaseStatus::Active);
        if !has_lease {
            // No active lease — blocked
            blocked_by_deps.push((unit.unit, vec![]));
            continue;
        }

        // Unit is ready
        ready.push(unit.unit);

        // Stop if we hit concurrency limit
        if ready.len() >= slots_available {
            break;
        }
    }

    Ok(ReadyUnits {
        ready,
        blocked_by_deps,
        blocked_by_concurrency,
    })
}

/// Compute topological levels for parallel execution.
///
/// Returns a vector of levels, where each level contains units that can be
/// executed in parallel (all their dependencies are in earlier levels).
/// Units within the same level have no dependencies on each other.
///
/// This uses Kahn's algorithm for topological sorting with level assignment.
pub fn topological_levels(graph: &GraphCandidate) -> Vec<Vec<UnitRef>> {
    use std::collections::{HashMap, VecDeque};

    // Build dependency maps
    let mut in_degree: HashMap<UnitRef, usize> = HashMap::new();
    let mut dependents: HashMap<UnitRef, Vec<UnitRef>> = HashMap::new();

    // Initialize all units with in_degree 0
    for unit in &graph.units {
        in_degree.insert(unit.unit, 0);
        dependents.insert(unit.unit, Vec::new());
    }

    // Build edges from dependencies
    for dep in &graph.dependencies {
        *in_degree.entry(dep.consumer).or_insert(0) += 1;
        dependents
            .entry(dep.producer)
            .or_insert_with(Vec::new)
            .push(dep.consumer);
    }

    // Find all units with no dependencies (level 0)
    let mut queue: VecDeque<UnitRef> = VecDeque::new();
    for unit in &graph.units {
        if in_degree.get(&unit.unit).copied().unwrap_or(0) == 0 {
            queue.push_back(unit.unit);
        }
    }

    let mut levels = Vec::new();
    let mut current_level = Vec::new();
    let mut next_queue = VecDeque::new();

    while !queue.is_empty() {
        // Process current level
        while let Some(unit_ref) = queue.pop_front() {
            current_level.push(unit_ref);

            // Reduce in-degree of dependents
            if let Some(deps) = dependents.get(&unit_ref) {
                for &dependent in deps {
                    if let Some(deg) = in_degree.get_mut(&dependent) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            next_queue.push_back(dependent);
                        }
                    }
                }
            }
        }

        if !current_level.is_empty() {
            levels.push(std::mem::take(&mut current_level));
        }

        // Move to next level
        std::mem::swap(&mut queue, &mut next_queue);
    }

    levels
}

/// Classify a failure for retry decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    /// Temporary failure — retry may succeed
    Temporary,
    /// Permanent failure — retry will not help
    Permanent,
    /// Resource exhaustion — wait and retry
    ResourceExhaustion,
    /// Unknown — needs investigation
    Unknown,
}

/// Proposal to retry a failed unit
#[derive(Debug, Clone)]
pub struct RetryProposal {
    pub unit: UnitRef,
    pub attempt: usize,
    pub delay_ms: u64,
}

/// Plan a retry for a failed attempt
pub fn plan_retry(
    unit: UnitRef,
    failure_class: FailureClass,
    attempt_count: usize,
    max_retries: usize,
) -> Result<Option<RetryProposal>, CoreError> {
    if attempt_count >= max_retries {
        return Ok(None); // Exhausted retries
    }

    let proposal = match failure_class {
        FailureClass::Temporary => Some(RetryProposal {
            unit,
            attempt: attempt_count + 1,
            delay_ms: 1000 * (attempt_count as u64 + 1), // Exponential backoff
        }),
        FailureClass::ResourceExhaustion => Some(RetryProposal {
            unit,
            attempt: attempt_count + 1,
            delay_ms: 5000, // Longer delay for resource recovery
        }),
        FailureClass::Permanent | FailureClass::Unknown => None,
    };

    Ok(proposal)
}

/// Evaluate whether a rejected run can be auto-approved via FinalReview.
///
/// Given evidence and verification records for a completed objective,
/// determines if all verifications pass and the reviewer is qualified,
/// enabling deterministic auto-approval without manual intervention.
///
/// Returns `Some(AutoApproveDecision)` when auto-approval is possible,
/// `None` otherwise.
pub fn evaluate_auto_approve(
    evidence: &FinalReviewEvidence,
    verifications: &[VerificationRecord],
) -> Option<AutoApproveDecision> {
    auto_approve_final_review_with_reason(evidence, verifications).ok()
}

/// Check if auto-approve applies for a set of verification records.
///
/// Convenience wrapper for `evaluate_auto_approve` when you have
/// a single verification and want a simple yes/no answer.
pub fn should_auto_approve(
    evidence: &FinalReviewEvidence,
    verification: &VerificationRecord,
) -> bool {
    auto_approve_final_review(evidence, std::slice::from_ref(verification)).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{active_lease, test_id_n};
    use dc_domain::ids::{EntityId, RowVersion};
    use dc_domain::work::{
        DependencyRelation, OutputContract, StopCondition, UnitRef, WorkBounds, WorkKind,
        WorkScope, WorkUnitSpec,
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

    fn test_unit(unit_id: EntityId, stage: UnitStage) -> WorkUnitSpec {
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
            stage,
            accepted_result: None,
            row_version: RowVersion::new(1).unwrap(),
        }
    }

    fn test_context() -> SchedulingContext {
        SchedulingContext {
            leases: vec![active_lease()],
            max_concurrent: 2,
            running_attempts: 0,
        }
    }

    #[test]
    fn select_ready_units_no_deps() {
        let id1 = test_id_n(10);
        let id2 = test_id_n(11);

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![
                test_unit(id1, UnitStage::Proposed),
                test_unit(id2, UnitStage::Proposed),
            ],
            dependencies: vec![],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let context = test_context();
        let result = select_ready_units(&graph, &context).unwrap();

        assert_eq!(result.ready.len(), 2);
    }

    #[test]
    fn select_ready_units_with_deps() {
        let id1 = test_id_n(20);
        let id2 = test_id_n(21);

        let mut unit1 = test_unit(id1, UnitStage::Accepted);
        let unit2 = test_unit(id2, UnitStage::Proposed);

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![unit1, unit2],
            dependencies: vec![dc_domain::work::WorkDependency {
                consumer: UnitRef {
                    unit_id: id2,
                    revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                },
                producer: UnitRef {
                    unit_id: id1,
                    revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                },
                relation: DependencyRelation::MustFinish,
            }],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let context = test_context();
        let result = select_ready_units(&graph, &context).unwrap();

        assert_eq!(result.ready.len(), 1);
        assert_eq!(result.ready[0].unit_id, id2);
    }

    #[test]
    fn select_ready_units_blocked_by_deps() {
        let id1 = test_id_n(30);
        let id2 = test_id_n(31);

        let unit1 = test_unit(id1, UnitStage::Proposed); // Not accepted
        let unit2 = test_unit(id2, UnitStage::Proposed);

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![unit1, unit2],
            dependencies: vec![dc_domain::work::WorkDependency {
                consumer: UnitRef {
                    unit_id: id2,
                    revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                },
                producer: UnitRef {
                    unit_id: id1,
                    revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                },
                relation: DependencyRelation::MustFinish,
            }],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let context = test_context();
        let result = select_ready_units(&graph, &context).unwrap();

        assert_eq!(result.ready.len(), 1);
        assert_eq!(result.ready[0].unit_id, id1);
        assert_eq!(result.blocked_by_deps.len(), 1);
    }

    #[test]
    fn select_ready_units_concurrency_limit() {
        let id1 = test_id_n(40);
        let id2 = test_id_n(41);
        let id3 = test_id_n(42);

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![
                test_unit(id1, UnitStage::Proposed),
                test_unit(id2, UnitStage::Proposed),
                test_unit(id3, UnitStage::Proposed),
            ],
            dependencies: vec![],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let mut context = test_context();
        context.max_concurrent = 2;
        context.running_attempts = 2;

        let result = select_ready_units(&graph, &context).unwrap();

        assert_eq!(result.ready.len(), 0);
        assert_eq!(result.blocked_by_concurrency.len(), 3);
    }

    #[test]
    fn plan_retry_temporary() {
        let unit = UnitRef {
            unit_id: test_id_n(50),
            revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
        };

        let result = plan_retry(unit, FailureClass::Temporary, 0, 3).unwrap();

        assert!(result.is_some());
        let proposal = result.unwrap();
        assert_eq!(proposal.unit, unit);
        assert_eq!(proposal.attempt, 1);
    }

    #[test]
    fn plan_retry_exhausted() {
        let unit = UnitRef {
            unit_id: test_id_n(50),
            revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
        };

        let result = plan_retry(unit, FailureClass::Temporary, 3, 3).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn plan_retry_permanent() {
        let unit = UnitRef {
            unit_id: test_id_n(50),
            revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
        };

        let result = plan_retry(unit, FailureClass::Permanent, 0, 3).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn topological_levels_empty_graph() {
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

        let levels = topological_levels(&graph);
        assert!(levels.is_empty());
    }

    #[test]
    fn topological_levels_single_unit() {
        let id1 = test_id_n(100);
        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![test_unit(id1, UnitStage::Proposed)],
            dependencies: vec![],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let levels = topological_levels(&graph);
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].len(), 1);
        assert_eq!(levels[0][0].unit_id, id1);
    }

    #[test]
    fn topological_levels_parallel_independent_units() {
        // Two units with no dependencies between them
        let id1 = test_id_n(101);
        let id2 = test_id_n(102);

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![
                test_unit(id1, UnitStage::Proposed),
                test_unit(id2, UnitStage::Proposed),
            ],
            dependencies: vec![],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let levels = topological_levels(&graph);
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].len(), 2);
    }

    #[test]
    fn topological_levels_sequential_chain() {
        // A -> B -> C (each depends on previous)
        let id_a = test_id_n(110);
        let id_b = test_id_n(111);
        let id_c = test_id_n(112);

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![
                test_unit(id_a, UnitStage::Proposed),
                test_unit(id_b, UnitStage::Proposed),
                test_unit(id_c, UnitStage::Proposed),
            ],
            dependencies: vec![
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_b,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_a,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_c,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_b,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
            ],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let levels = topological_levels(&graph);
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0].len(), 1);
        assert_eq!(levels[0][0].unit_id, id_a);
        assert_eq!(levels[1].len(), 1);
        assert_eq!(levels[1][0].unit_id, id_b);
        assert_eq!(levels[2].len(), 1);
        assert_eq!(levels[2][0].unit_id, id_c);
    }

    #[test]
    fn topological_levels_diamond_dependency() {
        // Diamond pattern: A -> B, A -> C, B -> D, C -> D
        // Level 0: A
        // Level 1: B, C (can run in parallel)
        // Level 2: D
        let id_a = test_id_n(120);
        let id_b = test_id_n(121);
        let id_c = test_id_n(122);
        let id_d = test_id_n(123);

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![
                test_unit(id_a, UnitStage::Proposed),
                test_unit(id_b, UnitStage::Proposed),
                test_unit(id_c, UnitStage::Proposed),
                test_unit(id_d, UnitStage::Proposed),
            ],
            dependencies: vec![
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_b,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_a,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_c,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_a,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_d,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_b,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_d,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_c,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
            ],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let levels = topological_levels(&graph);
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0].len(), 1);
        assert_eq!(levels[0][0].unit_id, id_a);
        assert_eq!(levels[1].len(), 2);
        assert!(levels[1].iter().any(|u| u.unit_id == id_b));
        assert!(levels[1].iter().any(|u| u.unit_id == id_c));
        assert_eq!(levels[2].len(), 1);
        assert_eq!(levels[2][0].unit_id, id_d);
    }

    #[test]
    fn topological_levels_complex_dag() {
        // Complex DAG: A, B (roots), C depends on A, D depends on A and B, E depends on C and D
        // Level 0: A, B
        // Level 1: C
        // Level 2: D
        // Level 3: E
        let id_a = test_id_n(130);
        let id_b = test_id_n(131);
        let id_c = test_id_n(132);
        let id_d = test_id_n(133);
        let id_e = test_id_n(134);

        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![
                test_unit(id_a, UnitStage::Proposed),
                test_unit(id_b, UnitStage::Proposed),
                test_unit(id_c, UnitStage::Proposed),
                test_unit(id_d, UnitStage::Proposed),
                test_unit(id_e, UnitStage::Proposed),
            ],
            dependencies: vec![
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_c,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_a,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_d,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_a,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_d,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_b,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_e,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_c,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
                dc_domain::work::WorkDependency {
                    consumer: UnitRef {
                        unit_id: id_e,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    producer: UnitRef {
                        unit_id: id_d,
                        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
                    },
                    relation: DependencyRelation::MustFinish,
                },
            ],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };

        let levels = topological_levels(&graph);
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0].len(), 2); // A, B
        assert_eq!(levels[1].len(), 2); // C, D
        assert_eq!(levels[2].len(), 1); // E
    }
}
