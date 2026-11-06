//! Integration tests for dc-worker.

use dc_worker::error::WorkerError;
use dc_worker::execute::{
    ExecuteRequest, ExecuteResult, WorkDispatcher, WorkResponse, execute_batch, execute_work_unit,
};

struct StubDispatcher;

impl WorkDispatcher for StubDispatcher {
    fn dispatch(&self, request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
        Ok(ExecuteResult::Success(WorkResponse {
            content: format!("processed: {}", request.execution_id),
            model: "stub-model".to_string(),
            tokens_used: Some(42),
            finish_reason: Some("stop".to_string()),
        }))
    }
}

struct RefusingDispatcher;

impl WorkDispatcher for RefusingDispatcher {
    fn dispatch(&self, _request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
        Ok(ExecuteResult::Refused(
            "content policy violation".to_string(),
        ))
    }
}

struct UnreachableDispatcher;

impl WorkDispatcher for UnreachableDispatcher {
    fn dispatch(&self, _request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
        Ok(ExecuteResult::Unreachable("connection timeout".to_string()))
    }
}

fn make_request(id: &str, spec: &str) -> ExecuteRequest {
    ExecuteRequest {
        execution_id: id.to_string(),
        work_unit_spec: spec.to_string(),
        system_prompt: "You are a Rust developer.".to_string(),
        user_prompt: "Fix the bug.".to_string(),
        max_tokens: 4096,
        temperature: 0.1,
    }
}

#[test]
fn execute_success_through_dispatcher() {
    let d = StubDispatcher;
    let req = make_request(
        "exec-001",
        r#"{"kind":"LocalChangeProposal","stage":"Proposed"}"#,
    );
    let result = execute_work_unit(&d, &req).unwrap();
    match result {
        ExecuteResult::Success(resp) => {
            assert!(resp.content.contains("exec-001"));
            assert_eq!(resp.model, "stub-model");
            assert_eq!(resp.tokens_used, Some(42));
        }
        _ => panic!("expected success"),
    }
}

#[test]
fn execute_refused() {
    let d = RefusingDispatcher;
    let req = make_request("exec-002", r#"{"kind":"LocalChangeProposal"}"#);
    let result = execute_work_unit(&d, &req).unwrap();
    assert!(matches!(result, ExecuteResult::Refused(_)));
}

#[test]
fn execute_unreachable() {
    let d = UnreachableDispatcher;
    let req = make_request("exec-003", r#"{"kind":"LocalChangeProposal"}"#);
    let result = execute_work_unit(&d, &req).unwrap();
    assert!(matches!(result, ExecuteResult::Unreachable(_)));
}

#[test]
fn execute_empty_spec_rejected() {
    let d = StubDispatcher;
    let req = make_request("exec-004", "");
    let result = execute_work_unit(&d, &req);
    assert!(matches!(result, Err(WorkerError::InvalidSpec(_))));
}

#[test]
fn execute_oversize_spec_rejected() {
    let d = StubDispatcher;
    let big_spec = "x".repeat(65_537);
    let req = make_request("exec-005", &big_spec);
    let result = execute_work_unit(&d, &req);
    assert!(matches!(result, Err(WorkerError::InvalidSpec(_))));
}

#[test]
fn batch_execution_mixed_results() {
    let d = StubDispatcher;
    let reqs = vec![
        make_request("batch-001", r#"{"kind":"LocalChangeProposal"}"#),
        make_request("batch-002", r#"{"kind":"LocalChangeProposal"}"#),
        make_request("batch-003", r#"{"kind":"LocalChangeProposal"}"#),
    ];
    let results = execute_batch(&d, &reqs);
    assert_eq!(results.len(), 3);
    for (i, r) in results.iter().enumerate() {
        assert!(r.is_ok(), "batch item {} failed", i);
    }
}
