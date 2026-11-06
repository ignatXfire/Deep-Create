use super::*;

#[path = "owner_rejection.rs"]
mod rejection;

fn unrejected_fixture(
    count: u32,
) -> (
    GraphCandidate,
    RootLifecycleState,
    OwnerRejectionDecision,
    OwnerReturnReview,
    GraphLimits,
) {
    let (mut g, packet, proof, limits) = completion_fixture();
    let graph = validate_work_graph(&g, &limits).unwrap();
    let mut rows = graph_dependency_rows(&graph);
    rows.push(packet.verification_policy.clone());
    let evidence = [AcceptedResultEvidence {
        packet: &packet,
        verification: &proof,
    }];
    let candidate = derive_completion_candidate(id(30), id(3), &graph, &evidence, &rows).unwrap();
    let reviewed = validate_final_review(
        &candidate,
        &FinalReviewEvidence {
            verification_id: id(31),
            candidate_id: id(30),
            scope: scope(),
            reviewed_results: candidate.results().to_vec(),
            verdict: VerificationVerdict::Pass,
            source_verification: Some(test_verification_record(id(31), &unit())),
        },
        &unit(),
        &test_objective(),
    )
    .unwrap();
    let mut state = RootLifecycleState {
        root: root(),
        scope: scope(),
        candidate: Some(candidate),
        final_review: Some(reviewed),
        pending_owner_rejection: None,
    };
    state.root.stage = RootStage::AwaitingOwnerAcceptance;
    state.root.row_version = row(5);
    g.roots[0] = state.root.clone();
    for n in 0..count {
        g.publications.push(ResultPublication {
            publication_id: id(100 + n as u128),
            producer_work_id: scope().work_id,
            producer_objective_id: scope().objective_id,
            producer_unit: packet.unit,
            export_revision: rev(n + 1),
            output: g.units[0].expected_output.clone(),
            result_id: packet.result_id,
            result_fingerprint: packet.fingerprint,
            verification_id: proof.verification_id,
            verification_level: VerificationLevel::IndependentReview,
            accepted_fields: vec![],
            source: PublicationSource {
                objective: g.objectives[0].clone(),
                unit: g.units[0].clone(),
                packet: packet.clone(),
                verification: proof.clone(),
                output_manifest_evidence_id: id(200 + n as u128),
                accepted_fields: vec![],
            },
            output_manifest_evidence_id: id(200 + n as u128),
            created_event_id: id(300 + n as u128),
            predecessor_publication_id: None,
        });
        g.publication_states.push(PublicationState {
            publication_id: id(100 + n as u128),
            eligibility: PublicationEligibility::Valid,
            reason_evidence_id: None,
            row_version: row(1),
        });
    }
    let decision = OwnerRejectionDecision {
        owner_event_id: id(40),
        candidate_id: id(30),
        scope: scope(),
        final_verification_id: id(31),
        reason_evidence_id: id(41),
    };
    let review = OwnerReturnReview {
        assessment_id: id(42),
        owner_event_id: id(40),
        reason_evidence_id: id(41),
        candidate_id: id(30),
        scope: scope(),
        final_verification_id: id(31),
        assessment_evidence_id: id(43),
        expected_graph_version: g.current_graph_version,
        expected_objective_row_version: g.objectives[0].row_version,
        repair_obligation_ids: vec![id(5)],
        publication_assessments: vec![],
    };
    (g, state, decision, review, limits)
}

fn receive(
    g: &mut GraphCandidate,
    state: &mut RootLifecycleState,
    decision: &OwnerRejectionDecision,
    review: &mut OwnerReturnReview,
    limits: &GraphLimits,
) -> OwnerRejectionPlan {
    let graph = validate_work_graph(g, limits).unwrap();
    let received =
        plan_owner_rejection(state, state.root.row_version, scope(), &graph, decision).unwrap();
    *state = received.replacement().clone();
    g.roots[0] = state.root.clone();
    for replacement in received.publication_replacements() {
        *g.publication_states
            .iter_mut()
            .find(|p| p.publication_id == replacement.publication_id)
            .unwrap() = replacement.clone();
    }
    g.current_graph_version = row(g.current_graph_version.get() + 1);
    g.expected_graph_version = g.current_graph_version;
    review.expected_graph_version = g.current_graph_version;
    received
}

fn fixture(
    count: u32,
) -> (
    GraphCandidate,
    RootLifecycleState,
    OwnerRejectionDecision,
    OwnerReturnReview,
    GraphLimits,
) {
    let (mut g, mut state, decision, mut review, limits) = unrejected_fixture(count);
    receive(&mut g, &mut state, &decision, &mut review, &limits);
    (g, state, decision, review, limits)
}

fn plan(
    g: &GraphCandidate,
    state: &RootLifecycleState,
    decision: &OwnerRejectionDecision,
    review: &OwnerReturnReview,
    limits: &GraphLimits,
) -> Result<OwnerReturnPlan, WorkError> {
    let graph = validate_work_graph(g, limits).unwrap();
    plan_owner_return(
        state,
        state.root.row_version,
        scope(),
        &graph,
        decision,
        review,
    )
}

fn impact(n: u128, effect: PublicationImpact) -> PublicationImpactAssessment {
    PublicationImpactAssessment {
        publication_id: id(n),
        expected_row_version: row(2),
        effect,
    }
}

fn add_consumer(g: &mut GraphCandidate, n: u128, publication_id: EntityId) -> EntityId {
    let consumer_scope = WorkScope {
        work_id: id(1000 + n),
        objective_id: id(2000 + n),
        ..scope()
    };
    let obligation_id = id(4000 + n);
    let mut objective = g.objectives[0].clone();
    objective.scope = consumer_scope;
    objective.owner_event_ids = vec![id(5000 + n)];
    objective.obligations[0].obligation_id = obligation_id;
    objective.obligations[0].objective_id = consumer_scope.objective_id;
    objective.completion_criteria = vec![obligation_id];
    let mut consumer = unit();
    consumer.scope = consumer_scope;
    consumer.unit.unit_id = id(3000 + n);
    consumer.obligation_ids = vec![obligation_id];
    consumer.acceptance_criteria = vec![obligation_id];
    g.roots.push(RootWork {
        work_id: consumer_scope.work_id,
        session_id: scope().session_id,
        active_objective_id: consumer_scope.objective_id,
        stage: RootStage::Executing,
        blocking_reasons: BTreeSet::new(),
        row_version: row(1),
    });
    let link_id = id(6000 + n);
    g.links.push(RootWorkLink {
        link_id,
        revision_id: id(7000 + n),
        row_version: row(1),
        producer_work_id: scope().work_id,
        consumer_work_id: consumer_scope.work_id,
        consumer_objective_id: consumer_scope.objective_id,
        consumer_targets: vec![ConsumerTargetRef::Unit {
            unit: consumer.unit,
        }],
        relation: RootWorkRelation::RequiresResult,
        selector: PublicationSelector::Pinned { publication_id },
        required_output_schema: rev(1),
        required_verification_level: VerificationLevel::IndependentReview,
        owner_acceptance: OwnerAcceptanceRequirement::NotRequired,
        condition: LinkCondition::Always {},
        freshness_policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::Policy,
                entity_id: id(9000),
            },
            expected_revision: row(1),
            fingerprint: None,
        },
        on_change: SourceChangePolicy::KeepPinned,
        accepted_basis_ref: id(8000 + n),
    });
    g.units.push(consumer);
    g.objectives.push(objective);
    link_id
}

#[test]
fn returned_publication_blocks_its_consumer_without_stopping_proven_independent_consumer() {
    let (mut g, mut state, decision, mut review, limits) = unrejected_fixture(2);
    let dependent = add_consumer(&mut g, 0, id(100));
    let independent = add_consumer(&mut g, 1, id(101));
    review.publication_assessments = vec![impact(
        101,
        PublicationImpact::ProvenUnaffected {
            evidence_id: id(44),
        },
    )];
    let before = g.clone();
    let v = validate_work_graph(&g, &limits).unwrap();
    let dependent_read = assess_link(&v, dependent).unwrap();
    let independent_read = assess_link(&v, independent).unwrap();
    assert!(dependent_read.blocking_reasons.is_empty());
    assert!(independent_read.blocking_reasons.is_empty());
    receive(&mut g, &mut state, &decision, &mut review, &limits);
    let pending_graph = g.clone();
    let v = validate_work_graph(&g, &limits).unwrap();
    assert!(
        !assess_link(&v, dependent)
            .unwrap()
            .blocking_reasons
            .is_empty()
    );
    assert!(
        !assess_link(&v, independent)
            .unwrap()
            .blocking_reasons
            .is_empty()
    );
    let result = plan(&g, &state, &decision, &review, &limits).unwrap();
    // Synthetic application of the WHOLE proposal; this is not a C-03 commit.
    g.roots[0] = result.replacement().root.clone();
    for replacement in result.publication_replacements() {
        *g.publication_states
            .iter_mut()
            .find(|p| p.publication_id == replacement.publication_id)
            .unwrap() = replacement.clone();
    }
    g.current_graph_version = row(3);
    g.expected_graph_version = row(3);
    let v = validate_work_graph(&g, &limits).unwrap();
    assert_eq!(
        assess_link(&v, dependent).unwrap().blocking_reasons,
        vec![BlockingReason::PublicationUnresolved {
            publication_id: id(100)
        }]
    );
    assert_ne!(
        assess_link(&v, dependent).unwrap().read_set,
        dependent_read.read_set
    );
    assert_ne!(
        assess_link(&v, independent).unwrap().read_set,
        independent_read.read_set
    );
    assert!(
        assess_link(&v, independent)
            .unwrap()
            .blocking_reasons
            .is_empty()
    );
    assert_eq!(g.publications, before.publications);
    assert_eq!(g.objectives, before.objectives);
    assert_eq!(g.units, before.units);
    assert_eq!(&g.roots[1..], &before.roots[1..]);
    // A requirement of another work cannot be smuggled into the repair list.
    let mut foreign = review.clone();
    foreign.repair_obligation_ids = vec![id(4000)];
    assert_eq!(
        plan(&pending_graph, &state, &decision, &foreign, &limits),
        Err(WorkError::UnknownObligation(id(4000)))
    );
}

#[test]
fn return_retires_exact_completion_and_preserves_provenance_and_all_read_conditions() {
    let (g, state, decision, mut review, limits) = fixture(3);
    review.publication_assessments = vec![
        impact(
            100,
            PublicationImpact::ProvenUnaffected {
                evidence_id: id(44),
            },
        ),
        impact(101, PublicationImpact::Affected),
    ];
    let before = (g.clone(), state.clone(), decision.clone(), review.clone());
    let result = plan(&g, &state, &decision, &review, &limits).unwrap();
    assert_eq!(result.replacement().root.stage, RootStage::Executing);
    assert_eq!(result.replacement().root.row_version, row(7));
    assert!(result.replacement().candidate.is_none());
    assert!(result.replacement().final_review.is_none());
    assert!(result.replacement().pending_owner_rejection.is_none());
    assert_eq!(result.expected_root_row(), row(6));
    assert_eq!(result.expected_graph_row(), row(2));
    assert_eq!(result.expected_objective_row(), row(1));
    assert_eq!(result.publication_reads(), &g.publication_states);
    assert_eq!(
        result.publication_replacements(),
        &[PublicationState {
            publication_id: id(100),
            eligibility: PublicationEligibility::Valid,
            reason_evidence_id: None,
            row_version: row(3),
        }]
    );
    assert_eq!(
        result.resolved_rejection(),
        state.pending_owner_rejection.as_ref().unwrap()
    );
    assert_eq!(
        result.returned_completion(),
        state.final_review.as_ref().unwrap()
    );
    assert_eq!(result.decision(), &decision);
    assert_eq!(result.review(), &review);
    assert_eq!(before, (g, state, decision, review));
}

#[test]
fn unknown_and_omitted_assessments_never_mean_unaffected() {
    let (g, state, decision, review, limits) = fixture(1);
    let omitted = plan(&g, &state, &decision, &review, &limits).unwrap();
    for effect in [PublicationImpact::Unknown, PublicationImpact::Affected] {
        let mut explicit = review.clone();
        explicit.publication_assessments = vec![impact(100, effect)];
        let p = plan(&g, &state, &decision, &explicit, &limits).unwrap();
        assert_eq!(
            p.publication_replacements(),
            omitted.publication_replacements()
        );
    }
}

#[test]
fn ineligible_publications_are_never_reopened_or_overwritten() {
    let (mut g, state, decision, mut review, limits) = fixture(2);
    for (n, eligibility) in [
        PublicationEligibility::Unresolved,
        PublicationEligibility::Withdrawn,
    ]
    .into_iter()
    .enumerate()
    {
        g.publication_states[n].eligibility = eligibility;
        g.publication_states[n].reason_evidence_id = Some(id(90 + n as u128));
        g.publication_states[n].row_version = row(u64::MAX);
    }
    let unknown = plan(&g, &state, &decision, &review, &limits).unwrap();
    assert!(unknown.publication_replacements().is_empty());
    assert_eq!(unknown.publication_reads(), &g.publication_states);
    review.publication_assessments = (0..2)
        .map(|n| PublicationImpactAssessment {
            publication_id: id(100 + n),
            expected_row_version: row(u64::MAX),
            effect: PublicationImpact::ProvenUnaffected {
                evidence_id: id(44 + n),
            },
        })
        .collect();
    assert!(
        plan(&g, &state, &decision, &review, &limits)
            .unwrap()
            .publication_replacements()
            .is_empty()
    );
}

#[test]
fn return_clears_only_its_own_gate_among_all_blocking_combinations() {
    let (mut g, mut state, decision, review, limits) = fixture(0);
    let own_gate = BlockingReason::OwnerRejectionPending {
        owner_event_id: decision.owner_event_id,
    };
    for mask in 0..(1u32 << gate_contract_rows().len()) {
        state.root.blocking_reasons = gate_contract_rows()
            .into_iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, (g, _))| g)
            .collect();
        state.root.blocking_reasons.insert(own_gate.clone());
        let mut remaining = state.root.blocking_reasons.clone();
        remaining.remove(&own_gate);
        g.roots[0] = state.root.clone();
        let p = plan(&g, &state, &decision, &review, &limits).unwrap();
        assert_eq!(p.replacement().root.blocking_reasons, remaining);
        assert_eq!(
            evaluate_gates(
                &p.replacement().root.blocking_reasons,
                &[GateOperation::Effect]
            )
            .unwrap()
            .is_clear(),
            remaining.is_empty()
        );
    }
}

#[test]
fn exact_owner_candidate_reason_and_review_bindings_are_required() {
    let (g, state, decision, review, limits) = fixture(0);
    for n in 0..10 {
        let mut d = decision.clone();
        let mut r = review.clone();
        match n {
            0 => d.candidate_id = id(99),
            1 => d.scope.objective_revision = rev(2),
            2 => d.final_verification_id = id(99),
            3 => r.owner_event_id = id(99),
            4 => r.reason_evidence_id = id(99),
            5 => r.candidate_id = id(99),
            6 => r.scope.work_id = id(99),
            7 => r.final_verification_id = id(99),
            8 => d.owner_event_id = id(99),
            9 => d.reason_evidence_id = id(99),
            _ => unreachable!(),
        }
        assert_eq!(
            plan(&g, &state, &d, &r, &limits),
            Err(WorkError::OwnerRejectionMismatch)
        );
    }
}

#[test]
fn repair_cannot_add_foreign_unknown_empty_or_duplicate_obligations() {
    let (g, state, decision, review, limits) = fixture(0);
    for repair in [
        vec![],
        vec![id(99)],
        vec![id(5), id(5)],
        vec![id(5); MAX_ITEMS + 1],
    ] {
        let mut r = review.clone();
        r.repair_obligation_ids = repair;
        assert!(plan(&g, &state, &decision, &r, &limits).is_err());
    }
}

#[test]
fn stale_graph_objective_publication_or_root_versions_are_rejected() {
    let (g, state, decision, review, limits) = fixture(1);
    let graph = validate_work_graph(&g, &limits).unwrap();
    assert_eq!(
        plan_owner_return(&state, row(4), scope(), &graph, &decision, &review),
        Err(WorkError::RowVersionConflict)
    );
    for n in 0..3 {
        let mut r = review.clone();
        match n {
            0 => r.expected_graph_version = row(3),
            1 => r.expected_objective_row_version = row(2),
            _ => {
                r.publication_assessments = vec![PublicationImpactAssessment {
                    expected_row_version: row(3),
                    ..impact(
                        100,
                        PublicationImpact::ProvenUnaffected {
                            evidence_id: id(44),
                        },
                    )
                }]
            }
        }
        assert!(plan(&g, &state, &decision, &r, &limits).is_err());
    }
    let mut other_state = state.clone();
    other_state.root.row_version = row(7);
    assert_eq!(
        plan(&g, &other_state, &decision, &review, &limits),
        Err(WorkError::ResultBasisMismatch)
    );
}

#[test]
fn malformed_publication_assessments_fail_without_partial_plan() {
    let (g, state, decision, review, limits) = fixture(1);
    let valid = impact(100, PublicationImpact::Unknown);
    for assessments in [
        vec![impact(999, PublicationImpact::Unknown)],
        vec![valid.clone(), valid.clone()],
        vec![valid; MAX_ITEMS + 1],
    ] {
        let mut r = review.clone();
        r.publication_assessments = assessments;
        assert!(plan(&g, &state, &decision, &r, &limits).is_err());
    }
}

#[test]
fn root_and_publication_overflows_leave_every_input_unchanged() {
    for root_overflow in [false, true] {
        let (mut g, mut state, decision, review, limits) = fixture(2);
        if root_overflow {
            state.root.row_version = row(u64::MAX);
            g.roots[0] = state.root.clone();
        } else {
            g.publication_states[1].row_version = row(u64::MAX);
            g.publication_states[1].eligibility = PublicationEligibility::Valid;
        }
        let before = (g.clone(), state.clone());
        assert_eq!(
            plan(&g, &state, &decision, &review, &limits),
            Err(WorkError::VersionOverflow)
        );
        assert_eq!(before, (g, state));
    }
}

#[test]
fn stale_completion_can_be_returned_but_cannot_be_accepted_again() {
    let (mut g, state, decision, mut review, limits) = fixture(0);
    // Changed evidence does not trap the owner in an un-rejectable completion.
    g.objectives[0].obligations[0].row_version = row(2);
    g.objectives[0].row_version = row(2);
    review.expected_objective_row_version = row(2);
    let returned = plan(&g, &state, &decision, &review, &limits).unwrap();
    let acceptance = validate_owner_acceptance(
        state.candidate.as_ref().unwrap(),
        &OwnerAcceptanceDecision {
            owner_event_id: id(45),
            candidate_id: id(30),
            scope: scope(),
            reviewed_results: state.candidate.as_ref().unwrap().results().to_vec(),
            final_verification_id: id(31),
            final_verdict: VerificationVerdict::Pass,
            accepted_owner_obligations: vec![],
        },
    )
    .unwrap();
    assert_eq!(
        plan_root_transition(
            returned.replacement(),
            row(7),
            scope(),
            None,
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&acceptance)
        ),
        Err(WorkError::InvalidTransition)
    );
}

#[test]
fn missing_review_bad_scope_and_nonwaiting_states_cannot_return() {
    let (g, state, decision, review, limits) = fixture(0);
    for n in 0..5 {
        let mut s = state.clone();
        match n {
            0 => s.final_review = None,
            1 => s.candidate = None,
            2 => s.scope.objective_revision = rev(2),
            3 => s.root.stage = RootStage::OwnerAccepted,
            _ => {
                s.root.stage = RootStage::Executing;
                s.candidate = None;
                s.final_review = None;
            }
        }
        assert!(plan(&g, &s, &decision, &review, &limits).is_err());
    }
}
