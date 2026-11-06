//! Long-running stability test: continuous operation for bounded time.
//!
//! Simulates hours of operation compressed into ~5 minutes by running
//! repeated objective→plan→execute→verify→persist cycles and monitoring
//! for:
//!   - Memory growth (RSS must stay bounded)
//!   - File handle / connection leaks (persistence must not fail)
//!   - Performance degradation (cycle time must not grow linearly)
//!
//! Run: cargo test -p dc-core --test stability_test -- --nocapture

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

/// Total wall-clock time to run the stability loop.
const STABILITY_DURATION_SECS: u64 = 300; // 5 minutes

/// Maximum allowed RSS growth over the entire run (bytes).
/// 512 MiB is generous for a test; real leaks show up as unbounded growth.
const MAX_RSS_GROWTH_BYTES: u64 = 512 * 1024 * 1024;

/// Maximum allowed ratio between the slowest cycle and the median cycle.
/// If cycles get slower over time (leak, fragmentation), this ratio explodes.
const MAX_SLOWDOWN_RATIO: f64 = 3.0;

/// Maximum allowed absolute cycle time (ms). Any cycle slower than this
/// indicates a stall or deadlock.
const MAX_CYCLE_TIME_MS: u128 = 10_000; // 10 seconds

// ============================================================================
// Test infrastructure (reused from end_to_end.rs patterns)
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
            export_name: "stability".into(),
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

struct StabilityDispatcher;

impl WorkDispatcher for StabilityDispatcher {
    fn dispatch(
        &self,
        request: &dc_worker::ExecuteRequest,
    ) -> Result<dc_worker::ExecuteResult, dc_worker::WorkerError> {
        Ok(dc_worker::ExecuteResult::Success(dc_worker::WorkResponse {
            content: format!("stability: {}", request.execution_id),
            model: "stability-test-model".to_string(),
            tokens_used: Some(10),
            finish_reason: Some("stop".to_string()),
        }))
    }
}

// ============================================================================
// Memory monitoring (Windows via windows-sys)
// ============================================================================

#[allow(unsafe_code)]
fn current_rss_bytes() -> u64 {
    #[cfg(target_os = "windows")]
    {
        // Use GetProcessMemoryInfo via windows-sys for accurate RSS
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
// Stability test
// ============================================================================

#[test]
#[ignore = "5-minute stability test; run manually with --ignored"]
fn long_running_stability() {
    let start = Instant::now();
    let deadline = start + Duration::from_secs(STABILITY_DURATION_SECS);

    let mut cycle_times_ms: Vec<u128> = Vec::new();
    let mut rss_samples: Vec<(Instant, u64)> = Vec::new();
    let baseline_rss = current_rss_bytes();
    rss_samples.push((start, baseline_rss));

    let mut cycle_count: u64 = 0;
    let mut persist_counter: u64 = 0;

    println!("=== Long-Running Stability Test ===");
    println!(
        "Duration: {}s | Baseline RSS: {} bytes",
        STABILITY_DURATION_SECS, baseline_rss
    );

    while Instant::now() < deadline {
        let cycle_start = Instant::now();
        cycle_count += 1;

        // ---------------------------------------------------------------
        // Phase 1: Fresh state + persistence (simulates fresh objective)
        // ---------------------------------------------------------------
        let store = MemoryStore::new();
        let mut persistence = CorePersistence::new(store);
        persistence
            .initialize(std::path::Path::new(":memory:"))
            .expect("persistence init must not leak");

        let mut state = CoreState::empty();

        // ---------------------------------------------------------------
        // Phase 2: Accept objective
        // ---------------------------------------------------------------
        let objective = ObjectiveContract {
            scope: test_scope(),
            statement: format!("Stability objective #{}", cycle_count),
            owner_event_ids: vec![],
            obligations: vec![],
            completion_criteria: vec![test_id(100)],
            row_version: RowVersion::new(1).unwrap(),
        };
        handle_command(&mut state, CoreCommand::AcceptObjective(objective))
            .expect("accept objective");

        // ---------------------------------------------------------------
        // Phase 3: Plan graph (1 unit per cycle keeps it fast)
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
        handle_command(&mut state, CoreCommand::AcceptGraph(graph)).expect("accept graph");

        // ---------------------------------------------------------------
        // Phase 4: Schedule
        // ---------------------------------------------------------------
        let sched_ctx = SchedulingContext {
            leases: vec![test_lease()],
            max_concurrent: 5,
            running_attempts: 0,
        };
        let ready = select_ready_units(&graph_clone, &sched_ctx).expect("schedule");
        assert_eq!(ready.ready.len(), 1, "exactly one unit must be ready");

        let unit_ref = ready.ready[0];
        handle_command(&mut state, CoreCommand::StartAttempt(unit_ref)).expect("start attempt");

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
        let context =
            compile_prompt_bundle(&unit_spec, &context_plan, &lease).expect("compile context");

        // ---------------------------------------------------------------
        // Phase 6: Dispatch
        // ---------------------------------------------------------------
        let dispatcher = CoreDispatcher::new(StabilityDispatcher, "stability-worker".to_string());
        let result = dispatcher
            .execute_unit(&unit_spec, &context.system_prompt, &context.user_prompt)
            .expect("dispatch");
        let response = match result {
            dc_worker::ExecuteResult::Success(r) => r,
            _ => panic!("expected success"),
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
        let result_packet = dispatcher
            .response_to_result(&unit_spec, &response, test_id(302), snapshot, policy)
            .expect("response_to_result");
        let result_packet_clone = result_packet.clone();

        handle_command(
            &mut state,
            CoreCommand::RecordResult {
                unit: unit_ref,
                result: result_packet,
            },
        )
        .expect("record result");

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
        handle_command(&mut state, CoreCommand::RecordVerification(verification))
            .expect("record verification");

        // ---------------------------------------------------------------
        // Phase 9: Apply verified result
        // ---------------------------------------------------------------
        handle_command(&mut state, CoreCommand::ApplyVerifiedResult(unit_ref))
            .expect("apply verified result");

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
        persistence
            .persist_state(
                persist_id,
                &persisted,
                dc_store::ExpectedVersion::MustNotExist,
            )
            .expect("persist must not leak");

        // Round-trip verify
        let loaded = persistence
            .load_state(&persist_id)
            .expect("load must succeed")
            .expect("must exist");
        assert_eq!(loaded.data.completed_units, vec![unit_ref.unit_id]);

        // ---------------------------------------------------------------
        // Phase 11: Machine verification (idempotent check)
        // ---------------------------------------------------------------
        let outcomes = run_machine_verification(&unit_spec, &result_packet_clone, test_id(600))
            .expect("machine verification");
        match outcomes {
            VerificationOutcome::Passed { record } => {
                assert_eq!(record.unit, unit_ref);
            }
            _ => panic!("expected verification pass"),
        }

        // ---------------------------------------------------------------
        // Cycle complete — record metrics
        // ---------------------------------------------------------------
        let cycle_elapsed = cycle_start.elapsed();
        cycle_times_ms.push(cycle_elapsed.as_millis());

        // Sample RSS every 10 cycles
        if cycle_count % 10 == 0 {
            let rss = current_rss_bytes();
            rss_samples.push((Instant::now(), rss));
        }

        // Progress log every 50 cycles
        if cycle_count % 50 == 0 {
            let elapsed = start.elapsed().as_secs();
            let rss = current_rss_bytes();
            let median = median(&cycle_times_ms);
            println!(
                "  cycle {:>5} | elapsed {:>3}s | RSS {:>8} KB | median cycle {:>4}ms | last {:>4}ms",
                cycle_count,
                elapsed,
                rss / 1024,
                median,
                cycle_elapsed.as_millis()
            );
        }
    }

    let total_elapsed = start.elapsed();
    let final_rss = current_rss_bytes();
    rss_samples.push((Instant::now(), final_rss));

    // ============================================================================
    // Analysis & Assertions
    // ============================================================================

    println!("\n=== Stability Test Complete ===");
    println!("Total cycles: {}", cycle_count);
    println!("Total time: {:.1}s", total_elapsed.as_secs_f64());
    println!(
        "Cycles/sec: {:.1}",
        cycle_count as f64 / total_elapsed.as_secs_f64()
    );

    // --- Memory analysis ---
    let max_rss = rss_samples.iter().map(|(_, r)| *r).max().unwrap_or(0);
    let rss_growth = final_rss.saturating_sub(baseline_rss);
    println!("Baseline RSS: {} KB", baseline_rss / 1024);
    println!("Final RSS:    {} KB", final_rss / 1024);
    println!("Peak RSS:     {} KB", max_rss / 1024);
    println!("RSS growth:   {} KB", rss_growth / 1024);

    // --- Performance analysis ---
    let median_cycle = median(&cycle_times_ms);
    let max_cycle = *cycle_times_ms.iter().max().unwrap_or(&0);
    let min_cycle = *cycle_times_ms.iter().min().unwrap_or(&0);
    let p95 = percentile(&cycle_times_ms, 95);
    let p99 = percentile(&cycle_times_ms, 99);

    println!(
        "Cycle times (ms): min={} median={} p95={} p99={} max={}",
        min_cycle, median_cycle, p95, p99, max_cycle
    );

    // Check for degradation: compare first-quartile median to last-quartile median
    let quarter = cycle_times_ms.len() / 4;
    let first_quarter_median = median(&cycle_times_ms[..quarter]);
    let last_quarter_median = median(&cycle_times_ms[3 * quarter..]);
    let slowdown_ratio = if first_quarter_median > 0 {
        last_quarter_median as f64 / first_quarter_median as f64
    } else {
        1.0
    };
    println!("First-quarter median: {}ms", first_quarter_median);
    println!("Last-quarter median:  {}ms", last_quarter_median);
    println!("Slowdown ratio:       {:.2}x", slowdown_ratio);

    // ============================================================================
    // Assertions
    // ============================================================================

    // 1. Memory growth must be bounded
    assert!(
        rss_growth <= MAX_RSS_GROWTH_BYTES,
        "MEMORY LEAK: RSS grew {} KB (max {} KB)",
        rss_growth / 1024,
        MAX_RSS_GROWTH_BYTES / 1024
    );

    // 2. No cycle should exceed absolute maximum
    assert!(
        max_cycle <= MAX_CYCLE_TIME_MS,
        "STALL DETECTED: slowest cycle {}ms (max {}ms)",
        max_cycle,
        MAX_CYCLE_TIME_MS
    );

    // 3. Performance must not degrade over time
    assert!(
        slowdown_ratio <= MAX_SLOWDOWN_RATIO,
        "PERFORMANCE DEGRADATION: last quarter {:.2}x slower than first quarter (max {}x)",
        slowdown_ratio,
        MAX_SLOWDOWN_RATIO
    );

    // 4. Must have completed a minimum number of cycles (sanity)
    assert!(
        cycle_count >= 50,
        "INSUFFICIENT COVERAGE: only {} cycles in {}s (expected >= 50)",
        cycle_count,
        total_elapsed.as_secs()
    );

    println!("\n✅ ALL STABILITY ASSERTIONS PASSED");
    println!(
        "   {} cycles over {:.1}s with no leaks, stalls, or degradation",
        cycle_count,
        total_elapsed.as_secs_f64()
    );
}

// ============================================================================
// Statistics helpers
// ============================================================================

fn median(values: &[u128]) -> u128 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted[sorted.len() / 2]
}

fn percentile(values: &[u128], p: usize) -> u128 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    let idx = (sorted.len() * p / 100).min(sorted.len() - 1);
    sorted[idx]
}
