//! Basic metrics counters for production observability.
//!
//! Global, lock-free counters (AtomicU64) for key operational events.
//! These are intentionally simple — no external metrics backend, no
//! cardinality explosion. Snapshot them via [`Metrics::snapshot`] for
//! health checks and diagnostics.
//!
//! Counters are process-local and reset on restart. For persistent
//! metrics, export snapshots to the store (future work).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Global metrics registry. Access via [`Metrics::global`].
pub struct Metrics {
    /// Process start time (for uptime calculation).
    started_at: Instant,
    /// Total objectives accepted since process start.
    pub objectives_processed: AtomicU64,
    /// Total graphs accepted since process start.
    pub graphs_accepted: AtomicU64,
    /// Total work-unit attempts started since process start.
    pub units_dispatched: AtomicU64,
    /// Total units that completed successfully (result recorded + verified).
    pub units_completed: AtomicU64,
    /// Total verification runs that passed.
    pub verifications_passed: AtomicU64,
    /// Total verification runs that failed.
    pub verifications_failed: AtomicU64,
    /// Total errors encountered (command rejections, broker failures, etc.).
    pub errors_total: AtomicU64,
}

impl Metrics {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
            objectives_processed: AtomicU64::new(0),
            graphs_accepted: AtomicU64::new(0),
            units_dispatched: AtomicU64::new(0),
            units_completed: AtomicU64::new(0),
            verifications_passed: AtomicU64::new(0),
            verifications_failed: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
        }
    }

    /// Access the global metrics registry.
    pub fn global() -> &'static Metrics {
        static INSTANCE: std::sync::OnceLock<Metrics> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(Metrics::new)
    }

    /// Increment the objectives-processed counter.
    pub fn inc_objectives(&self) {
        self.objectives_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the graphs-accepted counter.
    pub fn inc_graphs_accepted(&self) {
        self.graphs_accepted.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the units-dispatched counter.
    pub fn inc_units_dispatched(&self) {
        self.units_dispatched.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the units-completed counter.
    pub fn inc_units_completed(&self) {
        self.units_completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the verifications-passed counter.
    pub fn inc_verifications_passed(&self) {
        self.verifications_passed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the verifications-failed counter.
    pub fn inc_verifications_failed(&self) {
        self.verifications_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the errors counter.
    pub fn inc_errors(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Uptime in seconds since process start.
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Snapshot all counters as a JSON value (for health checks / logging).
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "uptime_secs": self.uptime_secs(),
            "objectives_processed": self.objectives_processed.load(Ordering::Relaxed),
            "graphs_accepted": self.graphs_accepted.load(Ordering::Relaxed),
            "units_dispatched": self.units_dispatched.load(Ordering::Relaxed),
            "units_completed": self.units_completed.load(Ordering::Relaxed),
            "verifications_passed": self.verifications_passed.load(Ordering::Relaxed),
            "verifications_failed": self.verifications_failed.load(Ordering::Relaxed),
            "errors_total": self.errors_total.load(Ordering::Relaxed),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_increment() {
        let m = Metrics::new();
        m.inc_objectives();
        m.inc_objectives();
        m.inc_units_dispatched();
        m.inc_errors();
        assert_eq!(m.objectives_processed.load(Ordering::Relaxed), 2);
        assert_eq!(m.units_dispatched.load(Ordering::Relaxed), 1);
        assert_eq!(m.errors_total.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn snapshot_has_all_fields() {
        let m = Metrics::new();
        let snap = m.snapshot();
        assert!(snap.get("uptime_secs").is_some());
        assert!(snap.get("objectives_processed").is_some());
        assert!(snap.get("units_completed").is_some());
        assert!(snap.get("errors_total").is_some());
    }
}
