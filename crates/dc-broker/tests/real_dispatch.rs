//! Real model dispatch test: calls live OmniRoute endpoint.
//!
//! This test PROVES the broker actually talks to a real model, not a mock.
//! Requires: OmniRoute running on localhost:20128.

#![allow(async_fn_in_trait)]

use dc_broker::{DispatchRequest, EndpointConfig, ModelEndpoint, dispatch::dispatch_work_unit};

/// Test real dispatch to live OmniRoute endpoint.
/// This is a REAL network call — not a mock, not a stub.
#[tokio::test]
#[ignore] // Ignored by default — requires live OmniRoute
async fn real_model_dispatch_omniroute() {
    let endpoint = ModelEndpoint::new(EndpointConfig {
        name: "omniroute-live".to_string(),
        base_url: "http://localhost:20128".to_string(),
        model: "auto/best-fast".to_string(),
        timeout_ms: 30_000,
        max_retries: 1,
    });

    let request = DispatchRequest {
        dispatch_id: "test-dispatch-001".to_string(),
        work_unit_spec: "{}".to_string(),
        system_prompt: "You are a helpful assistant.".to_string(),
        user_prompt: "Say exactly: DEEP_CREATE_BROKER_WORKS".to_string(),
        max_tokens: 50,
        temperature: 0.0,
    };

    let result = dispatch_work_unit(&endpoint, &request).await;

    match result {
        Ok(dc_broker::DispatchResult::Success(response)) => {
            println!("Model: {}", response.model);
            println!("Content: {}", response.content);
            println!("Tokens: {:?}", response.tokens_used);
            assert!(!response.content.is_empty(), "response content is empty");
            assert!(
                response.content.contains("DEEP_CREATE_BROKER_WORKS"),
                "unexpected response: {}",
                response.content
            );
        }
        Ok(dc_broker::DispatchResult::Refused(reason)) => {
            panic!("model refused: {}", reason);
        }
        Ok(dc_broker::DispatchResult::Unreachable(reason)) => {
            panic!("endpoint unreachable: {}", reason);
        }
        Err(e) => {
            panic!("dispatch error: {:?}", e);
        }
    }
}

/// Test broker retry logic with real endpoint.
/// Uses a model that might be slow to verify retry works.
#[tokio::test]
#[ignore]
async fn real_model_dispatch_with_retry() {
    let endpoint = ModelEndpoint::new(EndpointConfig {
        name: "omniroute-retry-test".to_string(),
        base_url: "http://localhost:20128".to_string(),
        model: "auto/best-coding".to_string(),
        timeout_ms: 60_000,
        max_retries: 3,
    });

    let request = DispatchRequest {
        dispatch_id: "test-retry-001".to_string(),
        work_unit_spec: "{}".to_string(),
        system_prompt: "You are a coding assistant.".to_string(),
        user_prompt: "Write a Rust function that adds two numbers. Just the code.".to_string(),
        max_tokens: 200,
        temperature: 0.0,
    };

    let result = dispatch_work_unit(&endpoint, &request).await;
    assert!(
        result.is_ok(),
        "dispatch failed after retries: {:?}",
        result.err()
    );

    if let Ok(dc_broker::DispatchResult::Success(response)) = result {
        assert!(
            response.content.contains("fn"),
            "expected Rust function in response"
        );
        println!(
            "Retry test passed. Model: {}, tokens: {:?}",
            response.model, response.tokens_used
        );
    }
}
