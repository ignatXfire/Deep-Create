# Deep Create: resource allocation and commit queue

Date: 2026-09-08. Status: clarification of the technical design prior to implementation. No code, running scheduler, or measured throughput is claimed by this document.

Area: points 2 and 3 of external criticism - promotion of several RootWorks with scarce resources and a limited path for teams to a single writer. The document complements 13, 14, 16, 17, 18, 19 and 20; does not supersede their responsibilities, modes of exploration, depth, independent development of the permissible area and the final acceptance of RootWork by the owner.

## 1. Decision and boundaries

1. The choice of useful content of the work remains LLM-centric. Models offer plans, explorations, branches, ways to test and improve. The deterministic policy determines whose already valid unit receives the scarce resource; it does not evaluate the truth of the answer and does not replace semantic testing.
2. Inter-worker selection has a versioned source policy `FairSchedulingV1`: comparison of flow rate in one physical dimension, limited owner weights and a separate aging rule. Monetary price, term, number of dependent nodes and “utility” do not add up to an arbitrary score.
3. There is no strict infinite foreground priority. A long-waiting valid RootWork gets a protected next move. The system continues to work independently until the resources needed for this move are freed.
4. The only subject writer is saved: `C-05 → C-03`. C-05 introduces a limited internal queue rather than a new process, second truth log, or `CommitManager`.
5. `stop/revoke` are not elements of this queue. The independent C-10/C-09 acts first, then the general journal accepts the defensive fact. An emergency stop does not call the model and does not occupy an LLM slot.
6. Acceleration of commit through batching is initially disabled. The queue and metrics are introduced before the bottleneck assumption; specific optimization is allowed after measurements and proof of order conservation.

## 2. Existing responsibilities remain

| Agreement | What remains his only responsibility | Clarification here |
|---|---|---|
| C-01 | pure subject rules; `evaluate_reservation`; without clock, network and base | Policy Form Validation, Service and Capacity Comparison Integer Arithmetic |
| C-03 | `commit_transition`, strings, events, deduplication and the only `resource_ledger` | atomic acceptance of the next move update along with the reservation; no public separate write-off |
| C-05 | `handle_command → decide_transition → commit_transition`, the only semantic acceptance | limited entry, select next command, wait for checkpoint and exact `CommandOutcome` |
| C-06 | `select_ready_units`, `filter_connections`, `rank_connections`, limited repeats | interwork choice with reasons and protected progress; the result remains a proposal |
| C-09 | rights, intentions, sole dispatch and observations | current checks directly from `D`; stable outcome is not lost when the core queue is full |
| C-10 | admission, point `F`, generation, independent stop and checkpoint | out-of-turn security entrance C-05; short anchor records do not wait for a stream of regular checkpoint requests |

`C01/C03/C05/C06/C09/C10` without a hyphen means the same passports C-01/C-03/C-05/C-06/C-09/C-10 of document 19, and not additional components. The internal function names below do not expand the public surfaces of these passports.

## 3. What exactly is distributed

### 3.1. The abstract "Astra slot" cannot be distributed

`ConnectionIdentity` saves all fields of document 16: supplier, endpoint, account, login method, exact `model_id`, reasoning option, adapter version and request forms. Two connections with the same model sign may have different quotas, admission quality, disclosure recipient and health.

Before choosing a move, the C-06 receives:

- version readiness of WorkUnit and `DependencySnapshot`;
- a positive set of exact valid connections or reasons for its absence;
- current installation/provider/connection/session/root/branch/unit limits;
- confirmed health, observation period and the next permissible moment of verification;
- the upper costs of the actual limited request;
- input capacity C-05 and reserved capacity for completing the attempt.

A strict filter of quality, independence, data disclosure, abilities, rights, tariff and upper budget precedes fairness. A busy but healthy allowed slot means contention; `Disabled`, `AuthRequired`, `QuotaExhausted`, unconfirmed quality or unknown upper price means another blocking fact. An unavailable model is not considered free just because its name is on the registry.

`fairness-eligible` means: there is a ready unit and a valid request that can be executed after freeing the already occupied end slots. This does not mean that all slots are free right now. A non-refundable cash limit, a closed fence, an unavailable connection, or a lack of required proof excludes a job from this category with a separate reason.

### 3.2. Unit of service and separate accounting of money

To make the first policy fair, only the time of the busy executive resource is used: `slot_milliseconds`. Upper move cost:

```text
U_service = parallel_slots_requested × enforced_attempt_wall_time_ms
```

This is an integer. The wall time constraint actually applies to the attempt and the observed tree; the estimate “the model usually responds in 8 seconds” is not an upper limit. Failure to demonstrably release a slot after exceeding the deadline creates `capacity_unknown` and removes that slot from availability; The timer itself does not release it.

Money in minimum monetary units, input/exit tokens, requests, place and time of the corresponding limit continue to be counted separately in C-03 under document 14, §4.17. `slot_milliseconds` does not become an exchange rate for money for quality. For a subscription connection, zero additional cost does not mean zero occupancy.

For each operation, the derived maintenance record references `logical_operation_id`, `attempt_id`, `reservation_id`, start/end observations, and the upper limit. Until proven completion, `U_service` is taken into account; upon completion, it is replaced by the observed time rather than added to it. If the end is unconfirmed, an upper estimate and a separate sign of uncertainty remain. Violation of the upper time is reflected in fact and increases the recorded service; it is not cut for beautiful statistics.

Resource postings preserve `reserved(U) → unknown(C,R) → settled(C)`: changing the state of one item does not create a second U. Model change, repeat, cancel, fairness debt and restart do not release unknown monetary value. If the observed count is higher than U, C-03 stores the fact, closes further expense and issues `resource_bound_breached`.

## 4. Initial deterministic policy `FairSchedulingV1`

### 4.1. Options

The following values are a suggested starting configuration for artificial tests, not a measured optimum, and not a promise of latency. The change is formalized by the setting version via C-05, with the unchangeable reason and check C-01.

| Field | Initial value | Meaning |
|---|---:|---|
| `policy_version` | `1` | completely defines the rules of comparison |
| `service_window_ms` | `300000` | window of completed service observations |
| `root_aging_ms` | `60000` | after this, the protected interworking passage is activated |
| `unit_aging_ms` | `120000` | protection of the old unit inside the selected work |
| `weight.foreground` | `4` | trusted foreground work |
| `weight.deadline_critical` | `3` | working with a trusted deadline and an explicit class |
| `weight.normal` | `2` | regular work |
| `weight.background` | `1` | background work |
| `weight.exploration` | `1` | research and search for improvements |
| `attempt_wall_time_cap_ms` | `300000` | one-move starting ceiling for test profile |
| `protected_claims_max` | `1` | one protected installation move at a time |
| `claim_drain_margin_ms` | `5000` | diagnostic stock after the upper limit of interfering attempts |

Long-term studies are not limited to five minutes: this is the ceiling of one limited attempt of the selected initial profile, after which the next unit with stored evidence is possible. A longer indivisible attempt requires a different proven version of the profile; It is forbidden to artificially trim it and pass off an incomplete answer as a high-quality one. All times given are reported as `ClockObservation` rather than read by a pure function.

Neither LLM nor the `priority` field in the untrusted result assigns a foreground. Urgency is not inferred from the volume of the text. There is no separate global class “all verifier above all works”: the verification receives a resource in the budget of its RootWork. Local recovery and stop are not included in any of these LLM classes.

### 4.2. Selection data

```text
SchedulingPolicy:
  policy_version, effective_from_event
  service_window_ms, root_aging_ms, unit_aging_ms
  weights[closed WorkPriority], attempt_wall_time_cap_ms
  protected_claims_max, claim_drain_margin_ms

RootScheduleState:
  root_work_id, row_version, policy_version
  owner_priority, priority_source_event_id
  waiting_turn_seq, eligible_since_observation
  last_dispatch_observation_id
  active_protected_claim_id?

ReadyCandidate:
  root_work_id, unit_id, unit_row_version, ready_seq
  dependency_snapshot_id, applicable_fence_versions
  connection_id, connection_revision, route_rule_version
  request_hash, route_valid_until, price_schedule_version
  required_slot_vector, resource_upper_bounds, U_service
  required_verification_profile, evidence_refs

SchedulingDecision:
  decision_id, policy_version, state_view_revision_refs
  selected_candidate?, selection_reason
  root_turn_seq, service_components, compared_weight
  skipped[(root_work_id, closed reason code)]
  protected_claim_proposal?, next_wakeup_condition
```

`waiting_turn_seq` - stable number in order of acceptance of readiness C-05/C-03. It does not replace EventSeq, dependency versions, or permissions. If the time is equal, this number is used, then the stable `root_work_id`/`unit_id`. After the actual observed start `D`, the job that has completed units left receives a new number in the tail. Denial of admission, expiration of a pending RouteDecision, and write error do not count as a received turn.

The unavailable connection wait is shown separately from the fair resource wait. It does not accumulate monetary credit and does not create a promise of time. When validity is restored, the saved old ticket does not overtake the already protected move; after this the normal order applies. If there is a long absence of eligibility, a new eligible queue is formed by a separate event, without erasing the general waiting history.

### 4.3. Common choice: one dimension, then a stable order

If there is no protected move and there is no work older than `root_aging_ms`, for each currently executable candidate the following is calculated:

```text
service_numerator(root, candidate) =
  observed_service_of_completed_attempts_in_window(root)
  + conservative_service_of_unfinished_attempts(root)
  + U_service(candidate)

normalized_service = service_numerator / owner_weight
```

Each operation enters exactly once. Older completed windows are not included; unfinished ones do not disappear from the load just because of age. The window is determined by trusted observations, and not timestamps sent by the model. Fraction comparisons are performed by proven integer cross multiplication; overflow - closed failure, not transition to `float` or zero.

The smallest `normalized_service` is selected, then `waiting_turn_seq`, then the stable ID. This is an explicable heuristic for occupancy distribution with weights, not a proof of exact max-min fairness for an arbitrary set of suppliers. It gives short, inexpensive moves a chance to free up the system, but in itself is insufficient against eternal waiting for a big task. The guarantee is provided by the following separate mechanism.

When selecting within RootWork, units older than `unit_aging_ms` through `ready_seq` are first considered; otherwise they are applied sequentially: end date in one time scale, ready mandatory check of an existing candidate, confirmed membership of the critical path, `ready_seq`, `unit_id`. The missing period comes after the given one. The number of dependent nodes and the utility prediction may explain the proposal of a new plan, but are not added to this comparison and do not change the specified quality.

For the selected unit, the exact route still selects `rank_connections` of document 16. The policy does not palm off a weak model for the sake of a smaller U. If a different route is allowed, the new solution saves the new recipient, selection, rate, and checks; The work waiting ticket is not reset by such a replacement.

### 4.4. Aging and protected next move

When several eligible jobs exceed `root_aging_ms`, the oldest `waiting_turn_seq` is selected. After the threshold has expired, new foreground, urgent work and cheap candidates cannot endlessly stand in front of it. The selected job gets one turn, non-monopoly, until the RootWork is completed.

If the required vector of slots is now free, the C-05 atomically accepts the normal C-03 reservation and preparation of the attempt. If not free, C-06 suggests `ProtectedCapacityClaim`:

```text
claim_id, root_work_id, unit_id, root_turn_seq
connection_id, connection_revision, request_hash
required_slot_vector[(limit_id, window_id, quantity)]
reservation_id, dependency_snapshot_id
created_observation, drain_deadline_observation
state: draining | dispatch_pending | consumed | withdrawn
withdraw_reason?, row_version
```

Claim is the obligation of the local planner not to occupy the vacated capacity before the selected move. This is not a CapabilityLease, permission to submit a request, or additional financial balance. The required upper cash/quota reservation and local waiting space are accepted by one existing `commit_transition`; resources that are still actually occupied by another process are not marked as free and are not double-reserved as a running slot.

For each affected group of slots, a new third-party launch is allowed only if after it the free capacity remains at least equal to the claim requirement. If the required capacity has not yet been released, new consumers are not allowed. Unaffected connections and redundant slots continue to be used. Everything necessary for the request must be prepared before the claim; a separate LLM for “deciding what to do” is not launched inside the claim.

New third-party reservations also do not consume the already secured claim balance. Its limited wait ends no later than the maximum remaining upper time of interfering attempts plus `claim_drain_margin_ms`. If the completion of these attempts is not proven, `CapacityReleaseUnconfirmed` is created: the slot remains unknown, the claim is revoked, the work retains the reason for waiting. This is a denial of promised availability, not a hidden age reset, and not an imaginary winning move.

A change in fare, entrances, health or eligibility requires a new admission. An inapplicable claim is withdrawn and recalculated; Only proven unsent reservations are released safely. A stuck external operation continues to hold its own unknown flow rate. `stop/revoke` immediately closes the claim of the corresponding area without waiting for its deadline; the entry in the main database catches up with the protective fact.

### 4.5. Conditional guarantee and its limit

For a specific RootWork, the final next move is guaranteed if the following conditions are met:

1. before its ticket there is a finite number of eligible jobs; new tickets are not inserted before the protected old one;
2. there is a specific ready-made valid request with a finite upper time and a sufficient already allowed budget; its significant inputs, editions, rights and admissibility remain stable long enough to wait for previous moves, preparation and admission from `D`;
3. the required precise connection maintains confirmed availability, and interfering processes are terminated/fenced with the observed release of capacity in a finite time;
4. C-05/C-03/C-10, disk, clock and channels make progress, the commit queue does not remain closed forever;
5. neither the owner nor the automatic result producers cancel or change the significant basis of this request indefinitely; the presence at any moment of some new valid request does not replace the stability of the selected one.

After the threshold before work there is a finite number of old tickets. Everyone gets a maximum of one protected move and moves to the back. Required slots that are released cannot be reoccupied before claim. Consequently, this ticket reaches dispatch or receives a specific condition-violating failure; an infinite foreground in itself is not such a failure.

Counterexample to the stronger promise: source `FollowHead` automatically changes the output between each selection and tolerance. Then repeated `DependencyChanged` are possible, although there is always a new candidate. This is not a preemption by other runs and is not a reason to send an old request. The variability of the base is recorded; consolidation of the editorial office is allowed only by a communication agreement out of 24, and not by a secret decision of the planner for the sake of an indicator of progress. The AC-SCH-13 test reproduces this case, then stops changing and checks for a move under the conditions met.

The wait estimate is `остаток aging + сумма конечных drain/prepare/commit интервалов старых tickets + собственный интервал`, rather than the unconditional “no more than 60 seconds.” Until the upper intervals are proven by the prototype, the numerical SLA is not published. Guaranteed to make a move, not get the result right, meet any deadline or complete an endless research goal. For WorkUnit, a similar aging queue is used inside the continuously ready RootWork; the constant emergence of new verifiers cannot forever displace the old valid unit.

Restoration is not done for the sake of fairness: the claim of the old generation does not come to life. The C-13 retrieves the service history and wait order as data, shows the clock break, stores the flow rate, and waits for permission to continue. Monotonous hours of different boot IDs are not subtracted; if the time is unreliable, the order of the tickets is preserved and the age is marked unknown. Lack of proven time is not issued as a completed move.

### 4.6. Example in four steps

The training numbers below are an artificial track, not measurements. One valid Astra-route has one slot. A — foreground, weight 4; B - normal, weight 2; C - background research, weight 1. In all three works, the mandatory quality is required by this valid route. Everyone has already used 0 slot-ms; U of the next attempt is 40000 slot-ms. C waits 59 seconds.

| Step | Observation and decision | What does this check |
|---|---|---|
| 1 | C has not yet reached the threshold. Comparison A=10000, B=20000, C=40000 selects A. Up to `D` the limit, intent and desired checkpoint are fixed. | Foreground has finite advantage, not absolute power. |
| 2 | On the artificial track, A realistically finishes in 10 seconds. Its U is replaced by 10000, slot release is proven. C now waits 69 seconds and has the oldest aged-ticket. A C is chosen, although a normal grade would again prefer an A. | Aging does not depend on invented utility and does not reset accounting. |
| 3 | While C occupies Astra, the new A2 cannot displace it. An independent D on another valid connection can go if installation slots and budget remain. If the installation had only one slot, D waits until it is finally freed rather than receiving a non-existent slot. | The full vector of constraints and the actual model availability are taken into account. |
| 4 | Ticket C has already been moved after the certified fact `D` according to §4.3. After proven completion and readiness of the next unit, C participates again with this order; completion does not rotate the ticket a second time. The oldest waiting B gets the next protected move if it reaches the threshold; Otherwise, the usual comparison applies. | The research is saved; One successful move does not give an eternal monopoly. |

A separate variation of step 2: if C demands two slots, one is freed, and the second is still occupied by A1, claim C prohibits A2 from occupying the first. Once the second one is proven to be freed, C is started. Without this rule, an endless stream of single-slot A2s could always interfere with dual-slot operation, even with proper aging.

## 5. Limited queue of a single writer

### 5.1. Different types of "accepted"

Transport response is not subject acceptance. C-05 distinguishes:

```text
CommandIngressOutcome:
  NotAdmitted { command_id, reason, retry_condition }
  QueuedVolatile { command_id, body_hash, queue_ticket, run_generation,
                   expires_at, durable: false }

CommandOutcome:
  Rejected { command_id, reason, current_version_refs }
  DBCommittedPendingCheckpoint { command_id, commit_receipt }
  Committed { command_id, commit_receipt, checkpoint_receipt }
  OutcomeNotYetKnown { command_id, query_condition }
```

`QueuedVolatile` - the command was checked at the input and received a place in the memory of the current core; it may still be rejected due to changed versions. This is not a durable commitment to restore the queue after a crash, nor is it a UI "change accepted" state. Until the terminal response, the sender stores the original command/evidence in its limited valid scope and does not trigger the dependent action. If the client can't save even that, it doesn't promise the user a saved command.

`NotAdmitted` means that there is no new location/new execution. But the lost response from the previous sending does not allow us to draw such a conclusion: first the status of the same `command_id` is requested. Repeating the same ID and canonical body returns the existing result or ticket; another body gives `ReplayConflict`. The Envelope of the current authenticated channel can have a new generation when querying the status; this cannot change the body and expected versions of the previous command.

`Committed` appears only after a mandatory checkpoint. Loss of communication between C and ACK returns unknown to the client rather than automatically retrying. ID/body/terminal outcome deduplication lives on the C-05/C-03 in the same subject commit, and not just in the queue cache. The old command with the old inputs does not run automatically after recovery; its incompleteness is shown and processed by a valid continuation contract.

### 5.2. Capacity in quantity and bytes

Initial test profile:

| Parameter | Meaning |
|---|---:|
| `max_outstanding_commands` | 256 |
| `max_outstanding_accounted_bytes` | 16 MiB |
| `max_single_command_wire_bytes` | 256 KiB |
| `max_single_command_accounted_bytes` | 512 KiB |
| `per_root_outstanding_commands` | 32 |
| `per_root_outstanding_accounted_bytes` | 2 MiB |
| `protected_ingress_reserve_commands` | 32 out of total 256 |
| `protected_ingress_reserve_bytes` | 2 MiB out of total 16 MiB |
| `high_watermark` / `low_watermark` | 75% / 50% |
| `max_inflight_main_transactions` | 1 |
| `max_pending_checkpoint_transitions` | 1 |

All limits are tested simultaneously. Outstanding includes pending, executing commands, transition from pending checkpoint, and not yet used obligations `completion_credit`. Moving from waiting to inflight does not free up the slot. The byte weight includes the bounded wire-body, proven upper decoded memory, indexes, and service wrapper; zero-copy and the value of the coefficients are checked on a real implementation. It is not possible to limit just the JSON length, leaving unlimited Vec/strings after parsing.

The large original response/artifact is sealed into C-04 under its own limit. The queue contains limited descriptor, hash, provenance and links; the link does not allow you to change the body later. Disk spool is also limited separately and does not turn RAM-backpressure into endless disk growth. An optional diagnostic stream does not become thousands of mandatory transactions.

Before allowing a new attempt, C-05 allocates `CompletionCredit { attempt_id, max_commands, max_accounted_bytes, expires_with_observed_attempt_lifecycle }` from the general limits. For a starting artificial contract - up to two compact simultaneously waiting teams of no more than 16 KiB accounted for each; the other format must declare its own proven boundary in advance. Credit reserves a place for completion/outcome, not an LLM slot, money or agreement to accept the outcome. When the result arrives, the reserved credit is replaced with the actual entry rather than being added a second time.

Credit is released only after the necessary observations have been accepted or the life cycle of the attempt has been proven to be closed. The death of a worker does not in itself release the unknown expense. Restart first checks guard and incomplete attempts; new starts are closed until the required tail is placed safely. If a contract can produce more independent mandatory messages than credit, running before the limit is increased is prohibited; Observations cannot be silently discarded after the fact.

### 5.3. Queue is not a second log

`QueuedCommand` contains only:

```text
queue_ticket, command_id, canonical_body_hash, authenticated_sender_ref
canonical_payload: BoundedCommandPayload
root_work_id?, command_kind, expected_rows, dependency_snapshot_id?
authority_epoch, run_generation, applicable_fence_versions
enqueue_clock_observation, expire_clock_observation
accounted_bytes, completion_credit_id?, ingress_class
```

`canonical_payload` is a limited copy of the already disassembled typed body `ValidatedCommand` belonging to this entry, not only its imprint. It is immutable until the end result or outright failure, enters `accounted_bytes` along with the overhead and does not borrow the sender's memory. Before re-inspection, C-05 verifies body conformity `canonical_body_hash`; Someone else's body with the same number will not be accepted. Large materials remain sealed evidence at the link and are not deployed in a queue. Without a body, the transition cannot be recomputed, so a single fingerprint receipt is not a sufficient form of queue.

Ticket and transient occupancy do not assert substantive truth. Full Accepted Transition and Mandatory Beacon still occur together in C-03. The queue does not have its own SQL writer, a separate truth DB, or an automatically “accepted” tail after a restart. The chosen memory limit is honestly different from the longevity of the command.

### 5.4. Alignment and back pressure

There are two closed C-05 input groups:

- `reconciliation`: certified facts of already occurred actions/expenses, barriers, loss of process and recorded recovery;
- `ordinary`: owner commands, intent preparation, packages, checks, semantic application and normal subject transitions.

Protective reserve is available only to the first group and completion of the already issued credit. It does not belong to any post by `urgent=true`. In ordinary, round-robin operates on RootWork with a FIFO ticket inside the work; trusted non-work-related commands have one limited installation group, rather than a new group per message. Reconciliation operates a FIFO of persistent observations with deduplication. For both non-empty groups, no more than three reconciliations in a row, then one ordinary one. This eliminates the endless displacement of the ordinary by the flow of honest accounting; stop has already worked independently.

The intent does not get the right to jump over its own armor, target, or checkpoint. Dependent commands become executable only after the required validated predecessor. An unexecutable head doesn't hold everyone back: the dependency gives an explicit `DependencyNotConfirmed`/deferred ticket with a deadline, rather than skipping the check. Permitted permutations do not change the sequence of already accepted events.

When any high-watermark is reached, the C-05 stops allowing new productive attempts and optional background entries. Already having credit ends; protective observations are taken into the provided reserve. Resume only after both indicators drop below the low-watermark, a state change event, and candidates are re-screened. A full regular queue returns `NotAdmitted(QueueFull)`; there is no hidden queue inside the provider, no new LLM “wait” call, or frequent busy polling.

`retry_condition` - notification of capacity release/change of a specific lock or a durable limited period from the general retry policy. The retry saves the ID, does not create a second card, and does not start a paid attempt until accepted. The capacity notification is not a permission: the sender goes through the atomic ingress again.

If both reserve and C-09 protective storage are exhausted, new dispatches are closed. An already known fact does not turn into “did not happen”: the emergency place and fail-closed of 18 are applied. There is no promise to limitlessly buffer observations with the final SSD. The main queue may be unavailable, but this does not prevent C-10 from closing admission and terminating processes.

## 6. Exact path from waiting to commit

```text
проверенный CommandEnvelope (C-02)
→ проверка границ ingress и ограниченный QueuedVolatile (C-05)
→ выбор исполнимого ticket без удержания AdmissionFence
→ свежий StateView + чистый decide_transition
→ краткий AdmissionFence: актуальные версии/права/зависимости
→ единственный main SQLite COMMIT: строки + ledger + события + command outcome
→ DB_COMMITTED_PENDING_CHECKPOINT; gate освобождена
→ C-10.record_journal_checkpoint(expected_previous, committed_head)
→ CHECKPOINT_CONFIRMED
→ Committed / публикация принятого снимка / разрешён следующий зависимый шаг
```

Waiting for a queue, collecting context, model, DNS, network response, waiting for a free slot and checkpoint do not hold gate. C-03 holds the required gate only up to point C main COMMIT for transitions defined by `AdmissionFence`; C-09 - before the actual transfer to OS D. Receiving a gate does not mean passing the test.

Immediately after the queue and at the acceptance point, `ExpectedRows`, significant `DependencySnapshot`, RootWork/WorkUnit status, applicable fence, era, generation, timing, material/check immutability, exact tariff/reserve for the corresponding transition are checked. The old version is not accepted because it once received a queue ticket. Changing the extraneous RootWork and growing the global EventSeq do not by themselves make the command stale.

`VersionConflict`, `DependencyChanged`, `FenceChanged`, `StaleGeneration`, `ExpiredCommand` terminate the current acceptance attempt with a refusal. Automatic replacement of `expected_version` with the current one is prohibited. A new plan calculation is allowed only as a new typed command with the same goal boundaries and with an explicit origin; it does not transfer the verification verdict to changed material.

The initial mode allows at most one main transition while waiting for a checkpoint. Until it is committed, the next normal main COMMIT does not begin; the queue remains limited, producers receive backpressure. This is a deliberately simple verifiable source sequence, and not a statement that its throughput will always be enough.

After C, but before the checkpoint, point F can protect the work. Checkpoint of a saved transition is allowed as a defensive local operation; it does not carry C retroactively. The UI after confirmation shows the result along with the current stop state and the dependent new action is prohibited F. Main rollback below the checkpoint and another MAC at the same position is a violation; failure before checkpoint requires a registered local C-13 reconciliation, not reapplying the command.

## 7. Emergency and short admission - outside the usual queues

The external proposal to put `revoke/stop/recovery fence` on Priority 0 of the common CommitPipeline is incorrect in its literal implementation. Even the first element of the queue will not be executed if core is frozen. The existing path is preserved:

```text
доверенный родной ввод
→ C-10 закрывает admission затронутой области для новых входов
→ краткая синхронизация с уже вошедшими операциями
→ F: устойчивый scope_fence/epoch + protection_event в authority.anchor
→ C-09 отражает отзыв, C-10 прекращает деревья и наблюдает завершение
→ позднее C-05/C-03 переносит факт по (origin_store_id, origin_event_id)
```

This path does not wait for the normal core queue, resource fairness, free model slot, or LLM response. It does not bypass owner authentication, command scope, anchor integrity, and unknown outcome rules. Recovery reasoning or network reconciliation does not receive emergency powers just by the name “recovery”.

Checkpoint and fence both record C-10. It does not accumulate the unlimited FIFO of regular checkpoint requests before stop: admission closures are handled by a separate guard input, and pending anchor entries are fetched with fencing priority. A short SQLite transaction that has already started cannot be declared instantly aborted without proof. Maximum gate/anchor-write retention is measured separately; the hang triggers the existing owner termination and reconciliation procedure. Until proven F/completion, the UI shows a running or unconfirmed stop, not `stop_confirmed`.

The late result is not applied after F. A certified account or observation of an already departed action is still imported by the service command; This is a closed exception for truthful accounting, not bypassing safety for the sake of “continuation”. Stopping A does not raise the global epoch or revoke unrelated B/Cs without a common cause.

## 8. Metrics and terms of discussion batching

### 8.1. What to measure

Derived performance metrics do not create a subject transaction for each measurement. Mandatory regime changes/violations are stored by ordinary events, but the histogram itself is not a recursive log of truth.

| Metric | Bounds/Dimension |
|---|---|
| `queue_wait_ms` | QueuedVolatile → start ticket processing |
| `transition_decide_ms` | fresh StateView → ready-made clean plan |
| `admission_wait_ms`, `admission_hold_ms` | gate wait separate from hold to C/D |
| `main_commit_ms` | start of SQLite transaction → confirmed COMMIT C |
| `checkpoint_ms` | confirmation request committed_head → corresponding receipt |
| `commit_pipeline_ms` | start processing → CHECKPOINT_CONFIRMED |
| `command_end_to_end_ms` | first ingress of this command → terminal confirmed response |
| `queue_count`, `queue_accounted_bytes` | waiting + inflight + pending checkpoint + unused completion credits |
| `queue_rejections_total`, `queue_expired_total` | by closed reason/class; short commits are not considered successful |
| `pending_checkpoint_age_ms`, `checkpoint_conflicts_total` | state of the only unconfirmed transition |
| `eligible_wait_ms`, `protected_turns_total`, `capacity_unknown_total` | progress of work separately from delay in recording |
| `backpressure_active_ms`, `completion_credit_utilization` | how long is entry limited and busy with obligations |

For latency, p50/p95/p99, number of observations N, window, build/policy version, limit configuration, load profile and histogram/resolution method are published. If there are no measurements - `not_measured`, not zero. When N is small, high percentiles are labeled as not stable enough; the results of incomplete/erroneous transitions have separate rows and a censored counter so that timeouts do not disappear from the beautiful p99.

Durations are taken from one monotonous scale or measured by the caller's round-trip. Other people's UTC hours of different processes/boots are not subtracted. The labels contain private classes and bounded IDs in the detail trail, rather than the infinite cardinality provider-body/root_id in each global metric. No percentile in this document is a measured outcome.

The test load must simultaneously describe workers, RootWorks, commands/sec, message sizes, ledger/verification/intents share, fsync/checkpoint delays and duration. “500 workers” or “3000 events/min” by themselves do not prove bottleneck: one event and one transaction are not identical, and most workers can wait for a model.

### 8.2. Batch is not enabled by default

Multiple rows/events of one original `TransitionPlan` are already atomic and are not a new optimization. Initial values: `max_commands_per_transaction = 1`, `max_committed_transitions_per_checkpoint = 1`.

Before expanding batch, a separate stored proof is required:

1. bottleneck is localized by measurements specifically in commit/checkpoint, and not in model requests, context composition, fsync of a faulty disk or a logical loop;
2. a closed-list of acceptable transition combinations is specified; there is no dependency within a batch that requires a pre-published preceding ACK/right;
3. each command saves the ID, version checking, result, order of events and the meaning of all-or-nothing; partial failure has an explicitly chosen semantics;
4. the final maximum number, bytes and gate retention is proven, emergency stop is saved;
5. checkpoint verifies an exactly continuous full prefix; no command behind a confirmed vertex receives an ACK;
6. fault-injection has been passed for all internal boundaries, repetitions and conflicts; reverse migration/reading does not lose outcomes.

You can’t just set a “collect another 50 ms” timer inside the AdmissionFence, confirm the set using the largest EventSeq, or accept the old version after accumulating a batch. Until such proof is achieved, optimization remains a proposal, not an implicit part of the implementation.

## 9. Internal functions, errors and events

### 9.1. Entry/exit without new managers

| Internal function | Owner/Caller | Input → Output; side effects |
|---|---|---|
| `validate_scheduling_policy` | C-01, calling C-05 | proposal + constraints → validated policy or `InvalidPolicy/UnboundedAttempt/ArithmeticOverflow`; clean |
| `compare_root_service` | C-01, calling C-06 | integer components + weights → Ordering and explanation; clean |
| `choose_fair_root_turn` | inside C-06.`select_ready_units` | StateView + ClockObservation + policy + availability → SchedulingDecision; no appointment/reserve |
| `plan_protected_capacity_claim` | C-06 | aged candidate + current slots/reservation → proposal or specific blocking; without issuing a license |
| `try_admit_command` | inside C-05.`handle_command` after C-02 | ValidatedCommand + bounded ingress state → IngressOutcome; only limited memory and ticket |
| `select_next_commit_command` | C-05 | queue + confirmed dependencies → executable ticket or wake condition; without DB entry |
| `revalidate_queued_command` | C-05 with C-01 | fresh StateView + command → plan or typed rejection; final preconditions remain with C-03/gate |
| `await_commit_checkpoint` | C-05 via existing C-10 | CommitReceipt + expected_previous → confirmed outcome/pending/failure; does not write anchor itself |
| `observe_pipeline_latency` | C-05, derived diagnostics | closed stage + monotonic observation → bounded histogram; not a separate truth |

Changes in schedule-state, accepted claim, resource postings, ticket-rotation according to certified D and reasons for failures are formalized by typed versions of existing C-05 commands. They are fixed by C-03 together with the original objective cause, where this is one atomic transition. Pure choice without acceptance does not write your own “decision diary”.

### 9.2. Closed reasons

`NoAdmissibleConnection`, `ConnectionUnavailable`, `QualityFloorUnmet`, `InsufficientIndependence`, `DisclosureDenied`, `LimitReached`, `UnknownUpperBound`, `CapacityBusy`, `CapacityReleaseUnconfirmed`, `CommitBackpressure`, `ClaimInputChanged`, `QueueFull`, `RootQueueFull`, `Oversize`, `DecodeBudgetExceeded`, `ExpiredCommand`, `DependencyNotConfirmed`, `DependencyChanged`, `VersionConflict`, `FenceChanged`, `StaleGeneration`, `ReplayConflict`, `CheckpointPending`, `HeadConflict`, `StorageUnavailable`, `ArithmeticOverflow`.

They differ: “not allowed”, “waiting for final resource”, “owner needed”, “state changed”, “recorded without checkpoint” and “outcome unknown”. A write failure after C does not return a failure mode, implying that there is definitely no record. The repetition method is determined by classification and observation, not by HTTP code alone.

### 9.3. Events

In C-03, when the change is accepted: `scheduling_policy_changed`, `root_wait_turn_registered`, `root_turn_dispatched`, `protected_capacity_claim_created`, `protected_capacity_claim_withdrawn`, `commit_backpressure_entered`, `commit_backpressure_left`, `queued_command_rejected_stale`, `capacity_release_unconfirmed`. The queue event does not state that a volatile team will survive the restart. Details of metrics and each unsuccessful cheap comparison are not reproduced in the log.

Existing `attempt_started`, `action_intent_committed`, `action_observed`, `result_committed`, `resource_bound_breached` and security events retain the same meaning. `root_turn_dispatched` arises from the trusted fact D, not from the intent or text of the worker. When the main database is full/unavailable, the backpressure state is available as a runtime observation with this origin; he can't imagine a long-lasting EventSeq. Metric transitions and security facts are transferred after recovery idempotently and with a separate fact time.

## 10. Acceptance scenarios before declaration of readiness

All scenarios below are future mandatory checks. PASSing a documentation checker does not mean executing them.

### 10.1. Planning

| ID | Script | Verified result |
|---|---|---|
| AC-SCH-01 | Ten RootWorks and one exact route; play one StateView/ClockObservation trace twice. | Same choice, reasons, tickets and reservation; neither float nor hash-map random order have any effect. Ticket rotates along D once, delayed completion does not allow protected re-selection of the previous number. |
| AC-SCH-02 | Continuously add foreground, keep background eligible longer than the threshold. | The oldest background gets a protected next move under the conditions of §4.5; new arrivals do not bypass it. |
| AC-SCH-03 | Two-slot work is waiting, single-slot work constantly frees/occupies one slot. | Claim stops occupying the required free balance; the two-slot one is launched after the observed drain. |
| AC-SCH-04 | Astra(route A) is unavailable, route B of the same name is prohibited due to data/quality; weak model available. | There is no hidden substitution; explicit `NoAdmissibleConnection`; other valid jobs are in progress. |
| AC-SCH-05 | Big U and different weights; real durations are less than U; repeat observations. | The top ones are replaced by a fact exactly once; integer sort; overflow blocks selection. |
| AC-SCH-06 | Money/tokens are unknown after losing a worker, then a late bill arrives. | Justice does not free U; C-03 takes into account the fact once; excess U is not trimmed. |
| AC-SCH-07 | While the claim is waiting, change the tariff, expected row, body, connection revision, fence one at a time. | A new calculation/approval or refusal to D is required; no portable pass. |
| AC-SCH-08 | Installation of 2 slots, claim requires 1 Astra; another allowed route is free. | Independent operation uses only the remaining capacity without taking away the required slot/reservation. |
| AC-SCH-09 | The connection flutters; termination of the interfering process has not been confirmed. | Anti-starvation SLA is not promised if conditions are violated; exact failure, unknown capacity and saved history. |
| AC-SCH-10 | The old research and self-improvement units compete with the new verifiers of the same RootWork. | Unit aging preserves their ability to move; Mandatory checks and boundaries of self-development are not cancelled. |
| AC-SCH-11 | Restart core with claim, unknown expense and old ticket. | No automatic start; the old generation is rejected; the accounting is not reset, continued on C-13. |
| AC-SCH-12 | Emergency stop A when the commit queue is full and all LLM slots are occupied. | C-10 closes A without model/core queue; B does not lose unrelated powers. |
| AC-SCH-13 | Automatic `FollowHead` changes the significant basis before each tolerance; the selected edition is then stabilized. | There is no false guarantee of running an outdated candidate and hidden pinning; after the conditions of §4.5 are met, the competing flow does not interfere with the final move. |

### 10.2. Queue and commit

| ID | Script | Verified result |
|---|---|---|
| AC-COMMIT-01 | Reach quantity limits, wire bytes, decoded bytes and per-root independently. | The following command gets the exact `NotAdmitted`; all counters take into account inflight, checkpoint and credits. |
| AC-COMMIT-02 | All the usual seats are taken; already admitted workers are terminated. | Their compact mandatory outcomes take credit without overflow/loss; new starts have been stopped. |
| AC-COMMIT-03 | Cross high/low-watermark in both directions with variable packet sizes. | Hysteresis checks quantity and bytes; no churn, busy polling or expensive LLM repeats. |
| AC-COMMIT-04 | Lose the response to ingress/COMMIT/checkpoint and repeat the ID with the same/different body. | One transition; previous result/ticket or unknown; another body - `ReplayConflict`. Changing the sender's memory after staging does not change the body belonging to the queue; an incomplete handle without required arguments is not allowed. |
| AC-COMMIT-05 | Kill core after `QueuedVolatile`, before C. | No false long-lived ACK; the sender sees the incompleteness and checks the ID; old versions are not overwritten when replayed. |
| AC-COMMIT-06 | Kill core after C before checkpoint, then after checkpoint before ACK. | In the first case, the pending-tail is verified without the second application; in the second, the confirmed outcome of the same ID is restored. |
| AC-COMMIT-07 | While the ticket is waiting, change the significant line one at a time, snapshot, right, fence, generation; change unrelated work separately. | All significant changes are rejected from acceptance; unrelated work does not produce a global false conflict. |
| AC-COMMIT-08 | Slow down/hang up checkpoint, fill in ordinary. | No more than one pending; no ACK/dependent dispatch; no infinite buffer; stop works separately. |
| AC-COMMIT-09 | The reconciliation stream does not end; ordinary contains the final executable commands. | After three reconciliations the service is ordinary; FIFO and root round-robin are deterministic. |
| AC-COMMIT-10 | C < F < checkpoint for result A; then a late B result in the same fenced area. | A is confirmed as written before F with the current stop; new consequences are prohibited; later B is not applied, the consumption is maintained. |
| AC-COMMIT-11 | Rollback mainDB below checkpoint, another MAC or someone else's journal_id; separately only a new unrelated event. | Damage closes admission; unrelated event does not replace the row/snapshot check. |
| AC-COMMIT-12 | No free core/guard entry; the owner makes a stop. | Admission closes regardless; confirmation of anchor/processes is shown honestly, outcomes are not declared cancelled. |
| AC-COMMIT-13 | Synthetically known durations, stuck/erroneous commands and an empty histogram. | Tested p50/p95/p99 and N; censored separately; no data - `not_measured`, not 0. |
| AC-COMMIT-14 | Try to enable batch without proof; then fault-injection inside the batch being studied. | Default is refusal; the future version must preserve the ID, full confirmed prefix, versions and limited gate. |
| AC-COMMIT-15 | Dependency command X requires Y, but Y is still a pending checkpoint; X earlier in ready-view. | X does not pass before Y is confirmed; the queue does not turn the order of delivery into proof of dependency. |

AC-SCH-02/03/10 and AC-COMMIT-04/06/07/08/10 require not only hand-selected examples, but the generation of valid event sequences with a reproducible seed and counterexample minimization. The specific model of such tests is drawn up separately; this chapter defines its expected properties rather than declaring an existing Stand.

## 11. Specific places of integration in future coding

| Place of the draft document 19 | Change to existing responsibility | Evidence/document stage 20 |
|---|---|---|
| `dc-domain/resource.rs`, `work.rs` | closed policy/types, verified counters, root/unit aging comparison, admission-capacity rules | stage 2; AC-SCH-01/05 and AC-COMMIT-01 |
| `dc-core/scheduler.rs`, `routing.rs` | `choose_fair_root_turn`, availability reason, claim and residual capacity; route quality remains in filter/rank | steps 6/7; AC-SCH-02—04/07—10 |
| `dc-core/command.rs`, `state_machine.rs` | bounded ingress, acceptance outcomes, two-class queues, freshness after waiting, checkpoint latch | steps 3/4; AC-COMMIT-01—10/15 |
| `dc-store/transition.rs`, `resource_ledger.rs`, migrations | schedule-state/claim on accepted transition, command outcome deduplication, one ledger without a second posting | stage 3; AC-SCH-06/11, AC-COMMIT-04—07 |
| `dc-broker` to C-09 | re-checking a specific request with D, durable observations with backpressure | step 4, then 7; AC-SCH-07/12, AC-COMMIT-02/12 |
| `dc-watchdog/{authority_anchor,admission,stop}.rs` | protective input beyond ordinary, pending checkpoint limit, measurement of short critical sections | stage 4; AC-COMMIT-08/10—12 |
| `tests/{contracts,transitions,recovery,end_to_end}`, `tools/test-harness` | artificial times/routes/flow, saturation, lost ACK, crash cuts and reproducible schedules | steps 2-4 to the present model; then through stage 7/15 |
| C-13/C-14 representation | why RootWork is waiting, claim/queue/unknown; don't show QueuedVolatile as accepted change | steps 5/9; AC-COMMIT-05/06/13 |

First, the subject fields and test traces are refined, then the bounded queue/main/checkpoint on artificial executors, then the real model and long-term load. The fairness setting is not a separate product mode: five depths, research and automatic development of the valid area use one path C-06/C-05/C-03 with different accepted budgets and assignments.

## 12. Response to external criticism

Point 2 is right: the list of limits and the choice of a useful unit have not yet defined complete inter-work fairness. Added selection rule, separate aging, specific availability, upper/real maintenance accounting, multi-slot stroke protection and conditional verifiable guarantee. The proposed deadline/age/cost amount without a general dimension is not accepted as a specification. Infinite strict P1 over the background is also not accepted.

Point 3 is right in the requirement to limit the queue in advance, provide backpressure and measure the main/checkpoint path. His bottleneck with hundreds of performers is still a hypothesis, not a measured defect. One writer, two-phase confirmation, pending-tail behavior and independent stopping have already been fixed. They do not need to be reinvented as a new control layer.

Superfluous or dangerous in the literal transfer of the sentence: new Manager/Service around existing C-responsibilities; stop/revoke, waiting for Priority 0 of the general queue; reduction in quality for the sake of fairness; batching one load impression at a time; turning a queue ticket into a durable agreement to accept the old version. Efficiency is enhanced by proven flow restrictions and observable policies, rather than by reductions in research or self-development opportunities.
