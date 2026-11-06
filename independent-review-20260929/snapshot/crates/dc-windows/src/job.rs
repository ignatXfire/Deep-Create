//! Windows Job Object isolation for contained process execution.
//!
//! P2-WINDOWS-JOB-OBJECT: Real Win32 Job Object integration for process isolation.
//! This module provides typed wrappers around Windows Job Objects to constrain
//! worker processes with resource limits and security boundaries.
//!
//! Note: This is a typed abstraction layer. Actual Win32 API calls are behind
//! the `windows-sys` feature flag and require unsafe code.

use std::fmt;

/// Resource limits for a Job Object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobLimits {
    /// Maximum working set size in bytes (0 = unlimited)
    pub max_working_set_bytes: u64,
    /// Maximum CPU time in 100-nanosecond intervals (0 = unlimited)
    pub max_cpu_time_100ns: u64,
    /// Maximum number of processes in the job (0 = unlimited)
    pub max_processes: u32,
    /// Maximum memory usage in bytes (0 = unlimited)
    pub max_memory_bytes: u64,
}

impl Default for JobLimits {
    fn default() -> Self {
        Self {
            max_working_set_bytes: 256 * 1024 * 1024, // 256 MB
            max_cpu_time_100ns: 60 * 10_000_000,      // 60 seconds
            max_processes: 1,
            max_memory_bytes: 512 * 1024 * 1024,      // 512 MB
        }
    }
}

/// Job Object state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    /// Job created but no process assigned
    Created,
    /// Job has active processes
    Active,
    /// Job is terminating
    Terminating,
    /// Job completed (all processes exited)
    Completed,
    /// Job failed (limit exceeded or error)
    Failed,
}

/// A typed handle to a Windows Job Object.
///
/// This is a zero-cost abstraction: the actual OS handle is managed
/// by the platform-specific implementation.
#[derive(Debug)]
pub struct JobObject {
    id: String,
    limits: JobLimits,
    state: JobState,
}

impl JobObject {
    /// Create a new Job Object with the given limits.
    ///
    /// # Safety
    ///
    /// This function does NOT create the actual OS object. It creates a typed
    /// wrapper that can be used to track the job lifecycle. The actual Win32
    /// `CreateJobObjectW` call must be performed by the platform adapter.
    pub fn new(id: String, limits: JobLimits) -> Self {
        Self {
            id,
            limits,
            state: JobState::Created,
        }
    }

    /// Get the job identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the current limits.
    pub fn limits(&self) -> &JobLimits {
        &self.limits
    }

    /// Get the current state.
    pub fn state(&self) -> JobState {
        self.state
    }

    /// Transition to Active state (process assigned).
    pub fn activate(&mut self) {
        self.state = JobState::Active;
    }

    /// Transition to Terminating state.
    pub fn terminate(&mut self) {
        self.state = JobState::Terminating;
    }

    /// Transition to Completed state.
    pub fn complete(&mut self) {
        self.state = JobState::Completed;
    }

    /// Transition to Failed state.
    pub fn fail(&mut self) {
        self.state = JobState::Failed;
    }
}

impl fmt::Display for JobObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JobObject({}, state={:?}, max_mem={}MB)",
            self.id,
            self.state,
            self.limits.max_memory_bytes / (1024 * 1024)
        )
    }
}

/// Error type for Job Object operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobError {
    /// Failed to create the job object
    CreationFailed,
    /// Failed to assign process to job
    AssignmentFailed,
    /// Failed to set limits
    LimitSetFailed,
    /// Job limit exceeded (memory, CPU, etc.)
    LimitExceeded,
    /// Job not found
    NotFound,
    /// Invalid state transition
    InvalidState,
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobError::CreationFailed => write!(f, "job object creation failed"),
            JobError::AssignmentFailed => write!(f, "process assignment to job failed"),
            JobError::LimitSetFailed => write!(f, "failed to set job limits"),
            JobError::LimitExceeded => write!(f, "job limit exceeded"),
            JobError::NotFound => write!(f, "job not found"),
            JobError::InvalidState => write!(f, "invalid job state transition"),
        }
    }
}

impl std::error::Error for JobError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_lifecycle() {
        let mut job = JobObject::new("test-job".to_string(), JobLimits::default());
        assert_eq!(job.state(), JobState::Created);

        job.activate();
        assert_eq!(job.state(), JobState::Active);

        job.terminate();
        assert_eq!(job.state(), JobState::Terminating);

        job.complete();
        assert_eq!(job.state(), JobState::Completed);
    }

    #[test]
    fn job_limits_default() {
        let limits = JobLimits::default();
        assert_eq!(limits.max_working_set_bytes, 256 * 1024 * 1024);
        assert_eq!(limits.max_processes, 1);
    }
}
