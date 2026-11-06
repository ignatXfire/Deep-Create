//! Core error types

use thiserror::Error;

/// Errors from core controller operations
#[derive(Debug, Error)]
pub enum CoreError {
    /// Command validation failed
    #[error("invalid command: {0}")]
    InvalidCommand(String),

    /// State transition invalid
    #[error("invalid transition: {0}")]
    InvalidTransition(String),

    /// Missing required permission
    #[error("missing permission: {0}")]
    MissingPermission(String),

    /// Resource limit reached
    #[error("resource limit reached: {0}")]
    LimitReached(String),

    /// Storage operation failed
    #[error("storage error: {0}")]
    Storage(#[from] dc_store::StoreError),

    /// Persistence operation failed
    #[error("persistence error: {0}")]
    Persistence(String),

    /// Domain validation failed
    #[error("domain error: {0}")]
    Domain(String),

    /// Protocol error
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Verification method mismatch or unqualified
    #[error("verification mismatch: {0}")]
    VerificationMismatch(String),

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Verification failed
    #[error("verification failed: {0}")]
    VerificationFailed(String),

    /// No work available
    #[error("no ready work units")]
    NoReadyWork,

    /// Objective not set
    #[error("no active objective")]
    NoObjective,

    /// Unit not found
    #[error("unit not found: {0}")]
    UnitNotFound(String),

    /// Dependency not satisfied
    #[error("dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),
}
