//! Synthetic C-10 recovery-evidence tests only. No `authority.anchor` file, no
//! broker `guard.db`, no live reconciliation handler and no OS dispatch is
//! exercised. These deterministic checks prove the typed classify and
//! reconcile-or-refuse logic, not OS authority or durable recovery.

use dc_domain::ids::{EntityId, RowVersion, RunGeneration};
use dc_domain::permission::{ActionKind, AuthoritySnapshot, CurrentFence, FenceScope};
use dc_watchdog::recovery::{
    ActionObservation, DispatchReceipt, ExternalActionStage, ExternalOutcome, OutcomeError,
    ReconciliationError, ReconciliationGrant, begin_reconciliation, classify_recovery_outcomes,
    observe_outcome, record_dispatch,
};

fn id(value: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | value)
        .expect("synthetic UUIDv7 fixture")
}

fn gv(value: u64) -> RunGeneration {
    RunGeneration::new(value).expect("nonzero fixture")
}

fn ver(value: u64) -> RowVersion {
    RowVersion::new(value).expect("nonzero fixture")
}

fn lease_scope() -> FenceScope {
    FenceScope::Lease(id(0xaa))
}

fn unit_scope() -> FenceScope {
    FenceScope::Unit(id(0xbb))
}

fn work_scope() -> FenceScope {
    FenceScope::RootWork(id(0xcc))
}

/// The complete support set of the admitting lease: root work, unit, lease.
/// Versions are the fence versions captured at dispatch.
fn support_set() -> [(FenceScope, u64); 3] {
    [(work_scope(), 2), (unit_scope(), 3), (lease_scope(), 4)]
}

/// A dispatch receipt with the common fixture identity: installation
/// `0x5ea1` (distinct from every intent id — P2-OUTCOME-ID-ALIAS requires
/// intent != installation), generation 3, epoch 7, and the full support-fence
/// set at its dispatch-time versions.
fn receipt(intent: u128) -> DispatchReceipt {
    DispatchReceipt {
        intent_id: id(intent),
        installation_id: id(0x5ea1),
        run_generation: gv(3),
        authority_epoch: gv(7),
        support_fences: support_set()
            .iter()
            .map(|(scope, version)| (*scope, ver(*version)))
            .collect(),
    }
}

fn authority(
    install: u128,
    epoch: u64,
    generation: u64,
    fences: &[(FenceScope, u64, bool)],
) -> AuthoritySnapshot {
    AuthoritySnapshot {
        installation_id: id(install),
        authority_epoch: gv(epoch),
        run_generation: gv(generation),
        fences: fences
            .iter()
            .map(|(scope, version, closed)| {
                (
                    *scope,
                    CurrentFence {
                        version: ver(*version),
                        closed: *closed,
                    },
                )
            })
            .collect(),
    }
}

/// An authority view that still knows every support fence at its captured
/// version, all open — the pre-revoke world the dispatch happened in.
fn open_authority() -> AuthoritySnapshot {
    let fences: Vec<(FenceScope, u64, bool)> = support_set()
        .iter()
        .map(|(scope, version)| (*scope, *version, false))
        .collect();
    authority(0x5ea1, 7, 3, &fences)
}

/// The same authority after the lease fence was closed and advanced by a
/// revoke: reconciliation must still be possible on it.
fn revoked_authority() -> AuthoritySnapshot {
    authority(
        0x5ea1,
        7,
        3,
        &[
            (work_scope(), 2, false),
            (unit_scope(), 3, false),
            (lease_scope(), 5, true),
        ],
    )
}

/// A reconciliation grant issued under the given generation/epoch. The grant
/// must match the *current* authority, not the historical dispatch.
fn grant_at(intent: u128, generation: u64, epoch: u64) -> ReconciliationGrant {
    ReconciliationGrant {
        intent_id: id(intent),
        action_kind: ActionKind::ReconcileExternalIntent,
        run_generation: gv(generation),
        authority_epoch: gv(epoch),
    }
}

fn grant(intent: u128) -> ReconciliationGrant {
    grant_at(intent, 3, 7)
}

#[test]
fn dispatch_receipt_with_aliased_intent_and_installation_is_rejected() {
    // P2-OUTCOME-ID-ALIAS: intent_id and installation_id are different identity
    // classes; a receipt that aliases them must be refused at the watchdog
    // boundary, fail-closed, before any outcome evidence can attach to it.
    let mut aliased = receipt(0x5ea1);
    aliased.installation_id = aliased.intent_id;
    let result = record_dispatch(aliased);
    assert!(matches!(result, Err(OutcomeError::IdentityAlias)));

    // And the distinct case still admits the dispatch.
    assert!(record_dispatch(receipt(1)).is_ok());
}

#[test]
fn unconfirmed_dispatch_observation_becomes_unknown() {
    let dispatched = record_dispatch(receipt(1)).expect("dispatch");
    assert_eq!(dispatched.stage, ExternalActionStage::DispatchStarted);
    // `dispatch_started` without a proven outcome is already unknown (§5.3).
    assert_eq!(dispatched.stage.outcome(), ExternalOutcome::OutcomeUnknown);

    let unknown = observe_outcome(&dispatched, ActionObservation::Unconfirmed).expect("classify");
    assert_eq!(unknown.stage, ExternalActionStage::OutcomeUnknown);
    assert_eq!(unknown.stage.outcome(), ExternalOutcome::OutcomeUnknown);
}

#[test]
fn confirmed_observation_settles_success_and_failure() {
    let ok = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Succeeded { evidence_id: id(9) },
    )
    .expect("settle");
    assert_eq!(ok.stage, ExternalActionStage::Succeeded);
    assert_eq!(ok.stage.outcome(), ExternalOutcome::Succeeded);

    let failed = observe_outcome(
        &record_dispatch(receipt(2)).expect("dispatch"),
        ActionObservation::Failed { evidence_id: id(9) },
    )
    .expect("settle");
    assert_eq!(failed.stage, ExternalActionStage::FailedConfirmed);
    assert_eq!(failed.stage.outcome(), ExternalOutcome::FailedConfirmed);
}

#[test]
fn late_result_is_rejected_for_settled_outcomes() {
    let settled = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Succeeded { evidence_id: id(9) },
    )
    .expect("settle");
    assert_eq!(
        observe_outcome(&settled, ActionObservation::Failed { evidence_id: id(9) }),
        Err(OutcomeError::LateResult)
    );
    assert_eq!(
        observe_outcome(&settled, ActionObservation::Unconfirmed),
        Err(OutcomeError::LateResult)
    );

    let failed = observe_outcome(
        &record_dispatch(receipt(2)).expect("dispatch"),
        ActionObservation::Failed { evidence_id: id(9) },
    )
    .expect("settle");
    assert_eq!(
        observe_outcome(&failed, ActionObservation::Succeeded { evidence_id: id(9) }),
        Err(OutcomeError::LateResult)
    );

    let unresolved = observe_outcome(
        &begin_reconciliation(
            &observe_outcome(
                &record_dispatch(receipt(3)).expect("dispatch"),
                ActionObservation::Unconfirmed,
            )
            .expect("unknown"),
            &grant(3),
            &open_authority(),
        )
        .expect("reconcile"),
        ActionObservation::Unconfirmed,
    )
    .expect("unresolved");
    assert_eq!(unresolved.stage, ExternalActionStage::Unresolved);
    assert_eq!(
        observe_outcome(
            &unresolved,
            ActionObservation::Succeeded { evidence_id: id(9) }
        ),
        Err(OutcomeError::LateResult)
    );
}

#[test]
fn blind_reobservation_of_an_unknown_outcome_is_refused() {
    let unknown = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Unconfirmed,
    )
    .expect("unknown");
    assert_eq!(
        observe_outcome(
            &unknown,
            ActionObservation::Succeeded { evidence_id: id(9) }
        ),
        Err(OutcomeError::RequiresReconciliation)
    );
    assert_eq!(
        observe_outcome(&unknown, ActionObservation::Failed { evidence_id: id(9) }),
        Err(OutcomeError::RequiresReconciliation)
    );
}

#[test]
fn reconciliation_refuses_without_a_permitted_matching_grant() {
    let unknown = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Unconfirmed,
    )
    .expect("unknown");

    assert_eq!(
        begin_reconciliation(&unknown, &grant(2), &open_authority()),
        Err(ReconciliationError::IntentMismatch)
    );
    let wrong_kind = ReconciliationGrant {
        action_kind: ActionKind::CallModelConnection,
        ..grant(1)
    };
    assert_eq!(
        begin_reconciliation(&unknown, &wrong_kind, &open_authority()),
        Err(ReconciliationError::WrongActionKind)
    );
    assert_eq!(
        begin_reconciliation(&unknown, &grant(1), &authority(2, 7, 3, &[])),
        Err(ReconciliationError::WrongInstallation)
    );
    // A grant issued under an older generation or epoch is stale against the
    // current authority; a grant "from the future" is equally refused.
    assert_eq!(
        begin_reconciliation(&unknown, &grant_at(1, 2, 7), &open_authority()),
        Err(ReconciliationError::StaleGrant)
    );
    assert_eq!(
        begin_reconciliation(&unknown, &grant_at(1, 3, 6), &open_authority()),
        Err(ReconciliationError::StaleGrant)
    );
    assert_eq!(
        begin_reconciliation(&unknown, &grant_at(1, 4, 7), &open_authority()),
        Err(ReconciliationError::StaleGrant)
    );
    // The authority view must still *know* every dispatch-time support fence;
    // a forgotten fence means the evidence history is incomplete.
    assert_eq!(
        begin_reconciliation(
            &unknown,
            &grant(1),
            &authority(
                0x5ea1,
                7,
                3,
                &[(work_scope(), 2, false), (unit_scope(), 3, false)]
            )
        ),
        Err(ReconciliationError::FenceHistoryLost)
    );
}

#[test]
fn reconciliation_survives_revoke_and_fence_closure_of_the_dispatch_lease() {
    // Audit regression: an external action was dispatched, its outcome is
    // unknown, the owner stops/revokes the old effect-authority (fence closes
    // and advances). Finding out whether the effect happened must still be
    // possible; the revoke must not fabricate a failure nor block the query.
    let unknown = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Unconfirmed,
    )
    .expect("unknown");

    let reconciling = begin_reconciliation(&unknown, &grant(1), &revoked_authority())
        .expect("reconcile on a revoked fence");
    assert_eq!(reconciling.stage, ExternalActionStage::Reconciling);

    // A confirmed reconciliation observation settles the outcome afterwards.
    let settled = observe_outcome(
        &reconciling,
        ActionObservation::Failed { evidence_id: id(9) },
    )
    .expect("settle after revoke");
    assert_eq!(settled.stage, ExternalActionStage::FailedConfirmed);
}

#[test]
fn reconciliation_survives_a_crash_and_a_new_run_generation() {
    // Audit regression: dispatch in generation 3, crash, recovery starts
    // generation 4 (contract 18 §3.5). The reconciliation grant is issued
    // under the new generation; the historical receipt is evidence, not a
    // freshness requirement.
    let unknown = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Unconfirmed,
    )
    .expect("unknown");

    let post_crash = authority(
        0x5ea1,
        7,
        4,
        &[
            (work_scope(), 2, false),
            (unit_scope(), 3, false),
            (lease_scope(), 5, true),
        ],
    );
    // A grant replayed from the dead generation is refused.
    assert_eq!(
        begin_reconciliation(&unknown, &grant_at(1, 3, 7), &post_crash),
        Err(ReconciliationError::StaleGrant)
    );
    // A grant issued by the current generation reconciles the old unknown.
    let reconciling =
        begin_reconciliation(&unknown, &grant_at(1, 4, 7), &post_crash).expect("new-generation");
    let settled = observe_outcome(
        &reconciling,
        ActionObservation::Succeeded { evidence_id: id(9) },
    )
    .expect("settle after crash");
    assert_eq!(settled.stage, ExternalActionStage::Succeeded);
}

#[test]
fn reconciliation_is_refused_outside_the_unknown_stage() {
    let dispatched = record_dispatch(receipt(1)).expect("dispatch");
    assert_eq!(
        begin_reconciliation(&dispatched, &grant(1), &open_authority()),
        Err(ReconciliationError::NotUnknown)
    );

    let settled = observe_outcome(
        &record_dispatch(receipt(2)).expect("dispatch"),
        ActionObservation::Succeeded { evidence_id: id(9) },
    )
    .expect("settle");
    assert_eq!(
        begin_reconciliation(&settled, &grant(2), &open_authority()),
        Err(ReconciliationError::NotUnknown)
    );
}

#[test]
fn unconfirmed_reconciliation_stays_unresolved_not_a_fabricated_failure() {
    let unknown = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Unconfirmed,
    )
    .expect("unknown");
    let reconciling =
        begin_reconciliation(&unknown, &grant(1), &open_authority()).expect("reconcile");
    assert_eq!(reconciling.stage, ExternalActionStage::Reconciling);

    let unresolved =
        observe_outcome(&reconciling, ActionObservation::Unconfirmed).expect("resolve");
    assert_eq!(unresolved.stage, ExternalActionStage::Unresolved);
    assert_eq!(unresolved.stage.outcome(), ExternalOutcome::OutcomeUnknown);
}

#[test]
fn confirmed_reconciliation_settles_the_outcome() {
    let unknown = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Unconfirmed,
    )
    .expect("unknown");
    let reconciling =
        begin_reconciliation(&unknown, &grant(1), &open_authority()).expect("reconcile");

    let ok = observe_outcome(
        &reconciling,
        ActionObservation::Succeeded { evidence_id: id(9) },
    )
    .expect("settle");
    assert_eq!(ok.stage, ExternalActionStage::Succeeded);

    let failed = observe_outcome(
        &begin_reconciliation(
            &observe_outcome(
                &record_dispatch(receipt(2)).expect("dispatch"),
                ActionObservation::Unconfirmed,
            )
            .expect("unknown"),
            &grant(2),
            &open_authority(),
        )
        .expect("reconcile"),
        ActionObservation::Failed { evidence_id: id(9) },
    )
    .expect("settle");
    assert_eq!(failed.stage, ExternalActionStage::FailedConfirmed);
}

#[test]
fn recovery_partition_buckets_and_blocks_resume_on_unknown() {
    let succeeded = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Succeeded { evidence_id: id(9) },
    )
    .expect("settle");
    let failed = observe_outcome(
        &record_dispatch(receipt(2)).expect("dispatch"),
        ActionObservation::Failed { evidence_id: id(9) },
    )
    .expect("settle");
    let unknown = observe_outcome(
        &record_dispatch(receipt(3)).expect("dispatch"),
        ActionObservation::Unconfirmed,
    )
    .expect("unknown");
    let unobserved = record_dispatch(receipt(4)).expect("dispatch");

    let partition = classify_recovery_outcomes([&succeeded, &failed, &unknown, &unobserved]);
    assert_eq!(partition.succeeded, vec![id(1)]);
    assert_eq!(partition.failed_confirmed, vec![id(2)]);
    assert_eq!(partition.unknown, vec![id(3), id(4)]);
    assert!(partition.has_unknown());

    let clean = classify_recovery_outcomes([&succeeded, &failed]);
    assert_eq!(clean.succeeded, vec![id(1)]);
    assert_eq!(clean.failed_confirmed, vec![id(2)]);
    assert!(clean.unknown.is_empty());
    assert!(!clean.has_unknown());
}

#[test]
fn unresolved_and_reconciling_count_as_unknown_in_recovery() {
    let unknown = observe_outcome(
        &record_dispatch(receipt(1)).expect("dispatch"),
        ActionObservation::Unconfirmed,
    )
    .expect("unknown");
    let reconciling =
        begin_reconciliation(&unknown, &grant(1), &open_authority()).expect("reconcile");
    let unresolved =
        observe_outcome(&reconciling, ActionObservation::Unconfirmed).expect("resolve");

    for record in [&unknown, &reconciling, &unresolved] {
        let partition = classify_recovery_outcomes([record]);
        assert_eq!(partition.unknown, vec![id(1)]);
        assert!(partition.succeeded.is_empty());
        assert!(partition.failed_confirmed.is_empty());
        assert!(partition.has_unknown());
    }
}
