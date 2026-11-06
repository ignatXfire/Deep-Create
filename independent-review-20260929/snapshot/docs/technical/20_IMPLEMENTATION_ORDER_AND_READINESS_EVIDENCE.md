# Deep Create: implementation order and proof of readiness

**Date:** 2026-09-06  
**Updated:** 2026-09-07  
**State:** mandatory sequence for creating the first version  
**Main boundary:** The next stage begins only after the verified output of the previous one. Having classes or a nice window does not count as working for the system.

---

## 1. Determination of readiness

An opportunity is considered created only if:

1. accepted requirement;
2. subject type and closed states;
3. one rule owner;
4. valid public function;
5. positive check;
6. prohibitory check;
7. check for failure or recovery if the status changes;
8. obligatory indicator of a significant transition for its owner; pure functions are covered by verification and do not maintain a separate log;
9. safe explanation in the interface;
10. entry in the correspondence matrix;
11. lack of a new workaround;
12. work in a clean build on the target Windows.

If only the normal successful path is tested, the opportunity is not ready.

---

## 2. First end-to-end chain

Before expanding the functions, one minimum chain must work:

```text
доверенное сообщение владельца
→ новая сессия
→ предложение цели
→ принятие цели
→ одна единица работы без внешних последствий
→ искусственный исполнитель
→ структурный пакет результата
→ независимая машинная проверка
→ кандидат завершения
→ принятие владельцем
→ закрытие приложения
→ восстановление того же принятого состояния
```

It still lacks a real model, a custom file, a network, an ability, and Hermes. This is deliberate: state, origin, completion and restoration are first proven.

---

## 3. Stage 0 - freezing of the technical project

### Create

- documents 13–21 and clarifications 23–26; historical report 22 is stored separately;
- dictionary of all states and events;
- map of open functions;
- map of resolved dependencies;
- matrix 51 solutions;
- a list of external properties that require checking on Windows;
- a journal of yet unproven engineering assumptions.

### Do not create

- production code;
- screen layout passed off as architecture;
- general executor of agents;
- extensions and backward compatibility with non-existent version.

### Exit

- there are no open decisions of the owner;
- each item of the first version has a future module, verification and beacon;
- detected inconsistencies in ownership and courses of action are eliminated and re-checked;
- unproven properties of Windows, storage and external services are listed with testing and prohibition of transition to the dependent stage; The absence of a comment while reading is not considered proof of the absence of all errors.

---

## 4. Stage 1 - reproducible empty build

### Create

- new separate project `D:/Projects/DeepCreate`;
- Rust workspace and Tauri shell;
- fixed dependency locking files;
- strict Rust and TypeScript settings;
- automatic checking of prohibited dependencies;
- reproducible assembly of the first required executable node; the remaining individual processes are added at their own stages, and not as empty packages;
- version page without access to Hermes.

### Check

- clean assembly without warnings;
- re-build uses the same fixed dependencies;
- the window does not have direct file, network and command line capabilities;
- run in a clean test environment without Hermes; For the sake of this test, you cannot delete or stop the current owner's profile;
- executable files have a single version number and a list of fingerprints;
- own `unsafe` is missing outside `dc-windows`; third-party dependencies are checked separately and are not passed off as code without `unsafe`.

### Exit

An empty stand-alone application launches and honestly shows that it does not yet know how to do the job.

---

## 5. Stage 2 - subject types and pure transitions

### Create

- identifiers;
- types of goal, obligation, main work, unit, attempt, result, right, action, event and limit;
- semantic stages and individual blocking boundaries;
- pure `decide_transition`;
- closed error classes;
- generators of valid and invalid sequences for testing.

### Check

- impossible transitions are not representable or are rejected;
- the target has exactly one current edition;
- the graph does not allow a cycle;
- a unit without one output or connection with an obligation is rejected;
- a descendant of rights is always narrower than the parent;
- parallel reserves do not exceed the limit;
- the late result of the old version is not accepted;
- blocking reasons do not inflate the semantic machine.

### Exit

All subject solutions are reproducibly tested in memory without database, network, time and Windows.

---

## 6. Stage 3 - main base and evidence

### Create

- `deepcreate.db` with one writer;
- numbered migrations;
- `commit_transition`;
- ordered table of events;
- archive of evidence with opaque names and protected integrity check;
- encryption of the database and objects with a key sealed by Windows;
- resource journal;
- agreed image and trial restoration.

### Check

- full synchronization is actually applied;
- foreign key and strict tables are included;
- a failure at each fixation point results in an old or new state;
- linking to an unsealed file is not possible;
- a lonely object is detected safely;
- damage to an event, file or connection enables limited mode;
- the decoy secret does not appear in the event database and normal diagnostic output;
- the backup copy is opened separately and does not return rights.

### Exit

The storage slice is reproduced through the test input: received message, one unit, artificial result, machine check and state reopening. The trusted window and individual executors will appear in the following steps. Process termination checks for process failure; real power loss or its reproducible low-level simulation is a separate test. One is not passed off as the other.

---

## 7. Stage 4 - watchman, era and artificial intermediary

### Create

- `authority.anchor` as a separate SQLite watchdog database with full synchronization, without a homemade disk recording format;
- separate `run_generation`;
- guardian of process trees;
- secure local channel;
- rental opportunities;
- artificial file handlers, networks and programs that do not change anything real;
- intention, observation and unknown outcome;
- immediate stop.

### Check

- the old process cannot send a message in the new generation;
- individual revocation does not extinguish other rights;
- the new global era extinguishes all old rights, and the fencing of one work preserves the rest;
- the text of the model does not create the right;
- unknown type of action is prohibited;
- the intention is always written down before the artificial effect;
- loss of response creates an unknown outcome and prohibits replay;
- a frozen kernel does not prevent the guard from stopping trees and closing the intermediary;
- review, sending and acceptance of the result have an observable ordering point: who passed first is confirmed by the log; an effect that has already been sent is not declared cancelled;
- the unconfirmed tail of the main log after a failure requires reconciliation with the guard checkpoint.

Additionally break the process at each boundary `DB_COMMITTED_PENDING_CHECKPOINT → CHECKPOINT_CONFIRMED`: A successful owner acknowledgment or dependent dispatch cannot be obtained until the second commit. Repeating a receipt with the same vertex is safe, the conflict of the previous vertex is rejected, and the receipt does not generate infinite events of itself. Check the pair `journal_id + event_seq`, the protected node and the transaction number, and not just one counter. These tests check inter-process consistency; they do not turn two bases into one atomic transaction.

Check the return to the old copy in the presence of newer observations of the intermediary: a new log line arises only after the confirmed decision of the owner and maintains a connection with the previous control point. The consumption of the lost tail is restored according to the operation/reserve/observation numbers without double counting. The unknown outcome holds the upper bound; the missing certificate prohibits new expenditure. The old copy does not reset the daily or total limit, does not erase the consequences, and does not restart old attempts.

### Exit

The security sequence passed the listed artificial tests. Access to real data remains dependent on Windows isolation testing, key storage, and specific I/O handlers.

---

## 8. Stage 5 - trusted window and management

### Create

- `UiProjection` as a single snapshot of the map, list and details;
- closed `UiIntent`;
- native confirmation of rights and sensitive settings;
- center of pending and existing rights;
- prepared and immediate stop;
- maintaining the position of the interface and ordinary unfinished forms without secrets and without turning a draft into an accepted solution.

### Check

- the web view cannot create a confirmation;
- changing the amount, object or version makes the question obsolete;
- map and list show one snapshot;
- the old screen does not change the new version;
- closing the window does not stop or continue work;
- the usual recall under L-04b is valid after the required confirmation of the owner and stable fencing; wait is not shown as a completed recall, immediate stop has a separate emergency path;
- control is available via keyboard and voiceover;
- the state is distinguishable without color or movement.

### Exit

The owner can observe and control the artificial end-to-end chain without the hidden power of the interface.

---

## 9. Stage 6 - context and artificial model

### Create

- plan for information needs;
- search in an artificial archive;
- final working selection;
- seven layers of instructions;
- individual `ModelResult`, `TrustedAttemptEnvelope` and collected by the local system `ResultPacket`: the model writes only the contents of `ModelResult`, the attempt identity, connection and observed flow are taken from the certified local trace;
- real versioned JSON Schema for these three forms and all nested forms from 15 §2.9, generated from canonical subject types; fixed in the unit `OutputContract` with numerical limits;
- internal conversion of `adapt_for_provider` to C-07 and verification of the exact body `validate_dispatch_preconditions` inside the C-09 shipment;
- verified abbreviation;
- artificial connections with different capabilities and data rules.

### Check

- broad search does not mean wide disclosure;
- the secret is excluded;
- contradictory sources are not hidden;
- a mandatory obsolete fact blocks sending;
- strong and weak settings change only the sixth layer;
- a malicious command in the external text does not create a right;
- reduction does not lose prohibition, denial and unknown outcome;
- the same fixed selection, setting and converter version create the same instruction bytes; updating an unrelated branch does not invalidate the result;
- a repeated verification unit does not create an endless chain of inspectors;
- unconfirmed independence is not upgraded to proven independence by changing the role name.

For schemes, perform forward and reverse serialization of valid examples, checking empty arrays, Cyrillic alphabet and each exact size boundary. Separately reject duplicate key, unknown field/value, extra text around JSON, incorrect encoding, missing mandatory criterion, non-existent material and model-substituted service fields. All closed `ResultParseError` from 15 §2.9 must have a test case. Check that a false shell, a mismatched mediator trace, and a modified expected pattern are not treated by asking the model to “write the correct numbers.”

After converting and waiting in line, change the recipient, rate, data class, entitlement, significant version, and actual request body in turn. Each case is required to require a new admission or refusal before dispatch; updating an unrelated branch does not itself invalidate a valid request. Admission should not turn into multiple permission.

### Exit

The full model path works reproducibly without a real supplier.

---

## 10. Stage 7 - real connection of the model

### Create

- one connector from one supplier;
- secure use of the secret without returning a string;
- exact connection identity;
- strict screening and explicable ordering;
- local queue with back pressure;
- long lasting repetitions;
- actual accounting of consumption;
- one independent test route.

### Check

- the bait secret is absent in all exits;
- the address cannot be changed by the model’s response;
- concurrency does not overflow the supplier's queue;
- temporary `429`, `502`, `503` and the break are restored within limits;
- `400` is repeated only according to a known time signature;
- permanent form error does not loop;
- restart saves the number of attempts and the time of the next retry;
- the hard limit cannot be circumvented by replacing the model;
- the verifier actually satisfies the independence requirement.

### Exit

One unit of reasoning goes through a true model path with complete origins. Transferring the selection to the supplier and the possible cost are taken into account as real consequences; This slice does not change user files, publications and messages to people.

---

## 11. Stage 8 - real local files

### Create

- assignment of a workplace separately from the main job;
- preliminary file parsing;
- the real identity of the Windows object;
- isolated working copy;
- checking differences and dependencies;
- local indivisible replacement;
- verifiable return.

### Check

- redirection point;
- changing the goal between checking and opening;
- intervolume operation;
- parallel change by the user;
- late result of an old branch;
- lack of space on each involved volume;
- completion of the process before and after replacement;
- restoration of previous contents.

### Exit

One explicitly allowed file edit is done through the full path and does not allow hidden direct access by the performer.

---

## 12. Stage 9 - full screen of work

### Create

- normal conversation;
- graph and peer list;
- details of the node;
- progress of the inspection;
- connections of evidence;
- incident center;
- settings of models, depth of work and memory, including a clear preview of the change suggested by a common phrase;
- restoring the position after loading the actual state.

### Check

- artificial delay of representations;
- the same actions from the map and list;
- scale up to 400 percent;
- contrast and control without color;
- reduced movement;
- return a pointer after closing the question;
- no loss of zoom, scrolling or selection;
- one notification per recurring error;
- lack of messages to the owner about each beacon;
- two main works: waiting for the right and stopping one do not extinguish the permitted independent work of the other;
- drafts of regular forms are restarted; Secret fields are not saved in drafts.

### Exit

The interface honestly shows a proven system, rather than simulating missing features.

---

## 13. Stage 10 - Hermes Bridge

Create only a manual reading snapshot of the selected session through an isolated input.

Check:

- lack of registration in Hermes;
- no transfer of his rights and secrets;
- malicious commands in history;
- partial and repeated import;
- origin and imprints;
- removing a bridge without breaking Deep Create.

A permanent two-way exchange is not created.

---

## 14. Level 11 - self-development suggestions

At first only the following are allowed:

- proposal for a new model setting;
- new derivative memory;
- new test script;
- candidate ability;
- a new whole version in a separate place;
- work in the shade.

Self-admission of a limited mutable area is enabled only after proof that the candidate does not change rights, guards, secrets, log, beacons, termination conditions, and own checks.

---

## 15. Long end-to-end test

The minimum duration is determined not by a calendar promise, but by the passage of all events:

- several starts and one abrupt restart;
- temporary unavailability of the supplier;
- filling the queue without losing tasks;
- exhaustion of one connection and safe replacement;
- reaching a hard limit;
- failure of one performer;
- unknown outcome of artificial action;
- request and denial of rights;
- prepared stop and continuation;
- immediate stop;
- lack of space;
- modification of the source file by the owner;
- checking the final materials;
- final acceptance by the owner.

Only after this the name “first working version” is used.

---

## 16. Checking each change

For each edit, formatting, assembling the affected path, checking for prohibited dependencies, and verifying the requirements map are required. The remaining tests are selected according to the amended contract, rather than being formally launched for each text correction:

| What's changed | What needs to be repeated |
|---|---|
| pure subject rule | positive and forbidden inputs, limit values, dependent transitions |
| message or interprocess protocol | form version, unknown view, redelivery, untrusted sender |
| state record | version conflict, late result, failure before and after commit, recovery |
| law, secret, isolation | bypass attempt, recall simultaneously with action, decoy secret, default failure |
| model connector | exact request and response, actual cost, change of address, repetitions and unknown outcome |
| saved data form | open old supported version, update copy, failed update, revert |
| performance | correctness of the image, keyboard, sensitive fields, saving position |
| explanatory document only | links, contract numbers, consistency with the source of the solution |

A general acceptance run is performed before releasing a version, when all selected change checks have already passed. The tests are run in a separate workstation with artificial secrets and files.

Changing the guard rule additionally requires adversarial testing and testing in which the condition is deliberately flawed to ensure that the test actually catches the bug.

---

## 17. Edit size rule

One edit must close one verified vertical meaning:

```text
тип
→ переход
→ запись
→ событие
→ проверка
```

Not accepted:

- a huge edit of the “entire frame” without a working path;
- hundreds of empty interfaces;
- one new layer without a caller;
- pure renaming along with a change in behavior;
- temporary bypass of security for the sake of demonstration;
- commented out old path;
- copy of the function for the second mode.

---

## 18. Stage report states

- **COMPLETE** - all stated positive, prohibitive, failure and recovery conditions have been proven.
- **GAPS** - lists the exact terms that are not covered; the next dependent stage does not start.
- **INVALID** — the check could not prove the result or the origin was violated; the work cannot be considered partially finished.

Nice manual behavior does not raise `GAPS` to `COMPLETE`.

## 19. Inclusion of clarifications 2026-09-08 in the current creation sequence

The documents [23](./23_EXTERNAL_CRITIQUE_AND_REFINEMENTS_20260908.md), [24](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md), [25](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) and [26](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md)] do not form a deferred wish list: the corresponding conditions are included in the output of already defined steps.

| Step | What is realized in it | Additional exit condition |
|---|---|---|
| 0 | Agreed Types, Responsibilities, Scenarios, and What's Not Yet Proven | Twelve observations are compared with decisions and C-passports; found gaps have been fixed; structural check is not called application check |
| 2 | Pure rules of interworking relationships, conditions, conflicts; choice of move; small testing machine | AC-XW-03 to AC-XW-05, AC-SCH-01/05 and corresponding AC-STATEs on clearly finite areas; deliberately spoiled version is discovered |
| 3 | Version lines, exact read_set, copies of evidence, maintenance accounting; limited queue | Broken copy, double command, lost acknowledgment, separate variable validity of result; AC-STORE and AC-COMMIT on artificial boundaries |
| 4 | Real Gatekeeper/Mediator/Windows Borders, Checkpointing and Surveillance | AC-STATEs are mapped to real breakpoints; AC-COMMIT-02/06/08/10/12 and independent overload stop; the absence of a record is not issued as a successful stop |
| 5 | Authenticated Input and Limited Representation of Asserted State | Owner acceptance is not counterfeited by internal provision; overview does not show queue as completed entry |
| 6 | Related works and complex branches with artificial responses; final inspection plans | AC-XW-01/02/03/15/17, AC-SCH on stable and continuously changing inputs; AC-CAL on famous artificial markings |
| 7 | Present permitted connections and qualification of solving methods | Metered maintenance and overload; initial qualification data; limitations of statistical inference; quality is not reduced for the sake of a free slot |
| 8 | Real shared work objects and transfer to authorized media | AC-XW-08—AC-XW-16, AC-STORE-01—AC-STORE-05; separate pooling and verification, refusal for unproven race, fair partial outcome |
| 9 | Full review → work → unit → attempt | AC-OV-01—AC-OV-03 and AC-OWNER-01; load, same card/list status, position saving and no unnecessary notifications |
| 11 | Tolerance of improvement, real-world follow-up, selective return | All AC-EVO; comparison expense has a permitted path, hidden tests were not leaked to the candidate, return does not cancel history/rights/consequences |
| Long run §15 | Combinations of multiple possibilities, not just isolated happy paths | Counterexamples are reproduced; the area, duration and truncated scenarios are named; beneficial outcome measured, not just the absence of a fall |

Passing a synthetic profile does not replace the appropriate Windows or vendor verification. If the experience is negative, the specific technical agreement is corrected, the opportunity itself is not cut out for the sake of a green report. New features appear along with the real challenger and the test of its stage; future parts are not filled with empty stubs in advance.
