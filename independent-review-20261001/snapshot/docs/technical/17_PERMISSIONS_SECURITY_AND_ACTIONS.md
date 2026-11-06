# Deep Create: Permissions, Security and Actions

**Addition 2026-09-08:** [24, §§6–9](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md) clarifies general work areas: `WorkspaceUseRecord` - resource retention, not right; To merge, you need consumer rights to all read areas and the exact target. [26, §§2–5](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md) clarifies expiration testing at the actual validity boundary and selective return of the modified version. A refund does not restore revoked rights, does not reduce costs, and does not reverse external consequences that have already occurred. These clarifications are mandatory for all modes along with the agreements below.

**Date:** 2026-09-07  
**State:** binding contract of all actions beyond pure reasoning  
**Main boundary:** model suggests action; the controlling process connects it with the goal; the owner or a pre-adopted rule creates the right; the protective intermediary alone performs the registered action.

---

## 1. Real threat model of the first version

The system must protect work from:

- erroneous or malicious model;
- downloaded ability;
- malicious text, page, file, archive, image and metadata;
- accidental error of the owner;
- parallel race;
- late response;
- process, network and power failure;
- restoring an old copy;
- secret leaks into a request, log, screen or error message;
- an internal module trying to bypass a single path of action.

The first local version does not promise to defeat an administrator who has completely taken over Windows and replaced the executable files. It also does not promise protection against an arbitrary fully trusted malicious process with the same user permissions being able to read memory/files of trusted processes or apply its DPAPI. Isolation refers to actual limited executors, rather than arising from different EXE names. Protection against general machine rollback requires an independent external root of trust; it is not in the first version.

---

## 2. Closed list of actions

```rust
enum ActionKind {
    ReadEvidenceFragment,
    ReadWorkspaceFile,
    WriteWorkerScratch,
    PrepareWorkspaceChange,
    CommitWorkspaceChange,
    SpawnRegisteredProgram,
    CallModelConnection,
    CallRegisteredEndpoint,
    ReconcileExternalIntent,
    ImportHermesSnapshot,
}
```

Sending a real message, publishing and payment are not included in the first working version. They cannot be expressed by the general `CallRegisteredEndpoint`; the future type of consequence will require its own type, form, mediator, reconciliation and testing.

A new enumeration option is prohibited by the assembly until the following are added at the same time:

- exact form of the sentence;
- the rule of canonization of the object and recipient;
- required type of law;
- upper estimate of consequences;
- intermediary handler;
- method of observation and reconciliation;
- beacons;
- positive, prohibitive and faulty checks.

---

## 3. Action proposal

```rust
struct ActionProposal {
    proposal_id: EntityId,
    work_id: EntityId,
    unit_id: EntityId,
    attempt_id: EntityId,
    kind: ActionKind,
    resource: ProposedResource,
    recipient: Option<ProposedRecipient>,
    data_refs: Vec<ScopedDataRef>,
    expected_effect: EffectDescription,
    worst_case_cost: ResourceUsage,
    dependency_snapshot_id: EntityId,
}
```

The model can only create this shape. It does not indicate which right is considered valid, and does not convey the secret.

`canonicalize_action` turns the sentence into a real machine object. Unknown, ambiguous or changing submissions are rejected.

---

## 4. Right as limited rental opportunity

```rust
struct CapabilityLease {
    lease_id: EntityId,
    owner_event_id: EntityId,
    work_id: EntityId,
    unit_id: Option<EntityId>,
    action_kind: ActionKind,
    canonical_resource: CanonicalResource,
    recipient: Option<CanonicalRecipient>,
    allowed_data_classes: DataClassSet,
    max_calls: ExplicitLimit<u32>,
    max_bytes: ExplicitLimit<ByteCount>,
    max_cost: Vec<ResourceBound>,
    not_before_utc: DateTimeUtc,
    expires_at_utc: Option<DateTimeUtc>,
    monotonic_deadline: Option<MonotonicDeadline>,
    parent_lease_id: Option<EntityId>,
    delegation_depth: u8,
    authority_epoch: AuthorityEpoch,
    scope_fence_versions: Vec<ScopedFenceVersion>,
    confirmed_conditions_hash: ContentFingerprint,
    issued_dependency_snapshot_id: EntityId,
    lease_version: RowVersion,
    status: LeaseStatus,
}
```

### 4.1. Rules

`ExplicitLimit<T>` has only variants `Finite(T)` and `Unlimited`; no field is not a third option. `None` in a required field `expires_at_utc` means explicitly shown perpetuity, but omitting the field itself is rejected by the schema. The monotonic term is tied to `run_generation` of issue and is not carried over by the number through a restart; a new execution requires re-verification and a new date for the current run.

- a missing field does not mean “without limitation”;
- perpetuity, unlimited number and no monetary limit are separate explicit meanings;
- the child is created only as an intersection with the parent;
- expenses and the number of uses of descendants are also written off from the general balance of the ancestors; creating two children does not multiply the parent's limit;
- the recipient, path, data, number, term and consequences of the child are not wider than the parent;
- changing the field shown to the owner changes `confirmed_conditions_hash` and requires a new decision;
- during the current run, the deadline is additionally checked using monotonic time;
- set the system clock back or doubt freeze the temporary right;
- an individual review changes the state and version of one right;
- the global era is raised only for mass protective fencing;
- re-issuance after revocation creates a new `lease_id`.

### 4.2. Subset checking

`derive_child_lease(parent, requested)` returns the child only when proven:

```text
вид действия тот же или более узкий
И настоящий объект входит в область родителя
И получатель тот же или уже
И классы данных являются подмножеством
И срок не позже
И число и объём не больше
И стоимость и последствия не больше
И глубина передачи разрешена
```

The vagueness of any comparison is a failure.

---

## 5. Trusted confirmation of the owner

The model can write the explanation, but the question, fields and main buttons build native local code from the canonical action.

The verified fingerprint includes:

- type of action;
- real object;
- recipient;
- categories and upper volume of data;
- limit of money and number of actions;
- term;
- main work and unit;
- versions of significant confirmed objects and `DependencySnapshot`, not the general event number of the entire installation;
- current era of authority;
- one-time question number.

The web view can only request to show the native window. It does not generate a confirmation event.

Interface rules:

- one question - one solution;
- safe failure is selected by default;
- external content does not create buttons and does not change captions;
- the link does not confirm the action;
- an expired or changed question is rejected;
- before issuance and revocation, a clear confirmation of intent is shown, as required by L-04b; after the last decision is made, the feedback is immediately sent along the trusted path without a model queue and without another question;
- re-issuance requires new confirmation;
- a major external consequence in the future will require enhanced identity verification.

---

## 6. Protective vaults and secrets

`authority.anchor` - separate SQLite guard base with `WAL + FULL`, era, generation and local fences; `guard.db` - an intermediary base with rights, one-time permission of intent and observations. Their writers are not interchangeable. The main base is only subject history and the only resource accounting. The complete recovery scheme and boundaries are in document 14, §5.

### 6.1. Secrets

The code does not allow the function `get_secret() -> String`.

Only the contract is allowed:

```rust
fn use_secret(
    secret_ref: SecretRef,
    connection: ConnectionIdentity,
    purpose: SecretPurpose,
    request: PreparedRequest,
) -> Result<SanitizedResponse, SecretUseError>;
```

Protective process:

1. checks the right, connection, purpose and exact address;
2. prints the secret using Windows for the current user;
3. inserts it only into a predefined connector field;
4. completes the request;
5. clears headers, addresses and errors;
6. overwrites the temporary buffer as much as the library used allows;
7. returns only the cleared response.

The main encryption key for the database and archive is sealed through Windows data protection without a mode that allows decryption by any computer user.

DPAPI in the current user's area associates the decryption with his credentials, usually also with the computer; this is not a "only our EXE" limitation. Portability of backup keys to a new machine cannot be promised without a separate, proven key recovery procedure. Until it is proven, the copy is marked as “verified for this account and machine,” rather than a generic rescue copy. [Microsoft: CryptProtectData](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata).

Provider access keys, object encryption keys, and secure fingerprint keys are separated by purpose and version. Vendor secrets are not included in a normal backup; Sealed data keys are included only under a copy agreement. Changing the key does not lose the ability to read stored evidence. The secret is only allowed through secure input; a detected or owner-marked secret value in plain text quarantines the message without sending the model or opening the log entry. Detecting an arbitrary unknown secret from text alone is not guaranteed: a secure input path cannot be replaced by the promise of a perfect filter. The privileged encryption surface does not return an arbitrary key, but only the result of the object being sealed/allowed to read. Buffer overwriting is a lifetime limit, not a promise to remove all copies from OS memory, dump or libraries.

### 6.2. The Age of Empowerment

The maximum confirmed value is stored by the guard outside the normal backup copy of the main database. The main database contains the last observed value.

If there is a mismatch, limited mode is activated; It is forbidden to assign power through `max()` based on numbers from the restored databases. The older copy does not lower the anchor; a surprisingly larger number also warrant investigation. The missing anchor of an existing installation is not recreated with zero. Individual recall and work stop use the precision guard `lease_id`/`work_id`, and the global era only fence the entire installation. A full return of the entire Windows along with the anchor remains uncovered without an external witness.

---

## 7. Internal channels

All local communications use a Windows named pipe with an explicit DACL, disallowing remote clients, and checking the expected identity/generation of the process. A DACL based on one user SID does not distinguish between all of its fully trusted processes; "Only PID allowed" is not a separate Windows right. Trusted communication is additionally established through a handle/one-time secret passed at startup and process token verification, and for a limited executor through its separate AppContainer SID. An unknown client, repeated secret, incorrect sender, or unverified identity means the connection is broken. The open local port is not used.

Each message contains:

- protocol version;
- type of message from a closed list;
- request number;
- one-time connection number;
- `run_generation`;
- sender and expected recipient;
- limited body size;
- imprint of essential fields.

The durable `request_id` is separated from the one-time connection ID. The command contains the expected versions of the affected rows, a dependency snapshot, and the target scope. The response "received" does not equal "applied"; only the result with the ID of the subject transaction or security receipt confirms the fixation. The old `AuthoritySnapshot` cannot be presented as a portable executable pass.

When a process starts, it receives a handle or one-time channel secret through protected inheritance. All other descriptors are not inherited.

Re-delivering the command with the same number and the same fingerprint returns the same result. The same number with a different body is rejected as a violation.

---

## 8. Executor isolation in Windows

The Windows Jobs object controls the tree and boundaries, but is not itself a complete access boundary. Therefore they are used together:

- a separate AppContainer profile without network capabilities and with minimal file ACLs; one restricted token is not considered a proven equivalent replacement;
- a separate task object for each tree;
- prohibition of child processes exiting the task object;
- file rights only to the allocated temporary area;
- lack of rights to the main database, archive, secrets and user directories;
- lack of direct network;
- managed `TEMP` and `TMP` on the selected volume;
- prohibition of inheritance of unnecessary descriptors;
- limit of memory, time, processes and output;
- one tested channel to the control part.

A pure model executor does not execute the network request himself at all: he gives a prepared instruction to the connection intermediary.

A registered local program is run by a separate bounded process. If a program cannot be safely contained and observed, it is not allowed into the first version.

Windows profile specific permission checks for direct opening of user directory, named pipes, registry, network, child exit from Job Object, handle inheritance, and access to proxy/guardian processes. `CREATE_SUSPENDED → assign job → resume` or a proven equivalent sequence does not leave a window for the child process before joining the tree. Wide capabilities and ACL `ALL APPLICATION PACKAGES` are not added for the sake of compatibility. An application with an MSIX identity does not automatically become an AppContainer. [Microsoft: creating AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer).

---

## 9. Ability Life Path

The downloaded ability is not loaded into the kernel process as a library.

```text
изолированный вход
→ проверка настоящего вида файлов
→ предел размера и глубины распаковки
→ вычисление отпечатков
→ чтение заявленного назначения
→ выявление кода, программ и адресов
→ машинный перечень входов, выходов и требуемых прав
→ искусственные и враждебные испытания
→ работа в тени
→ отдельный допуск точной версии
→ отдельное право на каждый настоящий запуск
```

`SkillManifest` contains origin, version, imprints, purpose, entry and exit forms, required abilities, data classes, programs, addresses, known failures, checks and resource limit.

The text ability is also untrusted data. It cannot simply be added to the fixed basis of the instructions.

Installation does not imply the right to use. An updated fingerprint means a new unaccepted version.

---

## 10. File action

### 10.1. Preliminary analysis

Before a change unit is created, the individual unit sets:

- purpose of the file;
- the real identity of the Windows object;
- owner and consumers;
- addictions;
- security;
- permissible range of change;
- possible cascading damage;
- method of check and return.

### 10.2. Real personality

The path string is cast to the open handle. Redirection points and the combination of volume number and file number are checked. Checking then using the line “open again and replace” is not enough: substitution may occur after checking. The handler must associate the permission with the authenticated parent directory, the target, the original content, and the specific replacement operation. For a file that does not yet exist, the parent directory and the absence of a name are ensured, and the creation uses a collision failure. Unexpected hard links, ADS, device/UNC paths, reparse point in any component and changes in workspace binding are rejected to a separate tested tolerance. The file number does not prove that the contents are unchanged.

The first version allows replacement only on a tested local volume and if there is a proven absence of the replacement window in the selected Windows handler. If the desired atomic operation on the authenticated object/directory cannot be proven, fail `file_identity_race_unproven` rather than fall back on the path string. Repeating `GetFinalPathNameByHandle` before recording is not enough. A return is also a new change with a new right and check: it must not overwrite someone else's entry that appeared after the original replacement.

### 10.3. Change

1. create a separate working copy;
2. apply the change only to it;
3. check the difference, shape and dependencies;
4. record the intention of replacement;
5. create a fully prepared object on the target volume;
6. perform a local indivisible replacement, where the file system guarantees this;
7. reopen and check the total;
8. Save the return method and observation.

An intervolume move is not called an indivisible swap. Transferring the result from the working copy to the user file is a separate action with a consequence.

---

## 11. Network action and connector

The connector does not accept an arbitrary address from the model. His version states:

- scheme, node, port and valid path;
- redirection rules;
- login method;
- request and response form;
- maximum size;
- whether the service supports a stable key;
- how to ask the outcome of the operation;
- which answers are final;
- which errors are truly temporary;
- what data the service can store.

`send_registered` checks the address after all conversions. Unintended redirection is prohibited.

Error `400` itself is not temporary. Replay is only allowed on a tested combination of vendor, connector version, and normalized temporary cause attribute.

---

## 12. Intention, action and observation

```rust
struct ActionIntent {
    intent_id: EntityId,
    idempotency_key: IdempotencyKey,
    proposal_hash: ContentFingerprint,
    action_kind: ActionKind,
    canonical_resource: CanonicalResource,
    recipient: Option<CanonicalRecipient>,
    data_manifest_hash: ContentFingerprint,
    lease_id: EntityId,
    lease_version: RowVersion,
    authority_epoch: AuthorityEpoch,
    run_generation: RunGeneration,
    scope_fence_versions: Vec<ScopedFenceVersion>,
    dependency_snapshot_id: EntityId,
    reservation_id: EntityId,
    worst_case_usage: ResourceUsage,
    reconciliation_method: ReconciliationMethod,
}
```

Sequence:

1. canonize the proposal;
2. check eligibility and reserve the top cost;
3. write down your intention steadily;
4. enter the protocol §12.1 and immediately before sending, repeat the check of the era, fences, version of the law, dependencies, object, armor and conditions;
5. the protective mediator performs the action;
6. the outcome is first preserved as evidence;
7. observation, actual consumption and allowable change are recorded by the transaction;
8. loss of response creates `outcome_unknown`;
9. replay is blocked until reconciliation.

A local strong key only provides one-time use when the connector has proven that the key is supported by an external service. Otherwise, the unknown outcome is not automatically repeated.

A late result after expiration or revocation is retained as a fact, but does not trigger subsequent actions or be accepted as a work product. This does not prohibit recording the observation of an effect that has already occurred, the expense and the unknown: a prohibition of such recording would make the story false. Revocation does not revoke bytes already committed to the OS/service, and does not promise to stop remote computing or return any money charged.

### 12.1. Linear recall and tolerance point

The general protocol `AdmissionFence` belongs to the guard; it is used by the mediator before the actual start of the action and by the controlling process when `commit_transition` results. The initial prototype uses one short native gate per installation to have an unambiguous order; data is fenced into precise areas. Gate - synchronization of trusted processes, rather than independent protection from malicious processes with their credentials.

1. `begin_fenced_operation` does not create a future-proof token. Inside the held gate, it checks the current `AuthoritySnapshot`, area, term/version of the right, `CommitPreconditions` and `DependencySnapshot`, and for the consequences also the fixed intention and reservation. The unknown is a refusal. Waiting for model, DNS, connection establishment and HTTP response inside gate is prohibited; The connector prepares everything that does not transmit protected content in advance under its restrictions.
2. For the action, the intermediary commits `dispatch_started` to `guard.db`, then, without releasing the gate, passes the exact request/local operation to the OS. This is the starting point of `D`; no service response. To accept the result, the controlling process holds gate until the completion of `COMMIT` of the main SQLite database - point `C`. `finish_fenced_operation` releases the gate, but does not issue permission to repeat the action.
3. The accepted trusted review closes admission to new entries and requests the same gate without queuing normal work. Below it, the guard steadily raises the required fence and writes down the protective fact - point `F`. The mediator reflects the final state of the law, and the main base then catches up with the protective fact by its ID. A “revoked” receipt is issued after `F`; before this, the interface shows "revocation in progress" rather than premature success.
4. If `F < D` or `F < C`, action/acceptance with the old version must be refused. If `D < F`, the effect may have already gone away and remains confirmed or unknown; automatic cancellation is not promised. If `C < F`, the result was accepted before revocation and is not considered late; the next step after `F` is prohibited. Checking the right outside the gate without holding until `D`/`C` does not fulfill this agreement.
5. Critical areas must have a measured limited profile. When stuck, the guard terminates the owner process of gate, waits for observed completion, and reconciles the pending operation until the fence is confirmed. An abandoned gate/lost guard always closes admission rather than making the operation successful. If the OS does not acknowledge the I/O completion/status, `stop_unconfirmed` is shown; it is impossible to prove instant termination by such a time-out. Selected Windows primitives, I/O cancellation behavior, COMMIT interruption, and re-entry after the death of the owner are mandatory prototype evidence, not an implied warranty of the document.

Physical file replacement refers to `D`, and `C` accepts the verified semantic result: there is no atomicity between NTFS and SQLite. If there was a failure between them, the file might have changed; intent, old/new content, and observation are needed for reconciliation, not for asserting that the file “hasn’t changed.” Canceling an external/file effect that has already been done is a separate future action with a new right, if such a type is allowed at all.

The fence does not block a closed list of local protective transitions: recording a recall, observed outcome/expense, process loss, checkpoint and recovery report. They pass `commit_transition` with the current service identity, do not require revival of the revoked right, and cannot accept the model result, begin network reconciliation, or new effect. Otherwise, the stop would have banned its own honest magazine. After the point `C`, but before the checkpoint, the result is not yet shown as accepted; a fence that occurs in this interval does not carry forward `C` retroactively and prohibits dependent new actions.

---

## 13. Hard resource limit

Before a paid or limited operation, the worst allowable expense is atomically reserved:

```text
сумма единственных текущих вкладов resource_ledger
+ верхняя оценка нового действия
≤ предел владельца
```

If the upper estimate cannot be obtained, action under the hard limit is prohibited. The owner can separately set an explicit "unrestricted" state.

The formula of each position and the transitions `reserved(U) → unknown(C,R) → settled(C)` are defined only in document 14, §4.17. The reservation and unknown expense of one operation do not add up twice. Parallel branches share the parent remainder with one transaction `commit_transition`; the intermediary receives `reservation_id` and a confirmed upper flow, but does not change the overall ledger. Recall, network error and restart do not release the unknown cost. A replay observation is idempotent; a new potentially paid attempt is reserved additionally, and the previous unknown outcome remains occupied.

---

## 14. Hermes Bridge

Bridge process:

- runs only when the source is explicitly selected;
- has read rights only to the selected session or snapshot;
- copies data to the isolated Deep Create input;
- calculates origin, time and fingerprints;
- does not write to Hermes;
- does not receive Deep Create rights;
- is not a startup or recovery condition.

The imported session text does not create a setting or resolution.

---

## 15. Self-development and renewal

The new version is created in a separate place and is accepted in its entirety. The author of the change cannot:

- change the security checks that evaluate its version;
- change the guard, the issuance of rights, secrets and a single path of action;
- expand data classes;
- remove mandatory beacons;
- include your version.

Changing the protective basis requires the owner's explicit decision. Self-admission is only allowed for a reversible mutable region after old, new, hidden and hostile checks.

After a failure, the entire old or entirely new version is launched. Mixing of files is prohibited by signed list and fingerprint verification.

---

## 16. Security points and internal responsibilities

| Function | Sole Responsibility |
|---|---|
| `canonicalize_action` | get real closed view of action |
| `describe_consent` | build an accurate native confirmation representation |
| `record_owner_decision` | make a one-time decision on the expected version |
| `issue_lease` | create a right from a trusted decision |
| `derive_child_lease` | provably narrow the right |
| `revoke_lease` | after trusted confirmation, protect one right under §12.1, then reflect the final state |
| `verify_lease` | check area, term, version and era |
| `reserve_effect_budget` | internal part of `commit_transition` main ledger; not a self-employed intermediary writer |
| `prepare_intent` | steadily write down intention before consequences |
| `dispatch_verified_intent` | execute registered handler |
| `record_action_observation` | save observed outcome |
| `reconcile_intent` | find out unknown outcome without blind repetition |
| `store_secret` | seal new secret meaning |
| `use_secret` | apply secret without returning public value |
| `revoke_secret` | stop using value |
| `stage_skill` | put ability into isolated input |
| `admit_skill_version` | allow accurate verified version |
| `open_verified_file` | open and verify the real object |
| `commit_file_replacement` | perform verified local replacement |
| `spawn_registered_program` | run limited tree |

Table names are local responsibilities, not additional arbitrary notation points. The external command passes the only register `C-*` document 19: `handle_command → decide_transition → commit_transition`; the executing permit always additionally passes `AdmissionFence`. The public watchman operation `establish_fence(FenceScope)` selects a precise region or global epoch based on a received command; `begin_fenced_operation`/`finish_fenced_operation` are internal operations of the admission protocol, not independent model commands. The watchman does not receive the meaning of the goal and does not accept the result as a control process.

Functions cannot have variants that bypass the general check for a mode, provider, or ability.

---

## 17. Mandatory checks

1. Model text and imported "permission" do not create a right.
2. The outdated native window does not confirm the changed object or amount.
3. The child does not extend the path, recipient, data, term and limit of the parent.
4. The revocation of one right does not revoke the others.
5. Stopping work extinguishes its old barriers, preserving unrelated work; general average extinguishes all rights of the old era.
6. Setting the clock back freezes temporary rights.
7. The performer cannot directly open the database, archive, user file or network.
8. The child process does not exit the job object.
9. The honeypot secret is missing from the order, response, error, log, interface, and normal backup.
10. Unauthorized address and redirection are prohibited.
11. The file redirection point and object change between check and replacement are detected.
12. Inter-volume movement is not used as an indivisible replacement.
13. Failure after intent and before response creates an unknown outcome.
14. An unknown outcome without external repetition does not repeat.
15. Two parallel reserves do not exceed the last balance of the limit.
16. After installation, the skill does not have the right to launch or network.
17. An updated ability imprint requires a new clearance.
18. The bridge cannot write to Hermes.
19. The new version cannot change its own set of mandatory checks.
20. Races `revoke/dispatch`, `revoke/COMMIT`, expiration and change of binding are executed in all orders before/within/after gate; after `F` there is no old `D`/`C`, before `F` the possible effect honestly remains taken into account.
21. Completing the gate owner before sending, at I/O and at COMMIT does not automatically return admission and does not create a false `stop_confirmed`.
22. Two children of the same right do not double the total remainder of the parent; The intermediary does not accept the same reservation for different purposes.
23. Another restricted artist's DPAPI is not available in the tested profile; the fully trusted process of the same user is clearly marked outside the promised boundary, rather than being used as proof of complete protection.
24. The absence of a proven top score prohibits paid admission; an unexpected bill above the reservation is fixed without cutting off and covers the following expenses.

---

## 18. Signs of false security

An implementation is rejected if:

- the intermediary is a normal function, but the performer has a direct network or files;
- the task object is used as the only security boundary;
- the secret can be obtained using an open string;
- the web view itself creates a confirmation;
- the supplier's address comes from the model's response;
- the downloaded library is loaded into the kernel;
- any module can add a string type of action;
- repetition is determined by one response code;
- a stable local key is issued as a guarantee of external non-recurrence;
- returning the base reduces the epoch;
- a right without a term, number or limit is silently considered perpetual and unlimited.
