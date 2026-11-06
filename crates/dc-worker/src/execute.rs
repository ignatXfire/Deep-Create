//! Execution logic: work-unit spec → dispatch → result.
//!
//! The worker does NOT validate authority — that is C-12's job. It does NOT
//! persist results — that is C-03's job. It does NOT schedule — that is C-01's job.
//! It does NOT know about the broker — dispatch is injected via the WorkDispatcher trait.

use crate::error::WorkerError;
use serde::{Deserialize, Serialize};

/// A request to execute a single work unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    /// Unique identifier for this execution (traceability).
    pub execution_id: String,
    /// The work-unit spec as a JSON string.
    pub work_unit_spec: String,
    /// System prompt for the model.
    pub system_prompt: String,
    /// User prompt for the model (the actual task).
    pub user_prompt: String,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Temperature (0.0 = deterministic, 1.0 = creative).
    pub temperature: f32,
}

/// The model's response content plus metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResponse {
    /// The model's output text.
    pub content: String,
    /// Model that generated this (for audit).
    pub model: String,
    /// Tokens used (prompt + completion).
    pub tokens_used: Option<u32>,
    /// Finish reason.
    pub finish_reason: Option<String>,
}

/// Result of executing a work unit.
#[derive(Debug)]
pub enum ExecuteResult {
    /// The model returned a valid response.
    Success(WorkResponse),
    /// The model refused or was blocked.
    Refused(String),
    /// The endpoint was unreachable after all retries.
    Unreachable(String),
}

/// Trait for dispatching work to a model. The worker does NOT own
/// network/secret/endpoint authority — that belongs to C-09 (broker).
/// The broker (or a test stub) implements this trait and injects it.
pub trait WorkDispatcher {
    fn dispatch(&self, request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError>;
}

/// Execute a work unit through the given dispatcher.
///
/// This is the ONLY public API of dc-worker. Everything else is internal.
pub fn execute_work_unit(
    dispatcher: &dyn WorkDispatcher,
    request: &ExecuteRequest,
) -> Result<ExecuteResult, WorkerError> {
    // Validate the spec before dispatching.
    if request.work_unit_spec.is_empty() {
        return Err(WorkerError::InvalidSpec("empty spec".to_string()));
    }
    if request.work_unit_spec.len() > 65_536 {
        return Err(WorkerError::InvalidSpec("spec exceeds 64KB".to_string()));
    }

    // Dispatch through the injected dispatcher (broker or test stub).
    dispatcher.dispatch(request)
}

/// Execute a batch of work units sequentially.
/// Returns a vec of results in the same order as the requests.
pub fn execute_batch(
    dispatcher: &dyn WorkDispatcher,
    requests: &[ExecuteRequest],
) -> Vec<Result<ExecuteResult, WorkerError>> {
    requests
        .iter()
        .map(|req| execute_work_unit(dispatcher, req))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubDispatcher;

    impl WorkDispatcher for StubDispatcher {
        fn dispatch(&self, request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
            Ok(ExecuteResult::Success(WorkResponse {
                content: format!("processed: {}", request.execution_id),
                model: "stub".to_string(),
                tokens_used: Some(10),
                finish_reason: Some("stop".to_string()),
            }))
        }
    }

    struct FailingDispatcher;

    impl WorkDispatcher for FailingDispatcher {
        fn dispatch(&self, _request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
            Err(WorkerError::Dispatch("endpoint unreachable".to_string()))
        }
    }

    fn test_request(id: &str) -> ExecuteRequest {
        ExecuteRequest {
            execution_id: id.to_string(),
            work_unit_spec: r#"{"kind":"code_review"}"#.to_string(),
            system_prompt: "You are a reviewer.".to_string(),
            user_prompt: "Review this code.".to_string(),
            max_tokens: 1024,
            temperature: 0.0,
        }
    }

    #[test]
    fn execute_success() {
        let d = StubDispatcher;
        let req = test_request("exec-1");
        let result = execute_work_unit(&d, &req).unwrap();
        match result {
            ExecuteResult::Success(resp) => {
                assert!(resp.content.contains("exec-1"));
                assert_eq!(resp.model, "stub");
            }
            _ => panic!("expected success"),
        }
    }

    #[test]
    fn execute_empty_spec_rejected() {
        let d = StubDispatcher;
        let mut req = test_request("exec-2");
        req.work_unit_spec = String::new();
        let err = execute_work_unit(&d, &req).unwrap_err();
        assert!(matches!(err, WorkerError::InvalidSpec(_)));
    }

    #[test]
    fn execute_oversize_spec_rejected() {
        let d = StubDispatcher;
        let mut req = test_request("exec-3");
        req.work_unit_spec = "x".repeat(65_537);
        let err = execute_work_unit(&d, &req).unwrap_err();
        assert!(matches!(err, WorkerError::InvalidSpec(_)));
    }

    #[test]
    fn execute_dispatch_failure() {
        let d = FailingDispatcher;
        let req = test_request("exec-4");
        let err = execute_work_unit(&d, &req).unwrap_err();
        assert!(matches!(err, WorkerError::Dispatch(_)));
    }

    #[test]
    fn batch_execution() {
        let d = StubDispatcher;
        let reqs: Vec<_> = (0..3)
            .map(|i| test_request(&format!("batch-{}", i)))
            .collect();
        let results = execute_batch(&d, &reqs);
        assert_eq!(results.len(), 3);
        for (i, r) in results.iter().enumerate() {
            assert!(r.is_ok(), "batch item {} failed", i);
        }
    }
}
