//! Error-injection tests for dc-broker dispatch paths.
//!
//! All tests use local TCP stubs — no external network access.
//! Covers: timeout, invalid JSON, HTTP 500, connection refused.

use dc_broker::dispatch::{DispatchRequest, DispatchResult, dispatch_work_unit};
use dc_broker::endpoint::{EndpointConfig, ModelEndpoint};
use dc_broker::error::BrokerError;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

/// Spawn a multi-shot HTTP stub that responds with the given status/body to N connections.
/// Returns the port it bound to.
fn spawn_http_stub_n(status: u16, body: &str, delay_ms: u64, connections: usize) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub");
    let port = listener.local_addr().unwrap().port();
    let body = body.to_string();

    thread::spawn(move || {
        for _ in 0..connections {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 4096];
                    let _ = stream.read(&mut buf);

                    if delay_ms > 0 {
                        thread::sleep(Duration::from_millis(delay_ms));
                    }

                    let status_text = match status {
                        200 => "OK",
                        500 => "Internal Server Error",
                        _ => "Unknown",
                    };
                    let response = format!(
                        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        status,
                        status_text,
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
                Err(_) => break,
            }
        }
    });

    port
}

/// Spawn a single-shot HTTP stub that responds with the given status/body.
/// Returns the port it bound to.
fn spawn_http_stub(status: u16, body: &str, delay_ms: u64) -> u16 {
    spawn_http_stub_n(status, body, delay_ms, 1)
}

fn test_request() -> DispatchRequest {
    DispatchRequest {
        dispatch_id: "err-inj-001".to_string(),
        work_unit_spec: r#"{"kind":"LocalChangeProposal","stage":"Proposed"}"#.to_string(),
        system_prompt: "You are a test responder.".to_string(),
        user_prompt: "Reply.".to_string(),
        max_tokens: 16,
        temperature: 0.0,
    }
}

fn endpoint_for_port(port: u16, timeout_ms: u64, max_retries: u32) -> ModelEndpoint {
    let cfg = EndpointConfig {
        name: "test-stub".to_string(),
        base_url: format!("http://127.0.0.1:{}", port),
        model: "test-model".to_string(),
        timeout_ms,
        max_retries,
    };
    ModelEndpoint::new(cfg)
}

/// 1. Network timeout — stub delays response beyond client timeout.
#[tokio::test]
async fn dispatch_timeout() {
    let port = spawn_http_stub(
        200,
        r#"{"choices":[{"message":{"content":"ok"},"finish_reason":"stop"}],"model":"test-model","usage":{"total_tokens":1}}"#,
        3000,
    );
    let ep = endpoint_for_port(port, 500, 0); // 500ms timeout, 0 retries
    let req = test_request();

    let result = dispatch_work_unit(&ep, &req).await;
    assert!(result.is_err(), "expected timeout error");
    let err = result.unwrap_err();
    assert!(
        matches!(err, BrokerError::Timeout(_)),
        "expected BrokerError::Timeout, got {:?}",
        err
    );
}

/// 2. Invalid JSON response — stub returns malformed JSON with 200.
#[tokio::test]
async fn dispatch_invalid_json() {
    let port = spawn_http_stub(200, "this is not json", 0);
    let ep = endpoint_for_port(port, 5000, 0);
    let req = test_request();

    let result = dispatch_work_unit(&ep, &req).await;
    assert!(result.is_err(), "expected invalid JSON error");
    let err = result.unwrap_err();
    assert!(
        matches!(err, BrokerError::InvalidResponse(_)),
        "expected BrokerError::InvalidResponse, got {:?}",
        err
    );
}

/// 3. HTTP 500 error — stub returns 500.
#[tokio::test]
async fn dispatch_http_500() {
    let port = spawn_http_stub(500, r#"{"error":"internal server error"}"#, 0);
    let ep = endpoint_for_port(port, 5000, 0);
    let req = test_request();

    let result = dispatch_work_unit(&ep, &req).await;
    assert!(result.is_err(), "expected HTTP 500 error");
    let err = result.unwrap_err();
    assert!(
        matches!(err, BrokerError::Http(_)),
        "expected BrokerError::Http, got {:?}",
        err
    );
    assert!(
        err.to_string().contains("500"),
        "error should mention 500, got {}",
        err
    );
}

/// 4. Connection refused — no listener on the port.
#[tokio::test]
async fn dispatch_connection_refused() {
    // Bind and immediately drop to get a port that will refuse connections.
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    };

    let ep = endpoint_for_port(port, 2000, 0);
    let req = test_request();

    let result = dispatch_work_unit(&ep, &req).await;
    assert!(result.is_err(), "expected connection refused error");
    let err = result.unwrap_err();
    // Windows TCP can surface refused connections as either a connect error
    // (EndpointUnreachable) or a timeout, depending on the network stack.
    assert!(
        matches!(
            err,
            BrokerError::EndpointUnreachable(_) | BrokerError::Timeout(_)
        ),
        "expected BrokerError::EndpointUnreachable or Timeout, got {:?}",
        err
    );
}

/// 5. Retry exhaustion — HTTP 500 with retries should exhaust and return last error.
#[tokio::test]
async fn dispatch_retry_exhaustion() {
    // 1 initial + 2 retries = 3 connections
    let port = spawn_http_stub_n(500, r#"{"error":"persistent failure"}"#, 0, 3);
    let ep = endpoint_for_port(port, 5000, 2); // 2 retries
    let req = test_request();

    let result = dispatch_work_unit(&ep, &req).await;
    assert!(result.is_err(), "expected retry exhaustion error");
    let err = result.unwrap_err();
    assert!(
        matches!(err, BrokerError::Http(_)),
        "expected BrokerError::Http after retries, got {:?}",
        err
    );
}

/// 6. Rate limiting — 429 with Retry-After header.
#[tokio::test]
async fn dispatch_rate_limited() {
    // First stub: 429 with Retry-After: 1
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub");
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        for i in 0..2 {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);

                if i == 0 {
                    // First request: 429 with Retry-After: 1
                    let response = "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nRetry-After: 1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                } else {
                    // Second request: success
                    let body = r#"{"choices":[{"message":{"content":"ok"},"finish_reason":"stop"}],"model":"test-model","usage":{"total_tokens":1}}"#;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                let _ = stream.flush();
            }
        }
    });

    let ep = endpoint_for_port(port, 5000, 1); // 1 retry
    let req = test_request();

    let result = dispatch_work_unit(&ep, &req).await;
    assert!(
        matches!(result, Ok(DispatchResult::Success(_))),
        "expected success after 429 retry, got {:?}",
        result
    );
}
