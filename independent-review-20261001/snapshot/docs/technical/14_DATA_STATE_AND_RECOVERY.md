# Deep Create: data, states and recovery

**Date:** 2026-09-07  
**Status:** Required first version schema before creating tables and subject types  
**Based on:** Documents 07, 10, 11 and 13

---

## 1. Main principle

Deep Create does not store a “long chat”, but a verifiable history of changes to one main job.

Each accepted change consists of two parts, recorded in one transaction:

1. new actual state of the subject object;
2. an unchangeable event that explains who, based on what rule and what previous version created this state.

An event without an accepted change may record a failure, observation, or incident. An accepted change without an event is disallowed.

The model does not write to the database. The window does not write to the database. The performer does not write to the database. The only writer of the main database is the control process. This does not make him a writer of security bases: the owners of `guard.db` and `authority.anchor` are distinguished in section 5. There is no common transaction between the three bases; incomplete exchanges are stored and reconciled against stable numbers, rather than being considered indivisible by default.

---

## 2. Three levels of data

### 2.1. Canonical data

This is data without which it is impossible to prove history or restore work:

- original messages from the owner;
- accepted interpretations of the goal;
- obligations and prohibitions;
- owner decisions;
- versions of the work graph;
- requests and issuance of rights;
- intentions and observed outcomes of actions;
- source materials and results;
- inspection reports;
- negative results and rejected paths;
- state events.

They are not automatically removed or replaced by a retelling.

### 2.2. Derived data

They can be recreated from the canonical ones:

- search index;
- semantic connections;
- abbreviated representations;
- preliminary calculation of the graph state;
- visual layout;
- cache of connection costs and availability.

Damage to the derived layer does not invalidate the canonical story. This layer is isolated and rebuilt.

### 2.3. Temporary data

- partially loaded answer;
- working copy of the file;
- unfinished material before the print is calculated;
- streaming visual output;
- temporary result of the performer until the result package is presented.

Temporary data is never the only place where a decision is stored.

---

## 3. Notations and general types

The external protocol and database use immutable random identifiers with approximately increasing generation time. The human-readable name is not the identity of the object and can change.

```rust
struct EntityId(UuidV7);
struct EventSeq(u64);
struct RowVersion(u64);
struct AuthorityEpoch(u64);
struct RunGeneration(u64);
struct ContentHash([u8; 32]);
struct EvidenceId(UuidV7);
struct ContentFingerprint { key_version: u32, purpose: FingerprintPurpose, value: [u8; 32] }
struct FenceVersion(u64);
struct IdempotencyKey(UuidV7);
struct SchemaVersion(u32);
```

Rules:

- the identifier is never reused;
- `EventSeq` is issued only by the main database in the order of fixation;
- `RowVersion` increases by exactly one when an object change is accepted;
- `AuthorityEpoch` only increases the protective process and never decreases it;
- `RunGeneration` is incremented each time the control process is started and invalidates messages from old processes;
- `ContentHash` applies to clear materials or ciphertext by exact bytes; for sensitive public content, use `ContentFingerprint` with a separate local destination key, rather than public SHA-256;
- `EvidenceId` sets a link and a random object name; the fingerprint checks the contents, but is not revealed through the file name;
- time is stored in UTC, and the validity period is additionally checked against the monotonic time of the current launch;
- money and quotas are stored in whole minimum units, not floating point numbers.

---

## 4. Main base `deepcreate.db`

### 4.1. `installation`

One line describes the local installation.

| Field | Meaning | Limit |
|---|---|---|
| `installation_id` | immutable installation identity | created once |
| `schema_version` | applied scheme | matches the last fully accepted migration |
| `app_version` | version that last wrote the database | only accepted whole version |
| `created_at_utc` | creation time | invariably |
| `last_event_seq` | last received event number | equal to the largest event |
| `journal_id` | current magazine line | changes only by a confirmed recovery procedure with a saved connection |
| `last_seen_run_generation` | generation adopted at last launch | is not the source of the generation itself |
| `restricted_reason` | reason for limited mode | missing or closed reason code |

There is no `is_clean_shutdown` field here: its absence does not prove a failure, and its presence does not prove integrity. Truth is derived from transactions, intents, processes, and storage verification.

### 4.2. `sessions`

A session is a long-lasting, visible container for conversation and work.

| Field | Meaning |
|---|---|
| `session_id` | resilient personality |
| `title` | changeable human-readable name |
| `interaction_mode` | `conversation` or `deep_create` |
| `status` | `open`, `stopped`, `archived` |
| `memory_access_profile_id` | accepted local search area |
| `autonomy_policy_id` | current set of rules for independence |
| `active_objective_id` | last accepted target or none |
| `created_at_utc`, `updated_at_utc` | actual times |
| `row_version` | late write protection |

Changing modes does not create a new session identity. A new independent goal can create a new session or a separate main work only after an explicit accepted parsing, and not by a guess from the model.

### 4.3. `root_works`

The main work is different from the session: the session stores the conversation and presentation, the main work stores one accepted goal and its verifiable history.

| Field | Meaning |
|---|---|
| `work_id` | unchangeable identity of the main job |
| `session_id` | visible container |
| `title` | changeable title |
| `active_objective_id` | current edition of the goal |
| `stage` | semantic stage of work |
| `depth_profile` | accepted depth of work |
| `created_by_owner_event_id` | source |
| `created_at_utc`, `updated_at_utc` | times |
| `row_version` | version |

The semantic stage does not store the reasons for the temporary blocking. A job can simultaneously be in progress, waiting for a resource, and protected by recovery; these characteristics are stored separately.

### 4.4. `owner_events`

Stores only events that arrived along the trusted path of the interface.

| Field | Meaning |
|---|---|
| `owner_event_id` | unique owner action number |
| `session_id` | scope |
| `kind` | closed view: message, confirmation, refusal, recall, stop, continue, change settings |
| `raw_evidence_id` | exact source text or form in evidence archive |
| `interpreted_payload` | machine interpretation by circuit version |
| `interpretation_version` | parser version |
| `status` | `received`, `needs_confirmation`, `accepted`, `rejected`, `superseded` |
| `created_at_utc` | time of receipt |

The original text does not change. Correcting the interpretation creates a new record and connection `supersedes`, rather than rewriting the past.

### 4.5. `objectives`

Each accepted target revision is a separate line.

| Field | Meaning |
|---|---|
| `objective_id` | target revision number |
| `work_id`, `session_id` | main work and its session |
| `revision` | increasing number |
| `statement` | short precise goal |
| `success_rule` | Rule of When to Start Final Review |
| `owner_event_id` | source |
| `status` | `draft`, `accepted`, `superseded`, `withdrawn` |
| `created_at_utc` | time |

At the same time, there can be only one accepted current version of the main work. The old one remains available. `sessions.active_objective_id` - the goal selected in the interface, not prohibiting several main works in the session; the source of the current edition remains `root_works.active_objective_id`.

### 4.6. `obligations`

Breaks the goal down into testable conditions.

| Field | Meaning |
|---|---|
| `obligation_id` | condition number |
| `objective_id` | goal editing |
| `parent_id` | optional compound condition |
| `kind` | `required`, `forbidden`, `preference`, `acceptance` |
| `statement` | clear condition |
| `verification_method` | machine known type of check or `owner_judgment` |
| `status` | `open`, `satisfied`, `violated`, `waived`, `unknown` |
| `evidence_policy` | minimum type of evidence required |
| `row_version` | version |

A preference cannot quietly become an obligation. Disclaimer requires the owner's trusted event and preserves the previous record.

### 4.7. `blocking_gates`

The blocking boundary answers not the question “what does work mean”, but the question “what is it currently allowed to do.”

| Field | Meaning |
|---|---|
| `gate_id` | personality boundaries |
| `scope_kind`, `scope_id` | installation, main work or unit |
| `kind` | recovery, integrity violation, lack of space, prepared stop, immediate stop, limit reached, unknown outcome, owner waiting |
| `allow_read`, `allow_reason`, `allow_plan`, `allow_effect` | explicit permitted job classes |
| `created_event_id` | proof of appearance |
| `cleared_event_id` | proof of withdrawal or absence |
| `details_evidence_id` | detailed report |

The unit is ready only if there is no prohibiting boundary in the entire chain “installation → main work → unit”. Removing one reason does not remove the others.

### 4.8. `work_units`

A unit of work is the minimum assigned and separately verified part.

| Field | Meaning |
|---|---|
| `unit_id` | unit personality |
| `work_id`, `session_id`, `objective_id` | main work, scope and editorial purpose |
| `parent_unit_id` | split origin |
| `kind` | planning, searching, creating, checking, reconciling, acting, combining |
| `title` | visible title |
| `task_spec` | precise work contract |
| `risk_class` | `reasoning`, `local_reversible`, `external_effect` |
| `required_capabilities` | closed list of required abilities |
| `status` | state from section 7 |
| `priority` | integer value calculated by the scheduler |
| `dependency_snapshot_id` | immutable `DependencySnapshot` from documents 15–16: target/unit revisions, read dependencies and rules |
| `accepted_result_id` | only verified accepted result |
| `created_at_utc`, `updated_at_utc` | times |
| `row_version` | protection against late results |

You cannot change `task_spec` for an already running unit. The change creates a new edition of the unit and cancels the acceptance of the late result of the old one.

### 4.9. `work_dependencies`

This contract describes the internal dependencies of units of one main work. Dependence on another work is not created by substituting someone else's `unit_id`: it goes through [24, §§3–5](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md). `result_publications`, their changeable admissibility and output bindings, `root_work_links`, unchangeable editions and the exact read set are specified there. All accepted connections and changes are serviced by the same C-03; the state of expectation is the cause of the boundary, not a new semantic stage of work.

| Field | Meaning |
|---|---|
| `unit_id` | dependent unit |
| `depends_on_unit_id` | predecessor |
| `kind` | `must_finish`, `must_supply`, `must_resolve`, `must_authorize` |

The `(unit_id, depends_on_unit_id)` pair is unique. The loop is prohibited until the graph is fixed. Waiting for permission blocks only reachable dependent branches, not all work.

### 4.10. `hypotheses`

| Field | Meaning |
|---|---|
| `hypothesis_id` | personality assumptions |
| `session_id`, `objective_id` | region |
| `parent_hypothesis_id` | branch origin |
| `statement` | verifiable statement |
| `status` | `proposed`, `testing`, `supported`, `weakened`, `rejected`, `merged` |
| `created_by_unit_id` | authorship |
| `row_version` | version |

A negative result is not removed. Merging does not rewrite the origin of branches.

### 4.11. `unit_attempts`

Each unit launch has a separate attempt.

| Field | Meaning |
|---|---|
| `attempt_id` | startup personality |
| `unit_id` | unit |
| `attempt_no` | serial number |
| `connection_id` | exact connection and model |
| `prompt_bundle_id` | unchangeable order |
| `worker_process_id` | observed local process |
| `authority_epoch` | era at launch |
| `run_generation` | launch control generation |
| `status` | `prepared`, `running`, `submitted`, `failed`, `lost`, `cancelled` |
| `started_at_utc`, `last_heartbeat_at_utc`, `finished_at_utc` | times |
| `failure_code` | exact closed code |

A retry creates a new attempt for the same unit. It does not create a new unit and does not lose the history of the cause.

### 4.12. `result_packets`

| Field | Meaning |
|---|---|
| `result_id` | result identity |
| `attempt_id` | source |
| `summary` | brief description of the result |
| `manifest_evidence_id` | complete list of outputs, statements and evidence |
| `output_schema_version` | form version |
| `dependency_snapshot_id` | the exact set of dependencies for which the executor worked |
| `status` | `submitted`, `verifying`, `accepted`, `rejected`, `stale` |
| `created_at_utc` | time |

The package does not contain hidden reasoning. It contains conclusions, verifiable statements, sources used, materials generated, negative findings, limitations, and requests for further work.

### 4.13. `verification_records`

To qualify the verification method, [26, §4](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md): `CalibrationEvidence` with initial observations, denominators and origin of markings; `QualificationDecision` with the scope of applicability and revision of the method. The test plan, admission and follow-up of the candidate have separate, unchangeable lists of evidence (§5). They are stored in the former C-04 archive and versioned C-03 records; C-08 and C-16 do not receive direct entry to the database.

| Field | Meaning |
|---|---|
| `verification_id` | check record |
| `target_kind`, `target_id` | result, action, graph or version |
| `verifier_attempt_id` | independent launch |
| `criteria_evidence_id` | exact verification conditions |
| `verdict` | `pass`, `gaps`, `reject`, `invalid` |
| `findings_evidence_id` | detailed conclusions |
| `independence_class` | how verification is independent of the author |
| `created_at_utc` | time |

`pass` does not complete the main job. It allows the controlling process to prepare a summary for the owner.

### 4.14. `evidence_objects` and `evidence_links`

Addendum 2026-09-08: [Document 23, §6](./23_EXTERNAL_CRITIQUE_AND_REFINEMENTS_20260908.md) defines `EvidenceLocation`, location revisions, verified copying, media unavailability, and corruption failure. This is a separate axis from `storage_state` below: The canonical source may be on a disabled media. The transfer itself does not change `EvidenceId`, the meaning, the right to disclose, or the validity of the conclusion. Location metadata is changed only by C-03 upon a received command from C-05.

`evidence_objects` indexes canonical files by random `evidence_id`. The public hash of personal data does not serve as a file name, external identifier, or general log lookup key.

| Field | Meaning |
|---|---|
| `evidence_id` | stable random object identity and name in storage |
| `content_fingerprint` | fingerprint of exact public bytes: protected for non-public classes, with purpose and key version |
| `ciphertext_hash` | control of the exact written encrypted bytes, not a fingerprint of the original text |
| `byte_length` | size |
| `media_type` | verified view |
| `data_class` | open, personal, sensitive, secret data |
| `encryption_key_ref` | link to sealed key, but not key |
| `storage_state` | `staged`, `canonical`, `quarantined`, `missing` |
| `created_at_utc` | time |

`evidence_links` contains `target_kind`, `target_id`, `relation`, `evidence_id`. Removing the last link to a canonical proof does not mean deleting the file. All links to the archive (`manifest_artifact_hash`, `criteria_artifact_hash`, `findings_artifact_hash`, `details_hash`, `evidence_hash` in previous sketches) in the created diagram have the type `EvidenceId` and the corresponding name `*_evidence_id`; raw hashes are not accepted instead of links. The contents of sensitive subject fields themselves are also encrypted before being transmitted to SQLite: single archive encryption does not protect text in database rows, WAL, temporary tables or index.

### 4.15. `context_projections` and `prompt_bundles`

`context_projections` stores the exact list of selected fragments, their origin, data category, reason for inclusion and reason for exclusion of sensitive candidates.

`prompt_bundles` connects the immutable fingerprints of all layers of the order, the model profile, the response form and the tool limit. The full text is stored as encrypted evidence; The magazine contains only safe composition and prints.

The same `prompt_bundle_id` means the same order byte byte.

### 4.16. `model_connections`, `capability_evidence`, `connection_health`

`model_connections` distinguishes between provider, address, entry method, connector, model, reasoning mode and data rules. Two different login methods for the same model are different connections.

`capability_evidence` stores the aptitude test version, result, problem set and date. Self-description of a model is not evidence of ability.

`connection_health` stores the last observed state, its expiration date, concurrency limit, and the time of the next cheap check. It is derivative and does not give the right to exceed the owner's limit.

### 4.17. `resource_limits` and `resource_ledger`

`resource_limits` contains:

- area: installation, session, branch, unit or connection;
- type: money, time, requests, input units, output units, place, parallelism;
- hard limit integer;
- action window;
- owner event;
- status `active`, `replaced`, `exhausted`.

The individual explicit state `unlimited` is different from the missing or unknown limit.

`resource_ledger` is the only place for cash and quota accounting, written only by the managing process. For each `(logical_operation_id, limit_id, window_id)` pair, there is one current position with `reservation_id`, `attempt_id`, `intent_id`, `upper_bound`, `confirmed_amount`, `unknown_remaining`, `row_version` and status `reserved`, `unknown`, `settled` or `released`. The history of transitions is saved by events. The intermediary checks and associates the reservation number with the intent once; his entry is not a second write-off.

Contribution of one position to the occupied limit: `reserved = U`; `unknown = C + R`, where `C` has already been confirmed, and `R` is the upper estimate of the still unclear remainder; `settled = C`; `released = 0`. For unknown transaction without partial account `C=0, R=U`. The transition `reserved(U) → unknown(0,U)` replaces the state of the same position and does not add a second `U`. The partial count replaces part of the remainder: `C+R ≤ U`, if the original upper bound is still proven. The full total `C` only releases the difference `U-C`. Recalling, restarting, and losing a process in itself does not free anything.

The new reservation, in one transaction, verifies the sum of the contributions of all positions and all applicable parent limits. The branch budget is the distribution of the parent's balance, not another actual expense; a physical operation is counted once in each applicable window. Redistribution does not hide expenditure already taken. Temporary concurrency slots are freed upon proven completion of a process and are not mixed with wasted quota. The type of time (`wall_clock`, `executor_time` or billable time) is specified in the limit rule, and is not guessed by the code.

A replay with proven external one-timeness saves `logical_operation_id`, key and position only if the top cost includes the replay; a new potentially paid attempt requires its own reservation while the old one is withheld unknown. The observation is applied exactly once according to `observation_id`; repetition of the same ID with different content is rejected. An actual invoice above `U` cannot be hidden, trimmed, or denied: it is recorded as a fact, blocks further charges, and creates `resource_bound_breached`. Therefore, the absence of a demonstrable upper flow rate prohibits the operation from being allowed under the hard limit.

### 4.18. `permission_requests`

The main database stores the side of the request that is understandable to the owner:

- source, session and unit;
- type of action;
- the real object or recipient;
- categories of transmitted data;
- upper limit of consequences and costs;
- required period and number of uses;
- owner's original text and exact interpretation;
- request status.

The valid right exists only in `guard.db` section 5 and is valid only if it matches the current snapshot of the guard's protected anchor.

### 4.19. `action_intents` and `action_observations`

The general present workplace specifies [24, §§6–9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md): `WorkspaceRef`, `WorkspaceRevision`, `WorkspaceUseRecord` and `WorkspaceConflict`. Accounting for use and an accurate listing of the original objects are associated with the intent and all supporting rights. An undefined use is not released once the process ends. Multiple file replacements and an SQLite transaction are not declared as a common atomic operation.

`action_intents` stores a unique intent number, the exact fingerprint of the action, unit, right, era, state and commit time before the start.

`action_observations` stores the observed outcome: success, confirmed failure, unknown, or the result of a subsequent reconciliation. The response from the external service is stored as evidence with the secrets cleared.

This is the subject history of the control process: `action_intents` records the prepared intent before transmission to the intermediary, and `action_observations` accepts certified observations from `guard_outcomes` via a stable number without re-counting. They do not replace the protective `guard_intents`/`guard_outcomes` and do not certify the actual shipment without an intermediary. Only C-05/C-03 write these master tables; only C-09 writes protective ones.

One intention number cannot correspond to two action fingerprints. One action does not receive a new number due to the loss of the response.

### 4.20. `incidents`

Identical repeated errors are combined by a secure cause fingerprint.

| Field | Meaning |
|---|---|
| `incident_id` | identity of the incident |
| `fingerprint` | class, object and root cause without secret |
| `severity` | `info`, `degraded`, `blocked`, `critical` |
| `state` | `open`, `retry_scheduled`, `human_required`, `resolved` |
| `first_seen_at`, `last_seen_at`, `occurrence_count` | repetition accumulation |
| `next_retry_at` | durable schedule |
| `owner_notification_state` | not yet sent, sent, confirmed |

Repetition does not create a new notification. The owner is notified only when there is a real need for a solution, a critical defense, or a final result.

### 4.21. `state_events`

This is the only organized subject journal.

| Field | Meaning |
|---|---|
| `journal_id`, `event_seq`, `event_id` | line, order within it and personality |
| `transaction_id` | group of indivisible changes |
| `aggregate_kind`, `aggregate_id` | modified object |
| `event_kind`, `event_schema_version` | private event type |
| `actor_kind`, `actor_id` | owner, control process, protective process, executor |
| `rule_id` | applied rule |
| `previous_version`, `new_version` | transition proof |
| `authority_epoch`, `run_generation` | protective era and generation process |
| `observed_at_utc` | time of the observed fact, if different from the record |
| `safe_payload` | data without secrets |
| `evidence_id` | detailed proof |
| `previous_event_mac`, `event_mac`, `mac_key_version` | versioned integrity chain verified against a separate checkpoint |
| `created_at_utc` | commit time |

Events are not updated or deleted. The correction creates a compensating event.

---

## 5. Two secure storages

The protected state is divided by record owner. One guard writes a small separate SQLite database `authority.anchor`; the name is stored as a role name, it is not a native binary format. One intermediary writes `guard.db`. Both databases are located on a local volume in separate access directories, outside the usual restored copy of `deepcreate.db`. The controlling process does not write to any of them. Diagnostic copies are not a source of valid rights.

### 5.1. Watchman's anchor `authority.anchor`

Table `authority_state` contains:

- `installation_id`;
- `authority_epoch`;
- `run_generation`;
- `anchor_sequence`;
- the reason for the last increase and the version of the scheme.

Table `scope_fences` contains `(scope_kind, scope_id, fence_version, closed, reason, owner_command_id)` for main activity, unit and individual entitlement. Table `protection_events` stores the minimum confirmed security facts before they are transferred to the main journal; the transfer is idempotent by its number. This is not a second history of goals. Snapshot `AuthoritySnapshot` includes installation identity, era, generation, anchor sequence, and applicable fence versions.

`journal_checkpoint` stores the last confirmed end of the main log: `(installation_id, journal_id, event_seq, event_mac, transaction_id)`. The only operation `record_journal_checkpoint(expected_previous, committed_head)` at the guard requires a match of the old end, the same setup/log line and a continuous new chain; any higher number will not be accepted. An accepted transition goes through the `DB_COMMITTED_PENDING_CHECKPOINT → CHECKPOINT_CONFIRMED` phases until the “accepted” response is received and the dependent action is allowed; the exact order of failure is specified in document 18, §2.1. The checkpoint receipt does not add a new recursively confirmed subject event to itself. `anchor_sequence` organizes security records, but its very change due to a checkpoint does not revoke unrelated rights: the admission checks the era, generation and applicable `scope_fences`.

The watchman is the only writer. The remaining processes receive the snapshot through an authenticated named pipe rather than trusting the copied file or unsigned number. For the base, `journal_mode=WAL`, `synchronous=FULL`, foreign keys and checking the actual values ​​after opening are selected; `-wal` and `-shm` are within the same access boundary. The epoch/fence change and guard event are written in one SQLite transaction; a success response is issued only after its completion. Homemade two cells, a current file switch, and a new anchor cryptographic scheme are not needed. `FULL` relies on a fair OS/drive reset and does not prove the stability of a particular computer without a power test. [Contract SQLite](https://www.sqlite.org/pragma.html#pragma_synchronous).

The global epoch increases when the entire installation is fenced: an old copy is restored, a rule shared by the installation is revoked, a key security material is changed, and a general inconsistency is detected. Stopping one main job increases it `fence_version`, and revoking one right - fencing this `lease_id`; unrelated rights are not extinguished. The intermediary additionally transfers the revoked right to the final state. The era and versions do not decrease; Counter overflow means limited mode, not zero crossing.

The start generation is increased only by the guard with each accepted start of the control process. It does not replace the epoch: its job is to reject messages, execution leases, and late responses from processes from the past run.

A missing/damaged anchor of an existing installation is not initialized to zero. Any unexplained discrepancy in identity, era, or protected reference state requires limited reconciliation; `max()` does not give out authority based on untrusted copies. A number from the main base above the anchor is also a violation, do not ask to quietly increase it. The rollback of the entire machine, including the anchor and keys, without an independent external witness is not guaranteed to be detected; a separate file and SQLite does not create such a witness.

### 5.2. Intermediary base `guard.db` and `capability_leases`

This database is written only by a protective intermediary. She contains the last seen `AuthoritySnapshot`, rights, intentions and observations, but her era copy has no power over the watchman's anchor.

Each right contains:

- `lease_id`;
- `authority_epoch`;
- `lease_version` and versions of applicable work/unit/right guards;
- `permission_request_id`;
- exact fingerprint of the scope;
- permitted type of action;
- the real object or recipient;
- acceptable data categories;
- limit on cost, frequency, number of uses and consequences;
- beginning and end;
- parental right, if there was a clear narrowing;
- status `active`, `consumed`, `expired`, `revoked`, `superseded`.

A right with an unknown field is invalid. A child cannot have a wider scope, term or limit than the parent.

### 5.3. `guard_intents` and `guard_outcomes`

The protective intermediary independently records the receipt of the intention and the actual initiation of the action. This allows you to verify the situation when the controlling process died after transmitting the request.

`guard_intents` binds `intent_id`, exact secure fingerprint, `reservation_id`, external one-time key, era/generation/fencing and state `prepared`, `dispatch_started`, `observed`, `unknown`. The intent number and its connection to the reservation are unique. `guard_outcomes` has a stable `observation_id` and a link to a sealed proof. Redelivery does not run the handler again. The main base first records the intent and reservation; the intermediary steadily accepts them and marks the beginning before the transfer of the OS; the observation is then transferred back. This is a sequence of two local transactions, not distributed atomicity.

The reservation receipt stores not only the ID, but also the immutable `logical_operation_id`, all applicable `limit_id/window_id`, units, upper marks, and rule version. Therefore, restoring the old main base does not lose evidence of possible post-snapshot consumption. Any launch capable of consuming a strictly limited resource has such a receipt before admission; `guard_outcomes` stores the observed flow rate or its unknown. These records do not calculate the second available balance and do not issue a budget: calculation and write-off still belong to one ledger of the main database.

The divergence of two bases does not automatically select a “winner”. `dispatch_started` without a proven outcome becomes unknown, even if the failure could have occurred before the first byte. `prepared` is allowed to be released only after proof that the old sender is protected and the beginning was not allowed. Restoring one master database does not restore rights. Lost `guard.db` freezes previous rights and keeps all potentially initiated actions unknown until reconciliation; an empty new base does not mean "nothing happened."

---

## 6. Writing a proof file

```text
назначить случайный evidence_id, класс данных и отдельный ключ объекта
→ получить поток байтов только в ограниченные буферы памяти
→ вычислять защищённый отпечаток и шифровать поток до первой дисковой записи
→ писать только шифротекст в временный файл с закрытым доступом на целевом томе
→ завершить стандартный проверенный формат аутентифицированного шифрования
→ принудительно сбросить и закрыть файл
→ неделимо переименовать в случайный путь evidence_id, без замены существующего объекта
→ открыть заново и проверить размер, шифротекст и аутентификацию всех блоков/конца
→ одной транзакцией добавить evidence_objects, ссылки и state_event
```

Until the last step, the object is non-canonical. Encryption uses a valid library and versioned format with order, length and termination checks; its own nonce/chunk protocol is not being invented. If the selected library does not provide proven bounded thread sealing, an object above the proven size is rejected before being written. Opening a temporary file and then “securely deleting” it is prohibited.

A lone completed object is moved to quarantine and joined only after checking its origin; It cannot be deleted just because of the lack of a link: it could have saved the only source or answer before it crashed. Only proven reproducible technical scraps can be automatically deleted according to a separate storage rule. Linking to missing file is not allowed. The application does not promise no open bytes in pagefile/hibernation/crash dump Windows: the native write limit does not control the entire OS; verification of such traces and the dump policy are part of the platform's clearance.

---

## 7. Main work states and units

### 7.1. Session

```text
open → stopped → open
open → archived
stopped → archived
```

The archive session is not doing the job. Reverting from an archive creates a new visible revision of the state, rather than deleting the fact of archiving.

### 7.2. Main work

```text
draft
→ executing
→ completion_candidate
→ verifying
→ awaiting_owner_acceptance
→ owner_accepted
awaiting_owner_acceptance → executing   после отклонения владельцем и принятого разбора причины
```

Waiting for rights, resources, restoration and stopping are blocking boundaries, not new semantic stages. This prevents the number of conditions from growing due to all possible combinations of causes.

An uncorrectable internal error creates a blocking boundary `invariant_failure`. Temporary unavailability of the supplier does not change the semantic stage of the entire work.

### 7.3. Unit of work

```text
proposed → ready → leased → running → result_submitted
result_submitted → verifying → accepted | rejected | stale
ready | leased | running → blocked
blocked → ready            после снятия точной причины
ready | leased | running → cancelling → cancelled
running → lost             при исчезновении процесса
```

Going to `accepted` requires a check record and `CommitPreconditions`. `stale` means changing the significant dependency `DependencySnapshot`, not any `EventSeq` or extraneous string; the result is saved but not applied. One `RowVersion` protects one mutable row. Unrelated results can be accepted sequentially if their dependency sets are still valid.

### 7.4. External action

```text
proposed → awaiting_permission → authorized
authorized → intent_committed → executing
executing → succeeded | failed_confirmed | outcome_unknown
outcome_unknown → reconciling → succeeded | failed_confirmed | unresolved
```

There is no transition from `outcome_unknown` back to `authorized`. A repetition is possible only as a new action after a confirmed verification that the previous effect did not occur, or through an external stable number that guarantees a single result.

### 7.5. Right

```text
requested → active → consumed | expired | revoked | superseded
```

No final state is returned in `active`. Reissue creates a new right.

---

## 8. Indivisible commands of the control process

Command options and failures are supplemented by passports [24, §9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md), [25, §§5–9](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) and [26, §§4.6, 5.6](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md). Queue C-05 is not a durable acceptance record: `QueuedVolatile`, the record without a checkpoint and the confirmed result are distinguishable. Maintenance accounting and resource reservation saves C-03; There is no separate scheduler database.

Each line modified by the command has its own `expected_row_version`. If the write target version or significant dependency has changed, the command does not attempt to merge the changes itself. The overall journal sequence is not used as a version of all work. Applying the result and initiating the consequence uses the fencing protocol of Document 17, §12.1; Validating a snapshot before entering a transaction without revoking serialization is not sufficient.

| Team | Changes in one transaction | Mandatory Event |
|---|---|---|
| accept owner's message | `owner_events`, if necessary `sessions` | `owner_event_received` |
| accept the goal revision | `objectives`, previous edition, `sessions`, `obligations` | `objective_accepted` |
| accept count | `work_units`, dependencies, branch limits | `work_graph_committed` |
| start trying | unit status, `unit_attempts`, the only reservation for this billable operation | `attempt_started` |
| accept result package | attempt, `result_packets`, evidence links | `result_submitted` |
| accept verification | `verification_records`, result status | `verification_completed` |
| apply result | target object, result, unit; flow is already reflected by individual certified observations, not by the verdict of the model | `result_committed` |
| request the right | `permission_requests`, unit condition | `permission_requested` |
| write down intention | `action_intents`, connection with an existing reservation or a new reservation for a separate operation, but not a second U of the same | `action_intent_committed` |
| record observation | `action_observations`, action, consumption | `action_observed` |
| start stop | main work, affected units | `stop_requested` |
| prepare a sequel | reconciliation status, new versions of inputs | `resume_prepared` |

A call to an external service never occurs inside a SQLite transaction. The transaction first commits the intent, then the guard process executes the action, then a separate transaction commits the observation.

---

## 9. Recovery after an unexpected restart

Additionally, unfinished uses of the common area are verified and results are provided for 24; derivative feedbacks are restored. The queue does not resurrect in memory as an allowed old attempt (25, §5). Combinations of failure, recall, expiration, checkpoint, and late flow are tested in 26, §§2–3; a transition recorded before the fence is not retroactively undone, and new consequences after the fence are not triggered.

The order is code, not model advice:

1. check the signature and integrity of the application version;
2. obtain from the watchman a complete photo of the protected anchor;
3. ask the guard to indivisibly increase the launch generation and get a new `AuthoritySnapshot`;
4. open the main database only locally;
5. check the integrity of SQLite and applied migrations;
6. check the continuity of `EventSeq` and compliance of object versions with the latest events;
7. check the presence of all canonical files to which there are links;
8. isolate lonely temporary and canonical files without links for subsequent parsing;
9. compare the epoch and control end of the log with the anchor; an older epoch does not revive rights, a database shorter than checkpoint or with a different MAC is not accepted; the unconfirmed tail undergoes local verification §2.1 of document 18;
10. reject all messages and leases from legacy launch generation;
11. mark incomplete attempts `lost`;
12. compare the intentions of the two bases and create `outcome_unknown` in the absence of a proven outcome;
13. check the free space on each participating volume and the ability to reliably write the next beacon;
14. write to the database the blocking recovery boundary that has already been in effect since the start;
15. generate a recovery report;
16. do not run previous units;
17. after an explicit “continue”, re-check the target, inputs, files, freshness of sources, limits and open actions;
18. remove only confirmed reasons for blocking and create new attempts without reviving old processes or rights.

Until step 1, the guard keeps admission closed: the window between the start and the boundary entry in step 14 does not allow operation. Reconciliation is allowed without a command only as a pre-registered local diagnostic, limited reading and checking, not the old reading/reasoning units. Network reconciliation of an unknown outcome requires a separate valid right to an accurate secure request and expense; the command “continue” does not in itself create such a right.

---

## 10. Backup and return

A consistent backup contains:

- a snapshot of the main database through the official SQLite mechanism;
- canonical proofs achievable from the snapshot;
- list of fingerprints;
- version of the application and scheme;
- encrypted data keys;
- copy verification report.

It is not considered suitable until a trial recovery to a separate location has passed integrity checks and selective decryption.

After returning the backup:

- the era of the guard's protected anchor does not decrease;
- all rights from the copy are invalid;
- open actions require reconciliation;
- the application remains in limited mode until the report and decision of the owner;
- a mixture of the old database and new files does not start.

The old snapshot below the current `journal_checkpoint` does not become the current base by simply removing the ban. For explicit recovery, the guard stores `restore_receipt`, linking the old protected end, the verified copy end, and the new random `journal_id`; the old line is not rewritten, the era rises. This operation is included only in the restoration agreement confirmed by the owner, and is not available to the usual `record_journal_checkpoint`. `EventSeq` is ordered within a line, the global event reference contains `(journal_id, event_seq)`. The first new end references the recovery receipt. If part of the previous history is unavailable, the report lists the unrecovered range and unknown activities; the continuation cannot be called the recovery of all data. Without an explicit decision by the owner about the indicated incomplete restoration, a limited mode remains. This is also not proof of external outcomes: their reconciliation and withheld expenses are stored against `guard.db` and secure receipts.

Before the resource ban is lifted, the restored ledger is checked against the combination of protection receipts for `logical_operation_id`, `reservation_id` and `observation_id`. One operation is not written off twice, but all potentially started operations after the snapshot again receive a confirmed contribution or an unknown upper estimate. You cannot treat missing lines as zero consumption or reset the window with a new `journal_id`. If `guard.db` is also lost or it is impossible to restore the expense limits, the affected limits remain blocked until a proven reconciliation / new accurate decision of the owner; the usual “continue” does not reset the flow rate.

---

## 11. Schema migration

Each migration is one part of an entire version of the application and contains:

- source and target versions;
- preconditions;
- verifiable transformation;
- checking the number and connectivity of records;
- a way to safely terminate before including a new version;
- a way to open an old copy as a separate old version if converting the data back is dangerous.

You cannot perform a permanent migration with a single untested copy. You cannot automatically run an old program on an already modified base. The mixed version is blocked.

---

## 12. Checks without which the scheme will not be accepted

1. Completing the process at each recording location results in a completely old or completely new state.
2. Two parallel results of the same version cannot both win silently.
3. The late result of a canceled unit remains in evidence and is not applied.
4. The dependency cycle deviates before committing.
5. The negative branch is preserved after merging.
6. Exhausting the total limit will block new reservations.
7. Individual branches do not exceed the joint limit when starting in parallel.
8. Returning a copy does not return rights.
9. Losing a response after an action does not cause blind replay.
10. The damaged search layer is recreated without changing the canonical data.
11. A missing canonical file puts the system into restricted mode.
12. Lack of space to the beacon is blocking the crossing.
13. Changing the system clock does not extend eligibility.
14. The window state after startup is loaded only after the actual running state.
15. Fifty-one owner responses have table, rule, check, and beacon paths in document 21.
16. The honeypot secret does not appear open in its own staging files, WAL, index, names or reports; Uncovered OS traces are recorded separately.
17. Transitions `reserved → unknown → settled`, partial billing, re-observation and re-delivery intent do not result in double debit or early release.
18. Damage, missing or rollback of anchor only/only `guard.db` includes prohibition; the rollback of the entire machine is marked uncovered without an external witness.
19. After creating a new working binding, the old path permission is not transferred: the new `WorkspaceBinding` has a new ID/version, a fresh object check and a new permission before the change.
20. Failure `DB_COMMITTED_PENDING_CHECKPOINT` does not allow the second execution of the command and does not issue a received ACK; a short/different chain relative to a single checkpoint is not considered a normal clean restart.
21. Returning a snapshot before a paid/unknown operation does not return the spent balance: guard receipts restore one deposit or the resource ban is maintained.

---

## 13. What this document prohibits from being implemented differently

- two writers of the main database;
- a separate text journal as a second truth;
- changing the accepted line instead of a new edition and event;
- a general arbitrary state object without closed types;
- storing money as a floating point number;
- restoring rights from a backup copy;
- conclusion of completion from the last message of the model;
- storing a single result only in process memory;
- automatic destruction of negative conclusions and primary sources;
- continuation of the previous attempt after a restart;
- repeating an external action due to a network error without verifying the persistent intent.
