//! Extended 30-minute stability soak test.
//!
//! Runs continuous objective→plan→execute→verify→persist cycles for 30 minutes.
//! Samples RSS every 60 seconds. Tracks throughput per 5-minute window.
//! Writes time-series data to EXTENDED_STABILITY_EVIDENCE.md.
//!
//! Run: cargo test -p dc-core --test extended_stability_soak -- --nocapture

use dc_core::{
    CoreCommand, CoreDispatcher, CorePersistence, CoreState, PersistedCoreState, SchedulingContext,
    VerificationOutcome, compile_prompt_bundle, handle_command, run_machine_verification,
    select_ready_units,
};
use dc_domain::ids::{AuthorityEpoch, EntityId, RowVersion, RunGeneration};
use dc_domain::permission::{
    ActionKind, CanonicalResource, CapabilityLease, LeaseExpiry, LeaseScope, LeaseStatus,
};
use dc_domain::resource::ExplicitLimit;
use dc_domain::work::{
    DependencyKey, DependencyKind, DependencySnapshot, GraphCandidate, ObjectiveContract,
    OutputContract, StopCondition, UnitRef, UnitStage, VerificationEvidence, VerificationLevel,
    VerificationRecord, VersionedDependency, WorkBounds, WorkKind, WorkScope, WorkUnitSpec,
};
use dc_store::MemoryStore;
use dc_worker::WorkDispatcher;
use std::collections::BTreeSet;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};

// ============================================================================
// Configuration
// ============================================================================

const SOAK_DURATION_SECS: u64 = 1800; // 30 minutes
const MAX_RSS_GROWTH_BYTES: u64 = 1024 * 1024 * 1024; // 1 GiB
const MAX_SLOWDOWN_RATIO: f64 = 3.0;
const MAX_CYCLE_TIME_MS: u128 = 10_000;

// ============================================================================
// Test infrastructure (reused from stability_test.rs)
// ============================================================================

fn test_id(n: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | n)
        .expect("synthetic RFC 9562 UUIDv7")
}

fn test_scope() -> WorkScope {
    WorkScope {
        session_id: test_id(1),
        work_id: test_id(2),
        objective_id: test_id(3),
        objective_revision: NonZeroU32::new(1).unwrap(),
    }
}

fn test_unit_spec(unit_id: EntityId) -> WorkUnitSpec {
    WorkUnitSpec {
        unit: UnitRef {
            unit_id,
            revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
        },
        scope: test_scope(),
        parent: None,
        kind: WorkKind::Research,
        expected_output: OutputContract {
            export_name: "soak".into(),
            schema_version: NonZeroU32::new(1).unwrap(),
            fields: vec![],
        },
        obligation_ids: vec![],
        acceptance_criteria: vec![],
        bounds: WorkBounds {
            max_attempts: NonZeroU32::new(3).unwrap(),
            max_steps: NonZeroU32::new(10).unwrap(),
            stop_conditions: vec![StopCondition::OutputProduced],
        },
        exploration: None,
        stage: UnitStage::Proposed,
        accepted_result: None,
        row_version: RowVersion::new(1).unwrap(),
    }
}

fn test_lease() -> CapabilityLease {
    CapabilityLease {
        lease_id: test_id(900),
        owner_event_id: test_id(901),
        installation_id: test_id(902),
        scope: LeaseScope {
            work_id: test_id(903),
            unit_id: None,
            action_kind: ActionKind::CallModelConnection,
            resource: CanonicalResource::ModelConnection {
                connection_id: test_id(905),
                revision: RowVersion::new(1).unwrap(),
            },
            recipient: None,
            data_classes: BTreeSet::from([dc_domain::permission::DataClass::Public]),
            max_calls: ExplicitLimit::Finite(100),
            max_bytes: ExplicitLimit::Finite(1_000_000),
            cost_limits: Default::default(),
        },
        not_before_utc_millis: 0,
        expiry: LeaseExpiry::Never,
        parent_lease_id: None,
        delegation_depth: 0,
        max_delegation_depth: 3,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        run_generation: RunGeneration::new(1).unwrap(),
        support_fences: vec![],
        confirmed_conditions_hash: [0u8; 32],
        issued_dependency_snapshot_id: test_id(904),
        lease_version: RowVersion::new(1).unwrap(),
        status: LeaseStatus::Active,
    }
}

struct SoakDispatcher;

impl WorkDispatcher for SoakDispatcher {
    fn dispatch(
        &self,
        request: &dc_worker::ExecuteRequest,
    ) -> Result<dc_worker::ExecuteResult, dc_worker::WorkerError> {
        Ok(dc_worker::ExecuteResult::Success(dc_worker::WorkResponse {
            content: format!("soak: {}", request.execution_id),
            model: "soak-test-model".to_string(),
            tokens_used: Some(10),
            finish_reason: Some("stop".to_string()),
        }))
    }
}

// ============================================================================
// Memory monitoring
// ============================================================================

#[allow(unsafe_code)]
fn current_rss_bytes() -> u64 {
    #[cfg(target_os = "windows")]
    {
        use std::mem::size_of;
        use windows_sys::Win32::System::ProcessStatus::{
            GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
        };
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        unsafe {
            let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
            counters.cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            let process = GetCurrentProcess();
            if GetProcessMemoryInfo(
                process,
                &mut counters,
                size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ) != 0
            {
                return counters.WorkingSetSize as u64;
            }
        }
        0
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let kb: u64 = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return kb * 1024;
                }
            }
        }
        0
    }
}

// ============================================================================
// Statistics helpers — bounded memory, O(1) per-cycle cost
// ============================================================================
//
// The previous implementation stored every cycle time in a Vec<u128>. Over a
// 30-minute soak at ~1000 cycles/sec this grew to 1.78M entries × 16 B ≈ 27 MB
// of RSS growth, and the periodic `median()` call did a full `to_vec() + sort()`
// on that vector every 500 cycles — O(n² log n) total work, which is what
// actually caused the observed throughput collapse.
//
// We now keep:
//   * scalar aggregates (count, sum, min, max) — exact;
//   * a log2-bucketed histogram for approximate percentiles — bounded;
//   * two small reservoir samples (first-quarter and last-quarter) for an
//     unbiased slowdown-ratio estimate — bounded.
//
// All structures are O(1) space and O(1) update.

const HISTOGRAM_BUCKETS: usize = 64; // log2 buckets cover 0..2^63 ms
const RESERVOIR_SIZE: usize = 4096;

struct CycleStats {
    count: u64,
    sum_ms: u128,
    min_ms: u128,
    max_ms: u128,
    /// log2 histogram; bucket i counts values in [2^i, 2^(i+1))
    hist: [u64; HISTOGRAM_BUCKETS],
}

impl Default for CycleStats {
    fn default() -> Self {
        Self {
            count: 0,
            sum_ms: 0,
            min_ms: 0,
            max_ms: 0,
            hist: [0; HISTOGRAM_BUCKETS],
        }
    }
}

impl CycleStats {
    fn record(&mut self, ms: u128) {
        if self.count == 0 {
            self.min_ms = ms;
            self.max_ms = ms;
        } else {
            if ms < self.min_ms {
                self.min_ms = ms;
            }
            if ms > self.max_ms {
                self.max_ms = ms;
            }
        }
        self.count += 1;
        self.sum_ms += ms;
        let bucket = bucket_index(ms);
        self.hist[bucket] += 1;
    }

    fn mean_ms(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_ms as f64 / self.count as f64
        }
    }

    /// Approximate percentile from the histogram. Returns the lower bound of
    /// the bucket that crosses the requested rank.
    fn percentile_ms(&self, p: f64) -> u128 {
        if self.count == 0 {
            return 0;
        }
        let rank = ((self.count as f64) * p / 100.0).ceil() as u64;
        let mut seen = 0u64;
        for (i, &c) in self.hist.iter().enumerate() {
            seen += c;
            if seen >= rank.max(1) {
                // Return bucket midpoint estimate
                let lo: u128 = if i == 0 { 0 } else { 1u128 << (i - 1) };
                let hi: u128 = 1u128 << i;
                return (lo + hi) / 2;
            }
        }
        self.max_ms
    }
}

fn bucket_index(ms: u128) -> usize {
    if ms == 0 {
        0
    } else {
        (127 - ms.leading_zeros() as usize).min(HISTOGRAM_BUCKETS - 1)
    }
}

/// Fixed-size reservoir sample (Algorithm R) for unbiased percentile/median
/// estimation over a stream — O(RESERVOIR_SIZE) space, O(1) amortised update.
struct Reservoir {
    samples: Vec<u128>,
    seen: u64,
    /// Simple xorshift PRNG; deterministic seed is fine for a test harness.
    rng: u64,
}

impl Reservoir {
    fn new() -> Self {
        Self {
            samples: Vec::with_capacity(RESERVOIR_SIZE),
            seen: 0,
            rng: 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_rand(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }

    fn record(&mut self, ms: u128) {
        self.seen += 1;
        if self.samples.len() < RESERVOIR_SIZE {
            self.samples.push(ms);
        } else {
            let j = (self.next_rand() % self.seen) as usize;
            if j < RESERVOIR_SIZE {
                self.samples[j] = ms;
            }
        }
    }

    fn median(&self) -> u128 {
        if self.samples.is_empty() {
            return 0;
        }
        let mut v = self.samples.clone();
        v.sort_unstable();
        v[v.len() / 2]
    }
}

// ============================================================================
// Extended soak test
// ============================================================================

#[test]
#[ignore = "30-minute soak test; run manually with --ignored"]
fn extended_stability_soak_30min() {
    let start = Instant::now();
    let deadline = start + Duration::from_secs(SOAK_DURATION_SECS);

    let mut stats = CycleStats::default();
    let mut reservoir_first_quarter = Reservoir::new();
    let mut reservoir_last_quarter = Reservoir::new();
    let mut rss_samples: Vec<(u64, u64)> = Vec::new(); // (elapsed_secs, rss_bytes)
    let baseline_rss = current_rss_bytes();
    rss_samples.push((0, baseline_rss));

    let mut cycle_count: u64 = 0;
    let mut persist_counter: u64 = 0;
    let mut last_rss_sample = Instant::now();
    let mut errors: Vec<String> = Vec::new();

    // Throughput windows: 5-minute buckets
    let mut window_cycles: Vec<u64> = Vec::new();
    let mut current_window_cycles: u64 = 0;
    let mut window_start = Instant::now();

    println!("=== EXTENDED STABILITY SOAK TEST (30 minutes) ===");
    println!(
        "Duration: {}s | Baseline RSS: {} bytes",
        SOAK_DURATION_SECS, baseline_rss
    );

    while Instant::now() < deadline {
        let cycle_start = Instant::now();
        cycle_count += 1;
        current_window_cycles += 1;

        // ---------------------------------------------------------------
        // Phase 1: Fresh state + persistence
        // ---------------------------------------------------------------
        let store = MemoryStore::new();
        let mut persistence = CorePersistence::new(store);
        if let Err(e) = persistence.initialize(std::path::Path::new(":memory:")) {
            errors.push(format!("cycle {}: persistence init: {}", cycle_count, e));
            continue;
        }

        let mut state = CoreState::empty();

        // ---------------------------------------------------------------
        // Phase 2: Accept objective
        // ---------------------------------------------------------------
        let objective = ObjectiveContract {
            scope: test_scope(),
            statement: format!("Soak objective #{}", cycle_count),
            owner_event_ids: vec![],
            obligations: vec![],
            completion_criteria: vec![test_id(100)],
            row_version: RowVersion::new(1).unwrap(),
        };
        if let Err(e) = handle_command(&mut state, CoreCommand::AcceptObjective(objective)) {
            errors.push(format!("cycle {}: accept objective: {}", cycle_count, e));
            continue;
        }

        // ---------------------------------------------------------------
        // Phase 3: Plan graph
        // ---------------------------------------------------------------
        let unit_id = test_id(200 + cycle_count as u128);
        let unit = test_unit_spec(unit_id);
        let graph = GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![unit.clone()],
            dependencies: vec![],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };
        let graph_clone = graph.clone();
        if let Err(e) = handle_command(&mut state, CoreCommand::AcceptGraph(graph)) {
            errors.push(format!("cycle {}: accept graph: {}", cycle_count, e));
            continue;
        }

        // ---------------------------------------------------------------
        // Phase 4: Schedule
        // ---------------------------------------------------------------
        let sched_ctx = SchedulingContext {
            leases: vec![test_lease()],
            max_concurrent: 5,
            running_attempts: 0,
        };
        let ready = match select_ready_units(&graph_clone, &sched_ctx) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("cycle {}: schedule: {}", cycle_count, e));
                continue;
            }
        };
        if ready.ready.len() != 1 {
            errors.push(format!(
                "cycle {}: expected 1 ready unit, got {}",
                cycle_count,
                ready.ready.len()
            ));
            continue;
        }

        let unit_ref = ready.ready[0];
        if let Err(e) = handle_command(&mut state, CoreCommand::StartAttempt(unit_ref)) {
            errors.push(format!("cycle {}: start attempt: {}", cycle_count, e));
            continue;
        }

        // ---------------------------------------------------------------
        // Phase 5: Compile context
        // ---------------------------------------------------------------
        let unit_spec = test_unit_spec(unit_ref.unit_id);
        let context_plan = dc_core::ContextPlan {
            memory_scope: vec![],
            allowed_data_classes: vec![dc_domain::permission::DataClass::Public],
            max_context_tokens: 4096,
        };
        let lease = test_lease();
        let context = match compile_prompt_bundle(&unit_spec, &context_plan, &lease) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("cycle {}: compile context: {}", cycle_count, e));
                continue;
            }
        };

        // ---------------------------------------------------------------
        // Phase 6: Dispatch
        // ---------------------------------------------------------------
        let dispatcher = CoreDispatcher::new(SoakDispatcher, "soak-worker".to_string());
        let result = match dispatcher.execute_unit(
            &unit_spec,
            &context.system_prompt,
            &context.user_prompt,
        ) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("cycle {}: dispatch: {}", cycle_count, e));
                continue;
            }
        };
        let response = match result {
            dc_worker::ExecuteResult::Success(r) => r,
            _ => {
                errors.push(format!("cycle {}: expected success", cycle_count));
                continue;
            }
        };

        // ---------------------------------------------------------------
        // Phase 7: Record result
        // ---------------------------------------------------------------
        let snapshot = DependencySnapshot {
            snapshot_id: test_id(300),
            scope: test_scope(),
            unit: unit_ref,
            read_set: vec![],
            policy_versions: vec![],
        };
        let policy = VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::VerificationCriteria,
                entity_id: test_id(301),
            },
            expected_revision: RowVersion::new(1).unwrap(),
            fingerprint: None,
        };
        let result_packet = match dispatcher.response_to_result(
            &unit_spec,
            &response,
            test_id(302),
            snapshot,
            policy,
        ) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("cycle {}: response_to_result: {}", cycle_count, e));
                continue;
            }
        };
        let result_packet_clone = result_packet.clone();

        if let Err(e) = handle_command(
            &mut state,
            CoreCommand::RecordResult {
                unit: unit_ref,
                result: result_packet,
            },
        ) {
            errors.push(format!("cycle {}: record result: {}", cycle_count, e));
            continue;
        }

        // ---------------------------------------------------------------
        // Phase 8: Verify
        // ---------------------------------------------------------------
        let verification = VerificationRecord {
            verification_id: test_id(400),
            result_id: test_id(302),
            result_fingerprint: [0u8; 32],
            unit: unit_ref,
            snapshot_id: test_id(300),
            verification_policy: VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::VerificationCriteria,
                    entity_id: test_id(401),
                },
                expected_revision: RowVersion::new(1).unwrap(),
                fingerprint: None,
            },
            verifier_attempt_id: test_id(402),
            verdict: dc_domain::work::VerificationVerdict::Pass,
            findings: vec![],
            evidence: VerificationEvidence {
                verification_plan_id: test_id(400),
                method_revision_id: test_id(403),
                scope: test_scope(),
                work_kind: WorkKind::Research,
                criterion_ids: vec![],
                level: VerificationLevel::MachineOnly,
                required_level: VerificationLevel::MachineOnly,
                machine_findings_id: test_id(404),
                policy: VersionedDependency {
                    key: DependencyKey {
                        kind: DependencyKind::Policy,
                        entity_id: test_id(405),
                    },
                    expected_revision: RowVersion::new(1).unwrap(),
                    fingerprint: None,
                },
                qualification: None,
                independence_assessment_id: None,
                protected_admission_evidence_id: None,
            },
        };
        if let Err(e) = handle_command(&mut state, CoreCommand::RecordVerification(verification))
        {
            errors.push(format!("cycle {}: record verification: {}", cycle_count, e));
            continue;
        }

        // ---------------------------------------------------------------
        // Phase 9: Apply verified result
        // ---------------------------------------------------------------
        if let Err(e) = handle_command(&mut state, CoreCommand::ApplyVerifiedResult(unit_ref)) {
            errors.push(format!("cycle {}: apply verified: {}", cycle_count, e));
            continue;
        }

        // ---------------------------------------------------------------
        // Phase 10: Persist
        // ---------------------------------------------------------------
        persist_counter += 1;
        let persist_id = test_id(500 + persist_counter as u128);
        let persisted = PersistedCoreState {
            current_objective: Some(test_id(3)),
            current_graph_version: 1,
            active_units: vec![],
            completed_units: vec![unit_ref.unit_id],
            objective_json: None,
            graph_json: None,
        };
        if let Err(e) = persistence.persist_state(
            persist_id,
            &persisted,
            dc_store::ExpectedVersion::MustNotExist,
        ) {
            errors.push(format!("cycle {}: persist: {}", cycle_count, e));
            continue;
        }

        let loaded = match persistence.load_state(&persist_id) {
            Ok(Some(l)) => l,
            Ok(None) => {
                errors.push(format!("cycle {}: load returned None", cycle_count));
                continue;
            }
            Err(e) => {
                errors.push(format!("cycle {}: load: {}", cycle_count, e));
                continue;
            }
        };
        if loaded.data.completed_units != vec![unit_ref.unit_id] {
            errors.push(format!("cycle {}: persistence mismatch", cycle_count));
            continue;
        }

        // ---------------------------------------------------------------
        // Phase 11: Machine verification
        // ---------------------------------------------------------------
        let outcomes = match run_machine_verification(&unit_spec, &result_packet_clone, test_id(600))
        {
            Ok(o) => o,
            Err(e) => {
                errors.push(format!("cycle {}: machine verification: {}", cycle_count, e));
                continue;
            }
        };
        match outcomes {
            VerificationOutcome::Passed { record } => {
                assert_eq!(record.unit, unit_ref);
            }
            _ => {
                errors.push(format!("cycle {}: verification not passed", cycle_count));
                continue;
            }
        }

        // ---------------------------------------------------------------
        // Cycle complete — record metrics (bounded, O(1))
        // ---------------------------------------------------------------
        let cycle_elapsed = cycle_start.elapsed();
        let cycle_ms = cycle_elapsed.as_millis();
        stats.record(cycle_ms);

        // Bucket into first/last quarter reservoirs based on elapsed wall time.
        // (We can't know cycle indices in advance, so use wall-clock quarters.)
        let elapsed = start.elapsed();
        let total = Duration::from_secs(SOAK_DURATION_SECS);
        if elapsed < total / 4 {
            reservoir_first_quarter.record(cycle_ms);
        } else if elapsed >= total * 3 / 4 {
            reservoir_last_quarter.record(cycle_ms);
        }

        // Sample RSS every 60 seconds
        if last_rss_sample.elapsed() >= Duration::from_secs(60) {
            let elapsed_secs = start.elapsed().as_secs();
            let rss = current_rss_bytes();
            rss_samples.push((elapsed_secs, rss));
            last_rss_sample = Instant::now();
        }

        // Track throughput per 5-minute window
        if window_start.elapsed() >= Duration::from_secs(300) {
            window_cycles.push(current_window_cycles);
            current_window_cycles = 0;
            window_start = Instant::now();
        }

        // Progress log every 500 cycles — use running mean (O(1)), not a sort.
        if cycle_count % 500 == 0 {
            let elapsed = start.elapsed().as_secs();
            let rss = current_rss_bytes();
            println!(
                "  cycle {:>6} | elapsed {:>4}s | RSS {:>8} KB | mean cycle {:>6.2}ms | last {:>4}ms | errors: {}",
                cycle_count,
                elapsed,
                rss / 1024,
                stats.mean_ms(),
                cycle_ms,
                errors.len()
            );
        }
    }

    // Capture final window
    if current_window_cycles > 0 {
        window_cycles.push(current_window_cycles);
    }

    let total_elapsed = start.elapsed();
    let final_rss = current_rss_bytes();
    let final_elapsed_secs = total_elapsed.as_secs();
    rss_samples.push((final_elapsed_secs, final_rss));

    // ============================================================================
    // Analysis
    // ============================================================================

    println!("\n=== EXTENDED SOAK TEST COMPLETE ===");
    println!("Total cycles: {}", cycle_count);
    println!("Total time: {:.1}s", total_elapsed.as_secs_f64());
    println!(
        "Overall cycles/sec: {:.2}",
        cycle_count as f64 / total_elapsed.as_secs_f64()
    );

    let max_rss = rss_samples.iter().map(|(_, r)| *r).max().unwrap_or(0);
    let rss_growth = final_rss.saturating_sub(baseline_rss);
    println!("Baseline RSS: {} KB", baseline_rss / 1024);
    println!("Final RSS:    {} KB", final_rss / 1024);
    println!("Peak RSS:     {} KB", max_rss / 1024);
    println!("RSS growth:   {} KB ({} bytes)", rss_growth / 1024, rss_growth);

    let median_cycle = stats.percentile_ms(50.0);
    let max_cycle = stats.max_ms;
    let min_cycle = stats.min_ms;
    let p95 = stats.percentile_ms(95.0);
    let p99 = stats.percentile_ms(99.0);

    println!(
        "Cycle times (ms): min={} median={} p95={} p99={} max={}",
        min_cycle, median_cycle, p95, p99, max_cycle
    );

    let first_quarter_median = reservoir_first_quarter.median();
    let last_quarter_median = reservoir_last_quarter.median();
    let slowdown_ratio = if first_quarter_median > 0 {
        last_quarter_median as f64 / first_quarter_median as f64
    } else {
        1.0
    };
    println!("First-quarter median: {}ms", first_quarter_median);
    println!("Last-quarter median:  {}ms", last_quarter_median);
    println!("Slowdown ratio:       {:.2}x", slowdown_ratio);

    println!("Errors encountered: {}", errors.len());
    if !errors.is_empty() {
        for (i, e) in errors.iter().take(20).enumerate() {
            println!("  error {}: {}", i + 1, e);
        }
        if errors.len() > 20 {
            println!("  ... and {} more", errors.len() - 20);
        }
    }

    // ============================================================================
    // Assertions
    // ============================================================================

    assert!(
        rss_growth <= MAX_RSS_GROWTH_BYTES,
        "MEMORY LEAK: RSS grew {} KB (max {} KB)",
        rss_growth / 1024,
        MAX_RSS_GROWTH_BYTES / 1024
    );

    assert!(
        max_cycle <= MAX_CYCLE_TIME_MS,
        "STALL DETECTED: slowest cycle {}ms (max {}ms)",
        max_cycle,
        MAX_CYCLE_TIME_MS
    );

    assert!(
        slowdown_ratio <= MAX_SLOWDOWN_RATIO,
        "PERFORMANCE DEGRADATION: last quarter {:.2}x slower than first quarter (max {}x)",
        slowdown_ratio,
        MAX_SLOWDOWN_RATIO
    );

    assert!(
        cycle_count >= 1000,
        "INSUFFICIENT COVERAGE: only {} cycles in {}s (expected >= 1000)",
        cycle_count,
        total_elapsed.as_secs()
    );

    assert!(
        errors.is_empty(),
        "ERRORS OCCURRED: {} errors during soak",
        errors.len()
    );

    println!("\n✅ ALL EXTENDED STABILITY ASSERTIONS PASSED");
    println!(
        "   {} cycles over {:.1}s with no leaks, stalls, degradation, or errors",
        cycle_count,
        total_elapsed.as_secs_f64()
    );

    // ============================================================================
    // Write evidence file
    // ============================================================================

    let evidence_path = std::path::Path::new("EXTENDED_STABILITY_EVIDENCE.md");
    let mut evidence = String::new();
    evidence.push_str("# Extended Stability Soak Test Evidence\n\n");
    evidence.push_str(&format!("- **Date**: {}\n", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));
    evidence.push_str(&format!("- **Duration**: {} seconds ({:.1} minutes)\n", total_elapsed.as_secs(), total_elapsed.as_secs_f64() / 60.0));
    evidence.push_str(&format!("- **Total cycles**: {}\n", cycle_count));
    evidence.push_str(&format!("- **Overall throughput**: {:.2} cycles/sec\n", cycle_count as f64 / total_elapsed.as_secs_f64()));
    evidence.push_str(&format!("- **Errors**: {}\n\n", errors.len()));

    evidence.push_str("## Memory (RSS) Time Series\n\n");
    evidence.push_str("| Elapsed (s) | RSS (KB) | RSS (MB) |\n");
    evidence.push_str("|-------------|----------|----------|\n");
    for (elapsed, rss) in &rss_samples {
        evidence.push_str(&format!("| {} | {} | {:.2} |\n", elapsed, rss / 1024, *rss as f64 / 1024.0 / 1024.0));
    }
    evidence.push_str("\n");

    evidence.push_str("## Throughput by 5-Minute Window\n\n");
    evidence.push_str("| Window | Cycles | Cycles/sec |\n");
    evidence.push_str("|--------|--------|------------|\n");
    for (i, &count) in window_cycles.iter().enumerate() {
        let cps = count as f64 / 300.0;
        evidence.push_str(&format!("| {}-{} min | {} | {:.2} |\n", i * 5, (i + 1) * 5, count, cps));
    }
    evidence.push_str("\n");

    evidence.push_str("## Cycle Time Statistics\n\n");
    evidence.push_str(&format!("- Min: {} ms\n", min_cycle));
    evidence.push_str(&format!("- Median: {} ms\n", median_cycle));
    evidence.push_str(&format!("- P95: {} ms\n", p95));
    evidence.push_str(&format!("- P99: {} ms\n", p99));
    evidence.push_str(&format!("- Max: {} ms\n", max_cycle));
    evidence.push_str(&format!("- First-quarter median: {} ms\n", first_quarter_median));
    evidence.push_str(&format!("- Last-quarter median: {} ms\n", last_quarter_median));
    evidence.push_str(&format!("- Slowdown ratio: {:.2}x\n\n", slowdown_ratio));

    evidence.push_str("## Memory Analysis\n\n");
    evidence.push_str(&format!("- Baseline RSS: {} KB\n", baseline_rss / 1024));
    evidence.push_str(&format!("- Final RSS: {} KB\n", final_rss / 1024));
    evidence.push_str(&format!("- Peak RSS: {} KB\n", max_rss / 1024));
    evidence.push_str(&format!("- RSS growth: {} KB ({} bytes)\n\n", rss_growth / 1024, rss_growth));

    evidence.push_str("## Verdict\n\n");
    let throughput_stable = slowdown_ratio <= 1.5;
    let rss_plateau = rss_growth < 100 * 1024 * 1024; // < 100 MB growth
    let verdict = if errors.is_empty() && throughput_stable && rss_plateau && rss_growth <= MAX_RSS_GROWTH_BYTES {
        "PASS — System stable over 30-minute soak"
    } else if !errors.is_empty() {
        "FAIL — Errors encountered"
    } else if !throughput_stable {
        "FAIL — Throughput degraded"
    } else {
        "FAIL — Memory growth exceeds threshold"
    };
    evidence.push_str(&format!("**{}**\n", verdict));
    evidence.push_str(&format!("- Throughput stable: {} (slowdown ratio {:.2}x)\n", throughput_stable, slowdown_ratio));
    evidence.push_str(&format!("- RSS plateauing: {} (growth {} KB)\n", rss_plateau, rss_growth / 1024));
    evidence.push_str(&format!("- Errors: {}\n", errors.len()));

    std::fs::write(evidence_path, evidence).expect("write evidence file");
    println!("\nEvidence written to: {}", evidence_path.display());
}
