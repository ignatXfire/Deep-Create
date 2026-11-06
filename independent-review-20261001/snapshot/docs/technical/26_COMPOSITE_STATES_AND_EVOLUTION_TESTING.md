# Deep Create: checking combinations of states and development

Date: 2026-09-08. Status: addition to the technical project; not performed test and not product code.

Area: points 4, 6, 7 and 12 of external criticism; documents 14, 16, 17, 18, 19 and 20. This document does not change Hermes, the profiles, the protected basis or the existing rights. He clarifies what needs to be implemented and presented as evidence.

## 1. What criticism repeats and what it really adds

| Point of criticism | Already accepted | This supplement |
|---|---|---|
| 4: failures between three storages and OS | 14 §5 divides the writers and expressly denies the general transaction; 17 §12.1 sets `D/C/F`; 18 §2.1 separates COMMIT and checkpoint; 18 §12 and 20 §7 require process breaks | Generated combination model, named failure points, reproducible trace, counterexample reduction and verification of compliance with this protocol |
| 6: Independence does not mean finding mistakes | 16 §§12.3–12.5 already require latent task proficiency, provenance, residual correlation, and final test chain; change of supplier is not evidence | Exact contract for measuring qualifications according to `WorkKind` and error class; denominators, markup, trust, offset, revisions and obsolescence in existing C-08 |
| 7: improvement is adjusted to check | 17 §15 already prohibits changing one's own tests and protected grounds; old, new, hidden and hostile checks are required; 20 §14 already contains work in the shadows | Version composition of tests, accounting for task disclosure, observation after inclusion and selective return of the version without rollback of rights, costs and consequences |
| 12: combinations of locally correct rules are dangerous | 14 §7 separates the semantic stage from the blocking boundaries; 17 §12.1 and 18 §§7–10 define the locality of the fence, stopping and protective mode | A compact executable specification of a critical protocol with explicitly finite boundaries; a systematic test of how rules interact, rather than a promise to go through the entire system |

Conclusion: separate `VerifierCalibrationRegistry`, a second log of truth, a new managing agent, or a universal “difficulty controller” are not needed. Need evidence and obligations in existing C-contracts. Complexity is not a reason to impoverish exploration, memory, pattern selection, or self-development.

## 2. Executable model of critical transitions

### 2.1. Purpose and size

The model is a small deterministic machine of the test node, and not a simulation of intelligence and not a second production controller. It stores only the facts on which sending permission, accepting the result, holding the resource, stopping and restoring depend. The target text, model tokens, UI, entire files, and exploration semantics are replaced with typed observations.

First final profile `StateModelBounds`:

```text
root_works = 2                         // A и независимая B
units_per_root <= 2
attempts_per_unit <= 2                 // действующая и поздняя
intents_per_root <= 2
observations_per_intent <= 2           // частичный/итоговый либо повтор
logical_time = BeforeDeadline | AtDeadline | AfterDeadline
version_relation = Current | Stale | Conflicting
durable_store_health = Writable | WriteFailed | Unavailable
critical_volumes <= 3                  // main, guard, anchor; одинаковые тома можно связать
crashes_per_trace <= 2
max_steps = задаётся в manifest прогона
retry_profile = NoAutomaticRetry
```

To run a final crawl, the run list must additionally specify the final sets of **all** remaining variables: limits and amounts, numbers/revisions, fingerprints, queue lengths, supporting rights and outcome alternatives. The missing set produces `UnboundedModelDomain` rather than implicitly generating an arbitrary number. The initial small profile uses the sums `0, 1, 2` and a separate observed case of exceeding the upper limit; he cannot be credited with checking the arithmetic of the entire bit depth. The machine integer boundary profile specifies separately `0, 1, max-1, max` and overflow operations with expected failure. The report lists the scope of each profile, the states explored, and the branches truncated; the number of steps without this data does not prove the completeness of the traversal.

These are the parameters of the specific space under study, not the limitations of the future Deep Create. Profiles with a large number of jobs, repetition through proven external one-time, counter overflow and storage loss are executed separately. Version numbers can be collapsed into relationships only for properties that do not need their magnitude. Overflow checking requires a separate numeric profile; “collapsed versions” does not mean “checked for overflow.”

`NoAutomaticRetry` deliberately excludes network repeats from the first model. Its property of “no more than one OS intent transmission” cannot be transferred to the retry handler. The `VerifiedExternalIdempotency` extension checks for an immutable foreign key, a proven one-time region, separate transport attempts, and a sufficient upper cost. One logical action does not become a new action from replacing an ID after a lost response.

### 2.2. Status: what exactly is stored

| Part `ModelState` | Minimum fields and reason |
|---|---|
| `authority` | `installation_id`, `authority_epoch`, `run_generation`, `scope_fences`, `admission_closed_scopes`, `anchor_health`; power is not derived from main |
| `gate` | `holder`, `operation_kind`, `scope`, `phase`, `holder_alive`; allows you to reproduce death between check and effect |
| `leases` | `lease_id`, `root_work_id`, `state`, `expiry_relation`, `issued_epoch`, `issued_generation`, `scope_versions`; old law does not come to life |
| `main` | `journal_id`, `committed_head`, `checkpoint_dependency`, `root_stages`, `blocking_gates`, `accepted_packet_refs`, `row_versions`, `dependency_read_sets`, `health` |
| `guard` | intentions with `logical_operation_id`, `intent_id`, `action_hash`, `reservation_witness`, `dispatch_marker`, `os_handoff`, `outcome`, `observation_ids`, `health` |
| `checkpoint` | `confirmed_head`, `expected_previous`, `pending_transaction_ids`; log confirmation is not a new permission |
| `ledger` | by `(logical_operation_id, limit_id, window_id)`: `reservation_id`, `upper_bound`, `confirmed_amount`, `unknown_remaining`, `state`, `applied_observation_ids` |
| `processes` | `attempt_id`, `root_work_id`, `generation`, `alive`, `termination_observed`; the performer's response does not prove its completion |
| `storage` | `volume_id`, `write_health`, `threshold_state`, `emergency_reserve_state`, `affected_store_ids`; shared volume failure is different from stopping A |
| `stop` | `scope`, `requested`, `admission_closed`, `durable_fence_ref`, `termination_refs`, `status` |

The states of the main base, guard, anchor and OS are updated by different transitions. The model step of “writing everything down at once” is not allowed. `dispatch_marker=true, os_handoff=unknown` is a valid incomplete condition, not a violation in itself.

### 2.3. Commands, Events and Breakpoints

Command - an attempt to change the state; an event is an already observed or stable recorded fact. The command failure is also checked. The generator is required to create old, repeated and invalid commands, not just the happy path.

| Model input | Transition/refusal | Production-owner |
|---|---|---|
| `StartAttempt`, `CommitIntent` | check dependencies and general limits; main records intent and reservation | C-05 → C-03 |
| `PrepareIntent` | transfer the exact intent and evidence of the armor; another hash with the same ID - failure | C-09 `prepare_intent` |
| `BeginDispatch`, `WriteDispatchMarker`, `HandOffToOs`, `EndGate` | repeat the tolerance, write marker to `D`; do not hold gate during remote response | C-09/C-10/C-12 |
| `AdvanceClock`, `ObserveClockUncertain` | the right is checked by the valid period, and not by the delivery of the timer message | C-09/C-10 |
| `RequestStop`, `CloseAdmission`, `CommitFence`, `ObserveTermination` | close new entrances; point `F`; then prove the completion of the region | C-10 `establish_fence`, `advance_stop` |
| `ReceiveResult`, `ApplyVerifiedResult` | keep the package separate from the ability to execute `C`; late package does not close the obligation | C-05/C-08/C-03 |
| `CommitMain`, `ConfirmCheckpoint`, `LoseCheckpointReply` | main COMMIT and anchor checkpoint are separated; repeat receipt is idempotent | C-03/C-10 |
| `ReceiveOutcome`, `ReceiveUsage`, `DuplicateObservation` | save the fact and apply it once in the ledger, even if semantic acceptance is prohibited | C-09 → C-05 → C-03 |
| `CrashProcess`, `RestartCore`, `RestoreOldMain`, `LoseGuard`, `CorruptAnchor` | only apparently temporary data is lost; new start closed until reconciliation | C-13/C-10 |
| `FailVolumeWrite`, `CrossDiskThreshold`, `ExhaustEmergencyReserve` | close the normal path of affected resources, preserve protective facts as much as possible | C-03/C-09/C-10/C-13 |

Named injection points `CrashPointId`: `BeforeMainCommit`, `AfterMainCommitBeforeCheckpoint`, `AfterCheckpointCommitBeforeAck`, `BeforeGuardPrepareCommit`, `AfterGuardPrepareCommit`, `BeforeDispatchMarkerCommit`, `AfterDispatchMarkerBeforeOsHandoff`, `AfterOsHandoffBeforeOutcomeSeal`, `AfterOutcomeSealBeforeGuardCommit`, `AfterGuardOutcomeBeforeMainImport`, `BeforeFenceCommit`, `AfterFenceCommitBeforeMainImport`, `DuringGateOwnerTermination`, `BeforeVersionSwitch`, `AfterVersionSwitchBeforeObservation`.

Failure before COMMIT and failure during COMMIT are different. During COMMIT, the test environment assumes every realistically possible stable outcome of SQLite and tests it after opening; does not imply the loss of the line “because the ACK did not arrive.” Power loss, process termination, and write error have different failure profiles. The integrity of the flush/drive is not inferred from the fact that it passed the process killing test.

### 2.4. Test node functions

```text
enabled_steps(ModelState, StateModelBounds) -> Vec<ModelInput>
reduce_model(ModelState, ModelInput) -> ModelStepResult
assert_invariants(ModelState, TracePrefix) -> Vec<InvariantFailure>
generate_trace(Seed, GeneratorRevision, Bounds) -> Trace
replay_trace(InitialState, Trace, ModelRevision) -> ModelRunReport
shrink_counterexample(Trace, FailureIdentity, Bounds) -> ShrinkReport
drive_protocol(Trace, ProductionAdapterRevision, FaultPlan) -> ObservedTrace
check_refinement(ModelTrace, ObservedTrace, TransitionMapRevision) -> RefinementReport
```

These functions are located only in `tests/state_model/` of the future project and have a caller - test runner/CI. They do not receive user secrets, do not run the real network, and do not become an instrument of the model. Replacement clocks, storage and OS surveillance support this runner. Production break inserts are only available to the test assembly and are themselves checked to ensure they are not included in the produced artifact.

`ModelStepResult` contains `next_state`, `decision: Applied|Rejected|Deferred`, `error_code`, `facts`, `linearization_refs`. Test contract errors: `ModelInvariantViolation`, `UnsupportedModelInput`, `BoundsExceeded`, `ReplayMismatch`, `RefinementMismatch`, `UnmappedProductionTransition`, `ShrinkPreconditionBroken`, `CounterexampleNotReproduced`. Going abroad does not turn into `pass`; such a trace is considered truncated.

### 2.5. Brute force, random generation and reduction

In a small finite profile, a systematic traversal of reachable states is performed to a given depth. For longer chains, generation from a saved seed is used. Partial reduction of permutations is only allowed for proven commuting transitions: A and B do not have a common gate phase, ledger line, checkpoint dependency, affected volume, or subject read/write intersection. A shared quota and checkpoint entry make seemingly independent commands cohesive for a specific property.

`StateModelRunManifest` stores:

- `run_id`, `model_revision`, `production_revision`, `transition_map_revision`, `generator_revision`, `fault_adapter_revision`;
- `seed`, exact initial fixture and `bounds`, dependency versions and storage settings;
- list of invariants, selected failure profiles, reduction methods and their prerequisites;
- reached states/transitions, unvisited classes, truncated traces and reason for termination;
- links to the original and reduced traces, `failure_identity`, repetition report.

Seed without generator versions, inputs and schedule is insufficient. For the production run, a controlled schedule, confirmation of reaching the crash point, and observations after a restart are saved. The irreproducibility of a real failure is not hidden: `CounterexampleNotReproduced` is stored at the reproducibility frequency and full original evidence.

Reduction first removes blocks of steps, then extraneous objects and data, then simplifies time intervals and break locations. It preserves the preconditions of the causal chain and the same violated invariant. You cannot “reduce” the trace to a parser failure and declare that the original defect has been found. If global minimality is not proven, the result is called a locally reduced counterexample. Each reproduced defect becomes a permanent regression scenario with the original cause.

The number of launches in itself is not a criterion for quality. The report “a million sequences passed” without boundaries, transition coverage and properties being tested is not accepted. More valuable: reaching dangerous boundaries, rearranging conflicting operations, intentionally breaking a rule, catching a counterexample, and repeating it on the production protocol.

### 2.6. Implementation Connection and Proof Boundary

`TransitionMap` associates each model step with a C contract, a production function, an observable and a stored fact. For example: model `CommitFence` → C-10 `establish_fence` → anchor COMMIT → `FenceReceipt`; `CommitMain` → C-03 `commit_transition` → main COMMIT → top of transaction; `ConfirmCheckpoint` → C-10 `record_journal_checkpoint` → checkpoint receipt. The matching scheme tests the causal order, not just the similarity of the resulting names.

There may be more low-level production steps. `check_refinement` allows internal steps without subject change, but not skipping an OS submit, ledger change, fence, COMMIT, or received ACK. A significant transition not displayed is a map error. The oracle of invariants cannot be built entirely by calling the same `decide_transition` that is being checked: then one error will give agreement between the model and the code. The test model and invariants are formulated separately; the message scheme may be general.

Special obligation for the term: `AdvanceClock` can occur inside the held gate. The test has no right to consider gate as stopping the clock. If the deadline has already expired before `D`/`C`, the previous deadline check is not sufficient in itself. The implementation is required to present an accurate temporary admission agreement and experience at the deadline; the inability to guarantee this contract by the chosen mechanism is noted by `DeadlineBoundaryUnproven`, and is not masked by model atomicity. It is impossible to silently weaken the term to “was valid at one time upon entry.” A deleted effect already started before the deadline does not become non-existent after the deadline.

The model does not prove: the truth of the semantic output, the independence of model training, the absence of errors outside the selected boundaries, the correctness of the crypto library and the OS, the physical stability of the drive, the safety of complete capture of the machine, the absence of general errors in the model and implementation. She doesn't go through a never-ending story. The progress properties are formulated conditionally: if the record works again, the processes actually terminate and the valid steps are executed, the reconciliation ends with a safe known state or an obvious unknown. A permanent disk failure is not required to complete `stopped` with a persistent acknowledgment.

## 3. Combination expiry + late result + stop + checkpoint + late usage + disk low

### 3.1. Not one “queue priority”, but five sources of facts

1. The current fence and the period determine the possibility of a new admission. A protective request closes new entrances; the stable point `F` defines a proven recall. The model response timestamp does not create the right.
2. `D`/`C` are determined by the observable linear points of the protocol, not by the order of the interface messages. `D` - OS transfer, `C` - main COMMIT semantic acceptance.
3. Checkpoint confirms the saved line. It does not recreate `C`, cancel `F`, or grant permission to take the next step.
4. Observed expenditure and external outcome are facts, even if the right has expired, the work is stopped, or the result is rejected. They are accepted in a closed official way, not the old right of the performer.
5. The actual recording error is more important than the free space estimate. No record does not equal no consequence or zero expenditure.

### 3.2. Baseline A with independent operation B

Initially: A has the right `L_A`, intention `I_A`, reservation `U`, `D_A` has already occurred; The outcome/score is not yet known. B does not depend on the result of A, does not exercise its rights and has its own actual dependencies. An installation may have shared physical storage and a parent limit - their impact is tested separately.

| Observation | What happens in A | What's not happening |
|---|---|---|
| `L_A` has expired | New submissions and late semantic acceptance under this right are prohibited regardless of the timer delay | The old `D_A` is not declared not to have happened; `U` is not released |
| Late result arrived | If recording is possible, the response is sealed; `late_result_rejected` with the reason for the deadline/fencing; obligations and accepted materials do not change | A new check pass is not created to bypass the revocation and continuation is not started |
| Stop A arrived | The new A entrances are immediately closed; caretaker seeks `F_A`, completion of affected trees and recording of facts | The global era does not increase and `run_generation` does not change only because of A |
| There is a pending checkpoint | The exact dependency on the COMMIT that has already been committed is maintained. If the anchor is available, the previous whole tail is confirmed, even after `F_A` | Checkpoint is not applied as a reacceptance of the response and does not trigger the next step A |
| Late invoice arrived | Certified observation is saved and imported into one ledger position by ID; partial bill leaves unknown balance | Expiration, stopping and rejected text will not void the fee; repeat delivery does not charge it twice |
| Not enough space | The normal record of the affected resources is closed; uses a proven emergency supply for permitted defense facts | You cannot purge canonical evidence or create a false ACK for the sake of completion |

Three different pending checkpoint options are checked separately:

- `C_A < F_A`, and the term was valid when `C_A`: acceptance has already occurred; after checking the tail, it remains historically accepted until the fence. Until the checkpoint is confirmed, it is not shown as confirmed; After confirmation, you can show the fact with a stop mark. Dependent new actions are prohibited.
- `F_A < C_A` or the right has expired before the new semantic `C_A`: the late result cannot be applied. A pending checkpoint may refer to a previous other transaction or observation record; it does not “meet” late acceptance.
- COMMIT could complete, but the process died before the receipt: recovery finds out the stable state of main and anchor. Neither the absence of an ACK nor a late response from the performer is proof that `C_A` did not exist.

If `dispatch_started` is saved, but there is no proof of `D_A`/outcome, `outcome_unknown` is set; It is assumed that the request never went through. The balance is held until proven reconciliation. `prepared` without a marker can be released only after the old sender has been protected and the absence of an accepted start has been confirmed. The unknown does not allow a repeat with the new `intent_id`.

If main is not available, but guard/anchor are available, they retain their resolved facts with the original ID and time. The main journal will later import them once; does not invent the same `EventSeq`. If the guard also cannot store the received score, its stable storage cannot be confirmed: known upper estimates are retained, the channel is not ACKed as reliably received, and the risk of loss of observation is shown. Repetition/reconciliation is permitted only in the safe manner provided; the network does not become authorized from the word “accounting”.

If the anchor cannot save `F_A`, admission remains closed in the live defense process, the trees are terminated, but the stable fencing status is unconfirmed. `stop_unconfirmed` is not replaced by `stopped`; the next launch is also closed until reconciliation. If the flow rate is higher than `U`, the actual amount and `resource_bound_breached` are recorded; the amount is not cut off according to the contract.

### 3.3. What does independence B mean?

Stopping A does not revoke `L_B`, make B's result obsolete via the shared `EventSeq`, or extinguish its branch. Changing `anchor_sequence` due to checkpoint is not a review of B. This is semantic and power independence, not a promise of a separate disk/budget for each work.

If the main/anchor shared volume is full, a mandatory journal is not possible, or the shared limit is reached, B can acquire its own fine blocking boundary `StorageUnavailable`/`LimitReached`. Her rights do not become revoked “because of A”. After restoration, only this proven eliminated cause is removed, with the rights and dependencies re-checked. If only A's private temporary volume is full and B has work log, reserve, and resources, B continues valid operation. A general security inconsistency, as opposed to an individual stop, may require the entire installation to be fenced off for a separate, clearly stated reason.

## 4. Calibration of the test method inside the C-08

### 4.1. What qualifies is not “the intelligence of the model in general”

Qualification unit: `VerificationMethodKey = {work_kind, criterion_kind, error_class, consequence_class, validation_method_revision, prompt_method_revision, projection_policy_revision, retrieval_policy_revision, exact_connection_revision, machine_validator_revision}`. For the machine method, the model fields are explicitly `NotApplicable` rather than omitted. Changing the critical part of the key requires a revision of the proof's portability.

A method may be good at checking arithmetic but weak at finding a lost negation. The total percentage for all `WorkKind` does not allow complex verification. For `Research`, provenance, relevance to issue, rebuttable factual statements, and limitations are tested; the lack of proven global novelty does not make the research idea false. For creative choices, consensus constraints are machine checked, and subjective preference is not given a dummy objective label.

`CalibrationEvidence` - immutable evidence C-08 in the existing evidence store; the index/link in main is created via C-05/C-03. This is not a new service or a stand-alone Writer/Registry.

| Field | Mandatory meaning |
|---|---|
| `calibration_id`, `schema_revision`, `method_key` | Identity and exact verifiable configuration |
| `evaluation_manifest_ref`, `label_manifest_ref`, `split_revision` | Frozen set, markup and division before launch |
| `eligible_unit_definition`, `assignment_manifest_ref` | What counts as one test and what was ordered; prohibits selecting only successful answers |
| `work_kind`, `criterion_kind`, `error_class`, `population_scope` | Which task does the conclusion apply to?
| `counts_by_label_and_decision`, `abstention_counts`, `technical_failure_counts` | All numerators, denominators and missing answers |
| `interval_method`, `confidence_level`, `cluster_definition`, `effective_sample_note` | Under what conditions is uncertainty assessed |
| `label_basis_refs`, `adjudication_revision`, `disputed_count`, `excluded_count`, `exclusion_reasons` | Why the label is trustworthy and what failed to tag |
| `author_strata`, `paired_error_evidence_ref`, `source_overlap_findings` | Observed commonality of errors and sources without fiction about training data |
| `exposure_manifest_ref`, `holdout_status`, `leakage_findings` | To whom are the tasks/tags disclosed and can they still be considered hidden |
| `measured_at_utc`, `review_due_at_utc`, `invalidating_conditions` | Revision Period and Early Obsolescence Events |
| `raw_observation_refs`, `limitations`, `qualification_decision_ref` | Reproducible support and output limit |

### 4.2. Exact denominators

The basic unit `EvaluationUnit` is one fixed candidate for one fixed decisive criterion, which is assigned weight and cluster before launch. If the acceptance of a whole package is measured, the unit is different: the whole package with the rule “an error in at least one mandatory criterion.” These two levels do not mix into the same denominator. The same candidate with ten paraphrases does not count as ten independent problems.

Label `ReferenceLabel = Acceptable | Unacceptable | Unresolved`. Verdict of the `MethodDecision = Accept | Reject | Abstain | InvalidRun` method. `gaps` refers to `Abstain` and not discreetly to `Reject`. The technical error relates to `InvalidRun`, not to the proof of correctness/incorrection of the candidate.

Let `E` be the entire preassigned set of units with the label `Acceptable` or `Unacceptable` set. `Unresolved` is taken into account and published separately with reasons, but is not included in the denominators of truth. You cannot delete difficult units after viewing the method response.

```text
N_bad  = count(E where label = Unacceptable)
N_good = count(E where label = Acceptable)
FA     = count(E where label = Unacceptable and decision = Accept)
FR     = count(E where label = Acceptable and decision = Reject)

false_accept_rate = FA / N_bad
false_reject_rate = FR / N_good

N_ce = count(заранее назначенных валидно размеченных пар
             EvaluationUnit × ErrorOpportunity с предъявленным материалом)
D_ce = count(таких пар, где метод указал именно эту ошибку/контрпример,
             а локализация и основание прошли заранее принятый matching rule)
counterexample_detection_rate = D_ce / N_ce
```

`Abstain` and `InvalidRun` are not false accept/reject and are not deleted from `E`. They are not error detection. In order for the “always be silent” method not to appear qualified, `decision_coverage = count(Accept|Reject)/|E|`, `invalid_run_rate`, `abstention_rate`, `acceptable_nonaccept_rate = count(label=Acceptable and decision!=Accept)/N_good`, delay and cost are required at the same time. If necessary, conditional FAR/FRR are also published only among `Accept|Reject`, with separate names and denominators. A low FAR with almost zero coverage does not provide tolerance.

For detection, one error possibility is considered to be found no more than once; Several errors in one candidate form a dependent cluster. A bare “disagree” does not count as finding a specific counterexample. Falsely reported errors are published as a separate indicator `unsupported_challenge_rate` with the number of all verifiable objections raised by the method in the denominator and the rule for marking these objections. An empty denominator always means `NotEstimable`, not `0%`, and not `100%`.

### 4.3. Where does the markup come from and how is uncertainty taken into account?

The basis of the label depends on the criterion: deterministic calculation/execution with a trusted fixture; primary source and accurate verifiable reference; controlled experiment; a proven historical error with a reproducible outcome; independent subject analysis with preserved arguments. The LLM vote itself is not a benchmark. The model-generated adversarial task is suitable after independently establishing the answer; it does not get the true label from the generator's confidence.

If the criterion is not objectively resolved or the experts disagree without decisive evidence, `Unresolved` is given. Markup resolved later creates a new `label_manifest_revision`; the previous report is not rewritten. The impact of the corrected label on valid qualifications is recalculated and saved in a separate decision.

The pre-launch plan fixes `min_labeled_units`, minimum volumes for each mandatory class, acceptable FAR/FRR/detection limits, minimum coverage, selected confidence limit, budget and collection stop rule. These values ​​depend on the consequences and class of tasks; There is no universal “100 examples are enough.” Zero errors in a small sample do not prove zero risk. Assuming independent tests of the same type and zero errors, the one-sided upper bound on the error probability of level `1-alpha` is equal to `1 - alpha^(1/n)`; This is a property of the adopted statistical model and is not a guarantee for unknown future problems.

For dependent examples, raw volumes and grouping units are shown: one source, source task family, RootWork, error template. Intervals on independent individual lines without taking into account this dependence are prohibited. The selected interval estimation/resampling method is recorded in evidence; if the premises are not justified, the estimate is labeled descriptive and a strong probabilistic conclusion is not made. Consistent peeking until a beautiful result and many comparisons require a pre-selected stopping/correction rule; otherwise, this is an exploratory result and a new closed run is needed.

A set of known hard errors is needed, but does not represent the frequency of errors in everyday work. The stress set and a representative sample of tasks are shown separately. Selection bias, artificiality, domain, language, freshness of sources, and missing classes are included in the limitations. Qualifications do not automatically transfer beyond their borders.

### 4.4. Correlation of errors with the author

`Provider=Different` does not mean independent errors. Unknown training data remains `Unknown`; You cannot write `training_overlap=0` using different model names. The similarity of known sources is established only by the observed retrieval/lineage.

Two different measurements are applied, not one ambiguous `correlation_with_author_family`:

1. `conditional_false_accept_by_author_stratum`: the proportion of accepted incorrect candidates among all incorrect candidates for this `author_family/author_method/error_class`. Numerator, denominator, abstain, intervals and difficulty comparability are published. This is a direct test of what errors the author misses.
2. `paired_independent_error_dependence`: on the same labeled problems, the author and a separate verification method independently obtain an answer before revealing the author’s solution. For the pair, `author_wrong`, `verifier_independent_wrong` are saved; The joint error rate and, where definable, the binary error correlation are calculated. We need a table of four outcomes, the number of pairs, clusters and uncertainty. In the absence of variation, the correlation `NotEstimable` is not zero.

The second experience is not equal to the usual review of an already presented candidate, and the first is not evidence of a difference in training. Comparisons are stratified by task complexity and type; otherwise the overall difficulty creates a misleading correlation. The high observed commonality of errors requires a different method/primary observation on the decisive criterion, but does not prohibit the use of these models for free generation and research.

### 4.5. Revisions, hidden set and validity period

Separate `development`, `qualification_holdout` and `post_admission` with independent manifest. The candidate and its optimizing branch do not receive hidden answers and a detailed evaluator trail. Each display of a task, failure reason, and outcome counts as a disclosure; multiple pass/fail responses can also carry a signal and consume the set attempt budget. The revealed element is transferred to regression/development rather than continuing to be called hidden.

A new prompt, model revision, source, retrieval policy, context schema, machine validator, corrected markup, or detected task shift can qualify `Stale` before the calendar deadline. `review_due_at_utc` - the upper date of the planned revision, not immunity from early obsolescence. Proof transfer is only allowed with a fixed parse of the affected key and limited re-run; The model name is not a stable revision.

`QualificationDecision` states: `Qualified`, `Restricted`, `InsufficientEvidence`, `Stale`, `Disqualified`. `Restricted` contains the exact valid `WorkKind/criterion/error/consequence` and a required additional method. Qualification is an input `plan_semantic_verification`, not a permission to external action and not a verdict on any future text.

### 4.6. Functions, faults and events C-08

| Function | Challenge and result | Significant Failures |
|---|---|---|
| `build_calibration_plan` | C-05; `MethodKey + QualificationPolicy + EvaluationManifest` → unchangeable test plan | `MissingLabelBasis`, `HoldoutExposed`, `PopulationMismatch` |
| `evaluate_calibration_evidence` | C-05; plan, raw observations and markings → `CalibrationEvidenceCandidate` | `AssignmentMismatch`, `DenominatorUndefined`, `LabelConflict`, `InsufficientSample` |
| `assess_method_qualification` | C-05/`plan_semantic_verification`; evidence, current key and policy → `QualificationDecision` | `CalibrationStale`, `CoverageTooLow`, `ErrorBoundExceeded`, `UnsupportedPopulation` |
| `assess_error_dependence` | internal stage C-08; paired observations/strata → constrained inference | `UnpairedEvidence`, `DependenceNotEstimable` |

This is the analysis of evidence and the preparation of decisions, not direct writers. Model calls remain normal C-06/C-07/C-11 limited units with resource accounting. Saving - via `handle_command → decide_transition → commit_transition`. Closed new command options C-05: `RecordCalibrationEvidence`, `RecordQualificationDecision`; the sender ensures that the model text is not such a command.

Events: `calibration_plan_fixed`, `calibration_evidence_recorded`, `method_qualification_changed`, `calibration_invalidated`, `evaluation_holdout_exposed`, `evaluation_label_revised`. Payload contains ID/revisions/counters/reason, not hidden answers. Full materials are available only for the appropriate projection of evidence.

## 5. Self-development without adjusting to your own tolerance

### 5.1. An ongoing cycle rather than a one-time exam

Deep Create can independently search for errors, improve methods, build new abilities, memory and model settings, compare options and allow reversible area to be allowed according to a pre-established rule. The freedom to formulate a research hypothesis is not the same as the right to replace the basis that allows its inclusion.

`EvaluationPoolManifest` - versioned list of evidence sets, not a new control component:

| Part | What saves | What does not prove in itself |
|---|---|---|
| `permanent_regression` | Known errors, including real historical failures and all reproduced state-model counterexamples | Generalization to new problems |
| `rotating_hidden` | Undisclosed tasks and tags with a limited number of hits | Eternal secrecy and no leakage through feedback |
| `fresh_challenge` | New problems and adversarial examples after candidate sealing, with independently set markup | The truth from the very fact of generating another LLM |
| `representative_work` | Various real types of work within the limits of permitted data processing | Missing Domain Coverage |
| `post_admission` | Observing benefits, regressions and costs after inclusion | Causality without a comparable base and taking into account changes in the composition of tasks |

Collections may cite the same source, but this is clearly reflected in the cluster/lineage and does not add to the independent evidence. Historical failures are not removed because they spoil the indicator. New data, accumulated memory, and design models maintain the origin, data classification, and scope of acceptable use.

### 5.2. The admission plan is fixed before the candidate is assessed

`EvolutionEvaluationPlan` contains `candidate_manifest_hash`, `baseline_manifest_hash`, `mutable_scope`, `protected_root_revision`, `admission_policy_revision`, `evaluation_pool_revision`, `qualification_refs`, `required_invariants`, `utility_criteria`, `regression_tolerances`, `cost_limits`, `max_candidate_revisions`, `max_semantic_rounds`, `exposure_budget`, `promotion_scope`, `observation_plan_ref`, `rollback_plan_ref`.

A candidate can propose a new test as research material, but not replace the test of his current admission, remove an old regression, change the threshold, change the label, or obtain a hidden set. The admission criteria and protected root belong to the protected accepted policy. Changes to this policy are subject to individual owner authority; it is not retroactively assigned to the current candidate.

Admission is not limited to GPA: mandatory rights/journal/data boundaries are not compensated by increased utility. For a variable area, the plan specifies the benefit to be tested, allowed class regressions, minimum coverage, price, and return conditions. The subjective part is clearly marked. Unproven benefit means “not proven to be included,” not “the study is pointless.”

### 5.3. Observation after switching on

`PostAdmissionObservationPlan`:

```text
promotion_id, candidate_manifest_hash, baseline_manifest_hash
scope_selector, dependency_selector, task_population, rollout_stage
minimum_observations_by_stratum, observation_window, maximum_exposure
primary_outcomes, hard_guardrails, delayed_outcomes
comparison_design, assignment_rule, confounders_to_record
review_schedule, sequential_decision_rule, missing_outcome_rule
rollback_triggers, pause_triggers, close_observation_rule
```

The calendar “seven days have passed” is not sufficient volume. The window depends on the outcome delay and the number of comparable tasks. If there are few observations, the status is `ObservationIncomplete`; version does not receive the undeserved status of proven long-term benefit. If the composition of tasks/models/sources has changed, this is included in the comparison; a simple “it got better after” is not declared to be the reason for the improvement.

Working in the shadows does not create hidden external consequences: a second paid request requires its own reserve/admission; an external action is not performed twice for A/B. The shadow receives the resolved observations/copies of the inputs, and the executing branch is determined before the action. Real private tasks are not sent to another route just because it is an evaluator.

Observation also records negative outcomes: restoration of old memory projection due to losses, increase in manual corrections, missed obligations, late errors, increased costs, false blocks and real successful results. The presence of one strong verification method does not close the measurement of benefit, and subjective disapproval of another LLM does not invalidate the proven passage of protected conditions.

### 5.4. Selective rollback, not rollback of the world

`EvolutionRollbackPlan` pre-links `promotion_id`, the whole verified base version, the exact `scope_selector`, `affected_dependency_keys`, a way to stop issuing new attempts to the candidate, schema/artifact compatibility, switch step and confirmation. A return is allowed only if it itself fits into a previously allowed reversible region; otherwise, the affected opportunity is closed and a specific decision by the owner is required.

Return procedure:

1. C-16 prepares a decision on a pre-assigned trigger; C-05 prohibits new attempts at the affected version. If necessary, C-10 will fence only the relevant units/area.
2. Current attempts either reach the allowed safe point or are abandoned. The result carries the exact version; it is not presented as produced by the basic version.
3. The entire authenticated changeable version/profile is switched. Partial mixture of files is prohibited. An incompatible scheme is not rolled back by blindly replacing the old base: the provided path is checked or `RollbackCompatibilityFailure` is returned.
4. Derived indexes/memory projections are invalidated on affected dependencies and rebuilt if necessary. Canonical sources, history of findings, negative results and origins are not removed. Already accepted results are not overwritten: possible impact flags and spot checks are created.
5. Switching observations and continuous reporting are saved; restoration of basic behavior is checked. Independent versions, works and results are not automatically returned.

`authority_epoch`, `run_generation`, `scope_fence_versions`, final revocation of rights, `resource_ledger`, already paid requests, checkpoint history and committed external/file consequences are never rolled back. Returning a version is not compensation for an external action. Compensation, if provided at all, is a distinct, precise action with its own right, object, expense, and observable outcome.

### 5.5. A finite tolerance should not prohibit development by an infinite LLM chain

Uses the existing final `VerificationPlan` of document 16 rather than the recursive requirement of "another smarter model". C-08 verifies form, origin, criteria and qualifications; C-16 applies the pre-fixed clearance decision. Neither the author nor his checking model include the version directly.

If the evidence is sufficient for the protected plan and the area of ​​admission is allowed, the system has the right to include the candidate without a new ritual LLM approval. If the evidence is insufficient, the exact missing criteria are recorded; new dimensions are selected from them, with a finite budget for editorial and disclosure. Independent research and generation of staged candidates can continue. You cannot bypass the limit by renaming one candidate, but you cannot block all future development with one old controversial review. The new substantive method/evidence forms a new related research thread with explicit contract and resources, preserving the history of the previous failure.

### 5.6. Functions, errors and events C-16

| Function | Input → Output | Border |
|---|---|---|
| `evaluate_staged_candidate` (addition to existing) | candidate/baseline, fixed plan, qualified evidence → `AdmissionDecision` | Does not change the plan and protected root; `EvaluationPoolMismatch`, `HoldoutExposed`, `RegressionDetected` are added |
| `promote_verified_candidate` (addition to existing) | current tolerance, whole version, observation/rollback plan → inclusion observation | `StaleAdmission`, `PartialVersion`, `ObservationPlanMissing`, `RecoveryRequired` |
| `assess_post_admission` | plan, linked real observations and basic comparison → `ContinueObservation|ConfirmedForScope|PauseScope|RollbackScope` | `InsufficientExposure`, `PopulationShift`, `UnresolvedDelayedOutcome`; no observations not `pass` |
| `prepare_scoped_rollback` | promotion, trigger, current dependencies and the whole base version → `ScopedRollbackPlan` | `RollbackScopeExpansion`, `RollbackCompatibilityFailure`, `UnaccountedEffect` |
| `apply_scoped_version_switch` | allowed current plan, confirmation of fencing of necessary attempts → `VersionSwitchObservation` | `StaleRollbackPlan`, `VersionIntegrityFailure`, `StorageUnavailable` |

The last function is a general internal step C-16 to include/revert an entire mutable version, not a public model tool. C-16 does not write main/ledger/anchor directly. Subject decisions go through C-05/C-03, fencing through C-10; The switching mechanism remains in the updater area. The new status writer does not appear.

Events: `evolution_evaluation_plan_fixed`, `evolution_candidate_evaluated`, `evolution_candidate_promoted`, `post_admission_observation_recorded`, `post_admission_review_completed`, `evolution_scope_paused`, `scoped_rollback_requested`, `scoped_rollback_completed`, `scoped_rollback_failed`, `derived_projection_invalidated`. The link to the unsuccessful version is saved. Commands C-05: `RecordEvolutionEvaluation`, `RecordPromotionObservation`, `RecordPostAdmissionReview`, `RequestScopedRollback`, `RecordVersionSwitchObservation`; their origin and valid caller are fixed by the C-02 scheme.

## 6. Acceptance conditions

The status of each condition as of the document date: `specified_not_executed`. Scenarios are not awarded as tests written or passed.

### 6.1. State Combinations

| ID | Test action | Condition of acceptance |
|---|---|---|
| `AC-STATE-01` | Rearrange expiry/revoke and dispatch with a break after checking/marker | After invalidation, there is no new admission until `D`; marker without a proven outcome does not declare the request unsent; time boundary implemented or explicit `DeadlineBoundaryUnproven` |
| `AC-STATE-02` | Lose response and retry intent/delivery | There is no second unintended OS handoff; another hash of the same ID is rejected; unknown has not been converted into a new right |
| `AC-STATE-03` | Stop, crash and partial/total/duplicate usage in different orders | One ledger; unknown balance withheld; the fact above U is saved and blocks the new flow |
| `AC-STATE-04` | All permutations of `C`, `F`, checkpoint and late package | `F<C` rejects semantic acceptance; `C<F` is not retroactively cancelled; no received ACK/dependent send before checkpoint |
| `AC-STATE-05` | Stop A; change its lines, checkpoint and `anchor_sequence` with live B | B's permissions and its independent dependencies are not revoked; general physical/resource locking named separately |
| `AC-STATE-06` | RestoreOldMain, LoseGuard, CorruptAnchor, restart with old process | There is no reduction in power/invigoration of the process; there is no zero flow from a blank copy; admission closed until proven reconciliation |
| `AC-STATE-07` | Full trace §3 with each critical volume and reserve depletion | Honest `outcome_unknown`, `stop_unconfirmed`, pending checkpoint; no false receipts or silent loss of canonical data |
| `AC-STATE-08` | Repeating the seed and reduced counterexample on pinned versions | The same violation and causal trace; truncation/irreproducibility indicated; defect falls into regression |
| `AC-STATE-09` | Compare model and production at all named phases | An unmapped significant transition is rejected; real observations confirm `D/C/F`, not just the final enum |
| `AC-STATE-10` | Intentionally remove fence-check, ledger dedup or prohibit ACK before checkpoint | The corresponding property catches the defect; consent of only equally damaged model/production does not count |
| `AC-STATE-11` | Do not restore the disk/process for a long time, then restore it | There is no false success with constant failure; under the stated prerequisites, recovery achieves a safe solution and does not hang due to the log's own ban |

### 6.2. Calibration

| ID | Test action | Condition of acceptance |
|---|---|---|
| `AC-CAL-01` | Fixture with correct/wrong/unresolved labels and four solutions | FAR, FRR, detection and coverage are recalculated from exact counts; abstain/invalid do not disappear |
| `AC-CAL-02` | Zero denominator, zero errors on a small sample, duplicates of one template | `NotEstimable` instead of a fictitious percentage; the scope and boundaries of uncertainty are fair; takes do not represent independence |
| `AC-CAL-03` | Markup by one LLM vote or conflict of primary sources | There is no standard without a basis; unresolved is published, not deleted after method response |
| `AC-CAL-04` | High average score if you fail one mandatory `WorkKind/error_class` | No transfer of general qualifications to a failed class |
| `AC-CAL-05` | Methods from different vendors repeat the same mistake; training unknown | The observed commonality of errors is clear; unknown training not declared independent |
| `AC-CAL-06` | Change prompt/model/retrieval revision or correct label | The old evidence remains; relevance reconsidered; unconfirmed transfer gives Stale/Restricted |
| `AC-CAL-07` | Open holdout, peek at many candidates, choose the best run | Disclosure/multiplicity taken into account; hidden status is not saved fictitiously; need an honest closed plan |
| `AC-CAL-08` | The method is always abstain or technically fails | Low FAR does not allow qualification without required coverage/reliability |
| `AC-CAL-09` | Bring model "QualificationDecision" without trusted origin | Direct recording/issuance of rights is not possible; C-08 only evaluates evidence, C-05/C-03 retain admissible decision |

### 6.3. Self-development

| ID | Test action | Condition of acceptance |
|---|---|---|
| `AC-EVO-01` | The candidate improves the open benchmark, but loses the historical error/hidden class | Admission rejected based on precise criteria; open score does not compensate for mandatory regression |
| `AC-EVO-02` | The candidate changes his evaluator, threshold, hidden labels, beacon or protected root | `ProtectedAreaChanged`/violation of plan; neither automatic switching nor hidden change of tolerance is possible |
| `AC-EVO-03` | After promotion, late real outcomes are worse with a comparable base | Observation leads to the intended PauseScope/RollbackScope, does not end with admission |
| `AC-EVO-04` | The window has ended without sufficient space or the composition of tasks has changed | `ObservationIncomplete`/`PopulationShift`, non-fictitious proof of benefit |
| `AC-EVO-05` | Returning version A with independent B, valid revocation, late usage and committed effect | Only the allowed version/area is returned; rights, ledger, authority and consequences are not rolled back; B is not extinguished |
| `AC-EVO-06` | Failure in every switching phase; file changed after evaluation; scheme incompatible | Whole old/new version or closed recovery; no mix, blind DB rollback or unproven success |
| `AC-EVO-07` | Sufficient Secured Plan Passed, New LLM Just “Doubts” | An endless test chain is not created; decision based on plan and evidence, development of acceptable area does not require ritual veto |
| `AC-EVO-08` | Insufficient admission of one candidate in other studies | There is no false promotion and no global research ban; the exact gaps and the ultimate next experience are named |
| `AC-EVO-09` | Returning a derived memory/route after a sense loss is detected | Canonical primary sources and origins are preserved; affected projections are rearranged; already accepted results are marked/rechecked without rewriting history |
| `AC-EVO-10` | Shadow/A-B for Toll Call or External Action | The double consequence is not satisfied for the sake of comparison; additional flow/transfer have their own allowable path |

## 7. Where to supplement existing documents

| Document | Exact addition |
|---|---|
| 14 §§4.13, 4.14, 4.21 | Links to `CalibrationEvidence`, `QualificationDecision`, evaluation/admission/observation manifests; versions and immutable origins; closed events without a new writer |
| 14 §§5, 7–9, 12 | Model-state boundary, marker/D/C/checkpoint separation, connection with `AC-STATE-*`, saving late flow/unknown |
| 16 §§12.3–12.5, 16 | Qualification Key, Precise Indicators/Denominators, Tags, Correlation, Obsolescence, End Role C-08; `AC-CAL-*` |
| 17 §§4.1, 12.1, 15, 17 | The verifiable contract expiry at the critical boundary; unchangeable own tolerance; selective rollback, no rollback of powers/consequences; `AC-STATE-*`/`AC-EVO-*` |
| 18 §§2.1, 3, 7–10, 12 | Scenario §3, fair statuses for disk low, calibration/development directories and named crash points |
| 19 C-01/C-03/C-05/C-09/C-10/C-12/C-13 | Links to model transitions and oracles, adapters responsibilities/errors, without new production controllers |
| 19 C-08 | Functions and evidence qualifications from §4.6 instead of a separate `VerifierCalibrationRegistry` |
| 19 C-16 | Post-acceptance surveillance, selective return preparation and single internal switching step from §5.6 |
| 20 §§5–7, 9–10, 14–16 | Small automatic on clean crossings; then the real crash/refinement; calibration prior to consequential verification; observation/return to automatic development; evidence manifests as exit steps |

When merging map 21 and final check 22, you should add links to new AC groups and leave the status “project set, checks not completed.” This document does not state 21/22 readings or changes and does not replace their reconciliation by those responsible for the overall project assembly.

Bottom line: Criticisms are addressed in the form of three specific enhancements - executable limited protocol model, measured proficiency of existing C-08, and ongoing review of C-16 development. They make a complex system more testable without turning research into a vote, memory into a changeable truth, and caution into an indefinite ban on self-development.
