# Multi-Unit Execution Ordering — Test Evidence

**Date:** 2026-10-01  
**Scope:** Verify dependency-aware scheduling produces correct topological ordering for multi-unit work graphs in Deep Create.

---

## Context

- `dc-core/src/planner.rs` — `plan_graph_from_objective` produces a single-unit graph today (multi-unit graphs are an owner-review decision, not planner output).
- `dc-core/src/scheduler.rs` — `select_ready_units` is the gating layer: a unit is ready only when all its producers are in `UnitStage::Accepted`.
- `dc-desktop/src/executor.rs` — `execute_accepted_graph` currently executes `units.iter()` in insertion order; **no dependency-aware ordering exists in the executor**. This is a known gap (the scheduler is the intended gatekeeper, but the executor bypasses it).
- `dc-domain/src/work.rs` — `reject_cycle` uses Kahn's algorithm for cycle detection on untrusted graphs.

Because the executor lacks dependency ordering, the tests below validate the **scheduler contract** (`select_ready_units`) as the authoritative ordering mechanism, and simulate execution by repeatedly calling it while marking units `Accepted` after each topological level.

---

## Tests Added

File: `crates/dc-core/tests/multi_unit_ordering.rs` (integration test, 247 lines)

| Test | Graph shape | What it proves |
|------|-------------|----------------|
| `linear_chain_a_b_c_executes_in_order` | A → B → C | Topological levels are `[A], [B], [C]`; scheduler-driven execution order is A, B, C. |
| `diamond_dependency_resolution` | A → B, A → C, B+C → D | Levels are `[A], [B, C], [D]`; B and C are parallel-ready; D executes only after both B and C. |
| `select_ready_units_respects_pending_deps` | A(Proposed) ← B(Proposed) | B is blocked; only A is ready. |
| `select_ready_units_unblocks_after_acceptance` | A(Accepted) ← B(Proposed) | B becomes ready once A is Accepted. |

### Helper: Kahn's topological levels

The test file includes a `topological_levels` function (Kahn's algorithm, BFS layer-aware) that partitions units into dependency levels. This is used to assert the *expected* shape before validating scheduler behavior.

---

## Test Run Evidence

### Command

```bash
cd /d/Projects/DeepCreate
source .local/hermes-organization-20260916/build-env.sh
cargo test -p dc-core --test multi_unit_ordering
```

### Output

```
running 4 tests
test diamond_dependency_resolution ... ok
test linear_chain_a_b_c_executes_in_order ... ok
test select_ready_units_respects_pending_deps ... ok
test select_ready_units_unblocks_after_acceptance ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

### Full dc-core suite

```bash
cargo test -p dc-core
```

```
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.31s
...
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
...
```

**Total: 58 tests, 0 failures.**

---

## Files Modified

| File | Change |
|------|--------|
| `crates/dc-core/tests/multi_unit_ordering.rs` | **New** — 4 integration tests + helpers. |

No production code was modified; this is a pure test addition.

---

## Known Gap (for parent agent)

`dc-desktop/src/executor.rs` `execute_accepted_graph` does **not** use `select_ready_units` or any topological sort. It executes `units.iter()` in insertion order. If units arrive in an order that violates dependencies, the executor will run them out of order. The scheduler contract is correct, but the executor needs to consume it (or perform its own Kahn's sort) before dispatch.
