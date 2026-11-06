//! C-06: Scheduling — select which work units are ready for execution.

use dc_domain::work::{GraphCandidate, UnitRef, UnitStage};
use dc_domain::permission::CapabilityLease;

use crate::error::CoreError;

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
    let slots_available = context.max_concurrent.saturating_sub(context.running_attempts);
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
                let dep_unit = graph.units.iter()
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
        let has_lease = context.leases.iter().any(|l| l.status == dc_domain::permission::LeaseStatus::Active);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{test_id_n, active_lease};
    use dc_domain::ids::{EntityId, RowVersion};
    use dc_domain::work::{WorkScope, WorkKind, WorkBounds, StopCondition, OutputContract, WorkUnitSpec, UnitRef, DependencyRelation};
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
}
