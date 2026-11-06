//! Error types for dc-worker.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkerError {
    /// The work-unit spec is malformed or missing required fields.
    #[error("invalid work-unit spec: {0}")]
    InvalidSpec(String),

    /// The dispatcher failed to produce a result.
    #[error("dispatch error: {0}")]
    Dispatch(String),

    /// The model returned a refusal or empty response.
    #[error("model refused or empty: {0}")]
    ModelRefused(String),

    /// Internal serialization error.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}
