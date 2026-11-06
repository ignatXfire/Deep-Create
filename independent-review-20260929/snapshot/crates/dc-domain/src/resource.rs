//! C-01 resource arithmetic, not a ledger writer or a reservation token.
//!
//! C-03 must supply every applicable ancestor limit and persist an assessment
//! with row/version checks in its one transaction. In particular, independent
//! assessments over the same old view are not safe parallel reservations.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ids::{EntityId, RowVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplicitLimit<T> {
    Finite(T),
    Unlimited,
}

impl<T: PartialOrd> ExplicitLimit<T> {
    pub fn is_subset_of(&self, parent: &Self) -> bool {
        match (self, parent) {
            (_, Self::Unlimited) => true,
            (Self::Finite(child), Self::Finite(parent)) => child <= parent,
            (Self::Unlimited, Self::Finite(_)) => false,
        }
    }
}

/// Each currency is a registered identity whose minor unit is fixed by C-03.
/// Different dimensions are never summed or silently converted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResourceKind {
    MoneyMinorUnits { currency: EntityId },
    WallClockMilliseconds,
    ExecutorMilliseconds,
    BillableMilliseconds,
    Requests,
    InputUnits,
    OutputUnits,
    StorageBytes,
    ConcurrentSlots,
    SlotMilliseconds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceScope {
    Installation(EntityId),
    Session(EntityId),
    RootWork(EntityId),
    Branch(EntityId),
    Unit(EntityId),
    Provider(EntityId),
    Connection(EntityId),
    Lease(EntityId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitStatus {
    Active,
    Replaced,
    Exhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLimit {
    pub limit_id: EntityId,
    pub scope: ResourceScope,
    pub window_id: EntityId,
    pub kind: ResourceKind,
    pub bound: ExplicitLimit<u64>,
    pub opens_at_utc_millis: i64,
    pub closes_at_utc_millis: Option<i64>,
    pub status: LimitStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpperBound {
    Known(u64),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LedgerKey {
    pub logical_operation_id: EntityId,
    pub limit_id: EntityId,
    pub window_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationRequest {
    pub key: LedgerKey,
    pub reservation_id: EntityId,
    pub attempt_id: EntityId,
    pub intent_id: EntityId,
    pub kind: ResourceKind,
    pub upper_bound: UpperBound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerState {
    Reserved,
    Unknown {
        confirmed_amount: u64,
        unknown_remaining: UpperBound,
    },
    Settled {
        confirmed_amount: u64,
    },
    Released,
}

/// This value can only be produced by an assessment, not by JSON deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerPosition {
    request: ReservationRequest,
    row_version: RowVersion,
    state: LedgerState,
    bound_breached: bool,
    observations: BTreeMap<EntityId, ResourceObservation>,
}

impl LedgerPosition {
    pub fn request(&self) -> &ReservationRequest {
        &self.request
    }
    pub fn state(&self) -> LedgerState {
        self.state
    }
    pub fn row_version(&self) -> RowVersion {
        self.row_version
    }
    pub fn bound_breached(&self) -> bool {
        self.bound_breached
    }

    /// Provenance of facts already applied to this exact row; no new accounting.
    pub fn observation_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.observations.keys().copied()
    }

    pub fn contribution(&self) -> Result<UpperBound, ResourceError> {
        match self.state {
            LedgerState::Reserved => Ok(self.request.upper_bound),
            LedgerState::Unknown {
                confirmed_amount,
                unknown_remaining,
            } => add_upper(UpperBound::Known(confirmed_amount), unknown_remaining),
            LedgerState::Settled { confirmed_amount } => Ok(UpperBound::Known(confirmed_amount)),
            LedgerState::Released => Ok(UpperBound::Known(0)),
        }
    }

    fn confirmed_amount(&self) -> u64 {
        match self.state {
            LedgerState::Unknown {
                confirmed_amount, ..
            }
            | LedgerState::Settled { confirmed_amount } => confirmed_amount,
            LedgerState::Reserved | LedgerState::Released => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationAssessment {
    pub position: LedgerPosition,
    pub total_after: UpperBound,
    /// An existing position is not permission to dispatch again.
    pub already_exists: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    OutcomeUnknown,
    /// Cumulative confirmed amount; the remaining bound must be evidenced.
    Partial {
        confirmed_amount: u64,
        unknown_remaining: UpperBound,
    },
    /// A cumulative bill, including a late correction above the original U.
    Settled {
        confirmed_amount: u64,
    },
    /// Trusted evidence of no dispatch; never inferred from timeout/revocation.
    NotDispatchedConfirmed,
    /// Releases only reusable concurrency, never money or spent quota.
    ProcessTerminationConfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceObservation {
    pub observation_id: EntityId,
    pub reservation_id: EntityId,
    /// One physical reservation may have many dimensional/ancestor positions.
    /// The trusted adapter must identify this exact application of its fact.
    pub ledger_key: LedgerKey,
    /// Includes currency identity for MoneyMinorUnits; never inferred from C.
    pub resource_kind: ResourceKind,
    pub kind: ObservationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationAssessment {
    pub position: LedgerPosition,
    pub already_applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResourceError {
    #[error("integer resource arithmetic overflow")]
    ArithmeticOverflow,
    #[error("limit is not active")]
    LimitInactive,
    #[error("limit window is invalid or does not include the supplied time")]
    WindowMismatch,
    #[error("reservation does not match the exact limit/window/dimension")]
    LimitMismatch,
    #[error("an applicable limit or upper bound is missing")]
    MissingApplicableLimit,
    #[error("hard limit cannot admit an unknown upper bound")]
    UnknownUpperBound,
    #[error("resource limit reached")]
    LimitReached,
    #[error("an earlier upper-bound breach blocks new spending")]
    BoundBreached,
    #[error("duplicate ledger key in the supplied state")]
    DuplicateLedgerKey,
    #[error("the same identity has different contents")]
    ReplayConflict,
    #[error(
        "observation belongs to a different reservation, ledger position, or resource dimension"
    )]
    ObservationMismatch,
    #[error("observation would regress known facts or release unconfirmed usage")]
    InvalidObservationTransition,
}

fn add_upper(a: UpperBound, b: UpperBound) -> Result<UpperBound, ResourceError> {
    match (a, b) {
        (UpperBound::Known(a), UpperBound::Known(b)) => a
            .checked_add(b)
            .map(UpperBound::Known)
            .ok_or(ResourceError::ArithmeticOverflow),
        _ => Ok(UpperBound::Unknown),
    }
}

fn sum_positions<'a>(
    positions: impl Iterator<Item = &'a LedgerPosition>,
) -> Result<UpperBound, ResourceError> {
    let mut known = 0_u64;
    let mut unknown = false;
    for position in positions {
        match position.contribution()? {
            UpperBound::Known(value) => {
                known = known
                    .checked_add(value)
                    .ok_or(ResourceError::ArithmeticOverflow)?
            }
            UpperBound::Unknown => unknown = true,
        }
    }
    Ok(if unknown {
        UpperBound::Unknown
    } else {
        UpperBound::Known(known)
    })
}

/// Deterministic calculation only; C-03 must atomically persist the new row.
pub fn evaluate_reservation(
    limit: &ResourceLimit,
    current: &[LedgerPosition],
    request: &ReservationRequest,
    now_utc_millis: i64,
) -> Result<ReservationAssessment, ResourceError> {
    if request.key.limit_id != limit.limit_id
        || request.key.window_id != limit.window_id
        || request.kind != limit.kind
    {
        return Err(ResourceError::LimitMismatch);
    }
    let mut keys = BTreeSet::new();
    for position in current {
        if !keys.insert(position.request.key) {
            return Err(ResourceError::DuplicateLedgerKey);
        }
    }
    let relevant: Vec<_> = current
        .iter()
        .filter(|p| {
            p.request.key.limit_id == limit.limit_id && p.request.key.window_id == limit.window_id
        })
        .collect();
    if relevant.iter().any(|p| p.request.kind != limit.kind) {
        return Err(ResourceError::LimitMismatch);
    }
    let occupied = sum_positions(relevant.iter().copied())?;
    if let Some(existing) = relevant.iter().find(|p| p.request.key == request.key) {
        if existing.request != *request {
            return Err(ResourceError::ReplayConflict);
        }
        return Ok(ReservationAssessment {
            position: (**existing).clone(),
            total_after: occupied,
            already_exists: true,
        });
    }
    if current.iter().any(|p| {
        p.request.reservation_id == request.reservation_id
            && (p.request.key.logical_operation_id != request.key.logical_operation_id
                || p.request.attempt_id != request.attempt_id
                || p.request.intent_id != request.intent_id)
    }) {
        return Err(ResourceError::ReplayConflict);
    }
    if limit.status != LimitStatus::Active {
        return Err(ResourceError::LimitInactive);
    }
    if now_utc_millis < limit.opens_at_utc_millis
        || limit
            .closes_at_utc_millis
            .is_some_and(|end| end <= limit.opens_at_utc_millis || now_utc_millis >= end)
    {
        return Err(ResourceError::WindowMismatch);
    }
    if relevant.iter().any(|p| p.bound_breached) {
        return Err(ResourceError::BoundBreached);
    }
    let total_after = add_upper(occupied, request.upper_bound)?;
    match (limit.bound, total_after) {
        (ExplicitLimit::Finite(_), UpperBound::Unknown) => {
            return Err(ResourceError::UnknownUpperBound);
        }
        (ExplicitLimit::Finite(max), UpperBound::Known(total)) if total > max => {
            return Err(ResourceError::LimitReached);
        }
        _ => {}
    }
    Ok(ReservationAssessment {
        position: LedgerPosition {
            request: request.clone(),
            row_version: RowVersion::MIN,
            state: LedgerState::Reserved,
            bound_breached: false,
            observations: BTreeMap::new(),
        },
        total_after,
        already_exists: false,
    })
}

/// Assess exactly one row per supplied applicable ancestor limit. The caller
/// derives that complete limit set from its trusted scope graph, not model JSON.
/// Failure returns no partial plan. The plan is not an atomic commit by itself.
pub fn evaluate_reservations(
    applicable_limits: &[ResourceLimit],
    current: &[LedgerPosition],
    requests: &[ReservationRequest],
    now_utc_millis: i64,
) -> Result<Vec<ReservationAssessment>, ResourceError> {
    if applicable_limits.is_empty() || requests.len() != applicable_limits.len() {
        return Err(ResourceError::MissingApplicableLimit);
    }
    let first = &requests[0];
    let mut seen = BTreeSet::new();
    let mut upper_by_kind = BTreeMap::new();
    let mut result = Vec::with_capacity(applicable_limits.len());
    for limit in applicable_limits {
        if !seen.insert((limit.limit_id, limit.window_id)) {
            return Err(ResourceError::DuplicateLedgerKey);
        }
        let matches: Vec<_> = requests
            .iter()
            .filter(|r| r.key.limit_id == limit.limit_id && r.key.window_id == limit.window_id)
            .collect();
        if matches.len() != 1 {
            return Err(ResourceError::MissingApplicableLimit);
        }
        let request = matches[0];
        if request.key.logical_operation_id != first.key.logical_operation_id
            || request.reservation_id != first.reservation_id
            || request.attempt_id != first.attempt_id
            || request.intent_id != first.intent_id
        {
            return Err(ResourceError::ReplayConflict);
        }
        if upper_by_kind
            .insert(request.kind, request.upper_bound)
            .is_some_and(|upper| upper != request.upper_bound)
        {
            return Err(ResourceError::LimitMismatch);
        }
        result.push(evaluate_reservation(
            limit,
            current,
            request,
            now_utc_millis,
        )?);
    }
    Ok(result)
}

/// Replaces a row's contribution; an observation is never a second debit.
/// Revocation/expiry are intentionally irrelevant to accounting already-seen facts.
pub fn apply_observation(
    position: &LedgerPosition,
    observation: ResourceObservation,
) -> Result<ObservationAssessment, ResourceError> {
    if observation.reservation_id != position.request.reservation_id
        || observation.ledger_key != position.request.key
        || observation.resource_kind != position.request.kind
    {
        return Err(ResourceError::ObservationMismatch);
    }
    if let Some(previous) = position.observations.get(&observation.observation_id) {
        if *previous != observation {
            return Err(ResourceError::ReplayConflict);
        }
        return Ok(ObservationAssessment {
            position: position.clone(),
            already_applied: true,
        });
    }
    let confirmed_before = position.confirmed_amount();
    let next_state = match observation.kind {
        ObservationKind::OutcomeUnknown => match position.state {
            LedgerState::Reserved => LedgerState::Unknown {
                confirmed_amount: 0,
                unknown_remaining: position.request.upper_bound,
            },
            LedgerState::Unknown { .. } => position.state,
            _ => return Err(ResourceError::InvalidObservationTransition),
        },
        ObservationKind::Partial {
            confirmed_amount,
            unknown_remaining,
        } => {
            if confirmed_amount < confirmed_before
                || matches!(
                    position.state,
                    LedgerState::Settled { .. } | LedgerState::Released
                )
            {
                return Err(ResourceError::InvalidObservationTransition);
            }
            LedgerState::Unknown {
                confirmed_amount,
                unknown_remaining,
            }
        }
        ObservationKind::Settled { confirmed_amount } => {
            if confirmed_amount < confirmed_before {
                return Err(ResourceError::InvalidObservationTransition);
            }
            // Even an unexpected bill after a prior no-dispatch observation is a
            // fact to retain. A positive correction to Released also flags breach.
            LedgerState::Settled { confirmed_amount }
        }
        ObservationKind::NotDispatchedConfirmed => {
            // A lost acknowledgement can first create Unknown. Subsequent
            // trusted broker reconciliation may prove that D never occurred.
            // This is new evidence, never an inference from the loss itself.
            if !matches!(
                position.state,
                LedgerState::Reserved
                    | LedgerState::Unknown {
                        confirmed_amount: 0,
                        ..
                    }
            ) {
                return Err(ResourceError::InvalidObservationTransition);
            }
            LedgerState::Released
        }
        ObservationKind::ProcessTerminationConfirmed => {
            if position.request.kind != ResourceKind::ConcurrentSlots {
                return Err(ResourceError::InvalidObservationTransition);
            }
            LedgerState::Released
        }
    };
    let mut next = position.clone();
    next.row_version = RowVersion::new(
        position
            .row_version
            .get()
            .checked_add(1)
            .ok_or(ResourceError::ArithmeticOverflow)?,
    )
    .ok_or(ResourceError::ArithmeticOverflow)?;
    next.state = next_state;
    // An overflowing C+R is retained as two exact facts and closes admission.
    // Refusing this observation would hide an already observed cost.
    let contribution = next.contribution();
    if let UpperBound::Known(upper) = position.request.upper_bound {
        next.bound_breached |= match contribution {
            Ok(UpperBound::Known(actual)) => actual > upper,
            Ok(UpperBound::Unknown) | Err(_) => true,
        };
    }
    next.bound_breached |= contribution.is_err();
    next.bound_breached |=
        matches!(position.state, LedgerState::Released) && next.confirmed_amount() > 0;
    next.observations
        .insert(observation.observation_id, observation);
    Ok(ObservationAssessment {
        position: next,
        already_applied: false,
    })
}

/// Upper service cost is a checked product, not a duration prediction.
pub fn upper_service_cost(
    parallel_slots: u64,
    enforced_wall_time_millis: u64,
) -> Result<u64, ResourceError> {
    parallel_slots
        .checked_mul(enforced_wall_time_millis)
        .ok_or(ResourceError::ArithmeticOverflow)
}
