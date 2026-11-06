# Deep Create: execution, model selection and verification

**Date:** 2026-09-07  
**Status:** technical agreement of the control cycle  
**Main boundary:** The scheduler decides what is ready to go; router - what connection is allowed; the performer offers the result; the inspector checks; only the controlling process accepts the change.

---

## 1. One control loop

```text
прочитать принятую цель и снимки зависимостей
→ найти готовые единицы
→ проверить зависимости и пределы
→ зарезервировать ресурсы
→ спланировать сведения и найти местных кандидатов
→ выбрать точное допустимое подключение
→ собрать адресную подборку, поручение и фактическое тело поставщика
→ повторно проверить права, свежесть, получателя и верхнюю стоимость
→ отправить через посредника в границе допуска
→ наблюдать жизнь и фактический расход
→ сохранить исходный ответ и доверенный след отдельно от модельного содержимого
→ местно собрать ResultPacket
→ выполнить машинные проверки
→ назначить независимую смысловую проверку при необходимости
→ одной транзакцией принять либо отклонить результат
→ пересчитать готовность зависимых единиц
```

The cycle runs on events and a durable schedule. It doesn't poll everything every few seconds or wake up an expensive model just to check availability.

The names `propose_work_graph`, `reserve_execution_slot`, `commit_route_decision`, `dispatch_attempt`, `commit_verified_result` and the rest below indicate logical stages, not individual public state writers. Public contracts are specified by the C-* register of document 19. The input `CommandEnvelope` is parsed and verified in C-02 with the identity of channel C-12. All changes then go through `handle_command(ValidatedCommand) → decide_transition(StateView, ValidatedCommand) → commit_transition(ExpectedRows, TransitionPlan, CommitPreconditions)`; pure solutions do not create events themselves. Accepting the result is a semantic variant of the typed command in this contract, and not an additional API with direct access to the database.

---

## 2. Responsibilities of parts

### 2.1. Scheduler

Has the right:

- propose a breakdown of the goal;
- create a limited number of branches of assumptions;
- identify dependencies;
- distribute the already established general limit between the branches;
- select the next finished unit;
- continue independent branches when one is waiting for an owner.

Does not have the right:

- expand the goal or general limit;
- consider waiting for one branch to be a stop for all;
- accept your own result;
- create an action with a consequence;
- endlessly create new branches.

### 2.2. Router

Selects not the model name, but the exact connection:

```rust
struct ConnectionIdentity {
    connection_id: EntityId,
    provider: ProviderId,
    endpoint: CanonicalEndpoint,
    account_label: AccountLabel,
    auth_method: AuthMethod,
    model_id: ProviderModelId,
    reasoning_variant: ReasoningVariant,
    adapter_version: SchemaVersion,
    request_schema_version: SchemaVersion,
}
```

He doesn't refer to the model. His output is the explainable `RouteDecision`.

### 2.3. Performer

Gets one unit, one selection, one time domain, and one limit. He does not plan all the main work and does not apply the result.

### 2.4. Inspector

Checks acceptance conditions and origin of evidence. It does not silently correct the result and does not complete the main work.

### 2.5. Host transition

The part of the control process that is the only one that can apply the tested proposition to the new version of the state.

---

## 3. Breaking down the goal

Version connections are applied between the main works [24, §§3–5](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md): conditions, accurate reporting of results and verification of future cycles are included in the acceptance of the graph. An arbitrary role or the phrase “wait for researcher” does not replace this data.

`propose_work_graph(objective, state, planning_limits)` returns only the graph clause.

Before committing, `validate_work_graph` proves:

- each unit is associated with at least one obligation, gap or required outcome;
- each unit has one testable output;
- no cycles;
- there is no unit without a stopping condition;
- the amount of branch reserves does not exceed the total limit;
- action with consequences is separated from reflection and preliminary analysis;
- verification of the result is a separate unit when the risk requires it;
- merging branches has its own check;
- the number of simultaneously ready units is limited by the main work policy.

A unit without connection to the target is rejected as junk work.

The Count is not required to know all future discoveries in advance. `Research` and `CreativeExploration` can have one output “report with alternatives and restrictions” and change the method inside `ExplorationContract` document 15. New units are limited proposals of the graph, checked against the same goal and total budget. A negative or indeterminate result may successfully close a research unit; it is not permission to declare a hypothesis an established fact.

---

## 4. Branches of assumptions

When branches from different works affect one real object, [24, §§6–8](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md). Association - a separate unit with a single responsible work and its own copy/verification; old verdicts of the original branches do not validate the result of the merge. This allows for more complex collaboration without giving away the hidden power to change the living tree.

For a checkout branch, the unknown must:

1. able to change a decision;
2. has a significant cost of error;
3. allows observation that distinguishes between options;
4. Can be tested within the allocated limit.

```rust
struct HypothesisBranchSpec {
    hypothesis_id: EntityId,
    parent_id: Option<EntityId>,
    discriminating_observation: Criterion,
    falsification_condition: Criterion,
    budget: ResourceReservation,
    workspace: WorkspaceRef,
    stop_condition: StopCondition,
}
```

Additional opinion without discriminating observation is not a new branch.

Creative exploration does not have to start with a falsifiable scientific hypothesis. It is issued `CreativeExploration` with a question, final output, width and cost limit; the detected variants can then spawn test branches. Division preserves the search for new ideas, but does not present several similar opinions as independent evidence.

When merging:

- negative results remain;
- a list of conflicts is constructed;
- contradictions are not averaged out;
- a new general edition is created;
- it is the combined total that is checked;
- a late result is not applied if its input dependencies or the affected join region have changed; unrelated concurrent results are not rejected due to the common event number.

---

## 5. Selection of connections

### 5.1. Hard elimination

`filter_connections` excludes connection if:

- it does not allow the required data class;
- there is no mandatory ability, type of input, means or form of response;
- the required selection does not fit;
- connection tolerance level is lower than required;
- health is outdated or in a state of mandatory cooling;
- the secret is missing or requires the participation of the owner;
- the independence of the inspector is violated;
- the circulation reserve exceeds the owner’s hard limit;
- allowed parallelism has been exceeded;
- the connector does not match the accepted version of the form.

Neither quality nor owner preference overrides the data ban or resource limit.

### 5.2. Ordering valid

After strict screening, an explicable lexicographical rule is applied, rather than one opaque score:

1. proven achievement of the mandatory quality threshold in this type of work;
2. absence of deteriorated condition;
3. compliance with the connection or group explicitly selected by the owner;
4. compliance with the required independence;
5. unlimited or already paid resource before a new monetary expenditure, if the quality is sufficient;
6. lower expected hard limit consumption;
7. lower observed latency;
8. stable identifier as the last way to remove equality.

The order is versioned. Each decision stores the reasons for excluding the remaining candidates.

### 5.3. `RouteDecision`

```rust
struct RouteDecision {
    unit_id: EntityId,
    dependency_snapshot_id: EntityId,
    selected: EntityId,
    selected_connection_revision: RowVersion,
    rejected: Vec<RejectedConnection>,
    required_capabilities: CapabilitySet,
    allowed_data_classes: DataClassSet,
    quota_snapshot: QuotaSnapshot,
    price_schedule_version: SchemaVersion,
    valid_until_utc: DateTimeUtc,
    resource_reservation_id: EntityId,
    independence: IndependenceAssessment,
    routing_rule_version: SchemaVersion,
}
```

The decision is not valid after a change in the significant dependency, applicable limit, health, tariff, data rules or precise connection, or after its own deadline. Accepting an unrelated branch does not undo it. The quota and cost of the photo serve as justification for the choice, but not permission to send it later without re-checking. Mandatory quality and independence are included in the strict screening process, and ranking cannot compensate for their absence with a cheap price.

---

## 6. Connection states

```rust
enum ConnectionState {
    Unverified,
    SyntheticOnly,
    Shadow,
    Active,
    Degraded,
    CoolingDown,
    QuotaExhausted,
    AuthRequired,
    Disabled,
}
```

Moving up the tolerance levels requires a saved test. An error in one login method does not disable another connection with the same model.

The quota restoration check is performed using a cheap local schedule no more often than the established deadline. If the supplier does not provide a cheap check, a rare minimal artificial task is performed and its cost is taken into account.

---

## 7. Concurrency and queue

The list of restrictions below is supplemented by a mandatory policy [25, §§3–9](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md). C-06 selects a valid connection and a fair move taking into account the wait, C-05 limits the acceptance queue by quantity and memory, C-03 maintains a single resource accounting. A free resource does not justify weak or prohibited substitution of a model; a long waiting move is only protected under explicitly specified availability conditions. The guard does not depend on the LLM queue or regular commands.

Restrictions exist simultaneously at the following levels:

- the entire installation;
- supplier;
- connections;
- sessions;
- main work;
- branches;
- units.

The actual parallelism is equal to the lowest effective limit.

`reserve_execution_slot` reserves a place in the queue and resources in one transaction. If the place does not appear before the end date:

- the attempt is not considered started;
- the reserve is released;
- a classified incident is created;
- a repeat is assigned later without creating a second card or unit.

You can't keep hundreds of requests inside a provider's internal queue. The local queue must apply back pressure before dispatch.

---

## 8. Attempt to contact

`dispatch_attempt` - sequence of one execution owner:

1. checks `DependencySnapshot`, unit revision and `RouteDecision`; calculates the upper cost of the actual body with a limit on output tokens and other charged units;
2. through a common command, atomically secures/specifies the reserve in a single resource log; the router and intermediary do not maintain a second balance;
3. stores `unit_attempt(prepared)`, instruction, converted request fingerprint and sending intent prior to network action;
4. creates a limited process tree; the performer receives instructions and opaque designations, but not the secret of connection;
5. directly at the sending, performs `validate_dispatch_preconditions`: read and expansion rights, the recipient after all transformations, the classes of the actual body and the absence of a secret, the freshness of the required sources, the required versions, the term and epoch of authority, the fence of work/units/rights, the launch generation, health, queue and the top cost at the current rate;
6. The intermediary in the short native tolerance boundary of document 17 serializes the final check with revocation/stopping and actual transmission to the OS. A saved photo or pass does not allow sending later: the body, address, tariff or generation have changed - a new permit is needed. The boundary is not held for the entire duration of the supplier's response;
7. saves the observation of the transmission (`running` is confirmed by the observed launch, not the text of the performer), accepts confirmations of life;
8. receives the original bytes and metadata of the transport; saves them in a secure archive without first correcting the answer;
9. records the observed flow rate with its origin; if flow is not known accurately, maintains a safe upper estimate/reserve until reconciliation rather than writing off zero;
10. builds `TrustedAttemptEnvelope` from its own records and passes the original model text to the local `ModelResult` parser.

The executor and the adaptation library do not have a key and independent network access. Network handling of the key is performed only by the secure connection broker. `AuthoritySnapshot` is evidence of a verified version and is not a substitute for the tolerance limit. Revoke after transmission is not able to return data that has already been sent, but it blocks new submissions and the acceptance of late content; Actual and expense observations are always saved.

A failure between the submission and its confirmation record leaves the outcome of the submission unknown. This is not a reason to immediately release the reserve or declare the challenge non-existent. Reconciliation uses the sustained intention and trace of the mediator; repetition of a pure model call is allowed only by a separate policy, taking into account possible repeated payment, and not as a proven one-time occurrence.

Exact refusals before shipping: `DependencyChanged`, `RouteExpired`, `PermissionExpired`, `PermissionRevoked`, `AuthorityFenced`, `FreshnessExpired`, `RecipientChanged`, `DataClassDenied`, `SecretDetected`, `RequestTooLarge`, `ConnectionUnavailable`, `ReservationInsufficient`, `CostBoundUnavailable`. They store the reason and safe links, not the text of personal data. The first require reassembly/reconciliation, prohibition of the right - a new valid authorized entry, the absence of an upper cost - blocking of the charged route; automatic border weakening is prohibited.

---

## 9. Repetitions

### 9.1. Failure classes

```rust
enum RetryClass {
    RetryableTransient,
    PermanentRequest,
    AuthenticationRequired,
    QuotaBlocked,
    UnknownOutcome,
    LocalInvariantViolation,
}
```

### 9.2. Rules

- `429`, confirmed queue overload, `502`, `503`, loss of connection before receiving a response and model timeout may be temporary;
- `400` is a permanent request error by default;
- `400` is repeated only if it matches a versioned indication of a known temporary cause from the provider, such as an explicit message that a temporary free space threshold has been reached on the service side;
- unknown text `400` does not loop;
- our form validation error is corrected before submitting again;
- each repetition creates a new attempt of the same unit and is taken into account in the hard limit;
- before each repetition, permissions, freshness, tariff and actual body are checked again; the old dispatch permit is not transferred;
- the delay grows with a limited random offset and is stored in the database;
- restarting the application does not reset the number of attempts and does not create a duplicate;
- reaching the retry limit creates one incident, not a stream of notifications.

For a pure model call, the loss of a response may allow a new attempt if the result could not have had an external consequence. For an action with a consequence, only the intent check from document 17 is applied.

### 9.3. Why is there only one repeat code?

Connectors return a classified failure and a recommended minimum time. They don't run their own loops. The only long-lived retry manager sees the total limit, number of attempts, running status, and stopping.

---

## 10. Replacing the connection

`select_fallback_connection` performs full hard screening again. Replacement is only allowed if the following are saved:

- data classes;
- required abilities;
- minimum proven quality;
- form of the result;
- remainder of the hard limit;
- required independence;
- the known outcome of a previous attempt.

The coincidence of the model name does not prove equality. The replacement is recorded by a separate route decision and a beacon.

---

## 11. Parsing the result package

`parse_result_packet` - local conversion of `raw_response + ожидаемая схема ModelResult + TrustedAttemptEnvelope + след посредника → ResultPacket | ResultParseError` (document 15). The model does not write a trusted shell. Before the semantic check, the stage checks:

- version of the form;
- correspondence of the trusted shell to its own attempt, unit, snapshot of dependencies, route and fingerprint of the actually sent order;
- absence of unknown control fields;
- syntax and resolvability of links; the material is considered to exist only after receiving from the intermediary and verifying the fingerprint, and not along the path declared by the model;
- compliance of the reported calls with the actual trace of the intermediary;
- lack of secrets in the stored fields;
- the presence of a typified division of the stated fact, conclusion, hypothesis and the unknown; the validity of this division is checked semantically;
- presence of negative results and limitations;
- the origin of the expense is from the records of the intermediary/supplier, and not from model JSON;
- suitability of the dependency snapshot for further verification.

The phrase "done" has no special meaning.

A schema-valid response with stale inputs is saved as `stale` rather than being masked by a JSON error. Shell error - violation of local quarantine agreement. Correction of the model form creates a new derived candidate or a new limited attempt with reference to the source; the parser does not complete “convenient” facts, identifiers, links or expense. `outcome=partial` is valid in form and can then obtain `gaps` or `pass` for an honest exploration of the unknown - the solution is determined by the conditions of the unit.

---

## 12. Independent verification

Detailing ability and independence - [26, §4](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md): compares exact method, task and error class, source marking, false acceptance/rejection rates and coverage. The name of another model does not replace such verification. Insufficient evidence limits the qualification of a particular decision method rather than the entire generation and research work. A new version of an order or source requires checking the applicability of the old qualification.

### 12.1. Machine check first

It always runs locally and checks form, fingerprints, versions, entitlements, consumption, material existence, file status and prerequisites in an accurate, verifiable manner.

### 12.2. Then semantic check

It is needed when the condition cannot be proven only by machine. The reviewer receives:

- acceptance conditions;
- package of approvals;
- newly collected initial evidence;
- created materials;
- information about the origin of the performer;
- specific questions of searching for counterexamples.

He does not automatically receive the author's persuasive move and his rights.

### 12.3. Degrees of independence

```rust
enum VerificationLevel {
    MachineOnly,
    IndependentReview,
    DiverseChallenge,
    ProtectedAdmission,
}

struct IndependenceAssessment {
    policy_id: EntityId,
    policy_version: SchemaVersion,
    required_level: VerificationLevel,
    candidate_lineage_ref: EntityId,
    verifier_lineage_ref: EntityId,
    comparisons: NonEmptyVec<IndependenceFinding>,
    qualification_record_refs: Vec<EntityId>,
    decision: IndependenceDecision, // Satisfied | Insufficient | Unknown
}
```

`IndependenceFinding { dimension, relation: Same|Different|Unknown, evidence_refs, explanation }` has a closed axle `Attempt`, `PromptMethod`, `Projection`, `EvidenceRetrieval`, `Connection`, `ModelFamily`, `Provider`, `CriteriaOrigin`, `ValidationMethod`. The relationship is established by the control part based on the saved contract and trace, not by the response of the checking model. A new hash or `different=true` without meaningfully different method/inputs is not sufficient. Unknown origin does not count as a difference.

The levels specify minimum requirements, not the probability of truth:

| Level | Required content |
|---|---|
| `MachineOnly` | Each condition has a deterministic method, a trusted input, and an observed result. The model is not a proof of the condition. |
| `IndependentReview` | Another try; separate verification profile with search for refutations; the criteria are fixed outside the candidate; the selection was re-obtained from canonical sources. The coincidence of the original source is acceptable, dependence only on the author’s retelling is not. The same model/connection is permissible only with a methodology qualified for hidden tasks and an explicit mark of residual correlation. |
| `DiverseChallenge` | All conditions of `IndependentReview`, plus other precise connection and proven distinction of family or verification method with independent observation (test execution, primary source, measurement). An identical request through another account is not suitable. The decisive conclusion and counterexample are checked, not just the agreement of the text. |
| `ProtectedAdmission` | `DiverseChallenge` plus candidate non-changeable criteria/hidden tests, separate cumulative change test and proof that candidate did not change their test. Changes to the protected basis are permitted only by the owner according to documents 13/17; no model verdict creates this right. |

The minimum policy of the first version is fixed before receiving the candidate:

| `WorkKind` | Minimum and what exactly is accepted |
|---|---|
| `DeterministicCheck` | `MachineOnly`, only if all criteria are truly machine; otherwise `IndependentReview` for the semantic part |
| `Research`, `CreativeExploration` | `IndependentReview`: relevance to the question, honest origin of factual statements, useful alternatives/limitations, not "proven novelty" of each idea |
| `Planning`, `Summary` | `IndependentReview` plus machine graph/links; for reduction - search for losses of negations, exceptions, rights and unknown outcomes by source |
| `Synthesis` | `DiverseChallenge` on the combined result and decisive contradictions |
| `PreflightAnalysis`, `LocalChangeProposal` | `DiverseChallenge` with verification of real objects and consequences; does not replace permission to perform an action |
| `ModelAdaptation` | `ProtectedAdmission` on a closed comparison with the base profile and regressions |
| `Verification` | Section 12.5 Restricted Procedure: Its entry is machine-checked and, if necessary, by one pre-assigned additional parse, a non-recursive copy of the same policy |

The cost of error increases the minimum: a factual conclusion on which an action with consequences or a significant decision depends requires at least `DiverseChallenge`, even if obtained by the unit `Research`. All criteria of one unit have a pre-selected method of verification; immeasurable aesthetic choices are left to the owner, and agreed-upon constraints are machine-checked. The absence of a qualified route gives `Insufficient`/`Unknown`: you can save the draft and continue independent work, but you cannot silently downgrade the acceptance. Qualification includes connection version, type of work, closed set of errors and revision period; The mere change of family or supplier does not prove independence.

### 12.4. Verdicts

- `pass` - the conditions are met and the origin is proven;
- `gaps` - the result is useful, but the exact open conditions are listed;
- `reject` - the result contradicts the obligation or evidence;
- `invalid` - verification is not possible due to shape, origin, old version or boundary violation.

`gaps` creates only the necessary subsequent units. `invalid` does not turn into a negative conclusion about the hypothesis itself.

### 12.5. Who checks the inspector and where the chain ends

Before the first check, `VerificationPlan` is saved: target `packet_hash`, dependency snapshot, policy version, criteria and methods for each criterion, required level, route qualification, resource limit, maximum semantic passes and disagreement rule. For the first version, one main semantic passage and no more than one additional analysis of controversial points are allowed (`max_semantic_rounds = 2`); the upper budget covers both. There are no semantic passages for `MachineOnly`. This is the chain limit of a single candidate revision, not permission to endlessly rename the same answer.

`max_candidate_revisions` - mandatory final positive limit of repair revisions in terms of main work, checked together with the general resource log; repeating, changing the role/route and restarting the application do not reset it. The number of two passes is a limitation of the first policy, not a proven universal optimality; the policy change requires closed trials and does not retroactively change a candidate's already issued plan.

The basis of trust is a verified version of the machine validator, protected pre-accepted criteria, observable evidence and qualification of the method on closed problems. It does not mean infallibility. The machine validator checks `VerificationRecord` for form, coverage of all criteria, links, independence, relevance of inputs and not exceeding the plan. The semantic evidence of the criterion must refer to the material/observation; “I agree” is not enough.

An additional pass is assigned only for a specific conflict or gap of the first, receives controversial criteria and independently collected evidence, and does not have the right to accept the majority opinion. If the conflict persists or the limit is exhausted, the result remains `gaps`/`invalid`, one incident is recorded; Only safe independent units are executed. The owner is informed only when his decision is really necessary. Neither `Verification` nor its verification generates another universal “verifying verifier”. A major revision creates a new revision of the candidate, but the expense, number of repair cycles, and cause continue to be included in the final budget for the underlying work.

`VerificationRecord` stores `target_packet_hash`, `dependency_snapshot_id`, `verification_plan_ref`, `machine_findings_ref`, `semantic_attempt_refs`, `criteria_results`, `independence_assessment_ref`, `verdict`, `open_gaps`, `rounds_used`. The verifier itself returns only candidate conclusions based on the criteria; reliable numbers, origin and final record are formed by the control part. A reviewer can propose a correction, but not simultaneously become the author of the corrected candidate and its accepting reviewer.

---

## 13. Acceptance of the result

Internal provision of verified result to other work regulates 24; it does not mean `owner_accepted`. The main work in `awaiting_owner_acceptance` does not block all others: the exact condition of each connection is taken into account. Queue and checkpoint confirmation - 25. Combination checks `C/F/checkpoint` - 26: an acceptance already recorded before the fence preserves the historical fact, but its later confirmation does not give the right to continue the stopped area.

`commit_verified_result` accepts:

- `CommitPreconditions`: snapshot of input dependencies, expected versions of affected record lines and current guard conditions;
- `ResultPacket`;
- suitable `VerificationRecord`;
- snapshot of current limits and rights;
- list of affected obligations.

When preparing a decision, then in the general acceptance transaction under the tolerance limit of document 17, the following are re-checked:

1. the version of the target and unit has not changed;
   all significant elements of `DependencySnapshot.read_set` and criteria versions match; changing an unrelated branch is not itself a conflict;
2. the attempt has not been canceled and belongs to the current launch generation;
3. the late result did not come after the right was revoked;
4. all materials exist and match the print;
5. the check satisfies the required independence;
6. the flow has an observable origin and is safely accounted for; an unknown amount does not release the reserve and does not close the resource reconciliation;
7. there is no unknown outcome on which the conclusion depends;
8. no hard limit is exceeded;
9. New units do not expand the goal.

`VerificationRecord` is associated with the candidate's exact fingerprint and set of criteria. The old candidate's verification is not reused for the corrected one without an explicit new verification contract. One pre-check `AuthoritySnapshot` is not enough: the final fence and SQLite COMMIT are serialized with a revoke/stop, so there is no unprotected gap left between "the right was valid" and the application. The general constraints and CAS of the affected rows are checked in the same transaction; automatic merging of conflicting records is prohibited.

Transaction at the same time:

- changes the state of the result and the unit;
- applies an acceptable subject change;
- connects the already observed consumption with the attempt and releases only the proven free balance; the write-off of an actually paid call does not depend on the acceptance of its content and is not duplicated;
- creates an event `result_committed` or `result_rejected`.

The main work after all internal checks goes to `awaiting_acceptance`. Only the owner's trusted decision transfers it to `accepted`.

A late/withdrawn response and expense incurred are saved as observations, but do not fulfill obligations, change accepted materials, or trigger a continuation. When independence or disagreement is exhausted, a false `pass` is not created for the sake of completion.

---

## 14. Logical stages of the control cycle and their owners

This table does not expand the public surface of C-* document 19. Selection and verification stages - internal functions of the respective owner; recording stages - options `TypedCommand` with a single acceptance path. Do not create public passthrough wrappers of the same name just for the sake of matching names in prose.

| Function | The only duty |
|---|---|
| `propose_work_graph` | create a limited graph proposal |
| `validate_work_graph` | prove the connection with the goal, finitude, dependencies and limits |
| `commit_work_graph` | accept one version of the graph by transaction |
| `select_ready_units` | find units with fulfilled dependencies |
| `reserve_execution_slot` | indivisibly reserve concurrency and resources |
| `filter_connections` | apply only strict prohibitions |
| `rank_connections` | explainable ordering of valid connections |
| `commit_route_decision` | save exact route solution |
| `validate_dispatch_preconditions` | check the current premises of the actual request; do not issue long-lived bypass pass |
| `dispatch_attempt` | create and run one limited attempt |
| `record_worker_heartbeat` | confirm the life of a specific generation process |
| `classify_attempt_failure` | return private failure class |
| `schedule_retry` | save one next launch with the same history |
| `parse_result_packet` | turn the original answer into a testable package |
| `run_machine_verification` | perform deterministic checks |
| `schedule_semantic_verification` | create an independent verification unit |
| `assess_verifier_independence` | apply version policy to observable origin, return `Satisfied`, `Insufficient`, or `Unknown` |
| `commit_verified_result` | only apply the verified result |

You cannot create separate variations of these features for normal talk, deep mode, or a specific provider. What is different is the entry contract, not the defensive sequence.

---

## 15. Beacons

| Beacon | What does it prove |
|---|---|
| `work_graph_proposed` | split origin |
| `work_graph_committed` | limb, version and limit distribution |
| `execution_slot_reserved` | no concurrency or quota exceeded |
| `connection_candidates_filtered` | reasons for hard failures |
| `connection_selected` | exact connection, revision, dependency snapshot and rule version |
| `dispatch_preconditions_checked` | body print, freshness, tariff, reserve and decision before the tolerance limit |
| `attempt_started` | attempt, process, generation, commission and reserve |
| `worker_heartbeat_observed` | life of the exact process, not the entire service |
| `attempt_failed` | closed class and proof of failure |
| `retry_scheduled` | number of attempts, time and remaining limit |
| `result_submitted` | separately immutable original answer, trusted shell and model package |
| `verification_completed` | criteria, origin of independence assessment, plan, spent passes and verdict |
| `verification_independence_failed` | missing/unknown measurements and qualifications without lowering the threshold |
| `verification_limit_reached` | specific disputed terms, exhausted limit and retained unaccepted outcome |
| `result_committed` | Matched Dependencies, Affected Row Versions, and Completed Commitments |
| `late_result_rejected` | reason for banning late commit |

---

## 16. Mandatory checks

1. Pending permission only blocks dependent branches.
2. The graph cycle and unit without connection with the goal are not fixed.
3. Four parallel branches cannot together exceed the common limit.
4. The local queue does not send a request beyond the connection limit.
5. Queue expiration creates one long-lived repeat, not a duplicate unit.
6. A connection prohibited for personal data is inferior to even a higher quality model.
7. The owner can require the model, but not accidentally override the data protection limit.
8. Unlimited sufficient connection is preferred to a paid one without losing the required quality.
9. Temporary error `400` is repeated only according to the known version of the attribute.
10. Permanent error `400` does not loop.
11. Restarting does not reset the number of attempts and does not start them until the “continue” command.
12. The connector does not contain its own hidden retry loop.
13. The later package of the old edition is retained and is not used.
14. The reviewer does not accept non-existent material.
15. Another name for a role with the same model, selection and method is not considered independence.
16. `pass` internal inspection does not complete the main work without the owner's decision.
17. Rights, freshness or value change in the queue: the previous submission is rejected before transmission to the OS, and not after leakage/expense.
18. Revocation competes with dispatch and with COMMIT: after the point of revocation there is no new transmission or acceptance of content; observation of what has already happened and consumption are not lost.
19. Two independent units are accepted for different numbers of the general event; changing the source read or the overall outcome affected creates a precise conflict.
20. The model substitutes trusted fields and zero flow: the form is rejected, real observations are not replaced.
21. For each `WorkKind`, the sufficient, insufficient and unknown independence routes were checked; another account of the same model does not create `DiverseChallenge`.
22. The first verifier and additional parsing diverge: after two semantic passes there is no third recursive verifier and false `pass`; the budget is not reset by the candidate's correction.
23. `Summary` one author’s retelling does not pass independence; a re-obtained source of the same source may be admissible under a qualified policy.
24. A creative report with new hypotheses and honest uncertainty is accepted under a research agreement without fictitious proof of each idea.
25. Changing the candidate after `pass` does not go through the old `target_packet_hash`; the unknown cost holds the reserve even if the answer is rejected.

---

## 17. Prohibited simplifications

- select a model by one name;
- reduce all properties to one point;
- consider any error `400` temporary;
- repeat inside the connector;
- give each branch a full general limit;
- create a branch for one more opinion;
- accept most identical models as truth;
- allow the evaluator to correct and accept his corrected answer;
- consider the lack of response as a proven lack of action;
- complete work on the text of the model;
- automatically resume the old attempt after launching the application;
- consider a snapshot of authority to be sufficient protection against revocation between check and action;
- invalidate any parallel work on a single global `row_version`;
- accept the model’s self-description as reliable execution metadata;
- set independence to one `bool` or require endless verification by the inspector;
- turn research uncertainty into a mandatory refusal or fictitious certainty.
