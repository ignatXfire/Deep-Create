//! C-01 comparison of trusted authority views. No issuance, clocks, I/O, or gate.
//!
//! These functions do not authenticate an owner event and their successful result
//! is not an executable capability. C-09/C-10 must obtain current trusted views
//! and repeat checks while holding AdmissionFence through dispatch/commit.
//! Exact object identities here do not prove Windows handle/ACL/race safety.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ids::{AuthorityEpoch, EntityId, RowVersion, RunGeneration};
use crate::resource::{ExplicitLimit, ResourceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    ReadEvidenceFragment,
    ReadWorkspaceFile,
    WriteWorkerScratch,
    PrepareWorkspaceChange,
    CommitWorkspaceChange,
    SpawnRegisteredProgram,
    CallModelConnection,
    CallRegisteredEndpoint,
    ReconcileExternalIntent,
    ImportHermesSnapshot,
}

/// Registered identities only: there is no arbitrary path, URL, or wildcard.
/// Canonicalization and actual-object verification are future C-09/C-12 work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalResource {
    EvidenceFragment {
        object_id: EntityId,
        fragment_id: EntityId,
    },
    WorkspaceFile {
        workspace_id: EntityId,
        binding_revision: RowVersion,
        object_id: EntityId,
    },
    WorkerScratch {
        attempt_id: EntityId,
        object_id: EntityId,
    },
    RegisteredProgram {
        program_id: EntityId,
        revision: RowVersion,
    },
    ModelConnection {
        connection_id: EntityId,
        revision: RowVersion,
    },
    RegisteredEndpoint {
        endpoint_id: EntityId,
        revision: RowVersion,
    },
    ExternalIntent {
        intent_id: EntityId,
    },
    HermesSnapshot {
        snapshot_id: EntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalRecipient {
    ModelConnection {
        connection_id: EntityId,
        revision: RowVersion,
    },
    RegisteredEndpoint {
        endpoint_id: EntityId,
        revision: RowVersion,
    },
    RegisteredProgram {
        program_id: EntityId,
        revision: RowVersion,
    },
}

/// Canonical sensitivity classes from contract 15 section 4.2.
/// These are not representations or executable handles. An opaque reference to
/// a secret has its own sensitivity; it does not relabel secret bytes as safe.
/// The set in a lease is explicit, not an implicit hierarchy from enum ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DataClass {
    Public,
    Local,
    Personal,
    Restricted,
    Secret,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseScope {
    pub work_id: EntityId,
    /// None explicitly permits units within this one work, never another work.
    pub unit_id: Option<EntityId>,
    pub action_kind: ActionKind,
    pub resource: CanonicalResource,
    /// None means no recipient, not any recipient.
    pub recipient: Option<CanonicalRecipient>,
    pub data_classes: BTreeSet<DataClass>,
    pub max_calls: ExplicitLimit<u32>,
    pub max_bytes: ExplicitLimit<u64>,
    /// Same dimension set is required on derivation; omission grants nothing.
    pub cost_limits: BTreeMap<ResourceKind, ExplicitLimit<u64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonotonicDeadline {
    pub run_generation: RunGeneration,
    pub at_millis: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseExpiry {
    Never,
    At {
        utc_millis: i64,
        monotonic: MonotonicDeadline,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockIntegrity {
    Trusted,
    RolledBack,
    Uncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockObservation {
    pub utc_millis: i64,
    pub monotonic_millis: u64,
    pub run_generation: RunGeneration,
    pub integrity: ClockIntegrity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FenceScope {
    RootWork(EntityId),
    Unit(EntityId),
    Lease(EntityId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopedFenceVersion {
    pub scope: FenceScope,
    pub version: RowVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentFence {
    pub version: RowVersion,
    pub closed: bool,
}

/// A supplied view, not a signed or transferable admission credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoritySnapshot {
    pub installation_id: EntityId,
    pub authority_epoch: AuthorityEpoch,
    pub run_generation: RunGeneration,
    pub fences: BTreeMap<FenceScope, CurrentFence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseStatus {
    Requested,
    Active,
    Consumed,
    Expired,
    Revoked,
    Superseded,
}

/// Only a native value-view for validation: deliberately no Deserialize or
/// issue/activate constructor. Constructing this struct is not owner approval.
///
/// ```compile_fail
/// use dc_domain::permission::CapabilityLease;
/// let forged = serde_json::from_str::<CapabilityLease>(r#"{"status":"Active"}"#);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityLease {
    pub lease_id: EntityId,
    pub owner_event_id: EntityId,
    pub installation_id: EntityId,
    pub scope: LeaseScope,
    pub not_before_utc_millis: i64,
    pub expiry: LeaseExpiry,
    pub parent_lease_id: Option<EntityId>,
    pub delegation_depth: u8,
    pub max_delegation_depth: u8,
    pub authority_epoch: AuthorityEpoch,
    pub run_generation: RunGeneration,
    /// Includes all authority supports, not just the receiving work's fence.
    pub support_fences: Vec<ScopedFenceVersion>,
    pub confirmed_conditions_hash: [u8; 32],
    pub issued_dependency_snapshot_id: EntityId,
    pub lease_version: RowVersion,
    pub status: LeaseStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PermissionError {
    #[error("scope, action, resource, recipient, or data expansion")]
    ScopeExpansion,
    #[error("expiration or not-before expansion")]
    ExpiryExpansion,
    #[error("resource or call limit expansion")]
    LimitExpansion,
    #[error("delegation lineage/depth is invalid")]
    InvalidDelegation,
    #[error("a support fence is missing, duplicated, or inconsistent")]
    InvalidSupportFences,
    #[error("lease is not active")]
    LeaseInactive,
    #[error("terminal lease state cannot revive or change terminal identity")]
    LeaseRevival,
    #[error("installation identity differs")]
    InstallationMismatch,
    #[error("authority epoch differs; neither larger nor smaller is accepted")]
    EpochMismatch,
    #[error("run generation differs")]
    StaleGeneration,
    #[error("clock rollback or uncertain observation closes time-based admission")]
    ClockUntrusted,
    #[error("lease has not started")]
    NotYetValid,
    #[error("lease has expired")]
    Expired,
    #[error("lease time interval is invalid")]
    InvalidTimeRange,
    #[error("support fence is absent, closed, or changed")]
    FenceChanged,
    #[error("expected lease version differs")]
    VersionChanged,
    #[error("action and canonical object/recipient do not match")]
    ActionMismatch,
    #[error("secret content cannot be disclosed to a model")]
    SecretDisclosureDenied,
    #[error("integer authority counter overflow")]
    ArithmeticOverflow,
}

fn validate_scope_shape(scope: &LeaseScope) -> Result<(), PermissionError> {
    // Credential injection inside C-09 is a distinct protected operation, not
    // caller-provided Secret content in a model request or ContextProjection.
    if scope.action_kind == ActionKind::CallModelConnection
        && scope.data_classes.contains(&DataClass::Secret)
    {
        return Err(PermissionError::SecretDisclosureDenied);
    }
    use ActionKind as A;
    use CanonicalResource as R;
    let compatible = match (&scope.action_kind, &scope.resource, &scope.recipient) {
        (A::ReadEvidenceFragment, R::EvidenceFragment { .. }, None)
        | (
            A::ReadWorkspaceFile | A::PrepareWorkspaceChange | A::CommitWorkspaceChange,
            R::WorkspaceFile { .. },
            None,
        )
        | (A::WriteWorkerScratch, R::WorkerScratch { .. }, None)
        | (A::ImportHermesSnapshot, R::HermesSnapshot { .. }, None) => true,
        (
            A::SpawnRegisteredProgram,
            R::RegisteredProgram {
                program_id,
                revision,
            },
            Some(CanonicalRecipient::RegisteredProgram {
                program_id: recipient,
                revision: recipient_revision,
            }),
        ) => program_id == recipient && revision == recipient_revision,
        (
            A::CallModelConnection,
            R::ModelConnection {
                connection_id,
                revision,
            },
            Some(CanonicalRecipient::ModelConnection {
                connection_id: recipient,
                revision: recipient_revision,
            }),
        ) => connection_id == recipient && revision == recipient_revision,
        (
            A::CallRegisteredEndpoint,
            R::RegisteredEndpoint {
                endpoint_id,
                revision,
            },
            Some(CanonicalRecipient::RegisteredEndpoint {
                endpoint_id: recipient,
                revision: recipient_revision,
            }),
        ) => endpoint_id == recipient && revision == recipient_revision,
        (
            A::ReconcileExternalIntent,
            R::ExternalIntent { .. },
            Some(
                CanonicalRecipient::ModelConnection { .. }
                | CanonicalRecipient::RegisteredEndpoint { .. },
            ),
        ) => true,
        _ => false,
    };
    if compatible {
        Ok(())
    } else {
        Err(PermissionError::ActionMismatch)
    }
}

/// Conservative subset: no path-prefix, resource hierarchy, endpoint alias, or
/// cross-action implication is guessed. Unknown comparisons fail closed.
pub fn is_scope_subset(parent: &LeaseScope, requested: &LeaseScope) -> Result<(), PermissionError> {
    validate_scope_shape(parent)?;
    validate_scope_shape(requested)?;
    if parent.work_id != requested.work_id
        || parent
            .unit_id
            .is_some_and(|unit| requested.unit_id != Some(unit))
        || parent.action_kind != requested.action_kind
        || parent.resource != requested.resource
        || parent.recipient != requested.recipient
        || !requested.data_classes.is_subset(&parent.data_classes)
    {
        return Err(PermissionError::ScopeExpansion);
    }
    if !requested.max_calls.is_subset_of(&parent.max_calls)
        || !requested.max_bytes.is_subset_of(&parent.max_bytes)
        || parent.cost_limits.len() != requested.cost_limits.len()
        || parent.cost_limits.iter().any(|(kind, bound)| {
            requested
                .cost_limits
                .get(kind)
                .is_none_or(|child| !child.is_subset_of(bound))
        })
    {
        return Err(PermissionError::LimitExpansion);
    }
    Ok(())
}

fn fence_map(lease: &CapabilityLease) -> Result<BTreeMap<FenceScope, RowVersion>, PermissionError> {
    let mut map = BTreeMap::new();
    for fence in &lease.support_fences {
        if map.insert(fence.scope, fence.version).is_some() {
            return Err(PermissionError::InvalidSupportFences);
        }
    }
    if !map.contains_key(&FenceScope::RootWork(lease.scope.work_id))
        || !map.contains_key(&FenceScope::Lease(lease.lease_id))
        || lease
            .scope
            .unit_id
            .is_some_and(|unit| !map.contains_key(&FenceScope::Unit(unit)))
        || lease
            .parent_lease_id
            .is_some_and(|parent| !map.contains_key(&FenceScope::Lease(parent)))
    {
        return Err(PermissionError::InvalidSupportFences);
    }
    Ok(map)
}

fn validate_lease_shape(lease: &CapabilityLease) -> Result<(), PermissionError> {
    validate_scope_shape(&lease.scope)?;
    fence_map(lease)?;
    if lease.delegation_depth > lease.max_delegation_depth
        || (lease.parent_lease_id.is_none() && lease.delegation_depth != 0)
        || (lease.parent_lease_id.is_some() && lease.delegation_depth == 0)
    {
        return Err(PermissionError::InvalidDelegation);
    }
    if let LeaseExpiry::At {
        utc_millis,
        monotonic,
    } = lease.expiry
    {
        if utc_millis <= lease.not_before_utc_millis {
            return Err(PermissionError::InvalidTimeRange);
        }
        if monotonic.run_generation != lease.run_generation {
            return Err(PermissionError::StaleGeneration);
        }
    }
    Ok(())
}

pub fn is_lease_subset(
    parent: &CapabilityLease,
    requested: &CapabilityLease,
) -> Result<(), PermissionError> {
    validate_lease_shape(parent)?;
    validate_lease_shape(requested)?;
    if parent.status != LeaseStatus::Active || requested.status != LeaseStatus::Active {
        return Err(PermissionError::LeaseInactive);
    }
    if parent.installation_id != requested.installation_id {
        return Err(PermissionError::InstallationMismatch);
    }
    if parent.authority_epoch != requested.authority_epoch {
        return Err(PermissionError::EpochMismatch);
    }
    if parent.run_generation != requested.run_generation {
        return Err(PermissionError::StaleGeneration);
    }
    if requested.lease_id == parent.lease_id
        || requested.parent_lease_id != Some(parent.lease_id)
        || requested.delegation_depth
            != parent
                .delegation_depth
                .checked_add(1)
                .ok_or(PermissionError::ArithmeticOverflow)?
        || requested.max_delegation_depth > parent.max_delegation_depth
    {
        return Err(PermissionError::InvalidDelegation);
    }
    is_scope_subset(&parent.scope, &requested.scope)?;
    if requested.not_before_utc_millis < parent.not_before_utc_millis {
        return Err(PermissionError::ExpiryExpansion);
    }
    match (parent.expiry, requested.expiry) {
        (LeaseExpiry::Never, _) => {}
        (
            LeaseExpiry::At {
                utc_millis: parent_utc,
                monotonic: parent_mono,
            },
            LeaseExpiry::At {
                utc_millis: child_utc,
                monotonic: child_mono,
            },
        ) if child_utc <= parent_utc
            && child_mono.at_millis <= parent_mono.at_millis
            && child_mono.run_generation == parent_mono.run_generation => {}
        _ => return Err(PermissionError::ExpiryExpansion),
    }
    let requested_fences = fence_map(requested)?;
    if fence_map(parent)?
        .iter()
        .any(|(scope, version)| requested_fences.get(scope) != Some(version))
    {
        return Err(PermissionError::InvalidSupportFences);
    }
    Ok(())
}

/// Recheck at each real D/C boundary, not once while preparing the request.
/// Whole ancestor-chain scope/cost checks and the actual gate remain C-09/C-10.
pub fn verify_lease(
    lease: &CapabilityLease,
    expected_lease_version: RowVersion,
    authority: &AuthoritySnapshot,
    clock: ClockObservation,
) -> Result<(), PermissionError> {
    validate_lease_shape(lease)?;
    if lease.status != LeaseStatus::Active {
        return Err(PermissionError::LeaseInactive);
    }
    if lease.lease_version != expected_lease_version {
        return Err(PermissionError::VersionChanged);
    }
    if lease.installation_id != authority.installation_id {
        return Err(PermissionError::InstallationMismatch);
    }
    if lease.authority_epoch != authority.authority_epoch {
        return Err(PermissionError::EpochMismatch);
    }
    if lease.run_generation != authority.run_generation
        || clock.run_generation != authority.run_generation
    {
        return Err(PermissionError::StaleGeneration);
    }
    if clock.integrity != ClockIntegrity::Trusted {
        return Err(PermissionError::ClockUntrusted);
    }
    if clock.utc_millis < lease.not_before_utc_millis {
        return Err(PermissionError::NotYetValid);
    }
    if let LeaseExpiry::At {
        utc_millis,
        monotonic,
    } = lease.expiry
        && (clock.utc_millis >= utc_millis || clock.monotonic_millis >= monotonic.at_millis)
    {
        return Err(PermissionError::Expired);
    }
    for (scope, version) in fence_map(lease)? {
        if authority
            .fences
            .get(&scope)
            .is_none_or(|current| current.closed || current.version != version)
        {
            return Err(PermissionError::FenceChanged);
        }
    }
    Ok(())
}

/// Validates state edges only. In particular Requested -> Active does not prove
/// an authenticated owner decision and does not issue a CapabilityLease.
pub fn validate_lease_transition(
    from: LeaseStatus,
    to: LeaseStatus,
) -> Result<(), PermissionError> {
    if from == to {
        return Ok(());
    }
    match (from, to) {
        (LeaseStatus::Requested, LeaseStatus::Active)
        | (
            LeaseStatus::Active,
            LeaseStatus::Consumed
            | LeaseStatus::Expired
            | LeaseStatus::Revoked
            | LeaseStatus::Superseded,
        ) => Ok(()),
        _ => Err(PermissionError::LeaseRevival),
    }
}

pub fn next_fence_version(version: RowVersion) -> Result<RowVersion, PermissionError> {
    RowVersion::new(
        version
            .get()
            .checked_add(1)
            .ok_or(PermissionError::ArithmeticOverflow)?,
    )
    .ok_or(PermissionError::ArithmeticOverflow)
}
