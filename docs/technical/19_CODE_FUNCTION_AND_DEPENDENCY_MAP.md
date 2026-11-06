# Deep Create: map code, features and dependencies

**Created:** 2026-09-06  
**Revised:** 2026-09-07  
**Status:** contract for future implementation; the listed functions have not yet been created.  
**Future workplace:** `D:/Projects/DeepCreate`

## 1. How to read and apply the map

Each C-01 through C-16 contract specifies the duty, permitted open surface, entry, exit, side effects, waivers, and evidence. Types are specific: when implemented, Rust may specify memory ownership and reference lifetimes, but not the authority or meaning of the transition.

Contracts do not require creating all packages at once. The first slice that works uses only the parts it needs. An empty package, a function without a caller, and a pass-through wrapper without its own responsibility are not added to the assembly.

A local pure function can be a detail of a single contract. It does not need a separate beacon: a significant transition is recorded by its owner. The function determining the right, expense, external action, acceptance or restoration is subject to this agreement regardless of the word `pub`.

Main links: [data and states](./14_DATA_STATE_AND_RECOVERY.md), [context](./15_CONTEXT_MEMORY_AND_INSTRUCTIONS.md), [execution](./16_EXECUTION_MODELS_AND_VERIFICATION.md), [protect](./17_PERMISSIONS_SECURITY_AND_ACTIONS.md), [events](./18_BEACONS_JOURNAL_STOP_AND_RECOVERY.md). The check result and event fields are defined there; new arbitrary dictionaries are not allowed.

## 2. The tree and the real reason for each part

```text
DeepCreate/
  Cargo.toml, Cargo.lock, rust-toolchain.toml
  package.json, pnpm-lock.yaml
  docs/technical/                 проверенная редакция договоров
  crates/
    dc-domain/                   значения, состояния, чистые правила
    dc-protocol/                 закрытые сообщения между процессами
    dc-store/                    единственный способ записи главной базы
    dc-core/                     принятие команд и планирование
    dc-broker/                   права, секреты подключений, последствия
    dc-watchdog/                 ограждения, поколения, независимая остановка
    dc-worker/                   одна изолированная попытка
    dc-windows/                  проверенные вызовы Windows
    dc-desktop/                  родной ввод и оболочка окна
  ui/                            отображение, навигация, обычные черновики
  tests/
    contracts/, transitions/, security/, recovery/, end_to_end/
  tools/test-harness/            воспроизводимые испытания отказов
```

`dc-bridge` and `dc-updater` are added only at steps 10 and 11 of document 20. Before that they are not in the assembly. The documentation acceptance tool lives next to the documentation and does not become part of the application.

Separation by process protects against freezing and access by an untrusted executor. It does not mean that each table or model role requires its own process. Planning, selection, routing and acceptance are modules of one control process. The verification model is executed as a normal single limited attempt.

## 3. Dependencies

| Part | Resolved dependencies and access |
|---|---|
| `dc-domain` | only subject meanings; without base, network, clock, environment and Windows |
| `dc-protocol` | `dc-domain` and checking the message form |
| `dc-windows` | minimal Windows types and calls; each consumer receives only his own adapter |
| `dc-store` | `dc-domain`, SQLite, Validated Encryption and Narrow Key/File Adapter |
| `dc-core` | domain, protocol, store; contacts broker/watchdog with messages |
| `dc-broker` | domain, protocol, Windows, your own storage of rights and observations; without memory works |
| `dc-watchdog` | domain, protocol, Windows, own database `authority.anchor`; without model and network |
| `dc-worker` | domain, protocol and time domain; no store, secrets, user files and direct network |
| `dc-desktop` | protocol and narrow native input; without main base and connection keys |
| `ui` | snapshots and valid command suggestions; without shell capabilities for arbitrary I/O |

The fact that Rust is a separate package does not limit the process's system calls. Access denials are proven by Windows rights and a C-12 test. Internal store/watchdog entries, temporary executor files, and window view saves have their own narrow owners; The "middleman only" rule applies to user files, running authorized programs, and network implications.

Loops between packages, importing executor code into a trusted process, and direct window access to the store block the build.

## 4. General rules for passports

`Result<T, E>` means either a complete exit or a closed failure code. Failure does not return the partially applied condition as a success. For an unknown external outcome there is a separate type, rather than the usual `Err`, after which it is convenient to repeat.

`ExpectedRows` - numbers of modified lines and their expected revisions. `DependencySnapshot` - read dependencies defined in document 15. The general event number does not replace any of them.

`CommandEnvelope` contains the form version, unique command number, installation and run number, authenticated sender, allowed private command, and expected versions. The identity of the sender is substituted by a verified channel, not a model body.

`TransitionPlan` contains private subject line changes, resource transactions, typed events, and pending versions. It does not contain SQL, executable text, or network call from the model.

`CommitReceipt` certifies only the local fixation and its number. The outward acknowledgment of the accepted result additionally follows the guard checkpoint protocol from document 14. There is no single common transaction between different bases: failure between phases is explicitly handled.

In the tables below, “test” is a future required scenario, not a test that has already been passed. AC references refer to document 21.

### C-01 - pure subject rules

**Location:** `dc-domain/{objective,work,permission,resource,result}.rs`. **Why:** the same rule applies to ordinary conversation, deep work and verification.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `validate_objective` | C-05; `ObjectiveCandidate + OwnerConstraints → ValidatedObjective` | no I/O; `MissingCriterion, ConstraintConflict` |
| `validate_work_graph` | C-05; `GraphCandidate + ObjectiveContract + LimitView + AffectedGraphView → ValidatedGraph` | `Cycle, UnboundObligation, UnboundedUnit`; precise editions of interworking communications and future cycles of 24; freedom of method is not prohibited |
| `is_lease_subset` | C-09; parent, requested area → narrowing proof | `ScopeExpansion, ExpiryExpansion, LimitExpansion` |
| `evaluate_reservation` | C-03; current postings, limit, request → allowed postings | `LimitReached, UnknownUpperBound`; money in whole minimum units |
| `assess_verifier_independence` | C-08; author/verification origin and work type rule → conclusion | `InsufficientIndependence, UnknownLineage`; another name is not proof |
| `derive_completion_candidate` | C-08; obligations, accepted results, checks → candidate or list of gaps | does not complete the main work |

**Evidence:** AC-L07, AC-L09A, AC-I10, AC-I12. Number bounds, negation, cyclic graph, and false independence are checked deterministically. A beacon is created by the caller when an offer is accepted or rejected.

Addition to C-01: [24, §9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md) clarifies `derive_completion_candidate` and graph verification; [25, §9.1](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) sets internal `validate_scheduling_policy` and `compare_root_service` with calling C-05/C-06. Both calculations are pure, without their own entry or LLM call. Reasons `UnknownPublication`, `InvalidLinkCondition`, `OutputSchemaMismatch`, `UnsupportedLinkPolicy` and arithmetic failure are not replaced by a general “not ready”. Test transitions and properties of C-01 are described in 26, §2.

### C-02 - Interprocess messages

**Location:** `dc-protocol/{envelope,commands,observations}.rs`. **Why:** model text cannot pretend to be an owner or mediator event.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `decode_envelope` | receiving party; limited bytes and current protocol → parsed message | without action; `Oversize, DuplicateField, UnknownField, UnsupportedVersion` |
| `validate_sender` | receiving party; message and `AuthenticatedPeer` from C-12 → `ValidatedCommand` or observation | `WrongRole, WrongInstallation, StaleGeneration, ReplayConflict` |
| `encode_envelope` | sender; private type → limited bytes | `Oversize, UnsupportedVersion`; without extension of rights |

Each pair “sender role → command type” is specified by a closed table. Repeating the same number with the same body returns the recorded total; another body with the same number is rejected. Deduplication of a significant command is stored in the owner of its state, and not just in the channel memory.

**Evidence:** AC-L03A, AC-I06, AC-I14. Role reversal, repeat, cropped body and previous launch message. The failure falls into a safe incident without the original sensitive body.

### C-03 - main base, transition and general flow

**Location:** `dc-store/{database,transition,resource_ledger}.rs`. **Why:** do not separate the state, event and accounting between different writers.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `open_store` | C-13 launch; verified place and `StoreKeyHandle` → `StoreHandle` | checking the circuit/settings; `Locked, IntegrityFailure, UnsupportedSchema, StorageUnavailable` |
| `read_state_view` | C-05/C-13; area and allowed view → agreed `StateView` | read only; `UnknownScope, IntegrityFailure` |
| `commit_transition` | C-05 only; `ExpectedRows + TransitionPlan + CommitPreconditions` → `CommitReceipt` | one main base transaction; `VersionConflict, FenceChanged, LimitReached, StorageUnavailable` |

`resource_ledger` does not have a public write path. Reserve, Confirmed Expenditure, Unknown Outcome Retention and Release change only within `commit_transition`. One flow rate is in exactly one of the considered states. The intermediary checks the associated reserve number and entitlement limit, but does not create a second general cash journal.

**Evidence:** AC-I03, AC-L07, AC-I05, AC-I17. Two simultaneous reservations for the last balance; failure before/after COMMIT; command repeat; unknown flow does not clear the limit. The event and lines have the same transaction number.

Addendum to C-03: [24, §9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md) adds expected versions of shared object relationships, grants, and holds; [25, §§5–6](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md)—limit one pending checkpoint and durable replay command recognition. The global event number does not replace the exact versions of the items read. Checks AC-XW-05/13/14, AC-COMMIT-04—AC-COMMIT-11.

### C-04 - encrypted evidence

**Location:** `dc-store/{evidence,keys}.rs`. **Why:** preserve the exact original source and display only the permitted fragment.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `seal_evidence` | C-05; flow, origin, class and limit → `SealedEvidenceDescriptor` | encryption before disk recording; the file is not canonical yet; `Oversize, StorageUnavailable, IntegrityFailure` |
| `open_scoped_evidence` | C-07/C-13; proof, permitted range and area → limited flow | access and integrity checks; `ScopeDenied, MissingEvidence, IntegrityFailure` |
| `verify_evidence` | C-13; descriptor → existence and integrity report | reading; `MissingEvidence, IntegrityFailure` |
| `stage_evidence_copy` | C-05/C-13 via verified permit C-09; source, destination and confirmed limit → `StagedLocationReceipt` | stream copy of encrypted bytes; does not change the canonical link; `InsufficientStorage, DestinationChanged, ScopeDenied, IntegrityFailure, StorageUnavailable` |

The canonical link only appears through C-03 after successful sealing. The file name is an opaque number, not a clear hash of private text. Storage keys and vendor keys have different purposes. C-12 issues the vault key only to the store owner; this feature is not included in the performer's IPC.

The copy passport, input certificates, failure phases and consumers are defined in [23, §6](./23_EXTERNAL_CRITIQUE_AND_REFINEMENTS_20260908.md). `open_scoped_evidence` and `verify_evidence` further differentiate `EvidenceOffline` and `StorageUnavailable` from proven loss. Selecting a trusted location is an internal step of these functions, not granting permissions. `AcceptEvidenceLocation` is a variant of the existing C-05 command with the usual transition C-03, not a new open entry into the database. Clearing the old copy remains a separate C-09 action; without the exact right it is not executed. Checks: AC-STORE-01—AC-STORE-05.

**Evidence:** AC-L14, AC-L15B, AC-I14V, AC-I17. Bait on disk before committing, damaged object, broken encryption, orphan without reference, full archive prohibited. Event `evidence_sealed` does not reveal content.

### C-05 - Commands, Settings and Single Acceptance

**Location:** `dc-core/{command,state_machine}.rs`. **Why:** separate the model proposal from the system solution.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `handle_command` | certified entry C-02; `ValidatedCommand → CommandOutcome` | reads the status, calls the pure rule and C-03; returns confirmation after the desired checkpoint |
| `decide_transition` | only `handle_command`; `StateView + ValidatedCommand → TransitionPlan` | pure function; `InvalidTransition, MissingPermission, DependencyChanged, InvariantViolation` |

Commands include `AcceptObjective, AcceptGraph, StartAttempt, RecordResult, RecordVerification, ApplyVerifiedResult, ProposeSettingChange, AcceptSettingChange, AcceptCompletion, RejectCompletion`. These are enumeration variants, not new public functions for each row of the table.

Setting with a normal phrase creates `SettingChangeProposal`: exact old/new values, clause source, scope, version and consequences. Power changes only after the current trusted decision; The model does not write the configuration directly. Sensitive settings are made through the native C-14 window.

**Evidence:** AC-L01, AC-L03A, AC-L10, AC-I03. Repeated confirmation, old form, changed data, refusal to complete, model command from someone else's sender. The beacons correspond to the accepted version of the command.

Addition to C-05: closed versions of commands from [24, §9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md), [25, §9](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) and [26, §§4.6/5.6](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md) are included in the same `handle_command`. Domestic `try_admit_command`, `select_next_commit_command`, `revalidate_queued_command`, `await_commit_checkpoint` and `observe_pipeline_latency` have precise passports of 25; do not become separate services or available workaround models. The body of the queue belongs to it, is limited in size and is not replaced by a single hash. The semantic transition to the control point is not declared successful. The watchman remains outside this line.

### C-06 - unit readiness and connection selection

**Location:** `dc-core/{scheduler,routing,retry}.rs`. **Why:** effectively move independent branches within the owner.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `select_ready_units` | control loop; `StateView + ClockObservation → ReadySelection` | pure choice with reasons; takes into account all blocking boundaries |
| `filter_connections` | routing; contract, information plan, register, limits → acceptable candidates and refusals | does not choose an unsafe path for the sake of quality |
| `rank_connections` | routing; valid candidates and rule → `RouteProposal` | explainable sequence of criteria; `NoAdmissibleConnection` |
| `classify_attempt_failure` | C-05; transport surveillance and version rule → `FailureClass` | code 400 by itself does not mean a temporary failure |
| `plan_retry` | control loop; history, reason and remainder → `RetryProposal` or exhaustion | no timer/recording; next launch saves C-05/C-03 |

The slot and flow rate are reserved after calculating the exact request; Before dispatch, their suitability is checked again. The connector makes one attempt and has no hidden loop. Five depths are versioned settings for the allowed amount of branching and checking, rather than five scheduler implementations. A strong model is selected based on the proven capabilities of a specific connection.

**Evidence:** AC-L05B, AC-L07, AC-L12, AC-I08, AC-I09, AC-I12. Queue, invalid 400, expired quota, change of recipient, independent work. Selection and trigger beacons are recorded via C-05.

Addition to C-06: `select_ready_units` reads interworking conditions and holds from [24, §9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md), and internal `choose_fair_root_turn` and `plan_protected_capacity_claim` from [25, §§4/9](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) select the course and offer of resource hold. Returning does not reserve a resource; acceptance goes through C-05/C-03. There are `AwaitingPublication`, `OwnerAcceptancePending`, `ConditionUnknown`, `WorkspaceBusy` and connection availability/qualification errors. Checks AC-XW-01—AC-XW-17 and AC-SCH-01—AC-SCH-13.

### C-07 - search, selection, assignment and analysis of the answer

**Location:** `dc-core/context.rs`. **Why:** transfer enough for a single unit, without transferring power and the entire archive.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `plan_context_needs` | C-06; contract and memory area → `ContextNeedPlan` | local search via C-04; `ScopeDenied, MissingRequiredEvidence` |
| `compile_context_projection` | C-06; plan, candidate fragments, exact connection, rules → `ContextProjection` | `DisclosureDenied, StaleEvidence, ContextOverflow` |
| `compile_prompt_bundle` | C-06; goal, unit, selection, capabilities, profile, scheme → `PromptBundle` | `PolicyConflict, MissingLayer, ContextOverflow` |
| `parse_result_packet` | C-05; sealed original response, expected pattern and limits `OutputContract`, `TrustedAttemptEnvelope`, certified intermediary trace → `ResultPacket` or `ResultParseError` | complete closed list of refusals - 15 §2.9, in particular `DuplicateKey, ResponseTooLarge, InvalidReference, EnvelopeMismatch, InvalidToolTrace`; does not accept the semantic result |

The search can only be broad within a pre-authorized local area. Sending always takes into account the exact recipient. Sources, arguments, and free response text do not become security instructions. Trusted numbers, times and flow are taken from the local trace and not from the JSON model. The semantic search and reduction assistant itself is executed by a regular unit, with the same rights and verification.

`adapt_for_provider` - C-07 internals in `dc-core/context.rs`: From `PromptBundle` and the pinned version of the converter, creates the final unclassified `ProviderRequest` body, layer mapping and secure fingerprint. It does not send a request, does not substitute a key, and does not issue access. Inability to maintain a binding contract - `UnsupportedPromptContract`. The authenticity of the body after the turn is checked by C-09; these two stages do not become separate public execution paths.

**Evidence:** AC-L08, AC-L15B, AC-I11, AC-I11A, AC-I13. Lost negation during reduction, alien source, flow field filled in by the model, change of significant dependence. Beacons 15 §8 belong to C-05.

### C-08 - Verification and Completion Candidate

**Location:** `dc-core/verification.rs`. **Why:** Verify claims and commitments while maintaining free inquiry.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `run_machine_verification` | C-05; package, criteria, dependencies and observations → `VerificationRecord` | reading permitted evidence; `InvalidEvidence, CriteriaNotMet` |
| `plan_semantic_verification` | C-05; candidate and WorkKind policy → `VerificationUnitProposal` or sufficient machine verification | `InsufficientIndependence, VerificationBudgetExhausted` |
| `build_completion_report` | C-05; obligations, results, checks and balances → `CompletionCandidate` | read only; does not approve final completion |

The verification unit uses C-06/C-07/C-11. Its output is machine checked for form and origin, and the admissibility of the semantic check is determined by the final rule of the document 16. There is no infinite “verifying verifier”. Having corrected the candidate, the reviewer becomes the author of the new candidate and does not accept it himself.

**Evidence:** AC-L10, AC-I10, AC-I11, AC-I18. False majority, missing primary source, unmet criterion, recursion stop. The final state of the main work is changed only by `AcceptCompletion` from the owner.

Addition to C-08: the qualification of the method has [exact passports in 26, §4.6](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md). `build_calibration_plan` is called by C-05 and returns the plan; `evaluate_calibration_evidence` - candidate evidence for plan/observations/marking; `assess_method_qualification` - versioned solution for C-05 or `plan_semantic_verification`. `assess_error_dependence` - internal matching of paired errors. None of these functions send a request indirectly or write state directly. Failures of origin, obsolescence, undersampling and coverage are distinguishable; a result with an undefined denominator does not become a zero error rate. AC-CAL-01—AC-CAL-09; a separate registry service is not created.

### C-09 - rights, secrets and actions of an intermediary

**Location:** `dc-broker/{leases,action,intent,secret}.rs`. **Why:** the only path of consequences from all modes.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `canonicalize_action` | C-05; typed clause and permitted scope → `CanonicalAction` | real file/recipient ID via C-12; `UnknownAction, TargetChanged, ScopeDenied` |
| `issue_lease` | trusted login; `OwnerDecision + CanonicalAction + AuthoritySnapshot → CapabilityLease` | one entry `guard.db`; `ExpiredDecision, StaleDecision, ScopeDenied` |
| `derive_child_lease` | C-05; parent and narrowing → new limited right | check C-01, write guard; `ScopeExpansion, Revoked` |
| `revoke_lease` | trusted solution; number/version → fencing confirmation | C-10 coordination and guard recording; the wait is not passed off as a final review |
| `prepare_intent` | C-05; already recorded `ActionIntent` with a stable number, exact action, right and confirmed reservation → `PreparedIntent` | persistent acceptance of the intent in guard with the same number; `NoReservation, Revoked, TargetChanged` |
| `dispatch_verified_intent` | C-05; prepared intention and current tolerance → `ActionObservation` | one registered processor; `ConfirmedFailure` or `OutcomeUnknown` differ |
| `reconcile_intent` | C-13/C-05; intention and proven method of reconciliation → `ReconciliationObservation` | only permitted reconciliation; the unknown does not turn into failure |
| `store_secret / use_secret / revoke_secret` | trusted input / registered handler / trusted review | key to secure storage / application inside a request / termination; no output returns public key |

External file, network and registered program handlers are closed parts of one `ActionKind`. You cannot export a separate direct `send_request` or `write_file` that bypasses the intent. The intermediary's observations enter the main story as certified facts through C-05, even if the meaningful application of the result is already prohibited by the recall.

`validate_dispatch_preconditions` is a closed stage of `dispatch_verified_intent` in C-09. Its input is the exact `PreparedIntent`, the final `ProviderRequest`, a confirmed resource reserve and current rights/fences/versions. It checks the recipient, body after conversion, allowed data classes, term, run generation, required dependencies, current rate and top cost. The freshness of subject dependencies is certified by C-05 through the general admission procedure, and not from a long-copied image. Success only allows this submission on the current C-10/C-12 short clearance, does not return a carryover pass. The secret substitution remains with the registered C-09 handler. A change in body, route or significant version requires a new admission; the failure maintains the distinction between `StaleInput`, `Revoked`, `DisclosureDenied`, `LimitReached` and `TargetChanged`.

When recording an intent, the intermediary stores an immutable reserve evidence with all `limit_id`, `window_id`, upper bounds, and transaction/reserve numbers. This is proof for recovery, not a second resource log: the amount of available balance is calculated by C-03 only. When recovering, the C-13 must verify these evidence and observations with the main base rather than lose the flow of the missing tail.

**Evidence:** AC-L02—L04B, AC-L09A, AC-I05—I07, AC-I17. Object/recall race, path/address spoofing, unknown outcome, hidden replay, honeypot leak. Beacons of rights and actions from 18; there are no secrets in them.

Addition to C-09: [24, §§6–9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md) clarifies real objects, `use_id`, all `supporting_lease_refs`, application conflicts and unknown partial consequences. [25, §§5–7](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) requires space for mandatory observations prior to launch; [26, §§2–3](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md) sets the check for outflow at the actual boundary. The uncertainty of the physical race gives an honest refusal, not the right to consider the entry safe by the path string.

### C-10 - guard, barriers and stop

**Location:** `dc-watchdog/{authority_anchor,admission,job_tree,stop}.rs`. **Why:** The controlling process and the stuck provider do not own the emergency stop.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `read_authority_snapshot` | verified core/broker; area → `AuthoritySnapshot` | read `authority.anchor`; photo is not permission to perform |
| `next_run_generation` | the only accepted launch core → new generation | transaction anchor; `AlreadyRunning, IntegrityFailure` |
| `establish_fence` | trusted recall/stop; area and reason → `FenceReceipt` | transaction anchor and admission procedure from 17 §12.1; `StorageUnavailable, AdmissionNotDrained` |
| `record_journal_checkpoint` | C-05; `journal_id`, expected previous peak and new `{event_seq, event_mac, transaction_id}`, launch generation → `CheckpointReceipt` | checked anchor record with the condition that the previous vertex matches; `HeadConflict, StaleGeneration, IntegrityFailure, StorageUnavailable` |
| `register_process_tree` | verified launch; suspended process, area and limit → `ProcessTreeRef` | binding to Job Object before execution; `ContainmentUnavailable` |
| `terminate_process_tree` | automatic stop → `TerminationObservation` | real cessation through C-12; `TerminationUnconfirmed` |
| `advance_stop` | owner/system surveillance; current stop state → next | the prepared and immediate branches are different; expectation/unknown are not equal `stopped` |

The global era refers to the installation. The fencing of a work, unit or right shall not suppress unrelated works. The epoch/fence change and the anchor event are captured together. A consequence that has already been sent retains the observed or unknown outcome. All internal termination actions remain with the watchman; the intermediary only requests them.

The main base record and the anchor confirmation are two different commits. After the first, C-05 is located in `DB_COMMITTED_PENDING_CHECKPOINT`; only `CHECKPOINT_CONFIRMED` allows you to issue a successful confirmation to the owner and allow action dependent on this record. The checkpoint receipt does not create another main log event, otherwise there would be an endless chain of confirmations. Unexpected tail after failure is checked by C-13 in limited mode; The guard does not pin an arbitrary new vertex on a single kernel request. Anti-rollback protection procedure and limit - document 14.

**Evidence:** AC-L04B, AC-L05, AC-L06, AC-L06A, AC-I04, AC-I14A. Frozen kernel/broker, death of security clearance holder, old generation, power outage, one of two jobs stopped.

Addition to C-10/C-12: the protective entrance and limited critical sections from [25, §7](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) are checked regardless of busy models and full queue. The named phases and observations `D/C/F` according to [26, §2](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md) link the artificial trace to the real Windows mechanism; The test model does not create a new production core.

### C-11 - isolated performer

**Location:** `dc-worker/{main,assignment}.rs`. **Why:** A limited attempt does not gain authority from the control system.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `run_assignment` | verified channel; `Assignment + ScopedTemporaryArea → WorkerObservationStream` | one attempt, temporary materials; `InvalidAssignment, Cancelled, ToolDenied, ResourceExceeded` |

Inside, it is possible to suggest the next step, request an authorized remedy, and confirm life. These are message options, not new public APIs. The model is called through an intermediary; the performer does not establish a connection with the supplier and does not open the owner's profile. The process number itself does not certify the attempt: the launch and the issued order are taken into account.

**Evidence:** AC-L09, AC-I01, AC-I06, AC-I14B. Forbidden file/network, child exit from Job Object, late message and false life signal.

### C-12 - Windows Small Layer

**Location:** `dc-windows/{ipc,process,file_identity,key_store,admission}.rs`. **Why:** do not spread privileged code throughout the program.

| Adapter | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `authenticate_pipe_peer` | C-02; channel handle → process/area verified identity | `PeerMismatch, AccessDenied`; channel name is not a secret |
| `create_contained_process` | C-10; approved binary, scope and limit → suspended tree | `ContainmentUnavailable`; launch without isolation is not substituted |
| `open_verified_target` | C-09; canonical domain and expected identity → public verified handle | `ReparseDenied, IdentityMismatch, SharingConflict` |
| `replace_verified_target` | C-09; handle, expected bytes and prepared replacement → watch | indivisibility only where proven; conflict is not silently overwritten |
| `protect_key / open_key_handle` | C-04 for data; C-09 for connection | sealed value/restricted key descriptor of its destination; `WrongPurpose, UnprotectFailed` |
| `enter_admission / leave_admission` | only C-03/C-09/C-10 under protocol 17 | live short admission/exemption; `FenceChanged, AbandonedAdmission` |

The credentials of this package are divided by adapter; access to the library does not give the executor access to its process. The storage key does not replace the vendor key. In the native code, all sections of `unsafe` are concentrated here, with a description of the requirements for descriptors and memory.

**Evidence:** AC-I07, AC-I14A, AC-I14V, AC-I17. Real Windows testing is required before private data. A successful API call and a file signature match do not prove the entire security boundary.

### C-13 - restoration and copies

**Location:** `dc-core/recovery.rs`, `dc-store/{backup,migration}.rs`. **Why:** restore meaning and unknown outcomes without resuming old actions.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `inspect_startup` | launch core; installation version and anchor → `RecoveryReport` | checking the database, files, tail, open actions; necessary transitions only C-05/C-03 |
| `prepare_resume` | current owner team → `ResumeProposal` | re-verification of dependencies, rights, deadlines and expenses; `UnresolvedOutcome, StaleInput, StorageUnavailable` |
| `create_backup` | allowed command; agreed shot → `BackupManifest` | separate place with check until suitability is recognized |
| `verify_backup` | checking the copy → `BackupVerification` | opening separately and decoding; `MissingObject, IntegrityFailure` |
| `stage_schema_upgrade` | approved version; copy of old database → verified candidate | the original base does not change; `UnsupportedUpgrade, ValidationFailure` |

The continuation command creates new attempts through the normal C-05/C-06 path. Restoring an old copy protects the former rights; no copy reduces anchor. Transferring an installation to another Windows is not considered a normal DPAPI key recovery: this is a separate procedure for the owner.

Returning to a verified old copy is a separate recovery operation confirmed by the owner, not a regular database opening. C-13 prepares proposal with lost tail composition and unknown consequences; C-14 certifies the exact solution, C-05/C-03 perform the transition, C-10 secures the new log line and connection to the previous control point. The new `journal_id` does not erase the history of the previous line and does not reduce the era; you cannot automatically declare an old vertex to be the new current one. These are internal steps to the existing recovery contract, not a universal bypass of the integrity check.

Before being able to proceed, C-13 restores the record by combining the main base records, immutable reserve evidence, and intermediary observations. Duplicates are cut off by `logical_operation_id`, `reservation_id`, `observation_id`; the unknown expense holds the upper limit, the proven unsent is released only after the old performers are fenced off. If there is insufficient data for the safe balance of the limit, the result is a prohibition of new consumption, rather than zero consumption. The owner's explicit decision to change the limit remains an ordinary protected decision and does not rewrite the facts. A complete rollback of the entire machine beyond the stated protection: the consistency of three old copies does not in itself prove freshness.

**Evidence:** AC-L05, AC-I04A, AC-I11A, AC-I15, AC-I17. Old copy, unknown outcome, missing file, change of circuit version, lack of space.

Addition to C-13: [24, §9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md) adds reconciliation of unfinished uses and restoration of feedback links; [25, §5](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) - distinguishing between lost space in queue memory and saved transition; [26, §§2–3](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md) - combinations of failures. Old attempts are not started automatically, and unknown holds are not released upon restart.

### C-14 - Window, Trusted Solutions and Regular Drafts

**Location:** `dc-core/projection.rs`, `dc-desktop/{consent,input}.rs`, `ui/`. **Why:** Full owner control without hidden page power.

| Function | Who is calling; input → output | Changes and cancellations |
|---|---|---|
| `build_ui_projection` | C-05; resolved `StateView`, `ViewQuery` → `UiProjectionPage` | reading; one verified photo for overview, map, list and details; `ProjectionRefreshRequired` if it is impossible to continue the previous page |
| `request_native_consent` | window offer; number of the current action → display of canonical conditions | conditions are requested from the trusted path; `StaleDecision, UnknownAction` |
| `emit_owner_decision` | only verified native input; shown conditions and nonce → authenticated command | `AlreadyUsed, ConditionsChanged`; JavaScript can't trigger confirmation as a ready-made fact |
| `save_view_draft / restore_view_draft` | shell; valid form/navigation → saved draft | separate narrow storage; secret fields excluded, `UnsupportedDraftVersion` |

A saved authorization form is not a valid confirmation. After a restart, it is re-compared with the current conditions. Sensitive contents open separately; a regular view stores information about the view, source, destination, and validation.

The exact composition of the review, page boundaries and reasons for waiting are described in [23, §4–5](./23_EXTERNAL_CRITIQUE_AND_REFINEMENTS_20260908.md). `UiProjectionPage` contains image metadata and a limited fragment `UiProjection`; does not create a different state of work. Checks AC-OV-01 through AC-OV-03 and AC-OWNER-01 complement the previous scenarios. Waiting for the final acceptance of one work is not a global ban on other works.

**Evidence:** AC-L03A, AC-L15—L15B, AC-I14. Two jobs on one graph, stale screen, fake confirmation, draft after crash, keyboard and scale without losing state.

### C-15 - late Hermes reading bridge

**Location:** future `dc-bridge/`, stage 10 only. **Entrance:** owner selected snapshot/session and read permission. **Output:** imported evidence with origin, without transfer of rights and secrets.

The only open operation `import_selected_snapshot` passes input isolation, size limit, duplicate check and C-04/C-05. It does not open Hermes for writing and does not run its tasks. Waivers: `SourceChanged, UnsupportedExport, Oversize, DisclosureDenied`.

**Evidence:** AC-I02, AC-I16, AC-I16A, AC-I19. Repeated/partial import, malicious instruction, deleted bridge. Until the stage has started, there is no package or plugs.

### C-16 - Late Acceptance of Improvements and Versions

**Place:** future `dc-updater/`, stage 11 only. **Why:** allow proven improvement of the variable area beyond the power of the candidate.

| Operation | Input → Output | Limit |
|---|---|---|
| `evaluate_staged_candidate` | unchangeable candidate list, area and independent reports → `AdmissionDecision` | `ProtectedAreaChanged, MissingEvidence, AssuranceMismatch`; the candidate does not change his admission rules |
| `promote_verified_candidate` | current clearance, whole verified version and return plan → switching observation | `StaleAdmission, PartialVersion, RecoveryRequired`; no mixture of parts versions |
| `assess_post_admission` | C-05; follow-up plan, linked real-world outcomes and baseline comparison → follow-up follow-up, confirmation for scope, hold or return | `InsufficientExposure, PopulationShift, UnresolvedDelayedOutcome`; no outcome is not considered success |
| `prepare_scoped_rollback` | C-05; inclusion, reason, current dependencies and the whole base version → `ScopedRollbackPlan` | `RollbackScopeExpansion, RollbackCompatibilityFailure, UnaccountedEffect`; no general database rollback |

The full composition of plans, errors and consequences is given by [26, §§5.3–5.6](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md). `apply_scoped_version_switch` - General internal C-16 enable/return step with current plan check and attempt fencing, not a public tool. It returns `VersionSwitchObservation`, does not edit main/ledger/anchor. With `StaleRollbackPlan`, `VersionIntegrityFailure`, `StorageUnavailable` success is not issued. `evaluate_staged_candidate` additionally tests fixed design/latent sampling/regressions; `promote_verified_candidate` requires a follow-up plan. Tests AC-EVO-01—AC-EVO-10.

Checking map completeness is part of the test node, not a new executable feature of the model. She doesn't issue permits. The protective base is not automatically replaced; changing his contract again passes the owner's decision.

**Evidence:** AC-L13, AC-I15, AC-I16B, AC-I18. The candidate removes the beacon/check, changes the area, slips the file after the assessment; switching is interrupted on each phase.

## 5. Which names from an early draft do not become separate functions

| Early name | Exact place of duty |
|---|---|
| `commit_work_graph`, `commit_route_decision`, `commit_verified_result`, `authorize_resume` | C-05 command options and one C-03 transition |
| `reserve_execution_slot`, `reserve_effect_budget`, `reserve_resource` | one atomic resource wiring C-03; suitability check for C-09 |
| `schedule_retry`, `dispatch_attempt` | sentence C-06, stored command C-05, execute C-11 |
| `schedule_semantic_verification` | proposal C-08 and normal unit path C-05/C-06 |
| `send_registered_request`, direct file operations | private C-09 handlers accessible only after intent |
| `terminate_process_tree` in mediator | request to the sole owner of the C-10 |
| `record_owner_decision` in an arbitrary module | certified C-14 → C-02 → desired state owner |
| `propose_summary_candidate`, `verify_summary_candidate` | types of ordinary units and checks C-07/C-08 |
| `verify_assurance_map` | documentation builder/checker; not the right application |
| `get_secret` | absent; use C-09 or key data descriptor C-04/C-12 |

References to early names in documents 14–18 and the matrix are read as semantic stages unless explicitly stated otherwise. This table closes the previous ambiguities; the implementation does not create reverse aliases to match old text.

## 6. What will break if you change the contract without checking the connections

| Change | Mandatory consumers | What to check before adoption |
|---|---|---|
| field or unit state | C-01, C-03, C-05, C-06, C-13, C-14 | old data, all transitions, readiness and apparent reasons for blocking |
| model response format | C-02, C-07, C-08, connector C-09 | unknown fields, old forms, origin, free text and size |
| personal data class | C-04, C-07, C-09, C-14 | Vendor Disclosure, Archive, Log, Screen and Derivatives |
| review/area of ​​law | C-01, C-03, C-09, C-10, C-12 | sending and COMMIT at the same time as revocation; neighboring work continues the permitted path |
| connection identity | C-06, C-07, C-09, C-13 | new selection for new recipient, check form, cost and independence |
| resource states | C-01, C-03, C-06, C-09 | parallel bookings, suspense without double counting, repeat and restart |
| reduction rule | C-07, C-08, C-04 | denial, rights, incompleteness, links to the source |
| log event | C-03, C-10, C-13, C-14 | versions, reproduction, checkpoint, old reader suitability |
| new ability | C-09, C-11, C-12, C-16 | new fingerprint, no inherited rights, self-traversal of network/files |
| map appearance | C-14 | single StateView, keyboard, drafts, no new state source |
| edition of provision or condition of communication | C-01, C-03, C-05, C-06, C-07, C-09, C-13, C-14 | fixed result and its changeable admissibility, future cycle, expectation of an accurate revision, feedback even after its restoration |
| fairness of service/queue form | C-01, C-03, C-05, C-06, C-09, C-10, C-13 | stable candidate, multiple types of resources, command body and memory, overload, lost acknowledgment, independent stop |
| location of archival evidence | C-03, C-04, C-05, C-07, C-09, C-13, C-14 | immutable EvidenceId, disconnect separate from loss, abort transfer, other verified copy, cleanup only by right |
| verification method/hidden sampling | C-04, C-05, C-06, C-07, C-08, C-16 | exact key and denominators, test leakage, applicability of old results, real benefit after approval |
| way to return a modified version | C-03, C-05, C-07, C-09, C-10, C-13, C-16 | whole version, scheme, dependent projections, preservation of rights/expenses/committed actions and independent area |

Automatic dependency search shows call locations. It does not prove semantic independence: the developer specifies the affected contracts and the reason for excluding the others. If the experience discovers a new connection, it first replenishes this card and the check, then the change is accepted.

## 7. Conditions for code approval

1. A significant function has a C-contract and a real caller.
2. The new behavior is covered by the requirement from 07/21 or by a specific engineering acceptance condition.
3. Transition, general expense and significant event do not have a second writer.
4. A pure function does not start I/O for its own beacon.
5. The failure is not hidden by a general error; unknown outcome and incompatible version are stored explicitly.
6. The implementation does not add empty generality, proprietary cryptography, or direct circumvention of the facility.
7. Checks correspond to the changed risk; having multiple tests is no substitute for bounds checking.
8. After the change, the contract, consumers, acceptance scenarios and recording of the test result were updated.

## 8. What remains to be proven during implementation

Contracts specify required behavior. They do not prove that the chosen Windows mechanism for isolation, key retention, fencing, and file replacement implements it correctly. Before real data, we need reproducible experiments from document 20 and the registry in the final check. The error of such experience changes the technical mechanism while maintaining the accepted user boundary.

Zero known errors after testing is a useful result. A promise of the absence of any future errors from one reading of a document is not engineering proof.
