//! Model endpoint configuration. Currently supports OmniRoute-compatible
//! HTTP endpoints (localhost:20128). Future: multiple providers, fallback.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for a single model endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// Human-readable name (e.g. "omniroute-local", "openrouter").
    pub name: String,
    /// Base URL (e.g. "http://localhost:20128").
    pub base_url: String,
    /// Model identifier (e.g. "dash/kimi-k3", "openai-codex/gpt-5.5").
    pub model: String,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Maximum retries before giving up.
    pub max_retries: u32,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            name: "omniroute-local".to_string(),
            base_url: "http://localhost:20128".to_string(),
            model: "dash/kimi-k3".to_string(),
            timeout_ms: 120_000,
            max_retries: 3,
        }
    }
}

impl fmt::Display for EndpointConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.model, self.base_url)
    }
}

/// A live model endpoint. Cheap to construct; the HTTP client is created
/// per-request to avoid stale connection pools.
#[derive(Debug, Clone)]
pub struct ModelEndpoint {
    config: EndpointConfig,
}

impl ModelEndpoint {
    pub fn new(config: EndpointConfig) -> Self {
        Self { config }
    }

    pub fn default_omniroute() -> Self {
        Self::new(EndpointConfig::default())
    }

    pub fn config(&self) -> &EndpointConfig {
        &self.config
    }

    /// Build the full URL for the chat-completions endpoint.
    pub fn chat_completions_url(&self) -> String {
        format!(
            "{}/v1/chat/completions",
            self.config.base_url.trim_end_matches('/')
        )
    }
}
