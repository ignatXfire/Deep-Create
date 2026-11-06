//! Wire-level test: proves the broker sends the correct payload to the model
//! endpoint, respects Retry-After, and does not multiply requests beyond
//! configured retry budget.
//!
//! This closes F04 (payload content), F05 (request count), and F06
//! (Retry-After semantics) at the wire level, not just at the unit level.
//!
//! The test starts a local TCP listener that mimics an OpenAI-compatible
//! chat completions endpoint, then dispatches a work unit against it.

use dc_broker::dispatch::{DispatchRequest, DispatchResult, dispatch_work_unit};
use dc_broker::endpoint::{EndpointConfig, ModelEndpoint};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn read_http_request(stream: &std::net::TcpStream) -> (String, String) {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap();

    let mut headers = std::collections::HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }

    let content_length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).unwrap();
    (
        request_line.trim().to_string(),
        String::from_utf8_lossy(&body).to_string(),
    )
}

fn write_http_response(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    extra_headers: &[(&str, &str)],
) {
    let mut resp = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        status,
        content_type,
        body.len()
    );
    for (k, v) in extra_headers {
        resp.push_str(&format!("{}: {}\r\n", k, v));
    }
    resp.push_str("\r\n");
    resp.push_str(body);
    stream.write_all(resp.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn success_body(content: &str) -> String {
    serde_json::json!({
        "choices": [{
            "message": {"role": "assistant", "content": content},
            "finish_reason": "stop"
        }],
        "model": "test-model",
        "usage": {"total_tokens": 42}
    })
    .to_string()
}

/// F04: The user_prompt sent to the model must contain the meaningful task
/// description, not Rust Debug format of the OutputContract struct.
///
/// This test verifies the payload that actually crosses the wire.
#[tokio::test]
async fn wire_payload_contains_task_not_debug_format() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let captured_body = Arc::new(std::sync::Mutex::new(String::new()));
    let captured = captured_body.clone();

    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let (_req_line, body) = read_http_request(&stream);
        *captured.lock().unwrap() = body;
        let resp = success_body("Task completed successfully");
        write_http_response(&mut stream, 200, "application/json", &resp, &[]);
    });

    let config = EndpointConfig {
        name: "test".to_string(),
        base_url: format!("http://127.0.0.1:{}", port),
        model: "test-model".to_string(),
        timeout_ms: 5000,
        max_retries: 0,
    };
    let endpoint = ModelEndpoint::new(config);

    let request = DispatchRequest {
        dispatch_id: "test-dispatch-001".to_string(),
        work_unit_spec: r#"{"unit_id":"test-unit","kind":"Research"}"#.to_string(),
        system_prompt: "You are a helpful assistant.".to_string(),
        user_prompt:
            "Task: Research the given topic thoroughly\nWork unit ID: test-unit-1\nKind: Research"
                .to_string(),
        max_tokens: 1024,
        temperature: 0.7,
    };

    let result = dispatch_work_unit(&endpoint, &request).await.unwrap();
    server.join().unwrap();

    // Verify success
    match &result {
        DispatchResult::Success(resp) => {
            assert_eq!(resp.content, "Task completed successfully");
            assert_eq!(resp.model, "test-model");
            assert_eq!(resp.tokens_used, Some(42));
        }
        other => panic!("expected Success, got {:?}", other),
    }

    // F04: verify the payload that crossed the wire
    let body = captured_body.lock().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let user_msg = parsed["messages"][1]["content"].as_str().unwrap();

    assert!(
        user_msg.contains("Research the given topic thoroughly"),
        "user prompt must contain meaningful task description, got: {}",
        user_msg
    );
    assert!(
        !user_msg.contains("OutputContract {"),
        "user prompt must NOT contain Debug format, got: {}",
        user_msg
    );
    assert!(
        !user_msg.contains("export_name:"),
        "user prompt must NOT contain struct field names, got: {}",
        user_msg
    );
    // stream must be explicitly false (OmniRoute SSE workaround)
    assert_eq!(parsed["stream"], serde_json::Value::Bool(false));
}

/// F05: On success, exactly ONE HTTP request is made (no executor-level retry).
/// The broker itself only retries on failure, never on success.
#[tokio::test]
async fn wire_success_makes_exactly_one_request() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let request_count = Arc::new(AtomicUsize::new(0));
    let count = request_count.clone();

    let server = std::thread::spawn(move || {
        // Accept exactly one connection; the test would hang if broker retried
        let (mut stream, _) = listener.accept().unwrap();
        count.fetch_add(1, Ordering::SeqCst);
        let (_req_line, _body) = read_http_request(&stream);
        let resp = success_body("ok");
        write_http_response(&mut stream, 200, "application/json", &resp, &[]);
        // Intentionally do NOT accept again — if broker retries, it gets connection refused
    });

    let config = EndpointConfig {
        name: "test".to_string(),
        base_url: format!("http://127.0.0.1:{}", port),
        model: "test-model".to_string(),
        timeout_ms: 3000,
        max_retries: 3, // even with retries allowed, success = 1 request
    };
    let endpoint = ModelEndpoint::new(config);

    let request = DispatchRequest {
        dispatch_id: "test-dispatch-002".to_string(),
        work_unit_spec: "{}".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: 100,
        temperature: 0.0,
    };

    let result = dispatch_work_unit(&endpoint, &request).await.unwrap();
    server.join().unwrap();

    assert!(matches!(result, DispatchResult::Success(_)));
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        1,
        "success must make exactly 1 request, not retry"
    );
}

/// F06: On 429 with Retry-After: 1, the broker retries after ~1 second
/// (not 1 millisecond, which was the old bug).
///
/// The server returns 429+Retry-After on first request, then 200 on second.
/// We measure elapsed time and count requests.
#[tokio::test]
async fn wire_retry_after_seconds_semantics() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let request_count = Arc::new(AtomicUsize::new(0));
    let count = request_count.clone();

    let server = std::thread::spawn(move || {
        // First request: 429 with Retry-After: 1 (1 second)
        let (mut stream, _) = listener.accept().unwrap();
        count.fetch_add(1, Ordering::SeqCst);
        let (_r, _b) = read_http_request(&stream);
        write_http_response(
            &mut stream,
            429,
            "application/json",
            r#"{"error":{"message":"rate limited","type":"rate_limit_error"}}"#,
            &[("Retry-After", "1")],
        );

        // Second request: 200 success
        let (mut stream, _) = listener.accept().unwrap();
        count.fetch_add(1, Ordering::SeqCst);
        let (_r, _b) = read_http_request(&stream);
        let resp = success_body("ok after retry");
        write_http_response(&mut stream, 200, "application/json", &resp, &[]);
    });

    let config = EndpointConfig {
        name: "test".to_string(),
        base_url: format!("http://127.0.0.1:{}", port),
        model: "test-model".to_string(),
        timeout_ms: 10000,
        max_retries: 3,
    };
    let endpoint = ModelEndpoint::new(config);

    let request = DispatchRequest {
        dispatch_id: "test-dispatch-003".to_string(),
        work_unit_spec: "{}".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: 100,
        temperature: 0.0,
    };

    let start = std::time::Instant::now();
    let result = dispatch_work_unit(&endpoint, &request).await.unwrap();
    let elapsed = start.elapsed();
    server.join().unwrap();

    assert!(matches!(result, DispatchResult::Success(_)));
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        2,
        "must retry once after 429"
    );

    // F06: Retry-After: 1 means 1 SECOND, not 1 millisecond.
    // Elapsed must be >= 1s. If the old bug (from_millis) were present, elapsed
    // would be ~1ms and this assertion would fail.
    assert!(
        elapsed >= std::time::Duration::from_millis(900),
        "Retry-After: 1 must wait ~1 second, not 1 millisecond; elapsed={:?}",
        elapsed
    );
    // Sanity: also not absurdly long (test shouldn't take 10s)
    assert!(
        elapsed < std::time::Duration::from_secs(8),
        "retry should not take absurdly long; elapsed={:?}",
        elapsed
    );
}

/// F05: On persistent failure, the broker makes exactly max_retries+1 requests
/// (initial + retries), then gives up. No hidden multiplication.
#[tokio::test]
async fn wire_failure_respects_retry_budget() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let request_count = Arc::new(AtomicUsize::new(0));
    let count = request_count.clone();
    let max_retries = 2usize;

    let server = std::thread::spawn(move || {
        // Accept max_retries+1 connections, each returns 500
        for _ in 0..=max_retries {
            if let Ok((mut stream, _)) = listener.accept() {
                count.fetch_add(1, Ordering::SeqCst);
                let (_r, _b) = read_http_request(&stream);
                write_http_response(
                    &mut stream,
                    500,
                    "application/json",
                    r#"{"error":{"message":"internal","type":"server_error"}}"#,
                    &[],
                );
            }
        }
    });

    let config = EndpointConfig {
        name: "test".to_string(),
        base_url: format!("http://127.0.0.1:{}", port),
        model: "test-model".to_string(),
        timeout_ms: 5000,
        max_retries: max_retries as u32,
    };
    let endpoint = ModelEndpoint::new(config);

    let request = DispatchRequest {
        dispatch_id: "test-dispatch-004".to_string(),
        work_unit_spec: "{}".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: 100,
        temperature: 0.0,
    };

    let result = dispatch_work_unit(&endpoint, &request).await;
    server.join().unwrap();

    // Broker returns Err(Unreachable) after exhausting retries
    assert!(result.is_err(), "expected error after persistent 500");

    // F05: exactly max_retries+1 requests — no executor multiplication
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        max_retries + 1,
        "must make exactly max_retries+1 requests, no hidden multiplication"
    );
}
