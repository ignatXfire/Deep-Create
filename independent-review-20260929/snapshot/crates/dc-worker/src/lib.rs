//! C-11 bounded work-unit executor. One responsibility: take a work-unit
//! spec, execute it through an injected dispatcher, and return the result.
//! No authority, no persistence, no scheduling, no direct broker dependency.
//!
//! The worker receives a `WorkDispatcher` trait object from the caller (dc-core
//! or test harness). It never touches network, secrets, store, or user files.

pub mod error;
pub mod execute;

pub use error::WorkerError;
pub use execute::{execute_batch, execute_work_unit, ExecuteRequest, ExecuteResult, WorkDispatcher, WorkResponse};
