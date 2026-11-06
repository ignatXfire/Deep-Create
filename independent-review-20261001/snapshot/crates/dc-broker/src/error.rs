//! Error types for dc-broker. Every failure is classified so the caller
//! (dc-worker or C-12) can decide retry vs abort without parsing strings.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrokerError {
    /// The endpoint is unreachable or returned a non-2xx status.
    #[error("endpoint unreachable: {0}")]
    EndpointUnreachable(String),

    /// The model returned a response that does not match the expected schema.
    #[error("invalid model response: {0}")]
    InvalidResponse(String),

    /// The request was malformed (missing fields, wrong types).
    #[error("malformed request: {0}")]
    MalformedRequest(String),

    /// The model endpoint returned a refusal or safety block.
    #[error("model refused: {0}")]
    ModelRefused(String),

    /// Internal serialization/deserialization error.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Network/HTTP error.
    #[error("http error: {0}")]
    Http(String),

    /// Request timeout.
    #[error("request timeout after {0}ms")]
    Timeout(u64),

    /// Rate limited by the endpoint.
    #[error("rate limited: retry after {0}ms")]
    RateLimited(u64),
}
