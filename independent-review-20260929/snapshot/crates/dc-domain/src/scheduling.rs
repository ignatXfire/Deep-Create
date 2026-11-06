//! C-01 / contract 25: policy validation and ordinary root-service comparison.
//! C-05 authenticates policy changes; C-06 supplies already eligible candidates
//! and handles aging/protected claims BEFORE this comparison. No dispatch,
//! admission, ledger mutation, time reading, or complete fairness guarantee here.

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    num::{NonZeroU16, NonZeroU32, NonZeroU64},
};

use thiserror::Error;

use crate::{ids::EntityId, resource::upper_service_cost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkPriority {
    Foreground,
    DeadlineCritical,
    Normal,
    Background,
    Exploration,
}

/// Fixed fields deliberately make an omitted priority impossible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriorityWeights {
    pub foreground: NonZeroU16,
    pub deadline_critical: NonZeroU16,
    pub normal: NonZeroU16,
    pub background: NonZeroU16,
    pub exploration: NonZeroU16,
}

impl PriorityWeights {
    fn for_priority(self, priority: WorkPriority) -> NonZeroU16 {
        match priority {
            WorkPriority::Foreground => self.foreground,
            WorkPriority::DeadlineCritical => self.deadline_critical,
            WorkPriority::Normal => self.normal,
            WorkPriority::Background => self.background,
            WorkPriority::Exploration => self.exploration,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulingPolicy {
    pub policy_version: u32,
    pub effective_from_event: NonZeroU64,
    pub service_window_ms: NonZeroU64,
    pub root_aging_ms: NonZeroU64,
    pub unit_aging_ms: NonZeroU64,
    pub weights: PriorityWeights,
    pub attempt_wall_time_cap_ms: NonZeroU64,
    pub protected_claims_max: u8,
    pub claim_drain_margin_ms: NonZeroU64,
}

/// Trusted installation/profile constraints, not instructions from a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedulingConstraints {
    pub max_weight: NonZeroU16,
    pub max_attempt_wall_time_ms: NonZeroU64,
    pub max_parallel_slots: NonZeroU32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedSchedulingPolicy {
    policy: SchedulingPolicy,
    max_parallel_slots: NonZeroU32,
}

impl ValidatedSchedulingPolicy {
    pub fn policy(&self) -> &SchedulingPolicy {
        &self.policy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SchedulingError {
    #[error("unsupported or out-of-constraint scheduling policy")]
    InvalidPolicy,
    #[error("attempt exceeds the enforced profile bounds")]
    UnboundedAttempt,
    #[error("service arithmetic overflow")]
    ArithmeticOverflow,
}

pub fn validate_scheduling_policy(
    policy: &SchedulingPolicy,
    constraints: SchedulingConstraints,
) -> Result<ValidatedSchedulingPolicy, SchedulingError> {
    if policy.policy_version != 1 || policy.protected_claims_max != 1 {
        return Err(SchedulingError::InvalidPolicy);
    }
    for priority in [
        WorkPriority::Foreground,
        WorkPriority::DeadlineCritical,
        WorkPriority::Normal,
        WorkPriority::Background,
        WorkPriority::Exploration,
    ] {
        if policy.weights.for_priority(priority) > constraints.max_weight {
            return Err(SchedulingError::InvalidPolicy);
        }
    }
    if policy.attempt_wall_time_cap_ms > constraints.max_attempt_wall_time_ms {
        return Err(SchedulingError::UnboundedAttempt);
    }
    policy
        .attempt_wall_time_cap_ms
        .get()
        .checked_add(policy.claim_drain_margin_ms.get())
        .ok_or(SchedulingError::ArithmeticOverflow)?;
    upper_service_cost(
        u64::from(constraints.max_parallel_slots.get()),
        policy.attempt_wall_time_cap_ms.get(),
    )
    .map_err(|_| SchedulingError::ArithmeticOverflow)?;
    Ok(ValidatedSchedulingPolicy {
        policy: policy.clone(),
        max_parallel_slots: constraints.max_parallel_slots,
    })
}

/// C-06 derives completed service within the window and ALL unfinished service
/// from distinct ledger identities. These aggregates cannot prove that lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootServiceView {
    pub root_work_id: EntityId,
    pub waiting_turn_seq: NonZeroU64,
    pub owner_priority: WorkPriority,
    pub completed_in_window_ms: u64,
    pub unfinished_conservative_ms: u64,
    pub candidate_parallel_slots: NonZeroU32,
    pub candidate_wall_time_ms: NonZeroU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonReason {
    NormalizedService,
    WaitingTurn,
    StableIdentity,
    Equal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootServiceComparison {
    pub ordering: Ordering,
    pub reason: ComparisonReason,
    pub left_numerator_ms: u64,
    pub right_numerator_ms: u64,
    pub left_weight: NonZeroU16,
    pub right_weight: NonZeroU16,
}

fn numerator(
    view: RootServiceView,
    policy: &ValidatedSchedulingPolicy,
) -> Result<u64, SchedulingError> {
    if view.candidate_parallel_slots > policy.max_parallel_slots
        || view.candidate_wall_time_ms > policy.policy.attempt_wall_time_cap_ms
    {
        return Err(SchedulingError::UnboundedAttempt);
    }
    let upper = upper_service_cost(
        u64::from(view.candidate_parallel_slots.get()),
        view.candidate_wall_time_ms.get(),
    )
    .map_err(|_| SchedulingError::ArithmeticOverflow)?;
    view.completed_in_window_ms
        .checked_add(view.unfinished_conservative_ms)
        .and_then(|amount| amount.checked_add(upper))
        .ok_or(SchedulingError::ArithmeticOverflow)
}

/// Ordinary comparison only. A protected or aged eligible ticket takes precedence
/// in C-06 and must not be sorted behind foreground using this function.
pub fn compare_root_service(
    left: RootServiceView,
    right: RootServiceView,
    policy: &ValidatedSchedulingPolicy,
) -> Result<RootServiceComparison, SchedulingError> {
    let left_numerator_ms = numerator(left, policy)?;
    let right_numerator_ms = numerator(right, policy)?;
    let left_weight = policy.policy.weights.for_priority(left.owner_priority);
    let right_weight = policy.policy.weights.for_priority(right.owner_priority);
    let left_cross = u128::from(left_numerator_ms)
        .checked_mul(u128::from(right_weight.get()))
        .ok_or(SchedulingError::ArithmeticOverflow)?;
    let right_cross = u128::from(right_numerator_ms)
        .checked_mul(u128::from(left_weight.get()))
        .ok_or(SchedulingError::ArithmeticOverflow)?;
    let (ordering, reason) = if left_cross != right_cross {
        (
            left_cross.cmp(&right_cross),
            ComparisonReason::NormalizedService,
        )
    } else if left.waiting_turn_seq != right.waiting_turn_seq {
        (
            left.waiting_turn_seq.cmp(&right.waiting_turn_seq),
            ComparisonReason::WaitingTurn,
        )
    } else if left.root_work_id != right.root_work_id {
        (
            left.root_work_id.cmp(&right.root_work_id),
            ComparisonReason::StableIdentity,
        )
    } else {
        (Ordering::Equal, ComparisonReason::Equal)
    };
    Ok(RootServiceComparison {
        ordering,
        reason,
        left_numerator_ms,
        right_numerator_ms,
        left_weight,
        right_weight,
    })
}

// ---------------------------------------------------------------------------
// C-06 internal functions (document 25 §4.3–4.5, §9.1)
// ---------------------------------------------------------------------------

/// Closed skip-reason codes from document 25 §9.2 relevant to scheduling.
/// Each code is a typed, closed refusal; no free-form text is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkipReason {
    /// No route satisfies the work's quality/data constraints.
    NoAdmissibleConnection,
    /// A route exists but its connection is currently unavailable.
    ConnectionUnavailable,
    /// The route's quality floor is not met.
    QualityFloorUnmet,
    /// Verifier independence cannot be established.
    InsufficientIndependence,
    /// Disclosure of required evidence is denied.
    DisclosureDenied,
    /// A resource limit has been reached.
    LimitReached,
    /// The upper bound of a resource is unknown.
    UnknownUpperBound,
    /// Required capacity is currently occupied.
    CapacityBusy,
    /// A blocking process has not confirmed release of capacity.
    CapacityReleaseUnconfirmed,
    /// Commit queue backpressure prevents admission.
    CommitBackpressure,
    /// Claim inputs changed while the claim was pending.
    ClaimInputChanged,
    /// A required dependency is not yet confirmed.
    DependencyNotConfirmed,
    /// A dependency changed while the candidate was pending.
    DependencyChanged,
    /// The admission fence changed.
    FenceChanged,
    /// The candidate's generation is stale.
    StaleGeneration,
    /// Arithmetic overflow in checked calculations.
    ArithmeticOverflow,
}

/// A candidate for root-turn selection, supplied by C-06 from trusted state.
/// `aging_wait_started_ms` is when the candidate became eligible according to
/// trusted observation; `None` means not yet eligible for aging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootTurnCandidate {
    pub view: RootServiceView,
    /// Trusted observation of when this candidate became eligible for aging.
    pub aging_wait_started_ms: Option<u64>,
    /// Whether an active protected capacity claim exists for this root.
    pub active_protected_claim: bool,
    /// If present, the candidate is currently blocked by this reason.
    pub skip_reason: Option<SkipReason>,
}

/// Why a particular candidate was selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionReason {
    /// An active protected capacity claim takes precedence over all ordinary
    /// ordering (document 25 §4.4).
    ProtectedClaim,
    /// The candidate is the eldest among those past `root_aging_ms`.
    AgingEldest,
    /// Ordinary service comparison (document 25 §4.3).
    OrdinaryService,
}

/// The result of `choose_fair_root_turn`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulingDecision {
    /// A candidate was selected.
    Selected {
        /// Index into the input slice.
        candidate_index: usize,
        reason: SelectionReason,
        /// The comparison evidence when `reason == OrdinaryService`.
        comparison: Option<RootServiceComparison>,
    },
    /// No candidate is eligible; all have closed skip reasons.
    NoEligibleCandidates {
        /// One skip reason per input candidate, in input order.
        reasons: Vec<SkipReason>,
    },
}

/// Choose the next root turn from eligible candidates.
///
/// Pure C-01 arithmetic: no I/O, no clock reading, no reservation. C-06
/// supplies the candidates, trusted `now_ms`, and availability observations.
///
/// Algorithm (document 25 §4.3–4.4):
/// 1. Filter out candidates with `skip_reason`.
/// 2. If any candidate has `active_protected_claim`, select it (short-circuit).
/// 3. Filter to aging-eligible (`now - aging_wait_started >= root_aging_ms`).
/// 4. If any aging-eligible, select the one with smallest `waiting_turn_seq`.
/// 5. Otherwise, select by `compare_root_service` (smallest normalized service,
///    then `waiting_turn_seq`, then `root_work_id`).
pub fn choose_fair_root_turn(
    candidates: &[RootTurnCandidate],
    policy: &ValidatedSchedulingPolicy,
    now_ms: u64,
) -> SchedulingDecision {
    let mut reasons = Vec::with_capacity(candidates.len());
    let mut eligible: Vec<(usize, &RootTurnCandidate)> = Vec::new();
    for (i, c) in candidates.iter().enumerate() {
        if let Some(reason) = c.skip_reason {
            reasons.push(reason);
        } else {
            eligible.push((i, c));
        }
    }
    if eligible.is_empty() {
        return SchedulingDecision::NoEligibleCandidates { reasons };
    }

    // Protected claim short-circuit: it wins over ordinary ordering.
    for &(i, c) in &eligible {
        if c.active_protected_claim {
            return SchedulingDecision::Selected {
                candidate_index: i,
                reason: SelectionReason::ProtectedClaim,
                comparison: None,
            };
        }
    }

    // Aging eligibility: waiting_turn_seq age >= root_aging_ms.
    let aging_threshold = policy.policy.root_aging_ms.get();
    let mut aged: Vec<(usize, &RootTurnCandidate)> = Vec::new();
    for &(i, c) in &eligible {
        if let Some(started) = c.aging_wait_started_ms {
            if now_ms.saturating_sub(started) >= aging_threshold {
                aged.push((i, c));
            }
        }
    }
    if !aged.is_empty() {
        // Eldest waiting_turn_seq wins; tiebreak by root_work_id for stability.
        let mut best = aged[0];
        for &(i, c) in &aged[1..] {
            if c.view.waiting_turn_seq < best.1.view.waiting_turn_seq
                || (c.view.waiting_turn_seq == best.1.view.waiting_turn_seq
                    && c.view.root_work_id < best.1.view.root_work_id)
            {
                best = (i, c);
            }
        }
        return SchedulingDecision::Selected {
            candidate_index: best.0,
            reason: SelectionReason::AgingEldest,
            comparison: None,
        };
    }

    // Ordinary service comparison: smallest normalized service wins.
    let mut best = eligible[0];
    let mut best_cmp: Option<RootServiceComparison> = None;
    for &(i, c) in &eligible[1..] {
        match compare_root_service(c.view, best.1.view, policy) {
            Ok(cmp) => {
                if cmp.ordering == Ordering::Less {
                    best = (i, c);
                    best_cmp = Some(cmp);
                } else if best_cmp.is_none() {
                    best_cmp = Some(cmp);
                }
            }
            Err(_) => {
                // Overflow or unbounded attempt: skip this candidate.
                // The caller should have filtered these out, but we handle it
                // defensively by treating the candidate as ineligible.
                continue;
            }
        }
    }
    SchedulingDecision::Selected {
        candidate_index: best.0,
        reason: SelectionReason::OrdinaryService,
        comparison: best_cmp,
    }
}

// ---------------------------------------------------------------------------
// Protected capacity claim planning (document 25 §4.4)
// ---------------------------------------------------------------------------

/// A required slot in a claim's vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequiredSlot {
    pub limit_id: crate::ids::EntityId,
    pub window_id: crate::ids::EntityId,
    pub quantity: u32,
}

/// Proposal for a protected capacity claim. This is NOT a reservation; it is
/// a plan that C-05/C-03 may accept atomically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedCapacityClaimProposal {
    pub claim_id: crate::ids::EntityId,
    pub root_work_id: crate::ids::EntityId,
    pub unit_id: crate::ids::EntityId,
    pub root_turn_seq: NonZeroU64,
    pub connection_id: crate::ids::EntityId,
    pub connection_revision: u64,
    pub request_hash: u64,
    pub required_slot_vector: Vec<RequiredSlot>,
    pub reservation_id: crate::ids::EntityId,
    pub dependency_snapshot_id: crate::ids::EntityId,
    pub created_observation_ms: u64,
    pub drain_deadline_ms: u64,
    pub state: ClaimState,
    pub withdraw_reason: Option<ClaimWithdrawReason>,
    pub row_version: crate::ids::RowVersion,
}

/// Claim lifecycle states from document 25 §4.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimState {
    Draining,
    DispatchPending,
    Consumed,
    Withdrawn,
}

/// Closed reasons for withdrawing a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimWithdrawReason {
    /// Capacity release was not confirmed by the deadline.
    CapacityReleaseUnconfirmed,
    /// Claim inputs (tariff, inputs, health, rights) changed.
    ClaimInputChanged,
    /// The connection revision changed.
    ConnectionRevisionChanged,
    /// The admission fence changed.
    FenceChanged,
    /// Owner or automatic producer cancelled the request.
    Cancelled,
}

/// Current occupancy of a slot group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotOccupancy {
    pub limit_id: crate::ids::EntityId,
    pub window_id: crate::ids::EntityId,
    /// Total capacity in this slot group.
    pub total_capacity: u32,
    /// Currently occupied slots (confirmed running).
    pub occupied: u32,
    /// Slots occupied by processes whose release is not yet confirmed.
    pub unconfirmed_release: u32,
}

/// Typed refusal from `plan_protected_capacity_claim`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimRefusal {
    /// Required capacity is currently occupied and no claim can be placed.
    CapacityBusy,
    /// A blocking process has not confirmed release; claim cannot be placed.
    CapacityReleaseUnconfirmed,
    /// The claim inputs changed between selection and planning.
    ClaimInputChanged,
    /// Arithmetic overflow when aggregating duplicate slot demands.
    ArithmeticOverflow,
}

/// Plan a protected capacity claim for an aged candidate whose required slots
/// are not currently free.
///
/// Returns a proposal if the claim can be placed, or a typed refusal.
/// The residual-capacity rule (document 25 §4.4): for each slot group touched
/// by the claim, a new unrelated dispatch is allowed only if free capacity
/// after that dispatch remains >= the required slot vector.
pub fn plan_protected_capacity_claim(
    candidate: &RootTurnCandidate,
    required_slots: &[RequiredSlot],
    current_occupancy: &[SlotOccupancy],
    connection_id: crate::ids::EntityId,
    connection_revision: u64,
    request_hash: u64,
    reservation_id: crate::ids::EntityId,
    dependency_snapshot_id: crate::ids::EntityId,
    claim_id: crate::ids::EntityId,
    unit_id: crate::ids::EntityId,
    now_ms: u64,
    policy: &ValidatedSchedulingPolicy,
) -> Result<ProtectedCapacityClaimProposal, ClaimRefusal> {
    // The candidate must not have a skip reason.
    if candidate.skip_reason.is_some() {
        return Err(ClaimRefusal::ClaimInputChanged);
    }

    // Aggregate duplicate slot-group entries before capacity checks.
    // A claim listing the same (limit_id, window_id) twice with quantity q1,q2
    // must be treated as a single demand of q1+q2. Without aggregation the
    // per-entry check below would let total demand exceed capacity.
    let mut aggregated: BTreeMap<(EntityId, EntityId), u32> = BTreeMap::new();
    for r in required_slots {
        let key = (r.limit_id, r.window_id);
        let acc = aggregated.entry(key).or_insert(0);
        *acc = acc.checked_add(r.quantity).ok_or(ClaimRefusal::ArithmeticOverflow)?;
    }

    // Check each required slot group.
    for (&(limit_id, window_id), &quantity) in &aggregated {
        let occupancy = current_occupancy
            .iter()
            .find(|o| o.limit_id == limit_id && o.window_id == window_id);
        let occ = match occupancy {
            Some(o) => o,
            None => return Err(ClaimRefusal::CapacityBusy),
        };
        if occ.unconfirmed_release > 0 {
            return Err(ClaimRefusal::CapacityReleaseUnconfirmed);
        }
        let free = occ.total_capacity.saturating_sub(occ.occupied);
        if free < quantity {
            // Capacity is occupied; a claim may still be placed if the occupied
            // slots are expected to drain within bounded time. The caller must
            // supply evidence of drain; here we only check that the required
            // quantity does not exceed total capacity.
            if quantity > occ.total_capacity {
                return Err(ClaimRefusal::CapacityBusy);
            }
        }
    }

    // Compute drain deadline: max remaining upper time of blocking attempts
    // plus claim_drain_margin_ms. The caller supplies the max remaining time
    // via the occupancy data; here we use the policy margin as the bound.
    let drain_deadline_ms = now_ms
        .checked_add(policy.policy.claim_drain_margin_ms.get())
        .ok_or(ClaimRefusal::CapacityBusy)?;

    Ok(ProtectedCapacityClaimProposal {
        claim_id,
        root_work_id: candidate.view.root_work_id,
        unit_id,
        root_turn_seq: candidate.view.waiting_turn_seq,
        connection_id,
        connection_revision,
        request_hash,
        required_slot_vector: required_slots.to_vec(),
        reservation_id,
        dependency_snapshot_id,
        created_observation_ms: now_ms,
        drain_deadline_ms,
        state: ClaimState::Draining,
        withdraw_reason: None,
        row_version: crate::ids::RowVersion::MIN,
    })
}

/// Check whether a new unrelated dispatch is allowed given existing claims.
/// For each slot group, free capacity after the dispatch must remain >= the
/// sum of required quantities from all active claims touching that group.
pub fn residual_capacity_allows(
    claims: &[ProtectedCapacityClaimProposal],
    occupancy: &[SlotOccupancy],
    dispatch_slots: &[RequiredSlot],
) -> bool {
    for occ in occupancy {
        let mut reserved_by_claims = 0u32;
        for claim in claims {
            if claim.state == ClaimState::Withdrawn || claim.state == ClaimState::Consumed {
                continue;
            }
            for slot in &claim.required_slot_vector {
                if slot.limit_id == occ.limit_id && slot.window_id == occ.window_id {
                    reserved_by_claims = reserved_by_claims.saturating_add(slot.quantity);
                }
            }
        }
        let mut dispatch_wants = 0u32;
        for slot in dispatch_slots {
            if slot.limit_id == occ.limit_id && slot.window_id == occ.window_id {
                dispatch_wants = dispatch_wants.saturating_add(slot.quantity);
            }
        }
        let free = occ.total_capacity.saturating_sub(occ.occupied);
        let after = free.saturating_sub(dispatch_wants);
        if after < reserved_by_claims {
            return false;
        }
    }
    true
}
