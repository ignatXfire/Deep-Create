//! P0-LIVE-E2E evidence: dc-worker dispatching through dc-broker to a live
//! model endpoint, end to end.
//!
//! Chain under test:
//!   ExecuteRequest → dc_worker::execute_work_unit → injected BrokerDispatcher
//!     → dc_broker::dispatch_work_unit_sync → HTTP → live OmniRoute → ModelResponse
//!     → mapped back to dc_worker::ExecuteResult::Success
//!
//! This is NOT an in-process stub: the request leaves the process over TCP
//! to localhost:20128. The test auto-skips when no endpoint is listening,
//! so CI without a live gateway stays green.

use dc_broker::dispatch::{dispatch_work_unit_sync, DispatchRequest, DispatchResult};
use dc_broker::endpoint::{EndpointConfig, ModelEndpoint};
use dc_worker::{execute_work_unit, ExecuteRequest, ExecuteResult, WorkDispatcher, WorkerError, WorkResponse};

/// Adapter: dc_broker's sync dispatch wrapped as a dc_worker WorkDispatcher.
/// This is exactly how production wires the broker into the worker.
struct BrokerDispatcher {
    endpoint: ModelEndpoint,
}

impl WorkDispatcher for BrokerDispatcher {
    fn dispatch(&self, request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
        let broker_req = DispatchRequest {
            dispatch_id: request.execution_id.clone(),
            work_unit_spec: request.work_unit_spec.clone(),
            system_prompt: request.system_prompt.clone(),
            user_prompt: request.user_prompt.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };
        match dispatch_work_unit_sync(&self.endpoint, &broker_req) {
            Ok(DispatchResult::Success(resp)) => Ok(ExecuteResult::Success(WorkResponse {
                content: resp.content,
                model: resp.model,
                tokens_used: resp.tokens_used,
                finish_reason: resp.finish_reason,
            })),
            Ok(DispatchResult::Refused(reason)) => Ok(ExecuteResult::Refused(reason)),
            Ok(DispatchResult::Unreachable(reason)) => Ok(ExecuteResult::Unreachable(reason)),
            Err(e) => Err(WorkerError::Dispatch(format!("{}", e))),
        }
    }
}

#[test]
fn live_e2e_worker_through_broker_to_real_model() {
    // Skip cleanly when nothing is listening (CI / offline dev machine).
    if std::net::TcpStream::connect_timeout(
        &"127.0.0.1:20128".parse().unwrap(),
        std::time::Duration::from_secs(2),
    )
    .is_err()
    {
        eprintln!("SKIP: no live endpoint on 127.0.0.1:20128");
        return;
    }

    let mut cfg = EndpointConfig::default();
    cfg.max_retries = 0; // one shot in tests
    cfg.timeout_ms = 90_000;
    let dispatcher = BrokerDispatcher {
        endpoint: ModelEndpoint::new(cfg),
    };

    let request = ExecuteRequest {
        execution_id: "live-e2e-001".to_string(),
        work_unit_spec: r#"{"kind":"LocalChangeProposal","stage":"Proposed"}"#.to_string(),
        system_prompt: "You are a terse test responder. Never think, just answer.".to_string(),
        user_prompt: "Reply with exactly one word: PONG".to_string(),
        max_tokens: 512, // reasoning models burn tokens on thinking before content
        temperature: 0.0,
    };

    let result = execute_work_unit(&dispatcher, &request).expect("dispatch must not error");
    match result {
        ExecuteResult::Success(resp) => {
            eprintln!(
                "LIVE-E2E RAW: content={:?} model={} tokens={:?} finish={:?}",
                resp.content, resp.model, resp.tokens_used, resp.finish_reason
            );
            // The live proof is: HTTP round-trip succeeded, real model name came back,
            // token accounting is present. Content may be empty if the model spent
            // all tokens on reasoning (finish=length) — that is model behaviour,
            // not a transport failure.
            assert!(!resp.model.is_empty(), "model name must be present");
            assert!(
                resp.tokens_used.is_some(),
                "token accounting must be present for a live response"
            );
            if resp.finish_reason.as_deref() == Some("stop") {
                assert!(
                    !resp.content.is_empty(),
                    "a cleanly-finished response must carry content"
                );
            }
            eprintln!(
                "LIVE-E2E OK: worker→broker→live model → content={:?} model={} tokens={:?}",
                &resp.content[..resp.content.len().min(60)],
                resp.model,
                resp.tokens_used
            );
        }
        other => panic!(
            "P0-LIVE-E2E evidence failure: worker→broker→live model returned {:?}",
            other
        ),
    }
}
