# Independent analysis of implementation - September 29, 2026

Base: Snapshot of the main project on `b194df41b7f94f30f99970bdf75a1b843c52d155`. Paths and line numbers refer to it. The source snapshot is located in `snapshot/`, the hashes are `evidence/snapshot-manifest.json`. This is a selective audit of critical chains, not a certificate of verification of each file. Old reports are not used as proof of serviceability.

P1 here means a blocking function defect or a serious violation of the promised boundary; P2 is an important defect that does not always block the first scenario. Severity does not mean that a leak or loss of money has already occurred.

## F01 · P1 · UI passes incompatible identifiers and arguments

`ui/src/application-info.ts:170–179` forms a UUID with hyphens. `crates/dc-domain/src/ids.rs:36–45` only accepts 32 lowercase hexadecimal digits without hyphens. Goal formation and planning cannot undergo this deserialization. Additionally, calls to `invoke` on lines 193, 209, 221, 235 use `objective_json`, `graph_json`, `unit_id`, and commands `dc-desktop/src/main.rs:162,175,198,212` are declared without `rename_all`. Tauri expects camelCase arguments by default. This is a separate failure that may occur before the UUID check.

Another incompatibility: `main.rs:85` outputs ID via Debug, but `application-info.ts:231` only outputs `WU-NNN`. Therefore, fixing the UUID alone is not enough to trigger the attempt.

Proof: code from both sides; Tauri's external agreement is confirmed by official documentation. Probe `actual_ui_dashed_id_is_rejected` calls the real EntityId type, not a copy of it. The actual window did not run in this audit.

Fix: single wire DTO, highlighting identities at the trusted Rust boundary, strict and consistent naming of IPC fields, returning a canonical string instead of Debug. Acceptance: the present path is to complete the goal → plan → accept → begin; negative cases: invalid UUID, missing field, unknown revision. Mocks invoke do not replace such checking.

## F02 · P1 · Enabling persistence results in panic; UI entries bypass persistence

If `DC_STORE_PATH` is present, `dc-desktop/src/main.rs:309` calls `EntityId::from_u128(1).unwrap()`. Validator `ids.rs:22–25` rejects the number 1: it does not have UUIDv7 version/variant. In this branch, panic is deterministic; This is not a probabilistic Windows bug. `main.rs:298` additionally ignores the initialize error.

Even after correcting the command ID, submit/start/accept (`main.rs:169,186,216`) calls the usual `handle_command`, not the persistent version. Executor also does not receive persistence. The presence of recovery serialization tests does not prove the persistence of the real window's actions.

Bugfix: trusted valid singleton identity, failure to start work mode on DB error, single durable command boundary with version and commit receipt. Acceptance: action via desktop command → process completion → restart → exact restoration of the goal, graph, attempts, unknown outcomes and costs. Separately fault injection before/after commit. This is not a power outage test.

## F03 · P1 · Broker's success leads to panic; after it the false acceptance path is hidden

`dc-desktop/src/executor.rs:125` receives `Success(_result_packet)` and does not use the actual content. On lines 142–147, creates an ID via `from_u128(1000+i)`, `2000+i`, `3000+i`, and unwrap. The first successful unit fails deterministically on an invalid ID.

Correcting these numbers is dangerous in itself to consider completing the task. Next, a new package is created with a zero fingerprint (`:153`). `dc-core/src/verification.rs:63–144` in the machine path only checks `stage == Submitted`, creates Qualified and empty criteria/find lists. Executor sets Submitted in advance, then directly writes Accepted (`executor.rs:180`). The actual model output and proof of fitness for purpose are not involved in this acceptance.

This is a statically confirmed logic defect; not a statement that the current UI has already successfully reached the false acceptance - F01/F02/the first part of F03 prevents this.

Correction: save real result/artifacts, fingerprints and origin; verify specific obligations using a qualified method; lack of method or proof returns Indeterminate. Prevent direct changes to Accepted from desktop. Acceptance: empty answer, incorrect answer, substitution of fingerprint, someone else's attempt, missing criterion should not become Accepted.

## F04 · P1 · The full-fledged task does not reach the model; dependencies don't control startup

`application-info.ts:178–179` discards the text of the criteria, leaving the ID. `dc-core/src/planner.rs:54–59` creates one Research unit with an empty `root_result`. `executor.rs:110–111` uses Debug from expected_output as prompt. Broker (`dc-broker/src/dispatch.rs:121–128`) sends only system/user prompts to the model; having a WorkUnitSpec in the local structure does not restore the missing target text.

Executor clones all `graph.units` (`:22–29`); `dc-core/src/dispatch.rs:130–142` makes a sequential zip/map without tolerance for graph dependencies and current status. The presence of a separate scheduler module does not mean that this execution branch uses it.

Fix: ObjectiveContract → provenance-linked ContextProjection → WorkRequest; The plan must retain the meaning of the criteria and dependencies. Each attempt must pass scheduler admission and verification of the current edition, and not just appear in the vector. Tests: two dependent units in reverse order, already Accepted unit, permission revoked, target change between plan and dispatch.

## F05 · P1 · Nested repeats after unknown outcome

`executor.rs:58–91` makes up to three external attempts. `dc-broker/src/dispatch.rs:138` executes `0..=max_retries`, the default in `endpoint.rs:30` is 3. Up to 12 HTTP POSTs per unit are obtained with the corresponding error sequence. This is the upper limit of attempts, not the proven 12 write-offs.

Timeout is repeated (`dispatch.rs:202–212`) without evidence that the server did not complete the previous request. `dispatch_id` is not passed to body/headers (`:64–76,145–151`). In this path there is no visible reconciliation of unknown and no reservation for a potentially toll retry. This is at odds with its own treaty 17 (§ Consequences and Costs), as well as the retry clause of non-idempotent requests in RFC 9110 §9.2.2.

Fix: single retry policy owner; distinguish between proven pre-shipment failure and possible implementation; unknown to save, do not turn into a regular retry. For replays, you need idempotency proven by the provider or a separate allowed attempt and its budget. Acceptance: The artificial local server accepts the POST and aborts the response - the second execution should not happen blindly.

## F06 · P2 · Retry-After reduced by a thousand times

`dc-broker/src/dispatch.rs:175–185`: the number from Retry-After is transferred to `Duration::from_millis`. According to the RFC, the number means seconds; HTTP date is also acceptable. For example, 120 is interpreted as 120 ms instead of 120 s, not counting its own backoff. Fix parser seconds/date and deadline, check with a controlled clock. This does not extend the repeat permission from F05.

## F07 · P1 · AppContainer is not yet a working execution boundary

`dc-windows/src/job_os.rs:99–109` in ContainedProcess::adopt creates a profile and Job Object, attaches an existing PID. There is no AppContainer applied to the process token. `appcontainer.rs:149–163` in is_process_appcontainer opens the process but returns `Ok(false)` without reading the token. Creating a profile does not sandbox an existing process.

No production callers of ContainedProcess were found by searching in crates. Therefore, the correct conclusion is that the mechanism is incomplete and not connected; it has not been proven that a particular existing worker is already violating the sandbox.

Correction: create a process with security capabilities/AppContainer, suspended → assign Job → check token/SID → resume. Negative tests: user files, network, child process, inheritance of handles, breakaway. Until then, real accounts should not be allowed based on the word contained in the type name.

## F08 · P2 · CPU limit declared but not applied

The `dc-windows/src/job.rs:18,29` max_cpu_time_100ns field has a default of 60 seconds. `job_os.rs:136–198` does not use it. Implement OS limit and QueryInformationJobObject/load synthetic process checking, or explicitly deny an unsupported request. The presence of a field in the configuration is not a resource limitation.

## F09 · P1 · Events and evidence allow destruction of previous contents

`dc-store/src/sqlite.rs:794,900,956` uses INSERT OR REPLACE for events/evidence/checkpoints. `dc-core/src/persistence.rs:234–291` generates an ID from the entity and milliseconds of time. If the ID matches, the second entry replaces the first with a successful result. SQLite documents exactly this kind of REPLACE semantics.

Record_event/record_evidence calls not found in production path: Current loss of running application history has not been proven. But the API defect is directly reproducible. Probe duplicate_event_id_silently_replaces_history calls the real SQLiteStore. Fix: insert-only identities, independent ID and sequence, exact idempotent repeat only if payload matches; other content with the same ID is a conflict.

## F10 · P1/P2 · Unknown database schema version is not rejected

`sqlite.rs:62–171,176–211`: initialize creates tables IF NOT EXISTS, INSERT OR IGNORE schema_version=2, but does not check for existing version. The base labeled 999 is accepted by the old client if the structure allows operations. Contract C-03 requires UnsupportedSchema. Probe future_schema_version_is_accepted uses an artificial separate database.

Correction: read version before modifying DDL; reject the unknown; migrations are atomic, backup/rollback and structure checking are required. You cannot change the version label for the sake of a failover.

## F11 · P2 · Authenticated decoder modifies bytes before replay check

`dc-protocol/src/auth.rs:206–209` re-serializes the parsed WireEnvelope; the formatting and order of the original keys are lost. The direct decoder preserves the original bytes (`lib.rs:253–270`), the `tests/authority.rs:441` test distinguishes between structurally equal but byte-byte different messages. ProofVerifier receives only &Envelope (`auth.rs:81–87,136`). The exact original bytes promise on both paths is not fulfilled.

This is a proven contract divergence, an unproven cryptographic bypass. Solution: either an explicitly canonical signed format with unambiguous rules, or storage/verification of the original bytes. Test through the authenticated path, and not just through the direct decoder.

## F12 · P1 to admit real resources · Some tests are not isolated and do not prove success

`dc-broker/tests/dispatch.rs:19–35` dispatch_stub_success sends the real request to the default localhost gateway and accepts Ok or Err. The file header promises no real HTTP. live_dispatch_against_local_omniroute (`:61+`) automatically contacts the gateway when the port is available. A real dispatcher was also found in `dc-core/tests/live_self_dev_e2e.rs`. Therefore, a regular general cargo test cannot be automatically considered an artificial test and is safe to run next to real connections.

In this audit, the overall run was interrupted while dependencies were being compiled; There are no completed results of his tests. A separate secure set is then selected. Fix: explicit opt-in live-tests, local fake server by default, no real credential/provider defaults, structured Skipped, prohibition of accepting any Err as proof of successful dispatch.

## What is useful and should not be thrown away

There is a meaningful subject model, closed wire types, protocol frame and replay cache size restrictions, SQL CAS by row_version, numerical bounds checks, WAL/FULL/FK/STRICT settings, real Win32 Job/DPAPI calls and RAII handles. This is the essential basis. However, the properties of an individual module are not automatically transferred to the desktop → core → broker → worker → verification → store chain.

There are also improvements relative to previous audit topics: the recovery code stores a separate outcome evidence; The watchdog anchor path contains a version precheck. Therefore, the old conclusion “there is no evidence of anchor outcome/verification at all” cannot be repeated without qualification. Generic storage anchor remains a different API, not equivalent to this protected path.
