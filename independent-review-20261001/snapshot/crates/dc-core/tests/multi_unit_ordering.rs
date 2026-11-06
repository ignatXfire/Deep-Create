//! Multi-unit execution ordering tests.
//!
//! Verifies that dependency-aware scheduling produces correct topological
//! ordering for linear chains and diamond dependencies.

use dc_core::scheduler::{SchedulingContext, select_ready_units};
use dc_domain::ids::{EntityId, RowVersion, UnitRevision};
use dc_domain::permission::{
    ActionKind, CanonicalResource, CapabilityLease, LeaseExpiry, LeaseScope, LeaseStatus,
};
use dc_domain::resource::ExplicitLimit;
use dc_domain::work::{
    DependencyRelation, GraphCandidate, OutputContract, StopCondition, UnitRef, UnitStage,
    WorkBounds, WorkDependency, WorkKind, WorkScope, WorkUnitSpec,
};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

/// Build a valid UUIDv7 EntityId for tests.
fn test_id_n(n: u128) -> EntityId {
    let timestamp = 0x123456789abc_u128;
    let rand_a = 0x123_u128;
    let rand_b = (0x123456789abcdef_u128 + n) & ((1u128 << 62) - 1);
    let value = (timestamp << 80) | (7 << 76) | (rand_a << 64) | (2 << 62) | rand_b;
    EntityId::from_u128(value).expect("constructed UUIDv7 must be valid")
}

/// Minimal active lease for scheduling tests.
fn active_lease() -> CapabilityLease {
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
        authority_epoch: dc_domain::ids::AuthorityEpoch::new(1).unwrap(),
        run_generation: dc_domain::ids::RunGeneration::new(1).unwrap(),
        support_fences: vec![],
        confirmed_conditions_hash: [0u8; 32],
        issued_dependency_snapshot_id: test_id_n(904),
        lease_version: RowVersion::new(1).unwrap(),
        status: LeaseStatus::Active,
    }
}

fn vid(n: u128) -> EntityId {
    test_id_n(n)
}

fn unit_ref(id: EntityId) -> UnitRef {
    UnitRef {
        unit_id: id,
        revision: UnitRevision::new(1).unwrap(),
    }
}

fn make_unit(id: EntityId, stage: UnitStage) -> WorkUnitSpec {
    WorkUnitSpec {
        unit: unit_ref(id),
        scope: WorkScope {
            session_id: vid(1),
            work_id: vid(2),
            objective_id: vid(3),
            objective_revision: UnitRevision::new(1).unwrap(),
        },
        parent: None,
        kind: WorkKind::Research,
        expected_output: OutputContract {
            export_name: format!("out_{}", id),
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

fn dep(consumer: EntityId, producer: EntityId) -> WorkDependency {
    WorkDependency {
        consumer: unit_ref(consumer),
        producer: unit_ref(producer),
        relation: DependencyRelation::MustFinish,
    }
}

fn graph_with_units(units: Vec<WorkUnitSpec>, deps: Vec<WorkDependency>) -> GraphCandidate {
    GraphCandidate {
        expected_graph_version: RowVersion::new(1).unwrap(),
        current_graph_version: RowVersion::new(1).unwrap(),
        roots: vec![],
        objectives: vec![],
        units,
        dependencies: deps,
        links: vec![],
        publications: vec![],
        publication_states: vec![],
        publication_bindings: vec![],
        owner_acceptances: vec![],
    }
}

fn scheduling_context() -> SchedulingContext {
    SchedulingContext {
        leases: vec![active_lease()],
        max_concurrent: 10,
        running_attempts: 0,
    }
}

/// Kahn's algorithm: returns topological levels (BFS layers).
/// Each level contains units whose dependencies are all in earlier levels.
fn topological_levels(units: &[WorkUnitSpec], deps: &[WorkDependency]) -> Vec<Vec<UnitRef>> {
    let mut in_degree: BTreeMap<UnitRef, usize> = units.iter().map(|u| (u.unit, 0)).collect();
    let mut outgoing: BTreeMap<UnitRef, Vec<UnitRef>> = BTreeMap::new();

    for d in deps {
        *in_degree.entry(d.consumer).or_default() += 1;
        in_degree.entry(d.producer).or_default();
        outgoing.entry(d.producer).or_default().push(d.consumer);
    }

    let mut levels = Vec::new();
    let mut current: Vec<UnitRef> = in_degree
        .iter()
        .filter_map(|(u, d)| (*d == 0).then_some(*u))
        .collect();

    while !current.is_empty() {
        levels.push(current.clone());
        let mut next = Vec::new();
        for node in &current {
            if let Some(targets) = outgoing.get(node) {
                for t in targets {
                    if let Some(deg) = in_degree.get_mut(t) {
                        *deg -= 1;
                        if *deg == 0 {
                            next.push(*t);
                        }
                    }
                }
            }
        }
        current = next;
    }

    levels
}

/// Simulate execution by walking topological levels and marking units Accepted
/// after each level. Returns the order in which units became ready.
fn simulate_execution_order(graph: &GraphCandidate) -> Vec<UnitRef> {
    let mut g = graph.clone();
    let mut execution_log = Vec::new();
    let ctx = scheduling_context();

    loop {
        let ready = select_ready_units(&g, &ctx).unwrap();
        if ready.ready.is_empty() {
            break;
        }
        // Execute all ready units (they are independent within this level)
        for unit_ref in &ready.ready {
            execution_log.push(*unit_ref);
            // Mark as Accepted so downstream units can become ready
            if let Some(u) = g.units.iter_mut().find(|u| u.unit == *unit_ref) {
                u.stage = UnitStage::Accepted;
            }
        }
    }

    execution_log
}

#[test]
fn linear_chain_a_b_c_executes_in_order() {
    let a = vid(100);
    let b = vid(101);
    let c = vid(102);

    let graph = graph_with_units(
        vec![
            make_unit(a, UnitStage::Proposed),
            make_unit(b, UnitStage::Proposed),
            make_unit(c, UnitStage::Proposed),
        ],
        vec![dep(b, a), dep(c, b)],
    );

    // Verify topological levels: [A], [B], [C]
    let levels = topological_levels(&graph.units, &graph.dependencies);
    assert_eq!(levels.len(), 3, "linear chain should have 3 levels");
    assert_eq!(levels[0].len(), 1);
    assert_eq!(levels[0][0].unit_id, a);
    assert_eq!(levels[1].len(), 1);
    assert_eq!(levels[1][0].unit_id, b);
    assert_eq!(levels[2].len(), 1);
    assert_eq!(levels[2][0].unit_id, c);

    // Verify scheduler-driven execution order
    let order = simulate_execution_order(&graph);
    assert_eq!(order.len(), 3, "all 3 units should execute");
    assert_eq!(order[0].unit_id, a, "A must execute first");
    assert_eq!(order[1].unit_id, b, "B must execute second");
    assert_eq!(order[2].unit_id, c, "C must execute third");
}

#[test]
fn diamond_dependency_resolution() {
    let a = vid(200);
    let b = vid(201);
    let c = vid(202);
    let d = vid(203);

    let graph = graph_with_units(
        vec![
            make_unit(a, UnitStage::Proposed),
            make_unit(b, UnitStage::Proposed),
            make_unit(c, UnitStage::Proposed),
            make_unit(d, UnitStage::Proposed),
        ],
        vec![dep(b, a), dep(c, a), dep(d, b), dep(d, c)],
    );

    // Verify topological levels: [A], [B, C], [D]
    let levels = topological_levels(&graph.units, &graph.dependencies);
    assert_eq!(levels.len(), 3, "diamond should have 3 levels");
    assert_eq!(levels[0].len(), 1);
    assert_eq!(levels[0][0].unit_id, a);
    assert_eq!(levels[1].len(), 2, "B and C should be parallel");
    assert!(levels[1].iter().any(|u| u.unit_id == b), "B in level 1");
    assert!(levels[1].iter().any(|u| u.unit_id == c), "C in level 1");
    assert_eq!(levels[2].len(), 1);
    assert_eq!(levels[2][0].unit_id, d);

    // Verify scheduler-driven execution order
    let order = simulate_execution_order(&graph);
    assert_eq!(order.len(), 4, "all 4 units should execute");
    assert_eq!(order[0].unit_id, a, "A must execute first");
    // B and C can be in any relative order (same level), but both before D
    let pos_b = order.iter().position(|u| u.unit_id == b).unwrap();
    let pos_c = order.iter().position(|u| u.unit_id == c).unwrap();
    let pos_d = order.iter().position(|u| u.unit_id == d).unwrap();
    assert!(
        pos_b < pos_d && pos_c < pos_d,
        "B and C must both execute before D"
    );
}

#[test]
fn select_ready_units_respects_pending_deps() {
    let a = vid(300);
    let b = vid(301);

    // A is Proposed (not yet Accepted), B depends on A
    let graph = graph_with_units(
        vec![
            make_unit(a, UnitStage::Proposed),
            make_unit(b, UnitStage::Proposed),
        ],
        vec![dep(b, a)],
    );

    let ctx = scheduling_context();
    let result = select_ready_units(&graph, &ctx).unwrap();

    // Only A should be ready; B is blocked by dependency on A
    assert_eq!(result.ready.len(), 1);
    assert_eq!(result.ready[0].unit_id, a);
    assert_eq!(result.blocked_by_deps.len(), 1);
    assert_eq!(result.blocked_by_deps[0].0.unit_id, b);
}

#[test]
fn select_ready_units_unblocks_after_acceptance() {
    let a = vid(400);
    let b = vid(401);

    // A is already Accepted, B depends on A
    let graph = graph_with_units(
        vec![
            make_unit(a, UnitStage::Accepted),
            make_unit(b, UnitStage::Proposed),
        ],
        vec![dep(b, a)],
    );

    let ctx = scheduling_context();
    let result = select_ready_units(&graph, &ctx).unwrap();

    // B should now be ready
    assert_eq!(result.ready.len(), 1);
    assert_eq!(result.ready[0].unit_id, b);
    assert!(result.blocked_by_deps.is_empty());
}
