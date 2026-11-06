//! Memory-leak detection test under sustained load.
//!
//! Strategy:
//! 1. A counting global allocator tracks live bytes + outstanding allocation count.
//! 2. On Windows, GetProcessMemoryInfo tracks WorkingSetSize before/after cycles.
//! 3. Sustained workload: build objectives → plan graphs → run machine verifications → drop.
//! 4. Assert allocator state returns exactly to baseline; RSS within tolerance.
//! 5. Evidence written to MEMORY_LEAK_EVIDENCE.md (test working directory).
//!
//! SAFETY-POLICY: `unsafe_code` is denied workspace-wide. This test file is an
//! additional audited unsafe boundary, justified block by block:
//! - `unsafe impl GlobalAlloc` is required by the language to register a
//!   counting allocator; it forwards to `System` and only adds atomic counters.
//! - The Win32 `GetProcessMemoryInfo` call is read-only against the current
//!   process pseudo-handle; no handles are created or leaked.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};

use dc_core::planner::plan_graph_from_objective;
use dc_core::verification::{VerificationOutcome, run_machine_verification};
use dc_domain::ids::{EntityId, RowVersion, UnitRevision};
use dc_domain::work::{
    DependencyKey, DependencyKind, DependencySnapshot, ObjectiveContract, Obligation,
    ObligationKind, ObligationStatus, ResultPacket, ResultStage, VerificationMethod,
    VersionedDependency, WorkScope,
};

// ---------------------------------------------------------------------------
// Counting allocator
// ---------------------------------------------------------------------------

struct CountingAllocator;

static LIVE_ALLOCS: AtomicIsize = AtomicIsize::new(0);
static LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            LIVE_ALLOCS.fetch_add(1, Ordering::SeqCst);
            LIVE_BYTES.fetch_add(layout.size() as isize, Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE_ALLOCS.fetch_sub(1, Ordering::SeqCst);
        LIVE_BYTES.fetch_sub(layout.size() as isize, Ordering::SeqCst);
    }
}

#[global_allocator]
static A: CountingAllocator = CountingAllocator;

fn live_bytes() -> isize {
    LIVE_BYTES.load(Ordering::SeqCst)
}

fn live_allocs() -> isize {
    LIVE_ALLOCS.load(Ordering::SeqCst)
}

// ---------------------------------------------------------------------------
// Windows RSS tracking (GetProcessMemoryInfo)
// ---------------------------------------------------------------------------

#[cfg(windows)]
mod win_rss {
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    pub fn working_set_size() -> u64 {
        unsafe {
            let process: HANDLE = GetCurrentProcess();
            let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
            counters.cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            let ok = GetProcessMemoryInfo(
                process,
                &mut counters,
                size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            );
            assert!(ok != 0, "GetProcessMemoryInfo failed");
            counters.WorkingSetSize as u64
        }
    }
}

#[cfg(not(windows))]
mod win_rss {
    pub fn working_set_size() -> u64 {
        0
    }
}

use win_rss::working_set_size;

// ---------------------------------------------------------------------------
// Domain-object builders (shapes mirror dc-core verification unit tests)
// ---------------------------------------------------------------------------

/// Build a valid UUIDv7 EntityId for tests.
fn vid(n: u128) -> EntityId {
    let v = (0x7u128 << 76) | (0x2u128 << 62) | (n & ((1u128 << 62) - 1));
    EntityId::from_u128(v).expect("valid v7 test id")
}

fn scope(n: u128) -> WorkScope {
    WorkScope {
        session_id: vid(n * 100 + 1),
        work_id: vid(n * 100 + 2),
        objective_id: vid(n * 100 + 3),
        objective_revision: UnitRevision::new(1).unwrap(),
    }
}

fn objective(n: u128) -> ObjectiveContract {
    ObjectiveContract {
        scope: scope(n),
        statement: format!("Sustained-leak-detection objective number {}", n),
        owner_event_ids: vec![],
        obligations: vec![Obligation {
            obligation_id: vid(n * 100 + 4),
            objective_id: vid(n * 100 + 3),
            parent_id: None,
            kind: ObligationKind::Required,
            statement: format!("Deliverable obligation for cycle {}", n),
            verification_method: VerificationMethod::Deterministic,
            status: ObligationStatus::Open,
            waiver_owner_event_id: None,
            row_version: RowVersion::new(1).unwrap(),
        }],
        completion_criteria: vec![vid(n * 100 + 5)],
        row_version: RowVersion::new(1).unwrap(),
    }
}

fn result_packet(graph: &dc_domain::work::GraphCandidate, n: u128) -> ResultPacket {
    let unit = &graph.units[0];
    ResultPacket {
        result_id: vid(n * 100 + 6),
        attempt_id: vid(n * 100 + 7),
        scope: unit.scope,
        unit: unit.unit,
        snapshot: DependencySnapshot {
            snapshot_id: vid(n * 100 + 8),
            scope: unit.scope,
            unit: unit.unit,
            read_set: vec![],
            policy_versions: vec![],
        },
        // Non-zero fingerprint so machine verification passes and exercises
        // the full record-building path (the heaviest allocation path).
        fingerprint: [(n % 251 + 1) as u8; 32],
        verification_policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::VerificationCriteria,
                entity_id: vid(n * 100 + 9),
            },
            expected_revision: RowVersion::new(1).unwrap(),
            fingerprint: None,
        },
        used_links: vec![],
        stage: ResultStage::Submitted,
        row_version: RowVersion::new(1).unwrap(),
    }
}

/// One full workload cycle: objective → graph → result → verification → drop.
fn run_cycle(n: u128) {
    let obj = objective(n);
    let graph = plan_graph_from_objective(&obj).expect("plan graph");
    let result = result_packet(&graph, n);
    let outcome = run_machine_verification(&graph.units[0], &result, vid(n * 100 + 10))
        .expect("machine verification runs");
    let _passed = matches!(outcome, VerificationOutcome::Passed { .. });
    // Explicit drops; everything must be reclaimed.
    drop(outcome);
    drop(result);
    drop(graph);
    drop(obj);
}

// ---------------------------------------------------------------------------
// The test
// ---------------------------------------------------------------------------

const CYCLES: u128 = 500;
/// Windows allocators retain freed pages in the working set; allow drift.
const RSS_TOLERANCE_BYTES: u64 = 8 * 1024 * 1024; // 8 MiB

#[test]
fn memory_returns_to_baseline_after_sustained_workload() {
    // Warm-up: stabilise lazy allocations, internal caches, thread-locals.
    for i in 0..10 {
        run_cycle(900_000 + i);
    }

    // Allocate the checkpoint log BEFORE baseline so its growth doesn't
    // pollute workload measurements; pre-reserve so push_str never reallocs
    // during the measured region.
    let mut log = String::with_capacity(64 * 1024);
    log.push_str("cycle | live_bytes (delta) | live_allocs (delta) | RSS bytes (delta)\n");

    let baseline_bytes = live_bytes();
    let baseline_allocs = live_allocs();
    let baseline_rss = working_set_size();

    let mut worst_byte_drift: isize = 0;
    let mut worst_alloc_drift: isize = 0;

    for cycle in 1..=CYCLES {
        run_cycle(cycle);

        if cycle % 50 == 0 {
            let b = live_bytes();
            let a = live_allocs();
            let r = working_set_size();
            let bd = b - baseline_bytes;
            let ad = a - baseline_allocs;
            let rd = r as i128 - baseline_rss as i128;
            worst_byte_drift = worst_byte_drift.max(bd);
            worst_alloc_drift = worst_alloc_drift.max(ad);
            log.push_str(&format!(
                "{:5} | {:12} ({:+5}) | {:11} ({:+4}) | {:12} ({:+8})\n",
                cycle, b, bd, a, ad, r, rd
            ));
        }
    }

    // Final measurement after all drops. Allocator counters must be EXACTLY
    // back at baseline — Rust drops are deterministic, no GC involved.
    //
    // IMPORTANT: the `log` String is part of the baseline (allocated before it),
    // so it must STILL BE ALIVE here. Measure BEFORE allocating the evidence
    // string so test-side allocations don't pollute the final reading.
    let final_bytes = live_bytes();
    let final_allocs = live_allocs();
    let final_rss = working_set_size();
    let byte_delta = final_bytes - baseline_bytes;
    let alloc_delta = final_allocs - baseline_allocs;
    let rss_delta = final_rss as i128 - baseline_rss as i128;

    let leak_found = byte_delta != 0 || alloc_delta != 0;

    let evidence = format!(
        "# Memory Leak Detection Evidence\n\n\
        **Test**: `memory_returns_to_baseline_after_sustained_workload` (dc-core)\n\
        **Method**: counting global allocator (live bytes + outstanding allocs) + \
        Windows `GetProcessMemoryInfo` WorkingSetSize\n\n\
        ## Configuration\n\
        - Cycles (objective → graph → result → machine verification → drop): {}\n\
        - Warm-up cycles: 10\n\
        - RSS tolerance: {} bytes ({} MiB)\n\n\
        ## Baseline (after warm-up)\n\
        - Live bytes: {}\n\
        - Outstanding allocations: {}\n\
        - Working set: {} bytes\n\n\
        ## Final (after all drops)\n\
        - Live bytes: {} (delta {:+})\n\
        - Outstanding allocations: {} (delta {:+})\n\
        - Working set: {} bytes (delta {:+})\n\n\
        ## Drift observed during run\n\
        - Worst live-byte drift at a checkpoint: {:+}\n\
        - Worst alloc-count drift at a checkpoint: {:+}\n\n\
        ## Verdict\n\
        - Allocator leak (bytes or allocs not back to baseline): **{}**\n\
        - RSS within tolerance: **{}**\n\n\
        ## Checkpoint log\n\
        ```\n{}```\n",
        CYCLES,
        RSS_TOLERANCE_BYTES,
        RSS_TOLERANCE_BYTES / (1024 * 1024),
        baseline_bytes,
        baseline_allocs,
        baseline_rss,
        final_bytes,
        byte_delta,
        final_allocs,
        alloc_delta,
        final_rss,
        rss_delta,
        worst_byte_drift,
        worst_alloc_drift,
        if leak_found {
            "LEAK DETECTED"
        } else {
            "no leak"
        },
        if rss_delta.unsigned_abs() <= RSS_TOLERANCE_BYTES as u128 {
            "yes"
        } else {
            "NO — RSS exceeded tolerance"
        },
        log
    );

    std::fs::write("MEMORY_LEAK_EVIDENCE.md", evidence).expect("write MEMORY_LEAK_EVIDENCE.md");

    assert_eq!(
        final_bytes, baseline_bytes,
        "allocator live bytes did not return to baseline (delta {byte_delta:+}); \
         see MEMORY_LEAK_EVIDENCE.md"
    );
    assert_eq!(
        final_allocs, baseline_allocs,
        "outstanding allocation count did not return to baseline (delta {alloc_delta:+}); \
         see MEMORY_LEAK_EVIDENCE.md"
    );
    assert!(
        rss_delta.unsigned_abs() <= RSS_TOLERANCE_BYTES as u128,
        "working set drifted {rss_delta:+} bytes, beyond tolerance {RSS_TOLERANCE_BYTES}; \
         see MEMORY_LEAK_EVIDENCE.md"
    );
}
