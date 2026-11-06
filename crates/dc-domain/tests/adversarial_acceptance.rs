//! Adversarial tests for stale review, failed checks, negative authority cases
//! and exact-revision full acceptance. These tests verify that the C-01 domain
//! logic correctly rejects invalid transitions and enforces strict checking.

use std::num::{NonZeroU32, NonZeroU64};

use dc_domain::ids::{EntityId, RowVersion};
use dc_domain::work::*;

// ============================================================================
// Test fixture helpers (duplicated to keep this test self-contained)
// ============================================================================

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
    ResultPacket {
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
    }
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
    advance_with_expected(g, p, target, record, p.row_version)
}

/// Like `advance`, but lets the caller supply the stored result row version
/// independently of the packet's version. This mirrors the real C-03 flow
/// where the expected version comes from the canonical store, not from the
/// incoming packet — otherwise a stale packet can never conflict.
fn advance_with_expected(
    g: &GraphCandidate,
    p: &ResultPacket,
    target: ResultStage,
    record: Option<&VerificationRecord>,
    expected_result_row_version: RowVersion,
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
            expected_result_row_version,
            active_attempt_id: p.attempt_id,
        },
    )
}

// ============================================================================
// ADVERSARIAL TESTS: Stale Review Cases
// ============================================================================

mod stale_review {
    use super::*;

    #[test]
    fn rejects_stale_result_row_version_on_verifying_transition() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        // Stored row version has advanced beyond what the packet carries
        // (e.g. another writer committed an intermediate transition).
        let result = advance_with_expected(&g, &p, ResultStage::Verifying, None, row(2));
        assert_eq!(result.unwrap_err(), WorkError::RowVersionConflict);
    }

    #[test]
    fn rejects_stale_result_row_version_on_acceptance() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        // Move to verifying first with correct version
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        // Now try to accept while the stored row version has moved on
        let result = advance_with_expected(
            &g,
            &verifying,
            ResultStage::Accepted,
            Some(&proof),
            row(999),
        );
        assert_eq!(result.unwrap_err(), WorkError::RowVersionConflict);
    }

    #[test]
    fn rejects_stale_unit_row_version() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        // Try to accept with wrong expected unit row version
        let v2 = validate_work_graph(&g, &limits()).unwrap();
        let rows = rows(&v2);
        let result = transition_result(
            &verifying,
            ResultStage::Accepted,
            Some(&proof),
            &ResultCommitView {
                graph: &v2,
                current_rows: &rows,
                expected_unit_row_version: row(999), // Wrong version
                expected_result_row_version: verifying.row_version,
                active_attempt_id: verifying.attempt_id,
            },
        );
        assert_eq!(result.unwrap_err(), WorkError::RowVersionConflict);
    }

    #[test]
    fn rejects_stale_dependency_snapshot() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let mut p = packet(&v, 0);
        // Corrupt the snapshot to reference an old row version
        p.snapshot.read_set[0].expected_revision = row(999);
        let result = advance(&g, &p, ResultStage::Verifying, None);
        assert!(matches!(result.unwrap_err(), WorkError::StaleInput(_)));
    }

    #[test]
    fn rejects_stale_graph_rows() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        // Modify an obligation row that the packet's snapshot actually read:
        // the snapshot captured row(1), but the canonical row is now row(2).
        let mut g2 = g.clone();
        g2.objectives[0].obligations[0].row_version = row(2);
        // Keep the linked unit row untouched so the packet's UnitRef still
        // resolves, but advance its stage to Verifying to match the packet.
        g2.units[0].stage = UnitStage::Verifying;
        let v2 = validate_work_graph(&g2, &limits()).unwrap();
        let rows = rows(&v2);
        let result = transition_result(
            &verifying,
            ResultStage::Accepted,
            Some(&proof),
            &ResultCommitView {
                graph: &v2,
                current_rows: &rows,
                expected_unit_row_version: row(1),
                expected_result_row_version: verifying.row_version,
                active_attempt_id: verifying.attempt_id,
            },
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, WorkError::StaleInput(_)),
            "expected StaleInput, got {err:?}"
        );
    }

    #[test]
    fn stale_transition_requires_actual_staleness() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        // Try to mark as stale when nothing is actually stale
        let result = advance(&g, &p, ResultStage::Stale, None);
        assert_eq!(result.unwrap_err(), WorkError::InvalidTransition);
    }
}

// ============================================================================
// ADVERSARIAL TESTS: Failed Verification Checks
// ============================================================================

mod failed_checks {
    use super::*;

    #[test]
    fn rejects_acceptance_with_failed_verification() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.verdict = VerificationVerdict::Reject;
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_acceptance_with_gaps_verdict() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.verdict = VerificationVerdict::Gaps;
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_acceptance_with_invalid_verdict() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.verdict = VerificationVerdict::Invalid;
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_wrong_result_id() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.result_id = id(9999); // Wrong result ID
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_wrong_fingerprint() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.result_fingerprint = [255; 32]; // Wrong fingerprint
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_wrong_snapshot_id() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.snapshot_id = id(9999); // Wrong snapshot
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_insufficient_level() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.evidence.level = VerificationLevel::MachineOnly; // Too low for Research
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_unqualified_method() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        if let Some(ref mut q) = proof.evidence.qualification {
            q.status = QualificationStatus::Disqualified;
        }
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_stale_qualification() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        if let Some(ref mut q) = proof.evidence.qualification {
            q.status = QualificationStatus::Stale;
        }
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_violated_finding() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.findings[0].status = ObligationStatus::Violated;
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_waived_finding() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.findings[0].status = ObligationStatus::Waived;
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn rejects_verification_with_missing_finding_for_criterion() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.findings[0].obligation_id = id(9999); // Wrong obligation
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }
}

// ============================================================================
// ADVERSARIAL TESTS: Negative Authority Cases
// ============================================================================

mod negative_authority {
    use super::*;

    #[test]
    fn rejects_acceptance_with_blocked_root() {
        let mut g = graph();
        g.roots[0]
            .blocking_reasons
            .insert(BlockingReason::Recovery {});
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert!(matches!(result.unwrap_err(), WorkError::Blocked(_)));
    }

    #[test]
    fn rejects_acceptance_with_resource_unavailable() {
        let mut g = graph();
        g.roots[0]
            .blocking_reasons
            .insert(BlockingReason::ResourceUnavailable {});
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert!(matches!(result.unwrap_err(), WorkError::Blocked(_)));
    }

    #[test]
    fn rejects_acceptance_with_integrity_failure() {
        let mut g = graph();
        g.roots[0]
            .blocking_reasons
            .insert(BlockingReason::IntegrityFailure {});
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert!(matches!(result.unwrap_err(), WorkError::Blocked(_)));
    }

    #[test]
    fn rejects_acceptance_with_stop_requested() {
        let mut g = graph();
        g.roots[0]
            .blocking_reasons
            .insert(BlockingReason::StopRequested {});
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert!(matches!(result.unwrap_err(), WorkError::Blocked(_)));
    }

    #[test]
    fn rejects_acceptance_from_draft_root() {
        let mut g = graph();
        g.roots[0].stage = RootStage::Draft;
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::InvalidTransition);
    }

    #[test]
    fn rejects_acceptance_from_owner_accepted_root() {
        let mut g = graph();
        g.roots[0].stage = RootStage::OwnerAccepted;
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::InvalidTransition);
    }

    #[test]
    fn rejects_verifying_transition_with_empty_gates() {
        // This tests that empty operations fail closed
        let result = evaluate_gates(&[], &[]);
        assert!(matches!(result.unwrap_err(), WorkError::InvalidField(_)));
    }

    #[test]
    fn rejects_submission_with_awaiting_owner_gate() {
        // AwaitingOwner allows read/reason/plan but not result submission
        let gates = vec![BlockingReason::AwaitingOwner {}];
        let result = evaluate_gates(&gates, &[GateOperation::SubmitResult]);
        let assessment = result.unwrap();
        assert!(!assessment.is_clear());
    }

    #[test]
    fn allows_submission_with_no_gates() {
        let result = evaluate_gates(&[], &[GateOperation::SubmitResult]);
        let assessment = result.unwrap();
        assert!(assessment.is_clear());
    }
}

// ============================================================================
// ADVERSARIAL TESTS: Exact-Revision Full Acceptance
// ============================================================================

mod exact_revision_acceptance {
    use super::*;

    #[test]
    fn full_acceptance_succeeds_with_exact_revisions() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        // Full path: Submitted -> Verifying -> Accepted
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        assert_eq!(verifying.stage, ResultStage::Verifying);
        assert_eq!(verifying.row_version, row(2));

        let accepted = advance(&g, &verifying, ResultStage::Accepted, Some(&proof)).unwrap();
        assert_eq!(accepted.stage, ResultStage::Accepted);
        assert_eq!(accepted.row_version, row(3));
    }

    #[test]
    fn acceptance_requires_exact_attempt_id() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        // Try to accept with wrong attempt ID
        let v2 = validate_work_graph(&g, &limits()).unwrap();
        let rows = rows(&v2);
        let result = transition_result(
            &verifying,
            ResultStage::Accepted,
            Some(&proof),
            &ResultCommitView {
                graph: &v2,
                current_rows: &rows,
                expected_unit_row_version: row(1),
                expected_result_row_version: verifying.row_version,
                active_attempt_id: id(9999), // Wrong attempt
            },
        );
        assert_eq!(result.unwrap_err(), WorkError::ResultBasisMismatch);
    }

    #[test]
    fn acceptance_requires_matching_unit_revision() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let mut p = packet(&v, 0);
        // Corrupt the unit revision
        p.unit.revision = rev(999);
        let result = advance(&g, &p, ResultStage::Verifying, None);
        assert_eq!(result.unwrap_err(), WorkError::ResultBasisMismatch);
    }

    #[test]
    fn acceptance_requires_matching_scope() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let mut p = packet(&v, 0);
        // Corrupt the scope
        p.scope.work_id = id(9999);
        let result = advance(&g, &p, ResultStage::Verifying, None);
        assert_eq!(result.unwrap_err(), WorkError::ResultBasisMismatch);
    }

    #[test]
    fn acceptance_requires_all_obligations_satisfied() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        // Change finding to a different obligation not in acceptance criteria
        proof.findings[0].obligation_id = id(9999);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn acceptance_rejects_extra_violated_findings() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        // Add a violated finding for an unrelated obligation
        proof.findings.push(ObligationFinding {
            obligation_id: id(8888),
            status: ObligationStatus::Violated,
            evidence_id: id(8889),
        });
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn double_acceptance_is_rejected() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let proof = verification(&p, 0);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let accepted = advance(&g, &verifying, ResultStage::Accepted, Some(&proof)).unwrap();
        // Try to accept again
        let result = advance(&g, &accepted, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::InvalidTransition);
    }

    #[test]
    fn rejection_preserves_facts() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        // Rejection requires a non-Pass verdict (fail-closed at C-01).
        let mut proof = verification(&p, 0);
        proof.verdict = VerificationVerdict::Reject;
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let rejected = advance(&g, &verifying, ResultStage::Rejected, Some(&proof)).unwrap();
        assert_eq!(rejected.stage, ResultStage::Rejected);
        // Submitted(row 1) -> Verifying(row 2) -> Rejected(row 3).
        assert_eq!(rejected.row_version, row(3));
        // Original packet data preserved
        assert_eq!(rejected.result_id, p.result_id);
        assert_eq!(rejected.attempt_id, p.attempt_id);
    }
}

// ============================================================================
// ADVERSARIAL TESTS: Boundary and Edge Cases
// ============================================================================

mod boundary_cases {
    use super::*;

    #[test]
    fn row_version_overflow_fails_safely() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let mut p = packet(&v, 0);
        p.row_version = row(u64::MAX);
        // Stored version matches, so the conflict check passes and the
        // transition must fail on checked_add overflow instead of panicking.
        let result = advance_with_expected(&g, &p, ResultStage::Verifying, None, row(u64::MAX));
        assert_eq!(result.unwrap_err(), WorkError::VersionOverflow);
    }

    #[test]
    fn row_version_max_conflict_detected_before_overflow() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let mut p = packet(&v, 0);
        p.row_version = row(u64::MAX);
        // Stored version disagrees: conflict must win over overflow handling.
        let result = advance_with_expected(&g, &p, ResultStage::Verifying, None, row(1));
        assert_eq!(result.unwrap_err(), WorkError::RowVersionConflict);
    }

    #[test]
    fn verifies_empty_criteria_rejection() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.evidence.criterion_ids = vec![]; // Empty criteria
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn verifier_cannot_be_same_as_producer() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.verifier_attempt_id = p.attempt_id; // Same as producer
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }

    #[test]
    fn policy_mismatch_rejected() {
        let g = graph();
        let v = validate_work_graph(&g, &limits()).unwrap();
        let p = packet(&v, 0);
        let mut proof = verification(&p, 0);
        proof.verification_policy.key.entity_id = id(9999);
        let verifying = advance(&g, &p, ResultStage::Verifying, None).unwrap();
        let result = advance(&g, &verifying, ResultStage::Accepted, Some(&proof));
        assert_eq!(result.unwrap_err(), WorkError::VerificationMismatch);
    }
}
