//! Synthetic C-01 contract examples, not database or external-effect tests.
use std::num::{NonZeroU32, NonZeroU64};

use dc_domain::ids::{EntityId, RowVersion};
use dc_domain::work::*;

#[test]
fn prepare_binds_exact_candidate_and_reports_expected_then_successor() {
    use dc_domain::topology::{TopologyHead, prepare_topology_change};

    let candidate = graph();
    let validated = validate_work_graph(&candidate, &limits()).expect("valid fixture");
    let head = TopologyHead {
        installation_id: id(900),
        revision: row(1),
    };
    let plan = prepare_topology_change(head, validated).expect("matching head prepares");

    // expected() echoes the exact head snapshot read with the affected graph.
    assert_eq!(plan.expected(), head);
    // successor() is exactly the next revision of the same installation.
    assert_eq!(
        plan.successor(),
        TopologyHead {
            installation_id: id(900),
            revision: row(2),
        }
    );
    // The plan borrows the exact immutable validated candidate rather than a
    // copy or a bare version integer detached from the graph.
    assert!(std::ptr::eq(plan.graph().candidate(), &candidate));
    assert_eq!(plan.graph().candidate().expected_graph_version, row(1));
    assert_eq!(plan.graph().candidate().current_graph_version, row(1));
}

#[test]
fn prepare_rejects_stale_head_and_stale_candidate_revisions() {
    use dc_domain::topology::{TopologyHead, prepare_topology_change};

    // Candidate validated at revision 2 while the held head snapshot is r1.
    let mut candidate = graph();
    candidate.expected_graph_version = row(2);
    candidate.current_graph_version = row(2);
    let validated = validate_work_graph(&candidate, &limits()).expect("valid fixture");
    let stale_head = TopologyHead {
        installation_id: id(901),
        revision: row(1),
    };
    assert_eq!(
        prepare_topology_change(stale_head, validated).unwrap_err(),
        WorkError::GraphVersionConflict
    );

    // Mirror image: the head already advanced past the validated candidate.
    let candidate = graph();
    let validated = validate_work_graph(&candidate, &limits()).expect("valid fixture");
    let advanced_head = TopologyHead {
        installation_id: id(901),
        revision: row(2),
    };
    assert_eq!(
        prepare_topology_change(advanced_head, validated).unwrap_err(),
        WorkError::GraphVersionConflict
    );
}

#[test]
fn precondition_rejects_same_revision_head_from_foreign_installation() {
    use dc_domain::topology::{TopologyHead, check_topology_precondition, prepare_topology_change};

    let candidate = graph();
    let validated = validate_work_graph(&candidate, &limits()).expect("valid fixture");
    let head = TopologyHead {
        installation_id: id(902),
        revision: row(1),
    };
    let plan = prepare_topology_change(head, validated).expect("matching head prepares");

    assert_eq!(check_topology_precondition(&plan, head), Ok(()));

    // An equal revision under another installation is not the same head.
    let foreign = TopologyHead {
        installation_id: id(903),
        revision: row(1),
    };
    assert_eq!(
        check_topology_precondition(&plan, foreign).unwrap_err(),
        WorkError::GraphVersionConflict
    );
}

#[test]
fn prepare_rejects_successor_revision_overflow() {
    use dc_domain::topology::{TopologyHead, prepare_topology_change};

    let mut candidate = graph();
    candidate.expected_graph_version = row(u64::MAX);
    candidate.current_graph_version = row(u64::MAX);
    let validated = validate_work_graph(&candidate, &limits()).expect("valid fixture");
    let head = TopologyHead {
        installation_id: id(904),
        revision: row(u64::MAX),
    };
    assert_eq!(
        prepare_topology_change(head, validated).unwrap_err(),
        WorkError::VersionOverflow
    );
}

#[test]
fn only_first_of_two_concurrent_plans_passes_the_updated_head_precondition() {
    use dc_domain::topology::{TopologyHead, check_topology_precondition, prepare_topology_change};

    // Two independent proposals are prepared against the same head snapshot r1.
    let candidate_a = graph();
    let validated_a = validate_work_graph(&candidate_a, &limits()).expect("valid fixture");
    let candidate_b = graph();
    let validated_b = validate_work_graph(&candidate_b, &limits()).expect("valid fixture");
    let head = TopologyHead {
        installation_id: id(905),
        revision: row(1),
    };
    let plan_a = prepare_topology_change(head, validated_a).expect("matching head prepares");
    let plan_b = prepare_topology_change(head, validated_b).expect("matching head prepares");

    // Both computed the identical successor, so the head admits only one.
    assert_eq!(plan_a.successor(), plan_b.successor());

    // The winner's in-transaction re-check matches the head it prepared from.
    assert_eq!(check_topology_precondition(&plan_a, head), Ok(()));
    // Once the observed head has advanced to that successor, the loser's
    // identical re-check no longer matches and must be rejected.
    let updated = plan_a.successor();
    assert_eq!(
        check_topology_precondition(&plan_b, updated).unwrap_err(),
        WorkError::GraphVersionConflict
    );
    // Rebasing the stale r1 candidate onto the r2 head is rejected as well.
    let rebased = validate_work_graph(&candidate_b, &limits()).expect("valid fixture");
    assert_eq!(
        prepare_topology_change(updated, rebased).unwrap_err(),
        WorkError::GraphVersionConflict
    );
}

fn id(n: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | n).expect("UUIDv7 fixture")
}
fn rev(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).expect("nonzero fixture")
}
fn row(n: u64) -> RowVersion {
    NonZeroU64::new(n).expect("nonzero fixture")
}
fn unit(n: u128) -> UnitRef {
    UnitRef {
        unit_id: id(100 + n),
        revision: rev(1),
    }
}
fn scope(n: u128) -> WorkScope {
    WorkScope {
        session_id: id(1),
        work_id: id(10 + n),
        objective_id: id(20 + n),
        objective_revision: rev(1),
    }
}
fn obligation(n: u128) -> EntityId {
    id(30 + n)
}
fn policy() -> VersionedDependency {
    VersionedDependency {
        key: DependencyKey {
            kind: DependencyKind::Policy,
            entity_id: id(40),
        },
        expected_revision: row(1),
        fingerprint: Some([1; 32]),
    }
}
fn output() -> OutputContract {
    OutputContract {
        export_name: "report".into(),
        schema_version: rev(1),
        fields: vec![FieldSchema {
            field_id: id(50),
            kind: ScalarKind::Boolean,
        }],
    }
}
fn limits() -> GraphLimits {
    GraphLimits {
        max_units: rev(100),
        max_edges: rev(200),
        max_attempts_per_unit: rev(3),
        max_steps_per_unit: rev(100),
        max_exploration_expansions: rev(10),
    }
}

mod combined_result_plan {
    use super::*;
    use dc_domain::ids::RunGeneration;
    use dc_domain::resource::*;
    use dc_domain::usage_binding::*;

    struct Fixture {
        graph: GraphCandidate,
        packet: ResultPacket,
        state: UnitAttemptState,
        expected: ExpectedLifecycleBasis,
        run: RunGeneration,
        result_row: RowVersion,
        active_attempt: EntityId,
        installation_gates: Vec<BlockingReason>,
        unit_gates: Vec<BlockingReason>,
    }

    impl Fixture {
        fn new() -> Self {
            let graph = graph();
            let packet = packet(&validate_work_graph(&graph, &limits()).unwrap(), 0);
            let run = RunGeneration::new(1).unwrap();
            let state = UnitAttemptState {
                unit: graph.units[0].clone(),
                attempt: Some(AttemptRecord {
                    attempt_id: packet.attempt_id,
                    scope: packet.scope,
                    unit: packet.unit,
                    run_generation: run,
                    stage: AttemptStage::Submitted,
                    row_version: row(3),
                }),
            };
            let expected = ExpectedLifecycleBasis {
                scope: packet.scope,
                unit: packet.unit,
                unit_row_version: row(1),
                attempt_id: Some(packet.attempt_id),
                attempt_row_version: Some(row(3)),
                run_generation: run,
            };
            Self {
                result_row: packet.row_version,
                active_attempt: packet.attempt_id,
                graph,
                packet,
                state,
                expected,
                run,
                installation_gates: vec![],
                unit_gates: vec![],
            }
        }

        fn plan(
            &self,
            target: ResultStage,
            proof: Option<&VerificationRecord>,
        ) -> Result<ResultTransitionPlan, WorkError> {
            self.with_context(|context| {
                plan_result_transition(&self.packet, target, proof, context, id(3100))
            })
        }

        fn with_context<T>(&self, action: impl FnOnce(&ResultTransitionContext<'_>) -> T) -> T {
            let graph = validate_work_graph(&self.graph, &limits()).unwrap();
            let current_rows = rows(&graph);
            let view = ResultCommitView {
                graph: &graph,
                current_rows: &current_rows,
                expected_unit_row_version: self.expected.unit_row_version,
                expected_result_row_version: self.result_row,
                active_attempt_id: self.active_attempt,
            };
            action(&ResultTransitionContext {
                result_view: &view,
                state: &self.state,
                expected: &self.expected,
                current_run: self.run,
                installation_gates: &self.installation_gates,
                unit_gates: &self.unit_gates,
            })
        }

        // Test-only replacement application; NOT a transaction implementation.
        fn apply(&mut self, plan: &ResultTransitionPlan) {
            self.packet = plan.packet().clone();
            self.result_row = self.packet.row_version;
            self.graph.units[0] = plan.unit().clone();
            self.state.unit = plan.unit().clone();
            self.expected.unit_row_version = plan.unit().row_version;
        }

        fn verifying() -> Self {
            let mut fixture = Self::new();
            let plan = fixture.plan(ResultStage::Verifying, None).unwrap();
            fixture.apply(&plan);
            fixture
        }
    }

    fn usage(attempt_id: EntityId) -> UsageBinding {
        usage_with_fact(
            attempt_id,
            ObservationKind::Partial {
                confirmed_amount: 1,
                unknown_remaining: UpperBound::Known(1),
            },
        )
    }

    fn usage_with_fact(attempt_id: EntityId, kind: ObservationKind) -> UsageBinding {
        let limit = ResourceLimit {
            limit_id: id(3300),
            scope: ResourceScope::RootWork(scope(0).work_id),
            window_id: id(3301),
            kind: ResourceKind::Requests,
            bound: ExplicitLimit::Finite(10),
            opens_at_utc_millis: 0,
            closes_at_utc_millis: None,
            status: LimitStatus::Active,
        };
        let position = evaluate_reservation(
            &limit,
            &[],
            &ReservationRequest {
                key: LedgerKey {
                    logical_operation_id: id(3302),
                    limit_id: limit.limit_id,
                    window_id: limit.window_id,
                },
                reservation_id: id(3303),
                attempt_id,
                intent_id: id(3304),
                kind: limit.kind,
                upper_bound: UpperBound::Known(2),
            },
            1,
        )
        .unwrap()
        .position;
        let position = apply_observation(
            &position,
            ResourceObservation {
                observation_id: id(3305),
                reservation_id: position.request().reservation_id,
                ledger_key: position.request().key,
                resource_kind: position.request().kind,
                kind,
            },
        )
        .unwrap()
        .position;
        bind_usage(
            &[ExpectedUsagePosition {
                key: position.request().key,
                reservation_id: position.request().reservation_id,
                kind: position.request().kind,
                row_version: position.row_version(),
            }],
            &[position],
            attempt_id,
        )
        .unwrap()
    }

    #[test]
    fn accepted_and_rejected_results_bind_usage_without_settling_again() {
        for target in [ResultStage::Accepted, ResultStage::Rejected] {
            let f = Fixture::verifying();
            let usage = usage(f.packet.attempt_id);
            let before = usage.clone();
            let mut proof = verification(&f.packet, 0);
            if target == ResultStage::Rejected {
                proof.verdict = VerificationVerdict::Reject;
            }
            let plan = f
                .with_context(|context| {
                    plan_accounted_result_transition(
                        &f.packet,
                        target,
                        &proof,
                        context,
                        &usage,
                        id(3306),
                    )
                })
                .unwrap();
            assert_eq!(plan.transition().packet().stage, target);
            assert_eq!(plan.usage(), &before);
            assert_eq!(usage, before);
            assert_eq!(
                plan.usage().positions()[0].state(),
                LedgerState::Unknown {
                    confirmed_amount: 1,
                    unknown_remaining: UpperBound::Known(1)
                }
            );
            assert_eq!(
                plan.usage().positions()[0].contribution(),
                UpperBound::Known(2)
            );
        }
    }

    #[test]
    fn accounted_plan_cannot_bypass_attempt_proof_stage_or_action_outcome_gate() {
        let mut f = Fixture::verifying();
        let bound = usage(f.packet.attempt_id);
        let mut proof = verification(&f.packet, 0);
        let call = |f: &Fixture, target, proof: &VerificationRecord, usage: &UsageBinding| {
            f.with_context(|context| {
                plan_accounted_result_transition(&f.packet, target, proof, context, usage, id(3306))
            })
        };
        assert!(call(&f, ResultStage::Accepted, &proof, &usage(id(3999))).is_err());
        for stage in [
            ResultStage::Submitted,
            ResultStage::Verifying,
            ResultStage::Stale,
        ] {
            assert!(call(&f, stage, &proof, &bound).is_err());
        }
        proof.result_fingerprint = [99; 32];
        assert!(call(&f, ResultStage::Accepted, &proof, &bound).is_err());
        proof = verification(&f.packet, 0);
        f.unit_gates.push(BlockingReason::UnknownOutcome {});
        assert!(matches!(
            call(&f, ResultStage::Accepted, &proof, &bound),
            Err(WorkError::Blocked(_))
        ));
    }

    #[test]
    fn breached_spend_prevents_acceptance_but_rejection_retains_the_fact() {
        let f = Fixture::verifying();
        let usage = usage_with_fact(
            f.packet.attempt_id,
            ObservationKind::Settled {
                confirmed_amount: 3,
            },
        );
        let mut proof = verification(&f.packet, 0);
        assert!(matches!(
            f.with_context(|context| plan_accounted_result_transition(
                &f.packet,
                ResultStage::Accepted,
                &proof,
                context,
                &usage,
                id(3306)
            )),
            Err(WorkError::Blocked(BlockingReason::ResourceUnavailable {}))
        ));
        proof.verdict = VerificationVerdict::Reject;
        let rejected = f
            .with_context(|context| {
                plan_accounted_result_transition(
                    &f.packet,
                    ResultStage::Rejected,
                    &proof,
                    context,
                    &usage,
                    id(3306),
                )
            })
            .unwrap();
        assert!(rejected.usage().positions()[0].bound_breached());
        assert_eq!(
            rejected.usage().positions()[0].contribution(),
            UpperBound::Known(3)
        );
        assert_eq!(rejected.transition().unit().stage, UnitStage::Rejected);
    }

    #[test]
    fn result_and_unit_advance_together_without_rewriting_attempt() {
        let mut f = Fixture::new();
        let original = f.state.clone();
        let verifying = f.plan(ResultStage::Verifying, None).unwrap();
        assert_eq!(f.state, original);
        assert_eq!(verifying.unit().stage, UnitStage::Verifying);
        assert_eq!(verifying.unit().row_version, row(2));
        assert_eq!(verifying.packet().row_version, row(2));
        assert_eq!(verifying.expected_basis(), &f.expected);
        assert_eq!(verifying.expected_result_row_version(), row(1));
        assert_eq!(verifying.event().verification_id, None);
        f.apply(&verifying);
        let proof = verification(&f.packet, 0);
        let accepted = f.plan(ResultStage::Accepted, Some(&proof)).unwrap();
        assert_eq!(accepted.unit().stage, UnitStage::Accepted);
        assert_eq!(accepted.unit().row_version, row(3));
        assert_eq!(
            accepted.unit().accepted_result,
            Some(AcceptedResultRef {
                result_id: accepted.packet().result_id,
                snapshot_id: accepted.packet().snapshot.snapshot_id,
                fingerprint: accepted.packet().fingerprint,
                result_row_version: accepted.packet().row_version,
            })
        );
        assert_eq!(
            accepted.event().verification_id,
            Some(proof.verification_id)
        );
        assert_eq!(accepted.event().result_from, ResultStage::Verifying);
        assert_eq!(accepted.event().unit_to, UnitStage::Accepted);
        assert_eq!(accepted.expected_basis().attempt_row_version, Some(row(3)));
        f.apply(&accepted);
        assert_eq!(f.state.attempt, original.attempt);
        assert!(f.plan(ResultStage::Accepted, Some(&proof)).is_err());
    }

    #[test]
    fn every_attempt_and_expected_basis_mismatch_is_rejected() {
        for case in 0..14 {
            let mut f = Fixture::new();
            match case {
                0 => f.state.unit.bounds.max_steps = rev(49),
                1 => f.expected.scope = scope(1),
                2 => f.expected.unit = unit(1),
                3 => f.expected.unit_row_version = row(2),
                4 => f.expected.attempt_id = Some(id(3999)),
                5 => f.expected.attempt_row_version = Some(row(4)),
                6 => f.run = RunGeneration::new(2).unwrap(),
                7 => f.state.attempt = None,
                8 => f.state.attempt.as_mut().unwrap().scope = scope(1),
                9 => f.state.attempt.as_mut().unwrap().unit = unit(1),
                10 => f.state.attempt.as_mut().unwrap().attempt_id = id(3999),
                11 => {
                    f.state.attempt.as_mut().unwrap().run_generation =
                        RunGeneration::new(2).unwrap()
                }
                12 => f.active_attempt = id(3999),
                13 => f.result_row = row(2),
                _ => unreachable!(),
            }
            let before = (f.state.clone(), f.packet.clone());
            assert!(f.plan(ResultStage::Verifying, None).is_err(), "case {case}");
            assert_eq!((f.state, f.packet), before);
        }
    }

    #[test]
    fn completion_uses_verified_findings_without_rewriting_obligation_basis() {
        let mut f = Fixture::verifying();
        let proof = verification(&f.packet, 0);
        let original_snapshot = f.packet.snapshot.clone();
        let accepted = f.plan(ResultStage::Accepted, Some(&proof)).unwrap();
        f.apply(&accepted);
        let completed = |g: &GraphCandidate| {
            let graph = validate_work_graph(g, &limits()).unwrap();
            derive_completion_candidate(
                id(3200),
                f.packet.scope.objective_id,
                &graph,
                &[AcceptedResultEvidence {
                    packet: &f.packet,
                    verification: &proof,
                }],
                &rows(&graph),
            )
        };
        assert_eq!(
            f.graph.objectives[0].obligations[0].status,
            ObligationStatus::Open
        );
        assert!(completed(&f.graph).is_ok());
        assert_eq!(f.packet.snapshot, original_snapshot);
        // A definition/read-basis rewrite is NOT just recording fulfilment.
        let mut changed = f.graph.clone();
        changed.objectives[0].obligations[0].status = ObligationStatus::Satisfied;
        changed.objectives[0].obligations[0].row_version = row(2);
        assert!(completed(&changed).unwrap_err().iter().any(|gap| matches!(
            gap,
            CompletionGap::InvalidAcceptedResult {
                reason: WorkError::StaleInput(_),
                ..
            }
        )));
    }

    #[test]
    fn only_submitted_attempt_can_enter_result_transition() {
        for stage in [
            AttemptStage::Prepared,
            AttemptStage::Running,
            AttemptStage::Failed,
            AttemptStage::Lost,
            AttemptStage::Cancelled,
        ] {
            let mut f = Fixture::new();
            f.state.attempt.as_mut().unwrap().stage = stage;
            assert!(f.plan(ResultStage::Verifying, None).is_err(), "{stage:?}");
        }
    }

    #[test]
    fn all_three_gate_scopes_block_verification_and_acceptance() {
        for target in [ResultStage::Verifying, ResultStage::Accepted] {
            for scope in 0..3 {
                let mut f = if target == ResultStage::Accepted {
                    Fixture::verifying()
                } else {
                    Fixture::new()
                };
                let proof = verification(&f.packet, 0);
                match scope {
                    0 => f.installation_gates.push(BlockingReason::StopRequested {}),
                    1 => {
                        f.graph.roots[0]
                            .blocking_reasons
                            .insert(BlockingReason::StopRequested {});
                    }
                    2 => f.unit_gates.push(BlockingReason::StopRequested {}),
                    _ => unreachable!(),
                }
                let proof = (target == ResultStage::Accepted).then_some(&proof);
                assert!(matches!(f.plan(target, proof), Err(WorkError::Blocked(_))));
            }
        }
    }

    #[test]
    fn rejected_and_stale_facts_do_not_become_accepted_bindings() {
        let mut f = Fixture::verifying();
        f.installation_gates.push(BlockingReason::StopRequested {});
        f.unit_gates.push(BlockingReason::Recovery {});
        let mut proof = verification(&f.packet, 0);
        proof.verdict = VerificationVerdict::Reject;
        let rejected = f.plan(ResultStage::Rejected, Some(&proof)).unwrap();
        assert_eq!(rejected.unit().stage, UnitStage::Rejected);
        assert!(rejected.unit().accepted_result.is_none());
        for verifying in [false, true] {
            let mut f = if verifying {
                Fixture::verifying()
            } else {
                Fixture::new()
            };
            assert!(f.plan(ResultStage::Stale, None).is_err());
            f.graph.objectives[0].obligations[0].row_version = row(2);
            let stale = f.plan(ResultStage::Stale, None).unwrap();
            assert_eq!(stale.unit().stage, UnitStage::Stale);
            assert!(stale.unit().accepted_result.is_none());
        }
    }

    #[test]
    fn invalid_missing_and_irrelevant_verification_cannot_enter_plan() {
        let f = Fixture::new();
        let proof = verification(&f.packet, 0);
        assert!(f.plan(ResultStage::Verifying, Some(&proof)).is_err());
        let f = Fixture::verifying();
        assert!(f.plan(ResultStage::Accepted, None).is_err());
        let mut proof = verification(&f.packet, 0);
        proof.evidence.method_revision_id = id(3999);
        assert!(f.plan(ResultStage::Accepted, Some(&proof)).is_err());
    }

    #[test]
    fn either_row_overflow_aborts_plan_without_input_mutation() {
        for unit_overflow in [false, true] {
            let mut f = Fixture::new();
            if unit_overflow {
                f.graph.units[0].row_version = row(u64::MAX);
                f.state.unit = f.graph.units[0].clone();
                f.expected.unit_row_version = row(u64::MAX);
            } else {
                f.packet.row_version = row(u64::MAX);
                f.result_row = row(u64::MAX);
            }
            let before = (f.state.clone(), f.packet.clone());
            assert!(matches!(
                f.plan(ResultStage::Verifying, None),
                Err(WorkError::VersionOverflow)
            ));
            assert_eq!((f.state, f.packet), before);
        }
    }
}

fn graph() -> GraphCandidate {
    let mut g = GraphCandidate {
        expected_graph_version: row(1),
        current_graph_version: row(1),
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
    for n in 0..2 {
        let scope = scope(n);
        g.roots.push(RootWork {
            work_id: scope.work_id,
            session_id: scope.session_id,
            active_objective_id: scope.objective_id,
            stage: RootStage::Executing,
            blocking_reasons: Default::default(),
            row_version: row(1),
        });
        g.objectives.push(ObjectiveContract {
            scope,
            statement: "Produce a bounded, checkable report".into(),
            owner_event_ids: vec![id(60 + n)],
            obligations: vec![Obligation {
                obligation_id: obligation(n),
                objective_id: scope.objective_id,
                parent_id: None,
                kind: ObligationKind::Required,
                statement: "Report satisfies its quality criterion".into(),
                verification_method: VerificationMethod::Deterministic,
                status: ObligationStatus::Open,
                waiver_owner_event_id: None,
                row_version: row(1),
            }],
            completion_criteria: vec![obligation(n)],
            row_version: row(1),
        });
        g.units.push(WorkUnitSpec {
            unit: unit(n),
            scope,
            parent: None,
            kind: WorkKind::Research,
            expected_output: output(),
            obligation_ids: vec![obligation(n)],
            acceptance_criteria: vec![obligation(n)],
            bounds: WorkBounds {
                max_attempts: rev(2),
                max_steps: rev(50),
                stop_conditions: vec![StopCondition::StepLimitReached],
            },
            exploration: Some(ExplorationContract {
                question: "Which alternatives are supported?".into(),
                allowed_scope: "Synthetic local evidence".into(),
                max_expansions: rev(5),
            }),
            stage: UnitStage::ResultSubmitted,
            accepted_result: None,
            row_version: row(1),
        });
    }
    g
}

fn link(n: u128, producer: u128, consumer: u128, selector: PublicationSelector) -> RootWorkLink {
    RootWorkLink {
        link_id: id(200 + n),
        revision_id: id(300 + n),
        row_version: row(1),
        producer_work_id: scope(producer).work_id,
        consumer_work_id: scope(consumer).work_id,
        consumer_objective_id: scope(consumer).objective_id,
        consumer_targets: vec![ConsumerTargetRef::Unit {
            unit: unit(consumer),
        }],
        relation: RootWorkRelation::RequiresResult,
        selector,
        required_output_schema: rev(1),
        required_verification_level: VerificationLevel::IndependentReview,
        owner_acceptance: OwnerAcceptanceRequirement::NotRequired,
        condition: LinkCondition::Always {},
        freshness_policy: policy(),
        on_change: SourceChangePolicy::KeepPinned,
        accepted_basis_ref: id(400 + n),
    }
}

fn publish(g: &mut GraphCandidate, n: u128, producer: u128, export_revision: u32) {
    let view = validate_work_graph(g, &limits()).unwrap();
    let mut source_packet = packet(&view, producer);
    source_packet.result_id = id(600 + n);
    source_packet.fingerprint = [n as u8; 32];
    source_packet.stage = ResultStage::Accepted;
    source_packet.row_version = row(3);
    let mut source_verification = verification(&source_packet, producer);
    source_verification.verification_id = id(700 + n);
    let mut source_unit = g
        .units
        .iter()
        .find(|u| u.unit == source_packet.unit)
        .unwrap()
        .clone();
    source_unit.stage = UnitStage::Accepted;
    source_unit.accepted_result = Some(AcceptedResultRef {
        result_id: source_packet.result_id,
        snapshot_id: source_packet.snapshot.snapshot_id,
        fingerprint: source_packet.fingerprint,
        result_row_version: source_packet.row_version,
    });
    let accepted_fields = vec![AcceptedField {
        field_id: id(50),
        value: CanonicalScalar::Boolean(true),
    }];
    let source = PublicationSource {
        objective: g
            .objectives
            .iter()
            .find(|o| o.scope == source_unit.scope)
            .unwrap()
            .clone(),
        unit: source_unit.clone(),
        packet: source_packet,
        verification: source_verification,
        output_manifest_evidence_id: id(2300 + n),
        accepted_fields: accepted_fields.clone(),
    };
    g.publications.push(ResultPublication {
        publication_id: id(500 + n),
        producer_work_id: scope(producer).work_id,
        producer_objective_id: scope(producer).objective_id,
        producer_unit: source_unit.unit,
        export_revision: rev(export_revision),
        output: output(),
        result_id: id(600 + n),
        result_fingerprint: [n as u8; 32],
        verification_id: id(700 + n),
        verification_level: VerificationLevel::IndependentReview,
        accepted_fields,
        source,
        output_manifest_evidence_id: id(2300 + n),
        created_event_id: id(2400 + n),
        predecessor_publication_id: None,
    });
    g.publication_states.push(PublicationState {
        publication_id: id(500 + n),
        eligibility: PublicationEligibility::Valid,
        reason_evidence_id: None,
        row_version: row(1),
    });
}

fn rows(g: &ValidatedGraph<'_>) -> Vec<VersionedDependency> {
    let mut rows = graph_dependency_rows(g);
    rows.push(policy());
    rows
}

fn packet(g: &ValidatedGraph<'_>, n: u128) -> ResultPacket {
    let exact_unit = g
        .candidate()
        .units
        .iter()
        .find(|u| u.unit.unit_id == unit(n).unit_id)
        .unwrap()
        .unit;
    let obligation_read = graph_dependency_rows(g)
        .into_iter()
        .find(|r| r.key.entity_id == obligation(n))
        .expect("obligation row");
    let mut p = ResultPacket {
        result_id: id(800 + n),
        attempt_id: id(900 + n),
        scope: scope(n),
        unit: exact_unit,
        snapshot: DependencySnapshot {
            snapshot_id: id(1000 + n),
            scope: scope(n),
            unit: exact_unit,
            read_set: vec![obligation_read],
            policy_versions: vec![policy()],
        },
        fingerprint: [2; 32],
        verification_policy: policy(),
        used_links: vec![],
        stage: ResultStage::Submitted,
        row_version: row(1),
    };
    for link in &g.candidate().links {
        if link.consumer_work_id != scope(n).work_id {
            continue;
        }
        let assessment = assess_link(g, link.link_id).expect("link assessment");
        p.used_links.push(LinkUse {
            link_id: link.link_id,
            link_revision_id: link.revision_id,
            selected_publication_id: assessment.selected_publication_id,
        });
        for read in assessment.read_set {
            if read.key.kind != DependencyKind::Policy && !p.snapshot.read_set.contains(&read) {
                p.snapshot.read_set.push(read);
            }
        }
    }
    p
}

fn verification(p: &ResultPacket, n: u128) -> VerificationRecord {
    VerificationRecord {
        verification_id: id(1100 + n),
        result_id: p.result_id,
        result_fingerprint: p.fingerprint,
        unit: p.unit,
        snapshot_id: p.snapshot.snapshot_id,
        verification_policy: p.verification_policy.clone(),
        verifier_attempt_id: id(1200 + n),
        verdict: VerificationVerdict::Pass,
        findings: vec![ObligationFinding {
            obligation_id: obligation(n),
            status: ObligationStatus::Satisfied,
            evidence_id: id(1300 + n),
        }],
        evidence: VerificationEvidence {
            method_revision_id: id(2800 + n),
            required_level: VerificationLevel::IndependentReview,
            verification_plan_id: id(2500 + n),
            scope: p.scope,
            work_kind: WorkKind::Research,
            criterion_ids: vec![obligation(n)],
            level: VerificationLevel::IndependentReview,
            machine_findings_id: id(2600 + n),
            policy: p.verification_policy.clone(),
            qualification: Some(MethodQualification {
                qualification_id: id(2700 + n),
                method_revision_id: id(2800 + n),
                work_kind: WorkKind::Research,
                criterion_ids: vec![obligation(n)],
                level: VerificationLevel::IndependentReview,
                policy: p.verification_policy.clone(),
                status: QualificationStatus::Qualified,
            }),
            independence_assessment_id: Some(id(2900 + n)),
            protected_admission_evidence_id: None,
        },
    }
}

fn advance(
    g: &GraphCandidate,
    p: &ResultPacket,
    target: ResultStage,
    record: Option<&VerificationRecord>,
) -> Result<ResultPacket, WorkError> {
    // Simulate C-03's related unit-stage update between packet transitions.
    let mut g = g.clone();
    if let Some(u) = g.units.iter_mut().find(|u| u.unit == p.unit)
        && matches!(u.stage, UnitStage::ResultSubmitted | UnitStage::Verifying)
    {
        u.stage = match p.stage {
            ResultStage::Verifying => UnitStage::Verifying,
            _ => UnitStage::ResultSubmitted,
        };
    }
    let v = validate_work_graph(&g, &limits())?;
    let rows = rows(&v);
    transition_result(
        p,
        target,
        record,
        &ResultCommitView {
            graph: &v,
            current_rows: &rows,
            expected_unit_row_version: row(1),
            expected_result_row_version: p.row_version,
            active_attempt_id: p.attempt_id,
        },
    )
}

fn bind_accepted(g: &mut GraphCandidate, p: &ResultPacket) {
    let u = g.units.iter_mut().find(|u| u.unit == p.unit).unwrap();
    u.stage = UnitStage::Accepted;
    u.accepted_result = Some(AcceptedResultRef {
        result_id: p.result_id,
        snapshot_id: p.snapshot.snapshot_id,
        fingerprint: p.fingerprint,
        result_row_version: p.row_version,
    });
}

#[test]
fn objective_requires_criteria_preserves_owner_constraints_and_validates_waivers() {
    let g = graph();
    let mut o = g.objectives[0].clone();
    assert!(
        validate_objective(
            &o,
            &OwnerConstraints {
                required_obligations: vec![obligation(0)],
                forbidden_obligations: vec![]
            }
        )
        .is_ok()
    );
    o.completion_criteria.clear();
    assert_eq!(
        validate_objective(&o, &OwnerConstraints::default()).unwrap_err(),
        WorkError::MissingCriterion
    );
    o = g.objectives[0].clone();
    o.obligations[0].kind = ObligationKind::Preference;
    assert_eq!(
        validate_objective(&o, &OwnerConstraints::default()).unwrap_err(),
        WorkError::ConstraintConflict
    );
    o = g.objectives[0].clone();
    o.obligations[0].status = ObligationStatus::Waived;
    assert_eq!(
        validate_objective(&o, &OwnerConstraints::default()).unwrap_err(),
        WorkError::ConstraintConflict
    );
    o.obligations[0].waiver_owner_event_id = Some(id(1400));
    assert!(validate_objective(&o, &OwnerConstraints::default()).is_ok());
}

#[test]
fn graph_rejects_foreign_internal_edges_and_obligation_bindings() {
    let mut g = graph();
    g.dependencies.push(WorkDependency {
        consumer: unit(0),
        producer: unit(1),
        relation: DependencyRelation::MustFinish,
    });
    assert!(matches!(
        validate_work_graph(&g, &limits()),
        Err(WorkError::ForeignScope(_))
    ));
    g.dependencies.clear();
    g.units[0].obligation_ids = vec![obligation(1)];
    assert!(matches!(
        validate_work_graph(&g, &limits()),
        Err(WorkError::UnknownObligation(_))
    ));
    g.units[0].obligation_ids.clear();
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::UnboundUnit(unit(0))
    );
}

#[test]
fn graph_rejects_unbounded_work_and_stale_affected_graph_version() {
    let mut g = graph();
    g.units[0].bounds.stop_conditions.clear();
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::UnboundedUnit(unit(0))
    );
    g = graph();
    g.units[0].bounds.max_attempts = rev(4);
    assert!(matches!(
        validate_work_graph(&g, &limits()),
        Err(WorkError::UnboundedUnit(_))
    ));
    g = graph();
    g.current_graph_version = row(2);
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::GraphVersionConflict
    );
}

#[test]
fn future_cycle_is_rejected_even_with_current_false_condition() {
    let mut g = graph();
    let mut a = link(
        0,
        0,
        1,
        PublicationSelector::AwaitOutput {
            producer_unit: unit(0),
            export_name: "report".into(),
        },
    );
    a.condition = LinkCondition::All {
        atoms: vec![ConditionAtom::ObligationHasStatus {
            obligation_id: obligation(0),
            status: ObligationStatus::Satisfied,
        }],
    };
    g.links.push(a);
    assert!(validate_work_graph(&g, &limits()).is_ok());
    g.links.push(link(
        1,
        1,
        0,
        PublicationSelector::AwaitOutput {
            producer_unit: unit(1),
            export_name: "report".into(),
        },
    ));
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::Cycle
    );
}

#[test]
fn future_owner_acceptance_node_participates_in_cycle_detection() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    let mut a = link(
        0,
        0,
        1,
        PublicationSelector::Pinned {
            publication_id: id(500),
        },
    );
    a.owner_acceptance = OwnerAcceptanceRequirement::RequiredForPublicationObjective;
    g.links.push(a);
    g.links.push(link(
        1,
        1,
        0,
        PublicationSelector::AwaitOutput {
            producer_unit: unit(1),
            export_name: "report".into(),
        },
    ));
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::Cycle
    );
}

#[test]
fn historical_publications_and_finite_iteration_are_not_root_level_cycles() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    publish(&mut g, 1, 1, 1);
    g.links.push(link(
        0,
        0,
        1,
        PublicationSelector::Pinned {
            publication_id: id(500),
        },
    ));
    g.links.push(link(
        1,
        1,
        0,
        PublicationSelector::Pinned {
            publication_id: id(501),
        },
    ));
    assert!(validate_work_graph(&g, &limits()).is_ok());
    g.publications.clear();
    g.publication_states.clear();
    g.links.clear();
    let mut a2 = g.units[0].clone();
    a2.unit = unit(2);
    g.units.push(a2);
    g.links.push(link(
        0,
        0,
        1,
        PublicationSelector::AwaitOutput {
            producer_unit: unit(0),
            export_name: "report".into(),
        },
    ));
    let mut back = link(
        1,
        1,
        0,
        PublicationSelector::AwaitOutput {
            producer_unit: unit(1),
            export_name: "report".into(),
        },
    );
    back.consumer_targets = vec![ConsumerTargetRef::Unit { unit: unit(2) }];
    g.links.push(back);
    assert!(validate_work_graph(&g, &limits()).is_ok());
}

#[test]
fn await_output_never_substitutes_a_different_unit_revision() {
    let mut g = graph();
    g.units[0].unit.revision = rev(2);
    publish(&mut g, 0, 0, 1);
    set_publication_revision(&mut g.publications[0], 3);
    g.links.push(link(
        0,
        0,
        1,
        PublicationSelector::AwaitOutput {
            producer_unit: g.units[0].unit,
            export_name: "report".into(),
        },
    ));
    let v = validate_work_graph(&g, &limits()).unwrap();
    let result = assess_link(&v, id(200)).unwrap();
    assert_eq!(result.selected_publication_id, None);
    assert_eq!(
        result.blocking_reasons,
        vec![BlockingReason::AwaitingPublication { link_id: id(200) }]
    );
    set_publication_revision(&mut g.publications[0], 2);
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert_eq!(
        assess_link(&v, id(200)).unwrap().selected_publication_id,
        Some(id(500))
    );
}

#[test]
fn owner_acceptance_requires_exact_objective_and_included_publication() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    let mut l = link(
        0,
        0,
        1,
        PublicationSelector::Pinned {
            publication_id: id(500),
        },
    );
    l.owner_acceptance = OwnerAcceptanceRequirement::RequiredForPublicationObjective;
    g.links.push(l);
    g.roots[0].stage = RootStage::OwnerAccepted;
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert_eq!(
        assess_link(&v, id(200)).unwrap().blocking_reasons,
        vec![BlockingReason::OwnerAcceptancePending {
            publication_id: id(500)
        }]
    );
    g.owner_acceptances.push(OwnerAcceptanceBinding {
        binding_id: id(1500),
        producer_objective_id: scope(0).objective_id,
        candidate_id: id(1501),
        owner_event_id: id(1502),
        included_publication_ids: vec![],
        valid: true,
        row_version: row(1),
    });
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert!(
        !assess_link(&v, id(200))
            .unwrap()
            .blocking_reasons
            .is_empty()
    );
    g.owner_acceptances[0]
        .included_publication_ids
        .push(id(500));
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert!(
        assess_link(&v, id(200))
            .unwrap()
            .blocking_reasons
            .is_empty()
    );
    assert!(
        assess_link(&v, id(200))
            .unwrap()
            .read_set
            .iter()
            .any(|r| r.key.kind == DependencyKind::OwnerAcceptanceBinding)
    );
}

#[test]
fn pinned_reads_eligibility_but_does_not_follow_head_implicitly() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    publish(&mut g, 1, 0, 2);
    g.publication_bindings.push(PublicationBinding {
        binding_id: id(1600),
        producer_work_id: scope(0).work_id,
        export_name: "report".into(),
        head_publication_id: id(501),
        row_version: row(2),
    });
    g.links.push(link(
        0,
        0,
        1,
        PublicationSelector::Pinned {
            publication_id: id(500),
        },
    ));
    let v = validate_work_graph(&g, &limits()).unwrap();
    let assessment = assess_link(&v, id(200)).unwrap();
    assert_eq!(assessment.selected_publication_id, Some(id(500)));
    assert!(
        assessment
            .read_set
            .iter()
            .any(|r| r.key.kind == DependencyKind::PublicationState)
    );
    assert!(
        !assessment
            .read_set
            .iter()
            .any(|r| r.key.kind == DependencyKind::PublicationBinding)
    );
    let p = packet(&v, 1);
    g.publication_states[0].eligibility = PublicationEligibility::Unresolved;
    g.publication_states[0].reason_evidence_id = Some(id(1601));
    g.publication_states[0].row_version = row(2);
    assert!(matches!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::StaleInput(DependencyKey {
            kind: DependencyKind::PublicationState,
            ..
        }))
    ));
}

#[test]
fn follow_head_policy_and_read_version_are_explicit() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    g.publication_bindings.push(PublicationBinding {
        binding_id: id(1600),
        producer_work_id: scope(0).work_id,
        export_name: "report".into(),
        head_publication_id: id(500),
        row_version: row(1),
    });
    g.links.push(link(
        0,
        0,
        1,
        PublicationSelector::FollowHead {
            binding_id: id(1600),
        },
    ));
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::UnsupportedLinkPolicy
    );
    g.links[0].on_change = SourceChangePolicy::RequireRevalidation;
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert!(
        assess_link(&v, id(200))
            .unwrap()
            .read_set
            .iter()
            .any(|r| r.key.kind == DependencyKind::PublicationBinding)
    );
    let p = packet(&v, 1);
    publish(&mut g, 1, 0, 2);
    g.publication_bindings[0].head_publication_id = id(501);
    g.publication_bindings[0].row_version = row(2);
    assert!(matches!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::StaleInput(_))
    ));
}

#[test]
fn false_condition_still_records_reads_and_unknown_never_allows_branch() {
    let mut g = graph();
    let mut l = link(
        0,
        0,
        1,
        PublicationSelector::AwaitOutput {
            producer_unit: unit(0),
            export_name: "report".into(),
        },
    );
    l.condition = LinkCondition::All {
        atoms: vec![ConditionAtom::ObligationHasStatus {
            obligation_id: obligation(0),
            status: ObligationStatus::Satisfied,
        }],
    };
    g.links.push(l);
    let v = validate_work_graph(&g, &limits()).unwrap();
    let a = assess_link(&v, id(200)).unwrap();
    assert_eq!(a.condition, ConditionResult::False);
    assert!(a.blocking_reasons.is_empty());
    let p = packet(&v, 1);
    g.objectives[0].obligations[0].status = ObligationStatus::Satisfied;
    g.objectives[0].obligations[0].row_version = row(2);
    assert!(matches!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::StaleInput(_))
    ));
    g.objectives[0].obligations[0].status = ObligationStatus::Unknown;
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert_eq!(
        assess_link(&v, id(200)).unwrap().condition,
        ConditionResult::Unknown
    );
    assert!(
        !assess_link(&v, id(200))
            .unwrap()
            .blocking_reasons
            .is_empty()
    );
}

#[test]
fn malformed_conditions_and_foreign_targets_are_rejected() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    let mut l = link(
        0,
        0,
        1,
        PublicationSelector::Pinned {
            publication_id: id(500),
        },
    );
    l.condition = LinkCondition::All {
        atoms: vec![ConditionAtom::AcceptedFieldEquals {
            publication_id: id(500),
            field_id: id(9999),
            value: CanonicalScalar::Boolean(true),
        }],
    };
    g.links.push(l);
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::InvalidLinkCondition
    );
    g.links[0].condition = LinkCondition::Always {};
    g.links[0].consumer_targets = vec![ConsumerTargetRef::Unit { unit: unit(0) }];
    assert!(matches!(
        validate_work_graph(&g, &limits()),
        Err(WorkError::ForeignScope(_))
    ));
}

#[test]
fn submitted_cannot_skip_verification_and_wrong_exact_proof_cannot_accept() {
    let g = graph();
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 0);
    let proof = verification(&p, 0);
    assert_eq!(
        advance(&g, &p, ResultStage::Accepted, Some(&proof)),
        Err(WorkError::InvalidTransition)
    );
    let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
    let mut wrong = proof.clone();
    wrong.result_fingerprint = [3; 32];
    assert_eq!(
        advance(&g, &verifying, ResultStage::Accepted, Some(&wrong)),
        Err(WorkError::VerificationMismatch)
    );
    wrong = proof.clone();
    wrong.snapshot_id = id(9999);
    assert_eq!(
        advance(&g, &verifying, ResultStage::Accepted, Some(&wrong)),
        Err(WorkError::VerificationMismatch)
    );
    wrong = proof.clone();
    wrong.verifier_attempt_id = p.attempt_id;
    assert_eq!(
        advance(&g, &verifying, ResultStage::Accepted, Some(&wrong)),
        Err(WorkError::VerificationMismatch)
    );
    let accepted = advance(&g, &verifying, ResultStage::Accepted, Some(&proof)).unwrap();
    assert_eq!(accepted.row_version, row(3));
    assert_eq!(
        advance(&g, &accepted, ResultStage::Verifying, None),
        Err(WorkError::InvalidTransition)
    );
    assert_eq!(p.stage, ResultStage::Submitted);
}

#[test]
fn unrelated_revision_allows_independent_result_but_read_change_does_not() {
    let mut g = graph();
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 0);
    g.units[1].row_version = row(2);
    g.objectives[1].obligations[0].row_version = row(2);
    let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
    let proof = verification(&verifying, 0);
    g.objectives[0].obligations[0].row_version = row(2);
    assert!(matches!(
        advance(&g, &verifying, ResultStage::Accepted, Some(&proof)),
        Err(WorkError::StaleInput(_))
    ));
    assert_eq!(
        advance(&g, &verifying, ResultStage::Stale, None)
            .unwrap()
            .stage,
        ResultStage::Stale
    );
}

#[test]
fn missing_mandatory_link_or_pinned_eligibility_read_is_rejected() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    g.links.push(link(
        0,
        0,
        1,
        PublicationSelector::Pinned {
            publication_id: id(500),
        },
    ));
    let v = validate_work_graph(&g, &limits()).unwrap();
    let mut p = packet(&v, 1);
    p.snapshot
        .read_set
        .retain(|r| r.key.kind != DependencyKind::PublicationState);
    assert!(matches!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::IncompleteSnapshot(_))
    ));
    p = packet(&v, 1);
    p.used_links.clear();
    assert!(matches!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::IncompleteSnapshot(_))
    ));
}

#[test]
fn stale_unit_revision_attempt_and_write_version_cannot_accept() {
    let mut g = graph();
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 0);
    let current = rows(&v);
    let view = ResultCommitView {
        graph: &v,
        current_rows: &current,
        expected_unit_row_version: row(2),
        expected_result_row_version: row(1),
        active_attempt_id: p.attempt_id,
    };
    assert_eq!(
        transition_result(&p, ResultStage::Verifying, None, &view),
        Err(WorkError::RowVersionConflict)
    );
    let view = ResultCommitView {
        expected_unit_row_version: row(1),
        active_attempt_id: id(9999),
        ..view
    };
    assert_eq!(
        transition_result(&p, ResultStage::Verifying, None, &view),
        Err(WorkError::ResultBasisMismatch)
    );
    g.units[0].unit.revision = rev(2);
    assert_eq!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::ResultBasisMismatch)
    );
}

#[test]
fn completion_candidate_is_not_owner_acceptance_and_owner_basis_is_exact() {
    let mut g = graph();
    g.objectives[0].obligations.push(Obligation {
        obligation_id: id(1700),
        objective_id: scope(0).objective_id,
        parent_id: None,
        kind: ObligationKind::Acceptance,
        statement: "Owner approves final deliverable".into(),
        verification_method: VerificationMethod::OwnerJudgment,
        status: ObligationStatus::Open,
        waiver_owner_event_id: None,
        row_version: row(1),
    });
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 0);
    let proof = verification(&p, 0);
    let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
    let accepted = advance(&g, &verifying, ResultStage::Accepted, Some(&proof)).unwrap();
    bind_accepted(&mut g, &accepted);
    let v = validate_work_graph(&g, &limits()).unwrap();
    let current = rows(&v);
    let candidate = derive_completion_candidate(
        id(1800),
        scope(0).objective_id,
        &v,
        &[AcceptedResultEvidence {
            packet: &accepted,
            verification: &proof,
        }],
        &current,
    )
    .unwrap();
    assert_eq!(candidate.stage(), RootStage::CompletionCandidate);
    assert_eq!(candidate.owner_judgment_obligations(), &[id(1700)]);
    assert_eq!(g.roots[0].stage, RootStage::Executing);
    let mut decision = OwnerAcceptanceDecision {
        owner_event_id: id(1801),
        candidate_id: candidate.candidate_id(),
        scope: candidate.scope(),
        reviewed_results: candidate.results().to_vec(),
        final_verification_id: id(1802),
        final_verdict: VerificationVerdict::Pass,
        accepted_owner_obligations: vec![id(1700)],
    };
    assert!(validate_owner_acceptance(&candidate, &decision).is_ok());
    decision.candidate_id = id(1803);
    assert_eq!(
        validate_owner_acceptance(&candidate, &decision),
        Err(WorkError::OwnerAcceptanceMismatch)
    );
    assert!(
        derive_completion_candidate(id(1804), scope(0).objective_id, &v, &[], &current).is_err()
    );
}

#[test]
fn withdrawn_used_input_prevents_completion_without_erasing_accepted_packet() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    g.links.push(link(
        0,
        0,
        1,
        PublicationSelector::Pinned {
            publication_id: id(500),
        },
    ));
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 1);
    let proof = verification(&p, 1);
    let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
    let accepted = advance(&g, &verifying, ResultStage::Accepted, Some(&proof)).unwrap();
    bind_accepted(&mut g, &accepted);
    g.publication_states[0].eligibility = PublicationEligibility::Withdrawn;
    g.publication_states[0].reason_evidence_id = Some(id(1900));
    g.publication_states[0].row_version = row(2);
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert!(
        derive_completion_candidate(
            id(1901),
            scope(1).objective_id,
            &v,
            &[AcceptedResultEvidence {
                packet: &accepted,
                verification: &proof
            }],
            &rows(&v)
        )
        .is_err()
    );
    assert_eq!(accepted.stage, ResultStage::Accepted);
}

#[test]
fn version_overflow_fails_without_mutation() {
    let g = graph();
    let v = validate_work_graph(&g, &limits()).unwrap();
    let mut p = packet(&v, 0);
    p.row_version = row(u64::MAX);
    assert_eq!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::VersionOverflow)
    );
    assert_eq!(p.stage, ResultStage::Submitted);
    assert_eq!(p.row_version.get(), u64::MAX);
}

#[test]
fn internal_predecessor_requires_accepted_output_and_exact_snapshot_rows() {
    let mut g = graph();
    let mut predecessor = g.units[0].clone();
    predecessor.unit = unit(2);
    predecessor.stage = UnitStage::Ready;
    g.units.push(predecessor);
    g.dependencies.push(WorkDependency {
        consumer: unit(0),
        producer: unit(2),
        relation: DependencyRelation::MustSupply,
    });
    let v = validate_work_graph(&g, &limits()).unwrap();
    let mut p = packet(&v, 0);
    assert!(matches!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::Blocked(BlockingReason::DependencyPending { .. }))
    ));
    g.units[2].stage = UnitStage::Accepted;
    g.units[2].accepted_result = Some(AcceptedResultRef {
        result_id: id(2000),
        snapshot_id: id(2001),
        fingerprint: [5; 32],
        result_row_version: row(3),
    });
    assert!(matches!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::IncompleteSnapshot(_))
    ));
    let v = validate_work_graph(&g, &limits()).unwrap();
    p.snapshot
        .read_set
        .extend(graph_dependency_rows(&v).into_iter().filter(|r| {
            matches!(
                r.key.kind,
                DependencyKind::UnitResultBinding | DependencyKind::AcceptedResult
            )
        }));
    let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
    let proof = verification(&p, 0);
    assert!(advance(&g, &verifying, ResultStage::Accepted, Some(&proof)).is_ok());
    g.units[2].row_version = row(2);
    assert!(matches!(
        advance(&g, &verifying, ResultStage::Accepted, Some(&proof)),
        Err(WorkError::StaleInput(_))
    ));
    g.dependencies[0].relation = DependencyRelation::MustAuthorize;
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 0);
    assert!(matches!(
        advance(&g, &p, ResultStage::Verifying, None),
        Err(WorkError::UnsupportedDependencyRelation(_))
    ));
}

#[test]
fn cancelled_lost_or_wrong_stage_unit_and_active_root_gate_cannot_accept() {
    let g = graph();
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 0);
    let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
    let proof = verification(&p, 0);
    for stage in [
        UnitStage::Cancelled,
        UnitStage::Cancelling,
        UnitStage::Lost,
        UnitStage::Ready,
        UnitStage::Running,
    ] {
        let mut changed = g.clone();
        changed.units[0].stage = stage;
        assert_eq!(
            advance(&changed, &verifying, ResultStage::Accepted, Some(&proof)),
            Err(WorkError::InvalidTransition)
        );
    }
    for reason in [
        BlockingReason::StopRequested {},
        BlockingReason::Recovery {},
        BlockingReason::UnknownOutcome {},
        BlockingReason::IntegrityFailure {},
    ] {
        let mut changed = g.clone();
        changed.roots[0].blocking_reasons.insert(reason.clone());
        assert_eq!(
            advance(&changed, &verifying, ResultStage::Accepted, Some(&proof)),
            Err(WorkError::Blocked(reason))
        );
        assert_eq!(verifying.stage, ResultStage::Verifying);
    }
}

#[test]
fn strict_wire_forms_reject_unknown_fields_zero_and_out_of_range_numbers() {
    // Canonical GraphCandidate is intentionally not deserializable. Only the
    // proposed unit wire form is parsed here; compile-fail docs cover the graph.
    let original = graph().units.remove(0);
    let serialized = serde_json::to_value(&original).unwrap();
    let mut value = serialized.clone();
    value["bounds"]["unlimited"] = serde_json::json!(true);
    assert!(serde_json::from_value::<WorkUnitSpec>(value).is_err());
    let mut value = serialized.clone();
    value["bounds"]["max_attempts"] = serde_json::json!(0);
    assert!(serde_json::from_value::<WorkUnitSpec>(value).is_err());
    let mut value = serialized.clone();
    value["bounds"]["max_attempts"] = serde_json::json!(4294967296u64);
    assert!(serde_json::from_value::<WorkUnitSpec>(value).is_err());
    let mut value = serialized.clone();
    value["kind"] = serde_json::json!("execute_arbitrary_code");
    assert!(serde_json::from_value::<WorkUnitSpec>(value).is_err());
    assert!(
        serde_json::from_str::<LinkCondition>(r#"{"kind":"all","atoms":[],"script":"allow()"}"#)
            .is_err()
    );
    assert_eq!(
        serde_json::from_value::<WorkUnitSpec>(serialized).unwrap(),
        original
    );
}

fn set_publication_revision(p: &mut ResultPublication, revision: u32) {
    p.producer_unit.revision = rev(revision);
    p.source.unit.unit = p.producer_unit;
    p.source.packet.unit = p.producer_unit;
    p.source.packet.snapshot.unit = p.producer_unit;
    p.source.verification.unit = p.producer_unit;
}

#[test]
fn every_zero_field_condition_and_gate_rejects_unknown_wire_fields() {
    assert!(
        serde_json::from_str::<LinkCondition>(r#"{"kind":"always","code":"allow()"}"#).is_err()
    );
    assert!(serde_json::from_str::<LinkCondition>(r#"{"kind":"always","kind":"always"}"#).is_err());
    for kind in [
        "recovery",
        "integrity_failure",
        "resource_unavailable",
        "stop_requested",
        "unknown_outcome",
        "awaiting_owner",
    ] {
        let unknown = format!("{{\"kind\":\"{kind}\",\"allow_effect\":true}}");
        assert!(
            serde_json::from_str::<BlockingReason>(&unknown).is_err(),
            "accepted unknown field: {unknown}"
        );
        let duplicate = format!("{{\"kind\":\"{kind}\",\"kind\":\"{kind}\"}}");
        assert!(serde_json::from_str::<BlockingReason>(&duplicate).is_err());
    }
}

#[test]
fn a_redundant_unbound_unit_is_rejected_even_when_another_unit_covers_objective() {
    let mut g = graph();
    let mut extra = g.units[0].clone();
    extra.unit = unit(2);
    extra.obligation_ids.clear();
    g.units.push(extra);
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::UnboundUnit(unit(2))
    );
}

#[test]
fn regression_acceptance_criterion_must_belong_to_unit_obligations() {
    let mut g = graph();
    let mut other = g.objectives[0].obligations[0].clone();
    other.obligation_id = id(2100);
    other.kind = ObligationKind::Preference;
    g.objectives[0].obligations.push(other);
    g.units[0].acceptance_criteria = vec![id(2100)];
    assert!(
        validate_work_graph(&g, &limits()).is_err(),
        "accepted criterion B even though unit only owns obligation A"
    );
}

#[test]
fn regression_publication_must_match_exact_canonical_producer() {
    let mut original = graph();
    publish(&mut original, 0, 0, 1);
    for corruption in 0..7 {
        let mut g = original.clone();
        let p = &mut g.publications[0];
        match corruption {
            0 => p.producer_unit.unit_id = id(2101),
            1 => p.producer_unit = unit(1),
            2 => p.producer_unit.revision = rev(2),
            3 => p.output.export_name = "uncontracted_output".into(),
            4 => p.result_id = id(2102),
            5 => p.result_fingerprint = [99; 32],
            _ => p.verification_id = id(2103),
        }
        assert!(
            validate_work_graph(&g, &limits()).is_err(),
            "accepted publication corruption {corruption}"
        );
    }
}

#[test]
fn regression_verification_level_uses_canonical_vocabulary() {
    for level in [
        "machine_only",
        "independent_review",
        "diverse_challenge",
        "protected_admission",
    ] {
        assert!(
            serde_json::from_value::<VerificationLevel>(serde_json::json!(level)).is_ok(),
            "missing canonical level {level}"
        );
    }
    assert!(serde_json::from_str::<VerificationLevel>("\"self_check\"").is_err());
}

#[test]
fn semantic_acceptance_requires_exact_qualified_evidence_not_level_label() {
    let g = graph();
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 0);
    let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
    let proof = verification(&p, 0);
    for corruption in 0..10 {
        let mut bad = proof.clone();
        match corruption {
            0 => bad.evidence.qualification = None,
            1 => bad.evidence.qualification.as_mut().unwrap().status = QualificationStatus::Stale,
            2 => bad.evidence.qualification.as_mut().unwrap().criterion_ids = vec![id(9999)],
            3 => bad.evidence.qualification.as_mut().unwrap().work_kind = WorkKind::Summary,
            4 => bad.evidence.level = VerificationLevel::ProtectedAdmission,
            5 => bad.evidence.independence_assessment_id = None,
            6 => bad.evidence.scope.objective_revision = rev(2),
            7 => bad.evidence.level = VerificationLevel::MachineOnly,
            8 => {
                bad.evidence
                    .qualification
                    .as_mut()
                    .unwrap()
                    .method_revision_id = id(9998)
            }
            _ => bad.evidence.required_level = VerificationLevel::ProtectedAdmission,
        }
        assert_eq!(
            advance(&g, &verifying, ResultStage::Accepted, Some(&bad)),
            Err(WorkError::VerificationMismatch),
            "accepted corruption {corruption}"
        );
    }
}

#[test]
fn machine_only_and_protected_admission_have_distinct_evidence_requirements() {
    for (kind, level) in [
        (WorkKind::DeterministicCheck, VerificationLevel::MachineOnly),
        (
            WorkKind::ModelAdaptation,
            VerificationLevel::ProtectedAdmission,
        ),
    ] {
        let mut g = graph();
        g.units[0].kind = kind;
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let mut proof = verification(&p, 0);
        proof.evidence.work_kind = kind;
        proof.evidence.level = level;
        proof.evidence.required_level = level;
        if level == VerificationLevel::MachineOnly {
            proof.evidence.qualification = None;
            proof.evidence.independence_assessment_id = None;
            assert!(advance(&g, &verifying, ResultStage::Accepted, Some(&proof)).is_ok());
            g.objectives[0].obligations[0].verification_method =
                VerificationMethod::IndependentReview;
            assert_eq!(
                advance(&g, &verifying, ResultStage::Accepted, Some(&proof)),
                Err(WorkError::VerificationMismatch)
            );
        } else {
            let qualification = proof.evidence.qualification.as_mut().unwrap();
            qualification.work_kind = kind;
            qualification.level = level;
            assert_eq!(
                advance(&g, &verifying, ResultStage::Accepted, Some(&proof)),
                Err(WorkError::VerificationMismatch)
            );
            proof.evidence.protected_admission_evidence_id = Some(id(3000));
            assert!(advance(&g, &verifying, ResultStage::Accepted, Some(&proof)).is_ok());
        }
    }
}

#[test]
fn historical_pinned_source_survives_new_objective_and_unit_revision_but_not_withdrawal() {
    let mut g = graph();
    publish(&mut g, 0, 0, 1);
    let historical = g.publications[0].clone();
    let mut new_scope = scope(0);
    new_scope.objective_id = id(3100);
    new_scope.objective_revision = rev(2);
    g.roots[0].active_objective_id = new_scope.objective_id;
    g.objectives[0].scope = new_scope;
    g.objectives[0].obligations[0].objective_id = new_scope.objective_id;
    g.objectives[0].obligations[0].obligation_id = id(3101);
    g.objectives[0].completion_criteria = vec![id(3101)];
    g.units[0].scope = new_scope;
    g.units[0].unit.revision = rev(2);
    g.units[0].expected_output.export_name = "new_report".into();
    g.units[0].obligation_ids = vec![id(3101)];
    g.units[0].acceptance_criteria = vec![id(3101)];
    g.links.push(link(
        0,
        0,
        1,
        PublicationSelector::Pinned {
            publication_id: id(500),
        },
    ));
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert!(
        assess_link(&v, id(200))
            .unwrap()
            .blocking_reasons
            .is_empty()
    );
    assert_eq!(g.publications[0], historical);
    g.publication_states[0].eligibility = PublicationEligibility::Withdrawn;
    g.publication_states[0].reason_evidence_id = Some(id(3102));
    g.publication_states[0].row_version = row(2);
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert_eq!(
        assess_link(&v, id(200)).unwrap().blocking_reasons,
        vec![BlockingReason::PublicationWithdrawn {
            publication_id: id(500)
        }]
    );
}

#[test]
fn publication_source_must_match_accepted_binding_manifest_and_qualification() {
    let mut original = graph();
    publish(&mut original, 0, 0, 1);
    for corruption in 0..7 {
        let mut g = original.clone();
        let p = &mut g.publications[0];
        match corruption {
            0 => p.source.unit.accepted_result = None,
            1 => p.source.unit.accepted_result.as_mut().unwrap().fingerprint = [99; 32],
            2 => p.source.packet.stage = ResultStage::Submitted,
            3 => p.output_manifest_evidence_id = id(9999),
            4 => p.source.verification.evidence.qualification = None,
            5 => p.verification_level = VerificationLevel::ProtectedAdmission,
            _ => p.source.unit.scope.session_id = id(9999),
        }
        assert!(
            validate_work_graph(&g, &limits()).is_err(),
            "accepted corrupt source {corruption}"
        );
    }
}

#[test]
fn all_condition_intentionally_requires_resolved_atoms_and_keeps_every_read() {
    let mut g = graph();
    g.objectives[1].obligations[0].status = ObligationStatus::Unknown;
    let mut l = link(
        0,
        0,
        1,
        PublicationSelector::AwaitOutput {
            producer_unit: unit(0),
            export_name: "report".into(),
        },
    );
    l.condition = LinkCondition::All {
        atoms: vec![
            ConditionAtom::ObligationHasStatus {
                obligation_id: obligation(0),
                status: ObligationStatus::Satisfied,
            },
            ConditionAtom::ObligationHasStatus {
                obligation_id: obligation(1),
                status: ObligationStatus::Satisfied,
            },
        ],
    };
    g.links.push(l);
    let v = validate_work_graph(&g, &limits()).unwrap();
    let a = assess_link(&v, id(200)).unwrap();
    assert_eq!(a.condition, ConditionResult::Unknown); // False + Unknown is not a proven exclusion.
    for id in [obligation(0), obligation(1)] {
        assert!(
            a.read_set
                .iter()
                .any(|r| r.key.kind == DependencyKind::Obligation && r.key.entity_id == id)
        );
    }
    if let LinkCondition::All { atoms } = &mut g.links[0].condition {
        atoms.reverse();
    }
    let v = validate_work_graph(&g, &limits()).unwrap();
    assert_eq!(
        assess_link(&v, id(200)).unwrap().condition,
        ConditionResult::Unknown
    );
}

#[test]
fn versioned_structural_limits_reject_oversized_names_and_conditions() {
    let structural = dc_domain::structure::STRUCTURAL_LIMITS_V1;
    let mut g = graph();
    g.units[0].expected_output.export_name = "a".repeat(structural.max_export_name_bytes + 1);
    assert!(matches!(
        validate_work_graph(&g, &limits()),
        Err(WorkError::InvalidField("export_name"))
    ));
    g = graph();
    let mut l = link(
        0,
        0,
        1,
        PublicationSelector::AwaitOutput {
            producer_unit: unit(0),
            export_name: "report".into(),
        },
    );
    l.condition = LinkCondition::All {
        atoms: vec![
            ConditionAtom::ObligationHasStatus {
                obligation_id: obligation(0),
                status: ObligationStatus::Open
            };
            structural.max_condition_atoms + 1
        ],
    };
    g.links.push(l);
    assert_eq!(
        validate_work_graph(&g, &limits()).unwrap_err(),
        WorkError::InvalidLinkCondition
    );
}

#[test]
fn deterministic_transition_sequences_never_accept_without_matching_proof() {
    let g = graph();
    let v = validate_work_graph(&g, &limits()).unwrap();
    let p = packet(&v, 0);
    let proof = verification(&p, 0);
    for target in [
        ResultStage::Submitted,
        ResultStage::Verifying,
        ResultStage::Accepted,
        ResultStage::Rejected,
        ResultStage::Stale,
    ] {
        let result = advance(&g, &p, target, Some(&proof));
        assert_eq!(result.is_ok(), target == ResultStage::Verifying);
    }
    for seed in 0..64u64 {
        let mut current = p.clone();
        let mut rng = seed + 1;
        for _ in 0..20 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let target = [
                ResultStage::Verifying,
                ResultStage::Accepted,
                ResultStage::Rejected,
                ResultStage::Stale,
            ][(rng % 4) as usize];
            let before = current.clone();
            match advance(&g, &current, target, Some(&proof)) {
                Ok(next) => {
                    assert_eq!(next.row_version.get(), before.row_version.get() + 1);
                    current = next;
                }
                Err(_) => assert_eq!(current, before),
            }
            if current.stage == ResultStage::Accepted {
                assert!(current.row_version.get() >= 3);
            }
        }
    }
}
