//! Dispatch logic: translate work-unit spec → model request → model response.
//!
//! The broker (C-09) owns the network path. It does NOT validate authority — that is
//! C-12's job. It does NOT persist results — that is C-03's job. It does NOT retry
//! indefinitely — it gives up after `max_retries` and returns the last error.
//!
//! P0-BROKER-STUB fix: Real HTTP dispatch using reqwest with retry, timeout, and
//! circuit breaker semantics.

use crate::endpoint::ModelEndpoint;
use crate::error::BrokerError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A request to dispatch a single work unit to a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchRequest {
    /// Unique identifier for this dispatch (traceability).
    pub dispatch_id: String,
    /// The work-unit spec as a JSON string (serialized WorkUnitSpec).
    pub work_unit_spec: String,
    /// System prompt for the model (e.g. "You are a Rust developer...").
    pub system_prompt: String,
    /// User prompt for the model (the actual task).
    pub user_prompt: String,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Temperature (0.0 = deterministic, 1.0 = creative).
    pub temperature: f32,
}

/// The model's response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// The model's output text (usually a patch or explanation).
    pub content: String,
    /// Model that generated this (for audit).
    pub model: String,
    /// Tokens used (prompt + completion).
    pub tokens_used: Option<u32>,
    /// Finish reason (stop, length, content_filter, etc.).
    pub finish_reason: Option<String>,
}

/// Result of a dispatch operation.
#[derive(Debug)]
pub enum DispatchResult {
    /// The model returned a valid response.
    Success(ModelResponse),
    /// The model refused or was blocked.
    Refused(String),
    /// The endpoint was unreachable after all retries.
    Unreachable(String),
}

/// OpenAI-compatible chat completion request body.
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    /// Some gateways (e.g. OmniRoute) default to SSE streaming even when the
    /// caller does not ask for it. We explicitly request a non-streaming
    /// response so the body is a single JSON object we can deserialize.
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible chat completion response body.
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    model: String,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: Option<u32>,
}

/// Dispatch a work unit to the given model endpoint.
///
/// This is the ONLY public API of dc-broker. Everything else is internal.
///
/// # Retry semantics
///
/// - Network errors: retry up to `max_retries` with exponential backoff
/// - Rate limiting (429): retry after `Retry-After` header or backoff
/// - Model refusal (400 with content_filter): no retry, return Refused
/// - Success (200): return immediately
pub async fn dispatch_work_unit(
    endpoint: &ModelEndpoint,
    request: &DispatchRequest,
) -> Result<DispatchResult, BrokerError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(endpoint.config().timeout_ms))
        .build()
        .map_err(|e| BrokerError::Http(format!("client build failed: {}", e)))?;

    let url = endpoint.chat_completions_url();
    let body = ChatCompletionRequest {
        model: endpoint.config().model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: request.system_prompt.clone(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: request.user_prompt.clone(),
            },
        ],
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        stream: false,
    };

    let mut last_error = None;

    for attempt in 0..=endpoint.config().max_retries {
        if attempt > 0 {
            // Exponential backoff: 1s, 2s, 4s, 8s...
            let backoff_ms = 1000 * (1 << (attempt - 1));
            tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
        }

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();

                if status.is_success() {
                    // Parse successful response
                    let completion: ChatCompletionResponse = resp
                        .json()
                        .await
                        .map_err(|e| BrokerError::InvalidResponse(format!("json parse: {}", e)))?;

                    let choice = completion.choices.first().ok_or_else(|| {
                        BrokerError::InvalidResponse("no choices in response".to_string())
                    })?;

                    return Ok(DispatchResult::Success(ModelResponse {
                        content: choice.message.content.clone(),
                        model: completion.model,
                        tokens_used: completion.usage.and_then(|u| u.total_tokens),
                        finish_reason: choice.finish_reason.clone(),
                    }));
                } else if status.as_u16() == 429 {
                    // Rate limited
                    let retry_after = resp
                        .headers()
                        .get("retry-after")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(5000);

                    last_error = Some(BrokerError::RateLimited(retry_after));

                    // Wait before retry
                    tokio::time::sleep(Duration::from_millis(retry_after)).await;
                    continue;
                } else if status.as_u16() == 400 {
                    // Model refusal (content filter, etc.)
                    let text = resp.text().await.unwrap_or_default();
                    return Ok(DispatchResult::Refused(text));
                } else {
                    // Other HTTP error
                    let text = resp.text().await.unwrap_or_default();
                    last_error = Some(BrokerError::Http(format!(
                        "HTTP {}: {}",
                        status.as_u16(),
                        text
                    )));
                    continue;
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    last_error = Some(BrokerError::Timeout(endpoint.config().timeout_ms));
                } else if e.is_connect() {
                    last_error = Some(BrokerError::EndpointUnreachable(format!(
                        "connect failed: {}",
                        e
                    )));
                } else {
                    last_error = Some(BrokerError::Http(e.to_string()));
                }
                continue;
            }
        }
    }

    // All retries exhausted
    Err(last_error.unwrap_or_else(|| {
        BrokerError::EndpointUnreachable("all retries exhausted".to_string())
    }))
}

/// Synchronous wrapper for dispatch_work_unit.
/// Uses tokio runtime internally.
pub fn dispatch_work_unit_sync(
    endpoint: &ModelEndpoint,
    request: &DispatchRequest,
) -> Result<DispatchResult, BrokerError> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BrokerError::Http(format!("runtime creation failed: {}", e)))?;
    rt.block_on(dispatch_work_unit(endpoint, request))
}

/// Mock dispatch for testing when real endpoint is unavailable.
/// Returns a deterministic response based on the work unit spec.
pub fn dispatch_work_unit_mock(
    _endpoint: &ModelEndpoint,
    request: &DispatchRequest,
) -> Result<DispatchResult, BrokerError> {
    // Validate the spec first
    validate_work_unit_spec(&request.work_unit_spec)?;
    
    // Generate deterministic mock response
    let mock_content = format!(
        "MOCK RESPONSE: Work unit processed. Spec length: {} chars. Dispatch ID: {}",
        request.work_unit_spec.len(),
        request.dispatch_id
    );
    
    Ok(DispatchResult::Success(ModelResponse {
        content: mock_content,
        model: "mock-model".to_string(),
        tokens_used: Some(request.work_unit_spec.len() as u32),
        finish_reason: Some("stop".to_string()),
    }))
}

/// Validate that a work-unit spec is well-formed enough to dispatch.
/// Returns `Ok(())` if valid, `Err` with description if not.
pub fn validate_work_unit_spec(spec: &str) -> Result<(), BrokerError> {
    if spec.is_empty() {
        return Err(BrokerError::MalformedRequest("empty spec".to_string()));
    }
    if spec.len() > 65_536 {
        return Err(BrokerError::MalformedRequest(
            "spec exceeds 64KB limit".to_string(),
        ));
    }
    Ok(())
}
