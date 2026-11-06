# Current defects and limits of proven

Line numbers refer to `snapshot/`. Static inference means checking the mechanism by code. Runtime probe means running the specified API on artificial data. No probe was connected to the model/account. P1 - blocking error of the promised function/boundary; P2 is an important API/failure violation that may not interfere with the first scenario.

## N01 · P1 · Try launch still breaks on identities

`ui/src/application-info.ts:234–237` receives and transmits 32hex. `crates/dc-desktop/src/main.rs:196–200` does `.parse::<u128>()`, which means decimal parsing. IDs with letters a-f are rejected; An ID made up of numbers only is interpreted in a different number system. Even earlier, `main.rs:87` generates a projection ID via Debug `EntityId(...)`, which the UI does not accept. Fixing the previous dashed UUID did not close this thread.

The check is static; a real UI click was not performed in this audit. Fix: one parser `EntityId::try_from` for canonical wire string, projection via Display/DTO, without WU-NNN/Debug/decimal transformations. Acceptance: from the present list of works, start an existing unit with letters in the ID, then check the attempt in the kernel; malformed/stale/foreign ID should be a failure.

## N02 · P1 · Window and executors do not save accepted changes in a single durable transition

`main.rs:184,205,237` calls a normal handle_command. Persistence is used during initialize/recovery (`:411–485`), but is not involved in these modification handlers. Both executors receive CoreState and endpoint, and the results/Accepted are written directly to memory (`executor.rs:232–236,895–898`). Searching all persistence references in these two files did not find durable write path.

Bottom line: a fixed startup crash does not prove recovery of user commands, results, or verification. `crates/dc-core/src/lib.rs:49` mentions commit_transition in a comment; no corresponding implementation was found in the crates studied. New canonical journal/ledger/evidence APIs do not have production callers in the scanned Rust outline: trait and sqlite implementation/tests found.

Fix: One team owner and one transaction state + event + budget/reservation/outcome + verification receipt. Issue a successful UI response after commit. Acceptance: action via IPC → confirmed commit → kill → restart → restoration of exact data; fault injection before/after commit and unknown receipt. Process kill should not be reported as a power outage.

## N03 · P1 · Machine verifier accepts someone else's result without proof of criteria

`dc-core/src/verification.rs:63–197` checks stage and non-zero fingerprint. The zero EntityId/zero NonZero revision checks are not possible for correctly constructed types. The contents of the artifact are not an argument to the function. Qualification is assigned as Qualified, criteria/content lists are empty; whether result.unit matches the expected unit is not checked in this path.

**Runtime confirmation:** probe `machine_pass_without_criterion_evidence_and_with_wrong_unit` passed Submitted, an arbitrary `[1;32]`, another unit/snapshot unit and non-empty obligations/criteria for expected work. The real run_machine_verification returned Passed. This is a reproduction of the API defect, not proof that a real account has already been affected.

Fix: first check scope/unit/attempt/snapshot/policy, then read and hash of real artifact, then qualified checks of each commitment. No method/artifact/origin - Indeterminate. Another unit is refusal. Only core transition has the right to issue Accepted. Separately check the independence of the author/verifier and the inability to self-designate Qualified.

## N04 · P1 · Run parallelism does not enforce dependencies

Added parallel executor, but desktop execute_graph (`main.rs:248`) calls sequential execute_accepted_graph. New code is detected as being called by a separate test; this is not proof of connection to the window command.

In a parallel path, the levels are calculated in advance (`executor.rs:643–689`). Refused/Unavailable/Err increments the counters (`:919–932`), after which the loop moves to the next level. There is no admission of the dependent unit according to the accepted status of the parents. Example A → B: A refused, but B is sent anyway. All units are cloned regardless of stage (`:630–636`). Scheduler contains its own readiness checks, but this path bypasses them.

The proof is static; The fake server experiment was not performed here. Fix: admission of each node from the current committed state, including dependency result, revision, fence, budget and attempt status. Acceptance: failure/unknown/cancel parent, obsolete target, already Accepted node, diamond DAG. The timing overlap check confirms concurrency, but not these properties.

## N05 · P1 · The model does not receive the full content of the goal and criteria

UI encoder continues to replace completionCriteria ID texts. Serial prompt (`executor.rs:95–123`) and parallel prompt contain kind/output-contract instead of the full origin of the target/obligations. The presence of ObjectiveContract in CoreState does not mean that it was included in HTTP messages. The root planner remains a simple single-job constructor; full decomposition and the associated long work are not proven by this.

Correction: saved semantic criteria → ContextProjection with source/revision/provenance → performer request. History should not be deleted for the sake of compression; the projection must explain which sources and versions are selected. Test: two targets with the same output scheme and different meaning should give different meaningful requests; changing the goal invalidates the previous result.

## N06 · P1 · Repeat unknown paid request saved

Removed external retry, but `dc-broker/src/dispatch.rs:159,231–242` continues to repeat timeout/transport errors up to max_retries; default 3 means up to four POSTs. No external idempotency key transfer found or evidence that the previous request failed. There is no end-to-end durable armor/unknown reconciliation on this route.

This is the limit of possible shipments, four write-offs have not been proven. Correction: one retry owner; prior to dispatch, proven refusal separately from possible execution; unknown save and verify. A new paid attempt requires a separate permit/reserve. Confirm a fake server that accepts POST and loses the response. Numeric Retry-After fixed; The HTTP date remains fallback (`:197–209`), which requires a separate parser/deadline test.

## N07 · P1/P2 · OS boundary remains incomplete; the new guard can change the thread

`dc-windows/src/job_os.rs:99–109`: ContainedProcess::adopt attaches existing PID to Job, but does not apply AppContainer token. `appcontainer.rs:149–163` does not read token, returns false. `max_cpu_time_100ns` is not applied in the viewed job configuration path. Production callers ContainedProcess/new NamedPipeServer were not found in the examined crates; these are API readiness defects, not verified current operation.

**New runtime/compile-time confirmation:** `named_pipe.rs:211–219` ImpersonationGuard contains only `()`, automatically Send+Sync. Probe confirmed these properties of the present type. Guard can be carried over and destroyed on another thread, although RevertToSelf is within the thread's impersonation context. Return value RevertToSelf is ignored. No real impersonation was performed in the probe.

Fix: guard !Send/!Sync, explicit fail-closed handling of RevertToSelf failure, real peer SID authentication. The process is created with AppContainer security capabilities, suspended → Job → token check → resume. Negative tests of files/network/child/handles. Profile/Job/name contained does not replace this evidence.

## N08 · P1/P2 · Future schema 10 erroneously becomes 3

`dc-store/src/sqlite.rs:249–250` executes `UPDATE store_meta SET value='3' ... value < '3'`. value has TEXT affinity. The string '10' is less than '3' lexicographically. initialize does not reject unknown schema prior to DDL.

**Runtime confirmation:** the artificial database with compatible tables and schema_version='10' has opened; the label became '3'. The old client erases the sign of the future scheme. Fix: strict version parsing before changes, supported versions, atomic migrations, unknown/future reject. Do not fix only CAST: the compatibility gate itself is required.

## N09 · P1/P2 · Deduplication evidence loses origin of second record

`sqlite.rs:1808–1827`: PK content_hash, INSERT OR IGNORE, return successful hash. A single line stores entity_id, type, depositor. `:1862–1873` searches for evidence based on this single entity_id.

**Runtime confirmation:** identical bytes from entities A/B were both successfully saved via the API, but evidence_list_by_entity(B) is empty. This is not a SHA-256 collision: the contents are intentionally the same. The production caller canonical API was not found, so loss of existing user data is not claimed.

Fix: dedup blobs by hash separately; immutable evidence records/links with their own ID and provenance separately. Acceptance: two works have the same document, different depositor/time/type, independent checks with the same results.

## N10 · P2 · Journal sequence does not match SQLite range

`stage3.rs:59–61` sequence - u64. `sqlite.rs:1653` cast as i64 without bounds, signed sort (`:1672`), head via MAX (`:1703`).

**Runtime confirmation:** sequence 1 and 2^63 saved; head is 1, the first in the sorted read is 2^63. The interface allows data that violates its order. UNIQUE limits duplicates, but does not prove gap-free/head+1 transition. Fix: closed range and compare expected head inside the same commit. This is not the normal size of a real story, but the incorrect entry should be closed.

## N11 P1 for acceptance certainty Part of the test does not prove anything about the claimed property

`dc-desktop/tests/e2e_persistence.rs:38–100` starts the process, waits, kills, checks the existence and size of the database. The reviewed test does not send a command to the target and does not check the restored target/result. The header promises submit/kill/verify persistence; actual verification is weaker.

`tools/validate-clean-machine.sh:123–145` does an env dump via `env -i ... env | sort`, then checks for the absence of prohibited lines. The script has set -u, but does not set the pipefail/status control of this dump; there is no required proof of a non-empty valid block. An empty file is able to pass negative greps. The script itself indicates that the machine is not a fresh VM. The presence of such a run does not close the clean VM/real installation.

`extended_stability_soak.rs:140+` if the RSS counter fails, returns 0. The measurement requires fail/unsupported instead of zero. The cycle Vec accumulation fix is ​​confirmed by the bounded stats code, but the new full 30-minute run is not in this audit. The old RSS growth should not be attributed to the core after the harness overhead was found; Failure to re-measure does not prove the absence of a leak.

Fix: assert positive measurement / exact expected data, saving binary SHA+source SHA+feature set+command+exit+logs. The test counter is an addition to the content of the checks.

## N12 · P1/P2 · The boundaries of runtime and reports diverge

Old broker/worker live tests are still able to automatically call localhost gateway; some new files are marked ignore. In this audit, a safe set was selected; the entire workspace was not launched blindly.

`dc-core/src/broker.rs:88–109` ProtocolDispatcher keeps returning Success with model='protocol-stub', without a real protocol dispatch. Desktop real HTTP uses another direct in-process branch. These are two different mechanisms; real HTTP in one does not prove production IPC in the other.

`dc-desktop/src/main.rs:565–601` signal handler calls process::exit, writes graceful shutdown initiated, but does not perform a full drain/cancel/persist barrier. WAL protects already committed transactions under its own conditions; it does not preserve uncommitted kernel state. There, Win32 unsafe is implemented outside of dc-windows, which is at odds with the current AGENTS.md.

Fix: Explicitly set supported routes; stub returns Unsupported/Unavailable in production; closing through an authoritative cancellation/persistence barrier with deadline and external watchdog. Place OS adapter in dc-windows. Before this, the journal word graceful/Success should not be considered evidence of the corresponding property.
