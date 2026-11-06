# Deep Create: beacons, log, stop and restore

**Date:** 2026-09-07  
**Status:** operating agreement of the first version  
**Main boundary:** the beacon proves a fact within the system; notice is addressed to the owner. These are different things.

---

## 1. One story, not three magazines

A significant event, a mandatory beacon and a change in subject state are recorded in one transaction of the main database.

Not created:

- a separate, mutable “truth log” file;
- a separate beacon table that the function may forget to update;
- own history of the interface state;
- a log of the performer, capable of confirming an action without an intermediary.

Complete requests, responses, files and detailed traces are stored as encrypted evidence. The main event contains only secure information and a link.

The exception is not hidden: the guard and the intermediary retain their own minimum protective facts in `authority.anchor` and `guard.db`, since they must act when the main base is inaccessible. These are fencing, admission and surveillance receipts, not a second count/history of targets. The main base accepts them later idempotently by `(origin_store_id, origin_event_id)`; The time of the fact is stored separately. Failure to write to the main log prevents new normal actions, but does not prevent admission from being closed or processes terminated. An unconfirmed protective fact cannot be passed off as an already existing `EventSeq`.

---

## 2. Event wrapper

```rust
struct StateEvent {
    event_seq: EventSeq,
    journal_id: EntityId,
    event_id: EntityId,
    transaction_id: EntityId,
    event_kind: EventKind,
    event_schema_version: SchemaVersion,
    aggregate: AggregateRef,
    causation_id: Option<EntityId>,
    correlation_id: EntityId,
    actor: ActorRef,
    rule_id: RuleId,
    previous_version: Option<RowVersion>,
    new_version: Option<RowVersion>,
    authority_epoch: AuthorityEpoch,
    run_generation: RunGeneration,
    recorded_at_utc: DateTimeUtc,
    observed_at_utc: Option<DateTimeUtc>,
    safe_payload: EventPayload,
    evidence_id: Option<EvidenceId>,
    data_class: DataClass,
    previous_event_mac: [redacted],
    event_mac: [redacted],
}
```

The order within the line is determined by `event_seq`, not time; the event reference includes `journal_id`. A new line is only possible under a confirmed contract for the restoration of document 14, §10, with a preserved connection to the previous control end. `observed_at_utc` separates the time of an external fact from the time of its recording. Form fields are closed; arbitrary string map is not allowed.

The event is never updated. The correction creates a new compensating event.

### 2.1. Integrity chain and control end external to the base

The MAC chain confirms the connectivity of the presented tail with the key saved, but does not itself detect a return to the previous whole tail. The guard stores a separate `journal_checkpoint { installation_id, journal_id, event_seq, event_mac, transaction_id }` in its SQLite database, not in the backup copy of the main database. The MAC algorithm, canonical representation, key version, and test vectors are pinned into a single version; The sensitive value fingerprint key is not reused for the log. This is an application of a proven cryptographic library, not a new secure anchor format.

The order for an accepted subject transition is: main SQLite transaction commits state and event (`DB_COMMITTED_PENDING_CHECKPOINT`) → `record_journal_checkpoint` guard compares expected previous end and stores new one (`CHECKPOINT_CONFIRMED`) → “accepted” response, publishing accepted snapshot, and dependent actions are allowed. These are two sequential transactions, not one cross-database transaction. The order `EventSeq` does not replace the subject dependency versions.

A failure before the second phase leaves a saved but not yet confirmed tail; it does not disappear or be reapplied. Recovery compares the chain, transaction and previous checkpoint in a limited mode. The expected whole tail can only be confirmed by a registered local reconciliation procedure; non-confirmation is shown in the report, and old units are not launched. The main base is shorter than the checkpoint, another MAC at the same position, an unknown installation identity or an arbitrary tail means a failure, not an automatic “catch up with the maximum”. The guard does not assert the truth of the model; it only records the sequence previously accepted by the control process. The coordinated rollback of all storages and machine keys is not covered by this mechanism.

---

## 3. Catalog of mandatory events

### 3.1. Owner and purpose

| View | Created when | Proves |
|---|---|---|
| `owner_event_received` | home node took action | source and exact original words |
| `owner_event_duplicate` | I received the same number and fingerprint again | lack of second execution |
| `owner_event_rejected_stale` | expected version has changed | ban decision on old screen |
| `deep_mode_proposed` | rule suggests going into deep work | reason for the proposal, but not agreement |
| `deep_mode_changed` | trusted event took transition | who chose which version |
| `objective_proposed` | machine interpretation created | difference between proposal and acceptance |
| `objective_accepted` | the owner or the exact previous decision was made by the editors | current purpose and commitment |
| `objective_superseded` | new edition accepted | why old results may be out of date |

### 3.2. Graph and units

| View | Proves |
|---|---|
| `work_graph_proposed` | split origin |
| `work_graph_rejected` | broken obligation, cycle or limit |
| `work_graph_committed` | accepted final version of the graph |
| `work_unit_ready` | fulfilled dependencies and no blocking boundary |
| `work_unit_blocked` | exact reason and area of ​​blocking |
| `work_unit_cancel_requested` | cancellation source |
| `work_unit_cancelled` | actual fencing attempts |
| `hypothesis_branch_created` | discriminative observation, limit and stopping condition |
| `hypothesis_branch_merged` | saved conflicts and negative results |

### 3.3. Context and mandate

| View | Proves |
|---|---|
| `context_need_plan_created` | questions, search scope and spaces |
| `context_projection_created` | exact sources, reasons and recipients |
| `data_disclosure_denied` | applied data class rule |
| `prompt_bundle_compiled` | versions and prints of seven layers |
| `instruction_injection_signal` | signal source and real limitation; not a verdict on the contents |
| `summary_candidate_created` | source range and correspondence table |
| `summary_candidate_verified` | losses found and verdict |

### 3.4. Connect and try

| View | Proves |
|---|---|
| `connection_candidates_filtered` | reasons for hard dropout |
| `connection_selected` | exact model, supplier, input, connector and rule |
| `execution_slot_reserved` | concurrency and resource reservation |
| `attempt_started` | process, generation, commission and reserve |
| `worker_heartbeat_observed` | life of one process |
| `attempt_response_sealed` | saving the exact answer until parsing |
| `attempt_failed` | closed failure class |
| `retry_scheduled` | attempt number, longevity time and limit remainder |
| `retry_exhausted` | exhaustion of machine recovery path |
| `late_result_rejected` | old version, generation or right |

### 3.5. Rights and actions

| View | Proves |
|---|---|
| `permission_requested` | exact object, recipient, data, term and limit |
| `permission_issued` | trusted decision and imprint of conditions shown |
| `permission_delegated` | proven narrowing of the parent |
| `permission_denied` | owner failure or rules |
| `permission_expired` | reaching deadline |
| `permission_revoked` | stable point F of the fence of one right, and not the moment of pressing the button |
| `scope_fence_advanced` | fencing of precise work/unit without revocation of unrelated rights |
| `authority_epoch_advanced` | mass protection of all old rights |
| `action_proposal_canonicalized` | present registered type of action |
| `action_intent_committed` | stable intention of the main base before transfer to the intermediary; his separate acceptance into the guard and admission are still required |
| `action_preflight_passed` | recheck immediately before execution |
| `action_dispatch_started` | saved fact of access to OS handoff; without outcome allows for uncertainty |
| `action_observed` | real success or confirmed failure |
| `action_outcome_unknown` | ban on blind replay |
| `action_reconciliation_started` | type and right of reconciliation |
| `action_reconciled` | confirmed result and proof |

### 3.6. Verification and completion

| View | Proves |
|---|---|
| `result_packet_parsed` | correspondence of form and origin |
| `result_packet_rejected` | exact broken contract |
| `result_submitted` | the package is saved and associated with the attempt; it has not yet passed semantic acceptance |
| `verification_started` | criteria and required independence |
| `verification_completed` | actual independence, verdict and gaps |
| `result_committed` | accepted new version of the state |
| `completion_candidate_created` | fulfillment of internal conditions |
| `owner_completion_accepted` | final decision of the owner |
| `owner_completion_rejected` | reason and new open commitment |

### 3.7. Storage, stop and version

| View | Proves |
|---|---|
| `storage_warning_entered` | volume, remainder, forecast and threshold |
| `storage_protection_entered` | blocking new changes and actions |
| `emergency_reserve_consumed` | emergency use of physical stock |
| `storage_protection_cleared` | checking the database and restoring stock |
| `stop_requested` | the main story took on the area and appearance of a stop; this is not confirmation of the termination of processes |
| `prepared_stop_requested` | area, owner words and cancel window |
| `prepared_stop_cancelled` | cancellation before termination and re-inspection |
| `prepared_stop_fencing_started` | ban on new units and actions |
| `emergency_stop_started` | direct received command to the guard and the exact area; not yet proven stop |
| `process_tree_terminated` | observable tree completion |
| `stop_confirmed` | protection of all intermediaries and known processes |
| `stop_unconfirmed` | unproven completion/fencing with exact holding condition |
| `startup_recovery_started` | new generation launch |
| `recovery_restricted` | reasons for limited mode |
| `recovery_report_ready` | full reconciliation result |
| `resume_waiting_owner` | prohibit automatic continuation |
| `resume_prepared` | current continuation plan after reconciliation; the plan itself does not trigger attempts |
| `resume_authorized` | explicit command and recheck |
| `backup_created` | agreed photograph and list of objects |
| `backup_restore_verified` | real trial recovery |
| `version_staged` | separate whole new version |
| `version_admission_rejected` | broken check or list of files |
| `version_promoted` | indivisible inclusion of the entire version |
| `version_rolled_back` | whole refund without reviving rights |
| `resource_bound_breached` | observed consumption is higher than proven reservation and prohibition of new consumption |

### 3.8. Addition after end-to-end verification of the code map

Below are names from Matrix 21 that cannot be left undescribed. Said contract C-* of document 19 owns the fact or decision; the recording of an accepted main story event still passes C-05/C-03. A pure evaluation event occurs only when its meaningful result is saved, not every time a function is called. The word "proves" means evidence of a specified observation with origin; it does not promise true model output, unlimited security, or access to the provider's internals.

| View | Owner; required content |
|---|---|
| `state_transition_committed` | C-03/C-05; transaction number, expected and new editions, subject events of the same transaction; not separate recommitment |
| `app_started` | C-13; installation identity, version and new generation; not automatic continuation |
| `scheduled_run_recovered` | C-13/C-06; the deadline saved, the reason for the delay and the state of waiting; not the fact of sending |
| `workspace_binding_changed` | C-05/C-09; certified new area and decision made; old rights are not transferred silently |
| `evidence_sealed` | C-04; opaque number, size, origin and verified integrity of the encrypted object |
| `evidence_scope_granted` | C-09; exact scope, subject, term and parental permission to read; not right of disclosure to any recipient |
| `sensitive_view_requested` | C-14/C-09; class, purpose of display and outcome of admission without sensitive text |
| `sensitive_match_recorded` | C-04; comparison purpose and protected fingerprints without original values ​​|
| `provider_request_adapted` | C-07; converter version, layer matching and secure fingerprint of the final unclassified body |
| `dispatch_preconditions_checked` | C-09; accurate dispatch, verified versions/receiver/reserve/fences and verification result; non-reusable pass |
| `budget_limit_reached` | C-03/C-06; limit number, window, confirmed expense, reservation and unknown balance without double billing |
| `connection_registered` | C-05/C-06; edition of the complete connection certificate without a key |
| `connection_admission_tested` | C-08; Proven Configuration, Tests, Applicability and Limitations |
| `connection_shadow_observed` | C-06/C-08; the result of an authorized comparative attempt, its separate expense; it does not accept the main result |
| `fallback_rejected` | C-06; candidate and exact non-matched replacement requirement |
| `delegation_denied` | C-09; parent, requested narrowing and violated condition; without issuing a subsidiary right |
| `action_path_denied` | C-09; unknown or prohibited type of action, without execution of its contents |
| `file_analysis_completed` | C-07/C-08; analyzing the purpose and dependencies of a specific version of a file and checking it; The model's word "finished" is not enough |
| `file_target_verified` | C-09/C-12; Identification of the actual open object and scope boundaries, not just path strings |
| `file_target_denied` | C-09/C-12; reason for unconfirmed object, substitution or leaving the area |
| `verification_independence_failed` | C-08; required and actually confirmed independence, holding condition |
| `verification_limit_reached` | C-08/C-06; exhausted finite number of checks/fixes and open obligations |
| `adaptation_profile_tested` | C-08/C-16; version of the setting, a set of closed tasks, comparison and identified degradation; not a promise for all tasks |
| `assurance_map_verified` | C-16; exact version of the responsibilities card and the results of mandatory candidate checks |
| `assurance_map_mismatch` | C-16; lost duty, check or beacon that disqualifies a candidate |
| `worker_orphaned` | C-10/C-06; verified attempt owner loss and fencing; a pause in response is not sufficient in itself |
| `ui_projection_built` | C-14; published subject edition for map/list; pointer movement and animation do not generate events |
| `owner_notification_emitted` | C-05/C-14; merged incident number, significant change and channel; without re-notifying the unchanged state |
| `restore_lineage_authorized` | C-13/C-14/C-10; confirmed solution, old and new lines, verified copy and connection with the previous control point |
| `budget_recovery_blocked` | C-13/C-03; missing evidence of expenditure and withheld limit; no line does not mean zero consumption |

Receipt `record_journal_checkpoint` remains a security guard entry rather than the new mandatory `StateEvent` in response to each receipt. The list above does not change the two-phase order of §2.1 and does not allow two processes to simultaneously own the same subject table.

---

### Update 2026-09-08: collaboration, growth and verifiable development

The exact manufacturers and conditions are specified in documents 23–26 and their C-passports. The table combines names, does not create another log. Observation before acceptance or when the base is not available does not receive a fictitious `EventSeq`; pure calculations and normal screen updates do not record every comparison. A recurring cause is combined into an incident under §6. The beacon does not necessarily notify the owner or trigger the LLM.

| View | Proves |
|---|---|
| `root_work_link_committed` | The exact edition of interworking communication has been adopted; rights not transferred |
| `root_work_link_cancelled` | Cancellation of connection saved without deleting its history |
| `result_publication_committed` | The verified result is provided within the system; not accepted by the owner |
| `publication_eligibility_changed` | Changed the validity of the specified result, separately from its bytes |
| `cross_work_dependency_blocked` | The specific consumer expects the exact basis |
| `cross_work_dependency_revalidated` | Re-check removes only the specified reason for waiting |
| `cross_work_dependency_changed` | The read binding has changed; Affected consumers identified by number |
| `workspace_use_reserved` | The use of this area is withheld; this is not right |
| `workspace_use_released` | There is a proven reason to release the hold |
| `workspace_conflict_detected` | The expected basis or the real object did not match |
| `workspace_merge_proposed` | A separate unit for combining precise fundamentals has been proposed |
| `workspace_revision_observed` | Recorded observation of a real tree, not automatic acceptance |
| `workspace_reconciliation_blocked` | The outcome of the application is not known enough to release or repeat |
| `scheduling_policy_changed` | The verified version of the service policy has been accepted |
| `root_wait_turn_registered` | Pending move order preserved |
| `root_turn_dispatched` | The move is confirmed by the certified fact of transfer, not by intention |
| `protected_capacity_claim_created` | A limited set of held resources has been set for a pending turn |
| `protected_capacity_claim_withdrawn` | The hold was lifted on the specified basis without apparent completion |
| `commit_backpressure_entered` | Queue limit reached; new regular starts limited |
| `commit_backpressure_left` | Proven return below lower overload threshold |
| `queued_command_rejected_stale` | Expected command rejected due to current significant conditions |
| `capacity_release_unconfirmed` | No proof of empty capacity |
| `calibration_plan_fixed` | A test plan for a specific verification method has been recorded |
| `calibration_evidence_recorded` | The initial observations, markings and calculations of the method have been accepted |
| `method_qualification_changed` | The qualification of the method in the exact area of ​​applicability has been changed |
| `calibration_invalidated` | Previous certificate no longer applicable for a specific reason |
| `evaluation_holdout_exposed` | The test is revealed; hidden status is not saved fictitiously |
| `evaluation_label_revised` | The reference marking has been changed while maintaining the same basis |
| `evolution_evaluation_plan_fixed` | The plan for assessing the candidate before comparing him is fixed |
| `evolution_candidate_evaluated` | There is a conclusion about the evaluation of a specific version, not the fact of inclusion |
| `evolution_candidate_promoted` | Received observation of inclusion of verified whole version |
| `post_admission_observation_recorded` | Saved linked real outcome after admission |
| `post_admission_review_completed` | There is a conclusion from the actual observation and the original plan |
| `evolution_scope_paused` | Exact affected development area suspended |
| `scoped_rollback_requested` | Suggested return of specific mutable version |
| `scoped_rollback_completed` | The return of the version has been confirmed without rollback of rights and consequences |
| `scoped_rollback_failed` | Return not confirmed; the reason is not hidden behind the imaginary success |
| `derived_projection_invalidated` | The derivative selection requires revision; source saved |
| `ui_projection_refresh_required` | A new consistent view snapshot is needed; derivative diagnostics |
| `evidence_location_verified` | Copy checked before canonical link changed |
| `evidence_location_committed` | New location accepted by regular transaction and checkpoint |
| `evidence_location_unavailable` | Location not available; this is not proof of the loss of the item |
| `evidence_relocation_interrupted` | The transfer is interrupted at a known phase; not declared successful |

## 4. Detailed factual trail

For each invocation of a model, tool, program, file, and external service, an observable trace is stored:

- exact type of operation;
- attempt and unit;
- secure login fingerprints;
- beginning, end and state;
- exit code or network total;
- actual consumption;
- created and modified materials;
- links to cleared evidence.

It does not contain the hidden reasoning of the model. The full sensitive request and response are in an encrypted archive and are issued only to a limited check or to the owner in a separate action.

The trace is not a second truth. Any action with a consequence still has `action_intent_committed` in the main sequence.

---

## 5. Confirmation of life without meaningless questioning

Each type of performer has a measured profile:

```rust
struct LivenessProfile {
    executor_kind: ExecutorKind,
    heartbeat_interval: Duration,
    warning_after: Duration,
    health_probe_after: Duration,
    orphan_after: Duration,
    graceful_stop_deadline: Duration,
    forced_stop_deadline: Duration,
    measurement_record_id: EntityId,
}
```

One term for all models and programs is prohibited.

Skip confirmation of life:

1. does not mean that the action did not occur;
2. stops issuing new consequences to this process;
3. launches cheap verification of the process and intermediary;
4. after the measured period, marks the attempt as lost;
5. creates a reconciliation of open intentions;
6. Does not send repeated messages to the owner with each survey.

Checking the availability of a supplier is carried out only when a long-term period has reached or a ready-made unit that needs it becomes available.

---

## 6. Incidents and notifications

### 6.1. Association

The incident has a safe imprint:

```text
класс ошибки
+ затронутая часть
+ коренная причина
+ требуемое действие владельца
```

Repeat increments the counter and updates the latest time. It does not create a new card and does not wake up the owner.

### 6.2. When the owner receives the message

Only if:

1. a decision is required that the system does not have the right to make;
2. the protective layer has detected a critical threat, loss of integrity or impossibility of stopping;
3. The final result of the main work is completely ready.

Temporary `400`, `429`, `502`, `503`, timeouts, connection cooling and repeat of an already known reason are processed by the system until the established path is exhausted.

### 6.3. One message contains

- what exactly is blocked;
- what continues to work;
- what automatic methods have already been tried;
- one precise solution required from the owner;
- consequences of each choice;
- link to details.

Once sent, the message is not repeated unless the cause, scope, or required solution has changed significantly.

---

## 7. Prepared stop

```text
requested
→ cancellation_window
→ fencing
→ reaching_safe_points
→ persisting
→ confirming_processes
→ stopped
```

### 7.1. `requested`

The owner's words, area, expected version, and affected units are saved. New external consequences are immediately prohibited.

### 7.2. `cancellation_window`

The initial value is five minutes, but this is a testable setting. Cancellation is only possible until `fencing`. After cancellation, rights periods, limits, files and sources are checked again; nothing “comes to life” from the previous screen image.

### 7.3. `fencing`

New units are prohibited except for pre-registered pins, returns and logs. Each process is sent one request to reach its own safe point.

“Fixing” here means to seal a partial material and state for restoration, not to accept a new semantic result after enclosure. "Return" is only allowed as an already resolved safe local step with a verified object; it is not a general permission to undo external effects or overwrite an owner-modified file. If a safe point without a new consequence cannot be reached, suspense is maintained and termination is applied rather than rights are expanded.

### 7.4. `reaching_safe_points` and `persisting`

The process either confirms the safe point or its measured time expires. Partial results are sealed; open actions receive an observable state or an unknown outcome.

### 7.5. `confirming_processes`

State `stopped` occurs only after affected attempts have been confirmed to be fenced and their trees have been observed to complete. The general `run_generation` installation is not raised to stop one branch: its area `scope_fence` is applied. An unreachable process does not have to respond verbally, but its termination and the closing of paths must be proven by the guard. If there is no evidence, `stop_unconfirmed` remains.

---

## 8. Immediate stop

The team goes their own way directly to the watchman.

1. After the owner's final clear confirmation, the guard closes admission of the affected area to new entries.
2. `establish_fence` executes the protocol `AdmissionFence` of document 17, §12.1: steadily increases the fencing of the main work/unit/right, and the global era - only when the entire installation is stopped or a general protective emergency.
3. The mediator stops new activities and the issuance of rights to the fenced area; unrelated work does not lose authority due to individual command.
4. The job objects of all affected processes are terminated.
5. The control process rejects late acceptance of results through the same gate, but retains the facts of actions and expenses that have already occurred.
6. Open intentions are translated into reconciliation.
7. The minimum security event is recorded in the anchor transaction; Emergency stock is used for transfer to the general journal and report.
8. The screen shows "stopped" only after the guardrail is observed.

If the control base is temporarily unavailable, persistent guard fencing still renders the old rights of the affected area unusable. After launch, the base catches up with the protective fact. If it is not possible to write the anchor itself, the guard keeps admission closed, stops the processes and shows that the stable stop is not confirmed; You cannot promise saved feedback at the next start. In any case, a new launch begins closed and requires reconciliation, rather than continuing from the old copy.

`stop_confirmed` means "local paths fenced, known affected processes terminated, late acceptance prohibited." It does not mean “the external service did nothing” or “the money was returned.” What has already been transmitted up to point F remains `succeeded`, `failed_confirmed` or `outcome_unknown`; the unknown outcome and retained value are visible separately even next to stopped work. Compensation does not start automatically.

---

## 9. Recovery

### 9.1. Order

1. Check the entire version of the program.
2. Seize the only lease of the control process.
3. Obtain from the watchman a complete photo of the protected anchor.
4. Ask the guard to indivisibly increase `run_generation` and issue a new `AuthoritySnapshot`.
5. Open the database and check the scheme, transactions and chain of events.
6. Compare the watchman's anchor with the last values ​​seen by the main base and the mediator; older meanings have no power, and any discrepancy freezes previous rights.
7. Reject messages from legacy processes.
8. Check all references to evidence.
9. Compare the intentions of the main base and `guard.db` of the intermediary.
10. Divide outcomes into confirmed success, confirmed failure and uncertainty.
11. Check all affected volumes and emergency stocks.
12. Write down the recovery blocking boundary that has already been in effect since the start.
13. Prepare a report.
14. Wait for an explicit “continue” command.

Admission is closed even before step 1. All listed points are a local restoration procedure, not a continuation of previous executive units. Until the anchor, log, keys, place and old trees are checked, neither the ready queue, nor the timer, nor the restored screen can start execution. An unknown external outcome is, if possible, determined only by authorized safe reconciliation; a regular HTTP request does not become free or non-transmitting from the word “diagnostics”.

### 9.2. After the command "continue"

The target, unit version, real files, `DependencySnapshot`, freshness of sources, connections, permissions, limits, open actions and working collections are re-checked. The command applies to the report and area shown; a changed report requires a new decision made. Only the verified recovery limit is removed, not automatically other prohibitions/unknown outcomes. New current generation attempts are being launched. The old process and the old law are not revived; “continue” does not give new permission to an unacknowledged external consequence.

A missed scheduled launch is a new limited unit of reading, reasoning, and freshness checking under the previously adopted rule L-05a. It is not a continuation of the failed attempt and does not catch up with the consequences. It can be created and allowed only after checking the general security boundaries, with new inputs, its own resource accounting and valid data transfer rights. Calling the old `attempt_id` a “missed launch” to bypass L-05 is prohibited.

---

## 10. Protecting the place

Archive growth and disconnected media are regulated by [23, §6](./23_EXTERNAL_CRITIQUE_AND_REFINEMENTS_20260908.md): the location is separated from the identity of the evidence, the copy is checked before switching, the release of the old copy is not permitted by the very fact of copying. These rules are mandatory along with the thresholds below; The transfer does not replace the backup copy and does not justify the disappearance of the source.

Each volume is controlled where:

- main base, `guard.db` intermediary and watchman's anchor;
- archive of evidence;
- time domain;
- user's working copy;
- backup copy;
- system temporary area of ​​the library used.

All child processes get managed by `TEMP` and `TMP` on the explicitly selected volume.

Before recording, the following is checked:

```text
свободное место
≥ максимум(
    защитный порог,
    верхняя оценка записи × запас неопределённости + физический аварийный резерв
  )
```

The initial values from the architecture - warning 15 gigabytes or 10 percent, protection 8 gigabytes or 5 percent, reserve 1 gigabyte - remain testing, not forever.

The emergency reserve must be an actual dedicated, uncompressed, unsparse file on each critical volume, rather than a single global file on another disk. The catalog and opening are checked against the real object; the executing part cannot spend it. If necessary, only the checked part is released for the stop and report events. This is a margin and not an absolute guarantee against disk failure, user quota, contention, or controller failure. The write error is more important than yesterday's free space indicator. Work does not continue until the reserve is created and verified again.

At the protective threshold, only diagnostics, verifiable transfer, database and reserve restoration are allowed. Canonical evidence is not automatically removed.

---

## 11. Backups

They differ:

- quick coordinated local image;
- a verified copy on another physical medium.

The readiness of the copy means a successful test recovery to an isolated folder with verification:

- databases and diagrams;
- chain of events;
- all selected prints;
- decoding of selected objects;
- absence of open secrets;
- impossibility of returning old rights;
- compatibility of the entire application version.

Simply copying a folder does not count as a verified backup.

---

## 12. Failure checks

1. End the process at each transaction transition.
2. Turn off the power after intention and before observation.
3. Kill the control process while the intermediary is alive.
4. Block the intermediary and check the work of the guard.
5. Send a late response from the previous `run_generation`.
6. Restore the database with a smaller epoch.
7. Set the clock back.
8. Fill in each volume used separately.
9. Use up the emergency reserve and check the continuation prohibition.
10. Damage the derived pointer and restore it.
11. Delete the canonical object with the reference and get restricted mode.
12. Create a lone fully written object without a reference and safely parse it.
13. Repeat one temporary error a hundred times and get one incident.
14. Leave the work in cooling until restarting and do not lose the repeat schedule.
15. Cancel the prepared stop before the barrier and re-check your license.
16. Try to cancel it after the fence and get rejected.
17. A stuck performer cannot prevent an immediate stop.
18. The old interface card cannot show `stopped` before actual confirmation.
19. Terminate the process between main COMMIT and checkpoint: there is no received ACK/new dispatch, the tail is saved and visible in recovery; a consistent rollback of the main base below the checkpoint is detected.
20. Stop one of two independent works: the second does not lose rights; General average, on the contrary, protects both.
21. Stuck inside the gate and when canceling I/O: `stop_confirmed` does not appear without proven completion; a request that has already passed remains in reconciliation and resource accounting.
22. Damage/fill exactly the anchor volume when the main base is live: new actions are closed, the absence of a durable fence is not hidden.
23. After restart, neither the old reasoning/read units nor the saved retry timer are resumed; the new missed launch is subject to a separate contract L-05a.
24. The security receipt for a fallen core is transferred exactly once with the original time, and does not create a fictitious previous EventSeq.

---

## 13. Prohibited implementations

- a message to the owner for each beacon;
- survey of an expensive model for the sake of the health of the process;
- repeated card of one mistake;
- one common life span for all performers;
- field `last_clean_shutdown` as the only proof of integrity;
- automatic continuation after recovery;
- emergency stop via model or frozen control loop;
- a sparse file instead of a physical emergency reserve;
- control only disk `C:`;
- backup without trial recovery;
- a technical journal with secrets or hidden reasoning;
- a separate interface history that may diverge from the base.
