use super::*;

fn reject(
    g: &GraphCandidate,
    state: &RootLifecycleState,
    decision: &OwnerRejectionDecision,
    limits: &GraphLimits,
) -> Result<OwnerRejectionPlan, WorkError> {
    let graph = validate_work_graph(g, limits).unwrap();
    plan_owner_rejection(state, state.root.row_version, scope(), &graph, decision)
}

fn owner_acceptance(state: &RootLifecycleState) -> OwnerAcceptedCandidate {
    let candidate = state.candidate.as_ref().unwrap();
    validate_owner_acceptance(
        candidate,
        &OwnerAcceptanceDecision {
            owner_event_id: id(55),
            candidate_id: candidate.candidate_id(),
            scope: scope(),
            reviewed_results: candidate.results().to_vec(),
            final_verification_id: state.final_review.as_ref().unwrap().verification_id(),
            final_verdict: VerificationVerdict::Pass,
            accepted_owner_obligations: vec![],
        },
    )
    .unwrap()
}

#[test]
fn initial_rejection_records_exact_pending_state_and_only_owns_its_new_holds() {
    let (mut g, mut state, decision, _, limits) = unrejected_fixture(3);
    state
        .root
        .blocking_reasons
        .insert(BlockingReason::StopRequested {});
    g.roots[0] = state.root.clone();
    g.publication_states[1].eligibility = PublicationEligibility::Unresolved;
    g.publication_states[1].reason_evidence_id = Some(id(80));
    g.publication_states[2].eligibility = PublicationEligibility::Withdrawn;
    g.publication_states[2].reason_evidence_id = Some(id(81));
    let before = (g.clone(), state.clone());
    let p = reject(&g, &state, &decision, &limits).unwrap();
    assert_eq!(p.expected_root_row(), row(5));
    assert_eq!(p.expected_graph_row(), row(1));
    assert_eq!(p.expected_objective_row(), row(1));
    assert_eq!(
        p.replacement().root.stage,
        RootStage::AwaitingOwnerAcceptance
    );
    assert_eq!(p.replacement().root.row_version, row(6));
    assert_eq!(p.replacement().candidate, state.candidate);
    assert_eq!(p.replacement().final_review, state.final_review);
    assert_eq!(
        p.replacement().root.blocking_reasons,
        BTreeSet::from([
            BlockingReason::StopRequested {},
            BlockingReason::OwnerRejectionPending {
                owner_event_id: id(40)
            }
        ])
    );
    assert_eq!(p.publication_reads(), &g.publication_states);
    assert_eq!(p.publication_replacements().len(), 1);
    let pending = p.replacement().pending_owner_rejection.as_ref().unwrap();
    assert_eq!(pending.decision(), &decision);
    assert_eq!(pending.completion(), state.final_review.as_ref().unwrap());
    assert_eq!(pending.publication_holds().len(), 1);
    assert_eq!(
        pending.publication_holds()[0].before(),
        &g.publication_states[0]
    );
    assert_eq!(
        pending.publication_holds()[0].held(),
        &p.publication_replacements()[0]
    );
    assert_eq!(before, (g, state));
}

#[test]
fn pending_rejection_blocks_old_acceptance_even_before_any_analysis() {
    let (g, state, decision, review, limits) = fixture(0);
    let accepted = owner_acceptance(&state);
    let own = BlockingReason::OwnerRejectionPending {
        owner_event_id: id(40),
    };
    assert_eq!(
        plan_root_transition(
            &state,
            row(6),
            scope(),
            None,
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&accepted)
        ),
        Err(WorkError::Blocked(own.clone()))
    );
    assert!(
        evaluate_gates(
            &g.roots[0].blocking_reasons,
            &[GateOperation::Reason, GateOperation::Plan]
        )
        .unwrap()
        .is_clear()
    );
    for operation in [
        GateOperation::Effect,
        GateOperation::SubmitResult,
        GateOperation::AcceptCompletion,
    ] {
        assert!(
            !evaluate_gates(&g.roots[0].blocking_reasons, &[operation])
                .unwrap()
                .is_clear()
        );
    }
    let mut inconsistent = state.clone();
    inconsistent.root.blocking_reasons.remove(&own);
    assert_eq!(
        plan(&g, &inconsistent, &decision, &review, &limits),
        Err(WorkError::OwnerRejectionMismatch)
    );
    assert_eq!(
        plan_root_transition(
            &inconsistent,
            row(6),
            scope(),
            None,
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&accepted)
        ),
        Err(WorkError::OwnerRejectionMismatch)
    );
    inconsistent = state.clone();
    inconsistent.pending_owner_rejection = None;
    assert_eq!(
        reject(&g, &inconsistent, &decision, &limits),
        Err(WorkError::InvalidTransition)
    );
    assert_eq!(
        plan(&g, &inconsistent, &decision, &review, &limits),
        Err(WorkError::OwnerRejectionMismatch)
    );
    assert_eq!(
        plan_root_transition(
            &inconsistent,
            row(6),
            scope(),
            None,
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&accepted)
        ),
        Err(WorkError::Blocked(own))
    );
    inconsistent = state;
    inconsistent.root.stage = RootStage::OwnerAccepted;
    assert!(
        plan_root_transition(
            &inconsistent,
            row(6),
            scope(),
            None,
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&accepted)
        )
        .is_err()
    );
}

#[test]
fn pending_rejection_cannot_be_replaced_and_unreceived_decision_cannot_resolve() {
    let (g, state, decision, _, limits) = fixture(0);
    for event_id in [id(40), id(99)] {
        let mut d = decision.clone();
        d.owner_event_id = event_id;
        assert_eq!(
            reject(&g, &state, &d, &limits),
            Err(WorkError::InvalidTransition)
        );
    }
    let (g, state, decision, review, limits) = unrejected_fixture(0);
    assert_eq!(
        plan(&g, &state, &decision, &review, &limits),
        Err(WorkError::OwnerRejectionMismatch)
    );
}

#[test]
fn initial_rejection_requires_exact_candidate_scope_root_and_versions() {
    let (g, state, decision, _, limits) = unrejected_fixture(1);
    for n in 0..3 {
        let mut d = decision.clone();
        match n {
            0 => d.candidate_id = id(99),
            1 => d.scope.objective_revision = rev(2),
            _ => d.final_verification_id = id(99),
        }
        assert_eq!(
            reject(&g, &state, &d, &limits),
            Err(WorkError::OwnerRejectionMismatch)
        );
    }
    let mut old = state.clone();
    old.root.row_version = row(4);
    assert_eq!(
        reject(&g, &old, &decision, &limits),
        Err(WorkError::ResultBasisMismatch)
    );
    let graph = validate_work_graph(&g, &limits).unwrap();
    assert_eq!(
        plan_owner_rejection(&state, row(4), scope(), &graph, &decision),
        Err(WorkError::RowVersionConflict)
    );
    old = state;
    old.root.stage = RootStage::OwnerAccepted;
    assert_eq!(
        reject(&g, &old, &decision, &limits),
        Err(WorkError::InvalidTransition)
    );
}

#[test]
fn initial_rejection_does_not_require_fresh_completion_evidence() {
    let (mut g, state, decision, _, limits) = unrejected_fixture(1);
    g.objectives[0].obligations[0].row_version = row(9);
    g.objectives[0].row_version = row(9);
    let p = reject(&g, &state, &decision, &limits).unwrap();
    assert_eq!(p.expected_objective_row(), row(9));
    assert_eq!(
        p.publication_replacements()[0].eligibility,
        PublicationEligibility::Unresolved
    );
}

#[test]
fn initial_rejection_overflow_never_leaves_partial_changes() {
    for which in 0..2 {
        let (mut g, mut state, decision, _, limits) = unrejected_fixture(2);
        if which == 0 {
            g.publication_states[1].row_version = row(u64::MAX);
        } else {
            state.root.row_version = row(u64::MAX);
            g.roots[0] = state.root.clone();
        }
        let before = (g.clone(), state.clone());
        assert_eq!(
            reject(&g, &state, &decision, &limits),
            Err(WorkError::VersionOverflow)
        );
        assert_eq!(before, (g, state));
    }
}

#[test]
fn initial_rejection_preserves_every_combination_of_preexisting_gates() {
    let (mut g, mut state, decision, _, limits) = unrejected_fixture(0);
    let gates: Vec<_> = gate_contract_rows()
        .into_iter()
        .map(|(gate, _)| gate)
        .filter(|gate| !matches!(gate, BlockingReason::OwnerRejectionPending { .. }))
        .collect();
    assert_eq!(gates.len(), 13);
    let own = BlockingReason::OwnerRejectionPending {
        owner_event_id: id(40),
    };
    for mask in 0..(1usize << gates.len()) {
        state.root.blocking_reasons = gates
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, gate)| gate.clone())
            .collect();
        g.roots[0] = state.root.clone();
        let p = reject(&g, &state, &decision, &limits).unwrap();
        let mut expected = state.root.blocking_reasons.clone();
        expected.insert(own.clone());
        assert_eq!(p.replacement().root.blocking_reasons, expected);
        assert_eq!(
            p.replacement().root.stage,
            RootStage::AwaitingOwnerAcceptance
        );
    }
}

#[test]
fn nonimpact_proof_cannot_remove_a_later_version_another_reason_or_withdrawal() {
    for which in 0..3 {
        let (mut g, state, decision, mut review, limits) = fixture(1);
        match which {
            0 => g.publication_states[0].row_version = row(3),
            1 => g.publication_states[0].reason_evidence_id = Some(id(99)),
            _ => g.publication_states[0].eligibility = PublicationEligibility::Withdrawn,
        }
        review.publication_assessments = vec![PublicationImpactAssessment {
            expected_row_version: g.publication_states[0].row_version,
            ..impact(
                100,
                PublicationImpact::ProvenUnaffected {
                    evidence_id: id(44),
                },
            )
        }];
        let p = plan(&g, &state, &decision, &review, &limits).unwrap();
        assert!(p.publication_replacements().is_empty());
        assert_eq!(p.publication_reads(), &g.publication_states);
    }
}

#[test]
fn a_matching_reason_label_is_not_proof_that_this_rejection_owns_the_hold() {
    let (mut g, mut state, decision, mut review, limits) = unrejected_fixture(1);
    g.publication_states[0] = PublicationState {
        publication_id: id(100),
        eligibility: PublicationEligibility::Unresolved,
        row_version: row(2),
        reason_evidence_id: Some(decision.reason_evidence_id),
    };
    receive(&mut g, &mut state, &decision, &mut review, &limits);
    assert!(
        state
            .pending_owner_rejection
            .as_ref()
            .unwrap()
            .publication_holds()
            .is_empty()
    );
    review.publication_assessments = vec![impact(
        100,
        PublicationImpact::ProvenUnaffected {
            evidence_id: id(44),
        },
    )];
    assert!(
        plan(&g, &state, &decision, &review, &limits)
            .unwrap()
            .publication_replacements()
            .is_empty()
    );
}

#[test]
fn restoring_owned_hold_increments_current_version_and_overflow_is_closed() {
    let (mut g, mut state, decision, mut review, limits) = unrejected_fixture(1);
    g.publication_states[0].row_version = row(u64::MAX - 1);
    receive(&mut g, &mut state, &decision, &mut review, &limits);
    review.publication_assessments = vec![PublicationImpactAssessment {
        expected_row_version: row(u64::MAX),
        ..impact(
            100,
            PublicationImpact::ProvenUnaffected {
                evidence_id: id(44),
            },
        )
    }];
    let before = (g.clone(), state.clone());
    assert_eq!(
        plan(&g, &state, &decision, &review, &limits),
        Err(WorkError::VersionOverflow)
    );
    assert_eq!(before, (g, state));
}

#[test]
fn resolving_rejection_never_clears_another_owner_event_or_general_wait() {
    let (mut g, mut state, decision, review, limits) = fixture(0);
    let other = BlockingReason::OwnerRejectionPending {
        owner_event_id: id(99),
    };
    state
        .root
        .blocking_reasons
        .extend([other.clone(), BlockingReason::AwaitingOwner {}]);
    g.roots[0] = state.root.clone();
    let p = plan(&g, &state, &decision, &review, &limits).unwrap();
    assert_eq!(
        p.replacement().root.blocking_reasons,
        BTreeSet::from([other, BlockingReason::AwaitingOwner {}])
    );
}

#[test]
fn pending_gate_has_strict_wire_shape_not_a_free_form_instruction() {
    let gate = BlockingReason::OwnerRejectionPending {
        owner_event_id: id(40),
    };
    let value = serde_json::to_value(&gate).unwrap();
    assert_eq!(
        serde_json::from_value::<BlockingReason>(value.clone()).unwrap(),
        gate
    );
    let mut extra = value.clone();
    extra["allow_effect"] = true.into();
    assert!(serde_json::from_value::<BlockingReason>(extra).is_err());
    let mut missing = value;
    missing.as_object_mut().unwrap().remove("owner_event_id");
    assert!(serde_json::from_value::<BlockingReason>(missing).is_err());
}

#[test]
fn newly_added_publications_are_not_silently_omitted_from_return_impact() {
    let (mut g, state, decision, mut review, limits) = fixture(1);
    let mut later = g.publications[0].clone();
    later.publication_id = id(101);
    later.export_revision = rev(2);
    later.created_event_id = id(301);
    g.publications.push(later);
    g.publication_states.push(PublicationState {
        publication_id: id(101),
        eligibility: PublicationEligibility::Valid,
        reason_evidence_id: None,
        row_version: row(1),
    });
    g.current_graph_version = row(3);
    g.expected_graph_version = row(3);
    review.expected_graph_version = row(3);
    let p = plan(&g, &state, &decision, &review, &limits).unwrap();
    assert_eq!(p.publication_reads().len(), 2);
    assert_eq!(
        p.publication_replacements(),
        &[PublicationState {
            publication_id: id(101),
            eligibility: PublicationEligibility::Unresolved,
            reason_evidence_id: Some(id(41)),
            row_version: row(2),
        }]
    );
}
