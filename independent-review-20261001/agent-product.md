# Independent audit of UI and execution - 2026-10-01

Base: unchangeable snapshot `independent-review-20261001/snapshot`, declared baseline f7444d1. Analysis is static; the application, tests and network did not start. All links below are relative to snapshot. Small target areas read; the entire project is not covered by this audit.

## Real fixes

- UI now sends UUID as 32 hex and commands have `rename_all = "snake_case"` (`ui/src/application-info.ts:172`, `crates/dc-desktop/src/main.rs:177,193,220,233`). Previously identified specific inconsistencies in format and argument names have been resolved.
- Recovery uses UUIDv7 instead of `from_u128(1)` (`main.rs:435`); initialization error is no longer silently discarded (`:417`). This resolves the previous specific cause of panic, but does not prove command persistence.
- In executor, ResultPacket identifiers now have version/variant UUIDv7, fingerprint is calculated from response.content. Old unwrap over 1000/2000/3000 and permanent zero fingerprint have been fixed.
- Added a separate executor with topological levels and limiting simultaneously executed tasks to 8 (`executor.rs:643-679,737-768`). The presence of concurrency code is confirmed; runtime operation has not been tested here.

## Residual defects

### P1: start_attempt still doesn't accept canonical ID

The UI allows 32 hex and sends the string without conversion (`application-info.ts:234-237`), but the native handler does `.parse::<u128>()`, that is, decimal parsing (`main.rs:196-200`). ID with a-f is rejected; consisting only of digits is treated as another number. Earlier blocker: CoreStateSource still generates id via Debug (`main.rs:87`), so the UI gets EntityId(...) instead of 32hex and rejects the attempt before IPC. Path: try button -> startAttempt -> IPC start_attempt. Validation: pass canonical existing ID with a-f. Fix: generic canonical parser EntityId, no manual decimal conversion; check the real IPC path.

### P1: acceptance of commands and results is not durable acceptance

Native submit/start/accept use the usual `handle_command` (`main.rs:184,205,237`), not the persistence option. Both executors receive only CoreState and endpoint configuration. Success directly modifies graph.units.stage and adds verification to memory (`executor.rs:232-236`, parallel path `:895-898`). The result, attempt, artifacts and durable verification are not recorded in a single transition. Check: run through desktop commands and restart, compare status/evidence; the test has not yet been run. Fix: single transaction point of application and acknowledgement only after commit.

### P1: the check still does not confirm the result of the task

`crates/dc-core/src/verification.rs:63` accepts a ResultPacket without result content. Added non-zero fingerprint and zero UUID/revision checks. UUID and revision are already limited to domain types, so these checks do not evaluate performance. A non-zero hash does not mean the criteria are met, and even an empty string can have a non-zero hash. The criterion_ids and findings lists do not contain any tests performed; the qualification is assigned by the code itself. Executor on Passed converts the unit to Accepted. Negative example: Submitted + valid IDs + arbitrary non-zero fingerprint, without evidence of achieving the goal. Indeterminate is expected, current logic allows Passed. Fix: Explicit verified criteria, real artifacts, and link to snapshot; Do not report unrunned checks as success.

### P1: Parallel levels do not block after prerequisite failure

`executor.rs:689` sequentially bypasses pre-computed levels. Refused/Unavailable/Err only increases the counter (`:919-932`), after which the next level is executed. The condition for successfully accepting dependencies before dispatch is missing from the read path. Negative example: A -> B, A returns Refused; B still starts. Additionally, the initial copy takes all units (`:630-636`), not just the allowed attempts. Fix: admission of each node after checking the current committed state of dependencies; negative diamond/DAG tests with parental refusal.

### P1: objective and criteria not delivered to new parallel prompt

The parallel path copies only units/dependencies (`:630-636`) and forms a prompt from a common phrase by kind, ID and OutputContract (`:710-735`). The sequential desktop path has the same problem (`:95-123`): the comment promises a priority objective statement, but the code selects kind_desc exclusively. The objective text and parent results are missing. The UI still replaces criteria strings with index IDs, discarding the strings themselves (`application-info.ts:180`). Thus, generic prompt does not specify a real user task. Fix: Validated ContextProjection with objective, criteria content and dependency results bound; test for the presence of exact custom wording in the mock broker request.

### P2: the new parallel implementation is not the desktop way

`execute_graph` calls `execute_accepted_graph`, not `execute_parallel_levels` (`main.rs:247-248`). Therefore, the presence of a separate parallel live test does not prove parallel execution from the UI. You should either integrate the new path after its failures are fixed, or explicitly describe it as a separate experimental API.

## Validation boundaries

Target areas: `ui/src/application-info.ts` encode/IPC/start; `dc-desktop/src/main.rs` commands/build persistence; `dc-desktop/src/executor.rs` parallel execution completely from reading the graph to final statistics, sequential result/verification in key areas; `dc-core/src/verification.rs` machine verification. This is not proof of compilation, network operation, GUI launch, correctness of the entire system, or the absence of other defects. Source corrections were not performed.
