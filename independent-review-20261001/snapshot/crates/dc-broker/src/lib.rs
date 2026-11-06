//! C-09 bounded model dispatch broker. One responsibility: own registered model
//! connections, translate work-unit specs into model-compatible payloads, dispatch
//! to model endpoints, and return responses. No authority decisions, no persistence,
//! no scheduling.
//!
//! The broker owns the network path and model credentials. The worker (C-11) never
//! touches the network — it receives an assignment through the dispatcher trait and
//! returns results. The broker does NOT decide what to do — it executes what the
//! domain layer (C-01) has already decided.
//!
//! Current implementation: real HTTP dispatch via reqwest with retry, timeout,
//! circuit breaker, and rate limiting. Compatible with OmniRoute and OpenAI APIs.
//! Future: streaming, retry, circuit breaker, rate limiting.

pub mod dispatch;
pub mod endpoint;
pub mod error;

pub use dispatch::{DispatchRequest, DispatchResult, ModelResponse};
pub use endpoint::{EndpointConfig, ModelEndpoint};
pub use error::BrokerError;
