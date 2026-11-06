# MASTER HANDOFF – Hermes / Deep Create
## Full continuation contract after Z.A.E.B.A.L. audit
**Transfer date:** 2026-09-16  
**Purpose:** This document is passed on to the next AI as an executable continuation contract, rather than as a reference note.

---

#0. CRITICAL INSTRUCTION TO THE NEXT AI

You don't start with another local fix `continue15.py`, `repair16.py`, one worker or one integrator.

Your task is to **continue all Hermes / Deep Create work in accordance with the user's original goal and the entire project history**, restoring the actual state on the machine and then autonomously bringing the work to the user's result.

Don't ask the user to re-explain something that has already been explained.  
Don't consider this handoff to be the absolute truth about the current runtime. This is a contract and a historical map, and **the live state is always double-checked against files, Git, processes, databases, logs, diffs and actually running scripts**.

Main principle:

> **Don't optimize for a local metric instead of a user goal.**

`running`, PID, HTTP 200, number of agents, number of lines of code, a beautiful report, exit code of an individual worker, or even several green unit tests - these are just intermediate signals. They do not prove that the system does what the user asked.

---

#1. ROLE OF THE MAIN MODEL

The main strong model is **architectural integrator, goal controller and final critic**, and not a manual dispatcher of each worker.

She should:

- keep the northern goal;
- notice the substitution of goals with proxy metrics;
- build or adjust the organization mechanism;
- delegate mass implementation to external models through Hermes / available routes;
- do not waste your own expensive resources on mechanical work that can be performed by external models;
- intervene in the middle only when the automated organization encounters a truly ambiguous architectural/authority blocker;
- conduct or arrange for an independent final review;
- do not take the performers’ statements at face value.

She **shouldn't** manually decide after each review who to launch next. Normal work must continue mechanically and autonomously.

---

#2. NORTHERN GOAL

The project consists of two related levels.

## 2.1 Hermes as a reliable foundation

Hermes should be a system in which:

- original user/assistant/tool ​​messages are not lost or changed due to compaction, model switch, restart or update;
- model context is separated from the durable user story;
- the update does not promote unverified runtime;
- the previous working version and user data do not disappear;
- local architectural overlay changes are not lost during upstream update;
- the Hermes change agent understands the affected architecture, invariants and checks;
- a critical change cannot be accepted just because the author said “done”;
- there is a normal trace request → agent → tool → change → verification → update;
- after compaction, the system can restore **the meaning of the work**, and not just a short summary of the latest messages.

Much of this foundation has historically already been implemented and tested. **Don't rewrite everything again without proven regression.**

## 2.2 Deep Create as a long-lived autonomous system for achieving goals

Deep Create is not a “multi-agent chat” or a fixed swarm.

This is a separate mode / system of long-term work in which the user can set a goal like:

- create a product;
- explore an unknown area;
- remake a large system;
- build a business;
- earn money;
- develop Deep Create itself.

The system itself must:

1. understand the exact purpose and authority;
2. extract only relevant past context with provenance;
3. present the unknown as unknown;
4. create competing hypotheses/plans;
5. determine what needs to be researched, written, tested, or built;
6. create a dynamic work schedule;
7. select models/tools/context for each work;
8. scale parallelism where it is really useful;
9. detect blockers and rebuild the plan;
10. conduct independent verification;
11. integrate compatible results;
12. do not accept unconfirmed results;
13. save the history of the reasons for decisions;
14. continue after restart / compaction / model change;
15. come to the final artifact/outcome, and not to a beautiful story about the work.

Formula:

```text
user request
→ exact goal + authority
→ obligations + hypotheses
→ required context/evidence
→ dynamic work graph
→ execution / research / creation
→ counterevidence + independent verification
→ repair / replan / integrate
→ user-visible outcome
→ durable history of why
```

---

#3. WHAT DOES THE USER MEAN BY “AGI THAT KNOWS EVEN WHEN IT DOESN’T KNOW”

Not omniscience.

Correct meaning:

```text
не знаю X
→ явно представить X как unknown
→ понять, влияет ли X на цель
→ придумать способ уменьшить неопределённость
→ выбрать исследование / эксперимент / инструмент / другого агента
→ получить evidence
→ обновить belief
→ не выдавать гипотезу за факт
```

The system is strong not because it knows everything in advance, but because it **knows how to work with its own ignorance**.

---

#4. MAIN UNITS OF DEEP CREATE

Do not mix “chat”, “model session” and “all work”.

```text
DeepCreateSession
  Долгоживущая работа вокруг одной большой цели.
  Может жить недели/месяцы.

Run / Job
  Один конкретный прогон внутри большой работы.

ObjectiveContract
  Машиночитаемая формулировка желаемого результата,
  критериев успеха, ограничений и authority.

Obligation
  Что должно быть сделано или доказано,
  чтобы результат можно было принять.

Hypothesis
  Проверяемое предположение/стратегия.

WorkUnit
  Ограниченная единица работы.

AgentAttempt
  Один конкретный запуск модели с PromptBundle.

Branch
  Альтернативная линия решения/гипотезы.

Worktree / Isolated Workspace
  Физически изолированная копия для изменяющей код работы.

ResultPacket
  Что утверждает worker, что изменил, чем доказал,
  что осталось неизвестным.

VerificationRecord
  Независимая проверка результата.

Decision / ADR
  Почему вариант принят/отклонён и на каком evidence.
```

---

#5. CONTEXT AND MEMORY

## 5.1 You cannot pass the “all memory as text” model

This creates noise, old contradictions and false anchors.

Needed:

```text
Raw immutable evidence
→ retrieval by purpose/scope
→ candidate evidence set
→ provenance / privacy / freshness / contradiction checks
→ ContextProjection для конкретной роли
```

The memory entry should have a meaning like:

```json
{
  "claim": "...",
  "source_event_id": "...",
  "source_type": "explicit_user_statement | artifact | model_hypothesis",
  "confidence": "...",
  "scope": ["..."],
  "freshness": "...",
  "supersedes": [],
  "contradictions": [],
  "allowed_purposes": ["..."]
}
```

The unverified output of the old agent **does not become a fact about the user**.

## 5.2 Prompt Bundle

The agent does not have one magical system prompt.

He gets the composition:

```text
1. Constitution
2. Role + authority
3. ObjectiveContract
4. ContextProjection
5. CapabilityLease
6. OutputSchema
7. Current WorkUnit
8. Open liabilities / contradictions
9. Relevant verification history
```

## 5.3 After compaction

Summary is **not a source of truth**.

After any compaction, the system must be rehydrated from the canonical state:

- ObjectiveContract;
- obligations;
- accepted decisions;
- open blockers;
- work graph;
- artifact IDs/hashes;
- verification verdicts;
- user authority;
- current candidate;
- latest source events.

Compaction can reduce the text of the model, but has no right to change the historical purpose or destroy liabilities.

---

#6. SKEPTICAL CONSTITUTION

This is a mandatory way of working, not a communication style.

1. **Don't agree with the user out of politeness.**
   If his hypothesis is not proven, say that it is a hypothesis and check it.

2. **Don't defend your own decision with words.**
   If the behavior is controversial, run/log/diff/test/user-visible artifact is a stronger explanation.

3. **Classify each significant statement internally:**
   - FACT - artifact/execution confirmed;
   - HYPOTHESIS - plausible, but not verified;
   - UNKNOWN - no data;
   - DISPROVED - the check has refuted.

4. **The worker's report is a statement, not a fact.**
   Check diff, artifact, exit code, test, runtime.

5. **`running` ≠ progress.**
   The PID may live in a useless loop.

6. **HTTP 200 ≠ useful result.**

7. **green unit tests ≠ correct user outcome.**

8. **healthy process ≠ correct runtime instance.**
   Check which process/DB/config/port actually serves the user path.

9. **don’t choose the first plausible reason.**
   Hold competing hypotheses until discriminating check.

10. **do not declare readiness based on the percentage of lines/files/agents.**
    Readiness is determined by obligations and outcome gates.

---

# 7. OPERATING MODES

Historically, five levels have been discussed and implemented:

```text
Лёгкий
Средний
Высокий
Очень высокий
Ультра
```

Key principle:

> A mode is a **ceiling of resources and depth**, and not an obligation to run a fixed number of agents.

Approximate semantics:

### Lightweight
Direct path, minimal work. One worker is acceptable if the task is really simple.

### Average
A small branching, testing the key hypothesis, verifier if necessary.

###High
Several independent branches, deeper research/implementation, mandatory verification of critical claims.

### Very tall
Dynamic multi-agent work, independent lineages, red-team, integration, advanced checks.

### Ultra
Long-lived organization, maximum **useful** concurrency, dynamic replanning, research, implementation, independent criticism and continuation to outcome / authority gate / proven impossible blocker.

A simple task, even in Ultra, can end up in one call if the additional branches don't have the expected value.

The system can recommend/automatically increase depth, but should not pointlessly fan-out one prompt eight times.

---

#8. AGENT INDEPENDENCE

Calling one `Author` and the other `Critic` is not enough.

If they have the same:

- model family;
- provider;
- context projection;
- prompt template;
- sources;
- retrieval;
- summary;

then the errors are correlated.

Need `IndependenceLineage`.

The Verifier should be as diverse as possible in source/context/model line/verification method and be able to say:

```text
NOT PROVEN
REWORK
HUMAN_REQUIRED
```

Consensus models are not truth.

---

#9. WORKING FORMATS THAT THE SYSTEM SHOULD COMPLETE

The user's request determines the configuration itself. Not every task has to go through the same pipeline.

Available WorkUnit types:

- objective/intent compilation;
- context retrieval / memory audit;
- research;
- source verification;
- implementation;
- test generation;
- deterministic verification;
- adversarial review;
- architecture critique;
- chaos/alternative-idea generation;
- integration;
- repair;
- runtime smoke;
- incident diagnosis;
- recovery;
- documentation/ADR;
- final acceptance.

Controller builds the desired combination of these functions.

---

# 10. CAPABILITY LEASE, NOT A HARD PRISON SCOPE

Worker receives limited `CapabilityLease`:

- paths;
- tools;
- network;
- runtime namespace;
- external side effects;
- budget;
-expiry.

But the critical mistake of the previous organization was that the worker discovered a defect in a neighboring module and **had no mechanism to bring the problem to someone who could actually fix it**.

Correct protocol:

```text
worker discovers blocker
→ BlockerPacket
→ controller classifies
→ choose one:
   A. expand existing lease safely
   B. spawn owner WorkUnit for affected component
   C. create cross-component integration WorkUnit
   D. fork hypothesis / replan
   E. human_required only for real authority/ambiguity
```

Worker is not obliged to silently climb anywhere.  
But the organization also does not have the right to send him to the same dead end four times.

---

#11. BLOCKER PACKET

Minimum:

```json
{
  "blocked_work_unit": "...",
  "blocking_obligation": "...",
  "type": "cross_component | missing_capability | missing_evidence | architecture_conflict | external_failure",
  "affected_components": ["..."],
  "evidence": ["artifact/ref/..."],
  "required_capability": ["..."],
  "minimal_unlock": "...",
  "previous_attempt_fingerprint": "...",
  "risk": "...",
  "recommended_next_action": "..."
}
```

Controller must respond with a structural change in the work graph, and not just a new retry.

---

#12. LOOP DETECTION – A MUST

The historical organization lost a lot of time in repeated reviews.

Need a fingerprint attempt:

```text
objective
+ WorkUnit
+ code/artifact hash
+ blocker fingerprint
+ verifier findings
+ relevant context projection
```

If the new attempt has the same blocker and there is no significant artifact/evidence delta:

```text
НЕ retry
→ replan
```

Possible actions:

- change owner;
- expand capability;
- select dependency;
- combine modules;
- change model/tool;
- create an experiment;
- recognize architecture conflict;
- escalate authority.

The number of repetitions should not be the magic “4”. Signal - **lack of useful delta**.

---

#13. PROGRESS

Key metrics:

- obligations closed;
- verified behavior added;
- blockers actually resolved;
- accepted artifacts;
- integration readiness;
- unknowns reduced;
- user-visible capabilities working.

Secondary:

- code diff;
- tests;
- elapsed time;
- calls;
- worker count.

Do not count as progress:

- just a living process;
- re-reading the same files;
- new report with the same blocker;
- repeated review of the same hash;
- HTTP 200;
- growth of token usage;
- number of launched models.

---

# 14. DYNAMIC PARALLELISM

The user wants scale, speed, and maximum use of available external models.

But "maximum" means:

> maximum **independent useful work**, not maximum processes.

Controller must evaluate:

```text
Expected Value of WorkUnit
≈ probability useful
× impact on obligations
× information gain
× unblock value
÷ (cost + conflict risk + correlated-error risk + route pressure)
```

Run more when:

- there are independent ready WorkUnits;
- no conflicting ownership;
- routes are healthy;
- reviewers/integrators did not become bottleneck;
- the results can be accepted.

Reduce fan-out when:

- workers wait for one common blocker;
- several models duplicate one inference lineage;
- integration queue is full;
- the provider is under resource pressure;
- all tasks touch one critical file.

---

#15. CODE ISOLATION

Historically correct user idea:

- each coding worker works in a separate worktree / isolated copy;
- branch/runtime namespace are separate;
- ports / DB / Redis prefix / temp / cache is issued by the runtime manager;
- you cannot change the product code just for the sake of the local port;
- dangerous shared files have special ownership/merge policy;
- the integrator does a 3-way merge;
- semantic conflicts are checked separately from Git conflicts;
- after merge, typecheck/lint/unit/integration/build/runtime tests are performed;
- the worker leaves a manifest of its changes.

The original project is not used as a sandbox.

---

# 16. WHAT HAS BEEN DONE HISTORICALLY AT HERMES
## IMPORTANT: these are historical facts from past work; live runtime recheck

At various points the following were implemented/tested:

- durable display history separate from the bounded model context;
- saving compacted originals;
- removing `.strip()` from the display path;
- safer edit/truncate;
- profile-aware DB;
- recovery Git ref before update;
- Windows UTF-8 subprocess fixes;
- `runtime_events` ledger;
- write-ahead `prompt.accepted`;
- model-switch as runtime event instead of synthetic user transcript marker;
- more complete branch envelope:
  reasoning / tool calls / IDs / timestamps / token metadata / platform IDs;
- auxiliary compression:
  `openai-codex/gpt-5.6-luna`;
- fallback compression route;
- `abort_on_summary_failure: true`;
- message integrity snapshots;
- Architecture Intelligence:
  `architecture/manifest.yaml`, query/impact/capsule/preflight/validate;
- Agent Change Gate;
- verifier evidence;
- updater regression tests;
- Desktop typecheck/build and migration rehearsal on candidate runtime.

Also, historically there was a situation when candidate Desktop was launched, but old sessions were not shown due to an incorrect runtime/DB/profile/projection link. This was important evidence:

> process smoke ≠ user-visible integrity.

You cannot assume that any of these items are now correct just because they once passed the test. But you can’t rewrite it from scratch without regression evidence.

---

# 17. DEEP CREATE: WHAT ALREADY HAS BEEN HISTORICALLY

There is a large volume:

- technical documents and contracts;
- Rust/TypeScript code;
- domain/protocol/storage/execution/context/model-routing/scheduler/broker/worker/watchdog/Windows/update/UI parts;
- acceptance/verification tooling;
- several generations of isolated candidates;
- external review/repair/integration artifacts.

Percentages from old answers **do not use as truth**. They were assessed in different ways and became part of the problem themselves.

Current readiness can only be determined through:

```text
requirement/obligation
→ implementation
→ test/evidence
→ integration
→ user-visible behavior
```

---

#18. CONFIRMED INCIDENT OF THE PREVIOUS ORGANIZATION

Historically, the following happened:

1. The user demanded that the development be completely transferred to an AI organization.
2. The organization itself had to:
   - plan;
   - create new tasks;
   - adjust course;
   - scale;
   - conduct review/repair/integration;
   - call the main model mainly for final acceptance.
3. Implementation gradually turned into fixed lanes/tasks/review passes.
4. Several workers received comments about neighboring modules that they did not have access to.
5. They went through the review again, but the blocker could not disappear.
6. Evidence of some passes was overwritten.
7. There were multiple compaction/review cycles; The request context of individual workers grew instead of decreasing.
8. The processes were alive and the API was responding, but useful progress stopped.
9. Instead of restructuring the organization, one cross-module integrator was created.
10. This removed the separate scope-blocker, but **did not restore the requested self-organizing system**.
11. The user rightly pointed out that this is again a substitution of the task.
12. Z.A.E.B.A.L.-audit stopped the current circuit.
13. The previous audit correctly noted the static-batch problem, but again narrowed the next step to “fix controller scenarios.”
14. The user rejected this as a **preliminary analysis and not a recovery of the entire target**.

---

#19. WRONG BELIEF THAT CANNOT BE REPEATED

The false belief went something like this:

> “If a fixed controller keeps workers alive, gives them multiple retry/review passes, keeps status and local tests green, then we are implementing self-organization.”

No.

A static pipeline can be part of an organization, but does not replace:

- dynamic work graph;
- dependency ownership;
- blocker routing;
- replanning;
- capability negotiation;
- useful-delta detection;
- dynamic scaling;
- semantic integration;
- durable goal/context recovery.

Second false substitution:

> "If the current controller is broken, the user's current task is to fix the controller."

No.

Controller is a means.  
The goal is to **autonomously bring Deep Create to a fully-fledged accepted application under fixed contracts**.

---

# 20. COMPACTING: WHAT IS PROVEN AND WHAT IS NOT

FACT:

- there have been many automatic context compactions in history;
- after the compaction part it was necessary to clearly restore the architectural context;
- created by `READ_FIRST_AFTER_COMPACTION.md`;
- the user separately asked to save not only documents, but also semantic “keys”, its wording and architectural scale;
- the late org actually started optimizing controller/review locally instead of the big target.

HYPOTHESIS:

- it was compaction that was the only or main reason for the degradation of the solutions of the main model.

This is **not proven**.

Two reasons are possible at the same time:

1. lossy context recovery after compaction;
2. initially insufficient meta-controller / orchestration architecture.

The new system should solve both, and not argue which was the main one.

---

# 21. FIVE TYPES OF CONDITION AFTER ANY RUN

Not only `success/fail`.

```text
ACCEPTED
  obligations доказаны и artifact принят.

REWORK
  путь сохраняется, но есть конкретные исправимые дефекты.

REPLAN
  текущая структура работы больше неадекватна.

PARTIAL
  часть obligations закрыта, часть открыта.

UNKNOWN / BLOCKED
  данных не хватает или нужен authority/external condition.
```

Rollback is a separate action, not “failure”.

---

#22. HOW THE NEW ORGANIZATION SHOULD WORK

## Control plane

It is desirable to be as deterministic as possible:

- canonical ObjectiveContract;
- requirement/obligation graph;
- work graph;
- artifact registry;
- leases;
- dependency graph;
- route/resource manager;
- event ledger;
- review fingerprints;
- policy transitions;
- compaction recovery state.

LLM is used for semantic planning/review/integration where deterministic logic is not sufficient.

## Planning loop

```text
observe canonical state
→ find open obligations
→ find blockers/unknowns
→ generate candidate WorkUnits
→ estimate independence/value/risk
→ schedule useful ready units
→ collect ResultPackets
→ verify
→ update graph
→ integrate/repair/replan
→ repeat
```

It should be a **cycle**, not one pre-written list of 27 tasks.

---

#23. FIRST THING THE NEXT AI DOES ON THE MACHINE
## FIRST READ-ONLY REALITY RECONSTRUCTION

Do not start the old controller automatically.

Check:

### Deep Create original
```text
D:/Projects/DeepCreate
```

### Main historical outline
```text
D:/Projects/DeepCreate\.local\delegated\20260913-execution-grid
```

First of all, if there are:

```text
READ_FIRST_AFTER_COMPACTION.md
INCIDENT_20260914.md
PROGRESS_EVIDENCE_20260914.md
```

### Last known unsuccessful integration candidate
Historically:
```text
...\20260913-execution-grid\repair16-retry1\integration
```

Don't automatically consider it the best. Find all candidates/workspaces and compare provenance.

### Check
- `git status`;
- branch/worktree map;
- latest commits;
- uncommitted diffs;
- running processes;
- exact command lines;
- status JSON;
- timestamps;
- artifact hashes;
- storage/free-space;
- current Hermes/OmniRoute routes, if needed;
- presence of incomplete external worker processes;
- which candidates contain which changes;
- original vs baseline;
- frozen technical docs;
- current Verify artifacts.

Take a **Reality Snapshot** before any changes.

---

# 24. DO NOT TRUST THE NAME `candidate`, `integration`, `accepted`

Check contents.

For each significant candidate:

```text
base/source provenance
changed files
diff
artifact reports
tests actually run
exit codes
review lineage
open blockers
technical-doc integrity
generated junk
runtime requirements
```

You cannot select newest folder only by time.

---

#25. BEFORE MASS RENEWAL - CHECK NEW CONTROL PLANE

The old controller cannot be patched endlessly.

He must go through real scenarios:

### Scenario A — cross-component blocker
Worker A has detected defect B, but lease does not allow changing B.

Expectation:
- blocker turns into BlockerPacket;
- the graph creates owner/integration WorkUnit;
- A does not loop;
- after B, dependency A is closed;
- old review is not repeated without delta.

### Scenario B — stale review
Reviewer checks the old hash.

Expectation:
- verdict is marked stale;
- does not block new artifact;
- the new check refers to the exact artifact ID/hash.

### Scenario C — repeated rejection, no delta
Expectation:
- REPLAN;
- not the fourth identical retry.

### Scenario D - worker crash
Expectation:
- saved artifacts remain;
- lease returns;
- WorkUnit safely continues another attempt;
- side effects are not duplicated.

### Scenario E - provider failure
Expectation:
- the route is classified separately from code failure;
- fallback/requeue by policy;
- the model result is not considered a rejected product.

### Scenario F - compaction/restart
Expectation:
- ObjectiveContract/WorkGraph/Blockers/Artifacts are restored;
- completed work is not restarted;
- the unknown does not turn into a fact.

### Scenario G - two workers, logical conflict
Expectation:
- isolated changes are saved;
- integration detects semantic conflict;
- verifier decides or marks human_required.

### Scenario H — final user-visible gate
Expectation:
- tests + build are not sufficient;
- the real application/flow is launched and the required behavior is checked.

---

#26. AFTER PROOF CONTROL PLANE - DON'T STOP

One successful self-development smoke is a **gate**, not an end goal.

After it, the system should:

1. build a complete coverage graph from technical contracts;
2. find all open obligations;
3. decompose them into dynamic WorkUnits;
4. launch independent useful work en masse;
5. continuously review/repair/integrate;
6. automatically create the next job as resources become available;
7. do not wait for the completion of a huge batch wave if the finished parts can already be checked;
8. Continue until complete candidate acceptance.

---

#27. HOW TO DETERMINE “APPLICATION IS READY”

Not “MVP is ready.” The user does not directly want to artificially delay a normal application under the guise of phases.

Before the final statement you need coverage:

```text
каждое обязательное требование
→ implementation artifact
→ tests/evidence
→ integrated candidate
→ runtime/user-visible proof
```

Critical unknown/blocker should not be hidden.

Minimum:

- full project build;
- relevant full test suites;
- negative tests;
- fault/recovery tests;
- architecture/contract validation;
- data persistence;
- restart/resume;
- provider errors;
- user-visible Desktop flow;
- self-development flow;
- integration conflict path;
- rollback/recovery where applicable;
- no accidental mutation of frozen/original sources;
- final independent review;
- exact accepted artifact hash/provenance.

---

#28.OUTCOME GATES

## GATE 1 - Reality
The current filesystem/runtime/repo state is proven by artifacts.

## GATE 2 - Orchestration
A real cross-component blocker is automatically redistributed and closed without manual micromanagement.

## GATE 3 - Compaction continuity
After forced/real compaction, work continues from the canonical state without losing the goal and without duplicate work.

## GATE 4 - Self-development
Deep Create independently performs the actual modification of its own project:

```text
goal
→ plan/work graph
→ workers
→ patch
→ tests
→ verifier
→ integration
→ accepted artifact
```

## GATE 5 - Full candidate
All mandatory requirements have evidence, full Verify is green, there are no critical unresolved obligations.

## GATE 6 - User-visible application
The assembled application actually runs and key user scripts run in the runtime that actually serves the user.

## GATE 7 - Promotion
The accepted candidate is transferred/promoted securely, preserving rollback/provenance and user data.

---

# 29. ANTI-PATTERNS - PROHIBITED

Do not repeat:

- “the process is alive → everything is fine”;
- “HTTP 200 → agent works useful”;
- “13 agents → large-scale development”;
- “fixed lanes → self-organization”;
- “reviewer said ACCEPTED → fact”;
- “unit tests are green → custom flow is working”;
- “works for a long time → complex useful task”;
- repeat the same prompt without changing the reasons;
- restart the controller without root-cause;
- give a blocker to the same worker without a new capability;
- one integrator to pass off as an organization;
- rewrite already solved modules without regression evidence;
- assess readiness by lines of code;
- lose negative verdicts with a new report;
- overwrite evidence of a previous attempt;
- consider summary after compaction to be a complete context;
- ask the user to explain the goals already written down again;
- agree with the user without verification;
- argue with a user without an artifact;
- make a local fix without checking whether it solves the Outcome Gate.

---

# 30. RULES FOR COMMUNICATION WITH USERS

Language: predominantly Russian.

The user does not want a stream of micro-statuses every minute.

Report:

- confirmed meaningful milestone;
- a real blocker that requires authority;
- identified error;
- final accepted result;
- upon direct request - actual cut.

Don't force the user to be the dispatcher.

If the user is annoyed/swears at the agent:

- perceive this as a signal of a possible system error;
- do not make excuses;
- conduct a Z.A.E.B.A.L. audit;
- find wrong belief;
- check execution evidence;
- do not continue the same path out of inertia.

---

#31. Z.A.E.B.A.L. — RULE FOR THE NEXT AI

If you fail again:

```text
STOP
→ reconstruct contract
→ find earliest divergence
→ FACT/HYPOTHESIS inventory
→ competing causes
→ discriminating check
→ wrong belief
→ outcome gate
→ only then mutation
```

There are two pitfalls to remember in particular:

### Sycophancy
Disagree simply because the user is angry.

### Hallucinated correctness
Do not defend your own code with beautiful explanations without run/log/artifact.

---

# 32. WHAT WAS WRONG IN THE PREVIOUS Z.A.E.B.A.L. AUDIT

He found the real symptom:

> static batch executor was accepted as an independent organization.

But then I narrowed the solution again:

> “fix the controller and some scripts.”

The user rejected this.

Why:

- the entire history from the Hermes foundation to Deep Create was not restored;
- the original North Star was not secured;
- the modes memory/context philosophy, skepticism, dynamic configuration were not enabled;
- a full-fledged handoff was not formed;
- `controller` again turned out to be the center of the task instead of the means;
- the user's setting was not restored: **completely transfer mass development to an autonomous AI organization and connect a strong model primarily for architectural control/final**.

---

# 33. CURRENT STATUS AS AT END OF PREVIOUS CHAT

Historically confirmed:

- there were useful results of many working lines;
- they should not be thrown away;
- the last cross-module integration candidate was **not accepted**;
- the previous controller/review loop was stopped during the Z.A.E.B.A.L. audit;
- original Deep Create should not have been replaced by this candidate;
- the user rejected the direction “fix the controller a little more” as insufficient.

**The current live state at the time of the new launch is UNKNOWN, the machine has not yet been checked.**

Don't say:

> “everything is stopped now”  
or  
> "it's still working now"

to process/filesystem check.

---

#34. FIRST REPORT OF THE NEXT AI AFTER REALITY RECONSTRUCTION

Brief but conclusive:

```text
1. CURRENT ORIGINAL
   commit / dirty state / hashes

2. LIVE PROCESSES
   exact processes and commands

3. CANDIDATES
   candidate IDs/paths + provenance

4. ACCEPTED EVIDENCE
   what is truly verified

5. OPEN LIABILITIES
   blockers / unknowns / stale reviews

6. ORCHESTRATION STATUS
   whether the old control plane satisfies the required dynamic behaviors

7. NEXT EXECUTING ACTION
   not a plan-only sentence, but what the autonomous organization is now doing

8. OUTCOME GATE
   what observable artifact will prove this stage complete
```

---

# 35. PRIORITY OF FURTHER WORK

If the living state does not show anything fundamentally new:

1. save all existing candidates/artifacts;
2. do not revive the old retry-loop;
3. build/correct a minimal **dynamic control plane** around an existing work;
4. test it with the scripts from section 25;
5. run one real self-development work end-to-end;
6. immediately after gate, launch a wide dynamic completion of the project;
7. use as many useful external models as possible, but do not create correlated noise;
8. integrate only evidence-backed results;
9. reach full candidate;
10. conduct user-visible acceptance;
11. Only then promotion in original.

---

# 36. DO NOT CONFUSE “AUTONOMY” AND “UNCONTROL”

The user wants enormous autonomy.

This does not mean:

- spend money quietly;
- do external legal/financial actions;
- delete original;
- accept conflict at random;
- replace user authority.

Correct formula:

> **maximally autonomous for reversible local actions + fail-closed only where authority or reliable ambiguity resolution is actually required.**

---

# 37. IF YOU NEED TO CHOOSE BETWEEN SPEED AND “BEAUTIFUL ORGANIZATION”

Choose to complete the goal.

Don't build an orchestrator for the sake of an orchestrator.

If some of the work can be safely done by a direct worker, do it.

If there is a lot of independent work, scale it up.

If one blocker holds twenty tasks, first unblock the blocker.

If the organization itself has become a bottleneck, simplify/rewrite the control plane.

Architecture serves as an outcome, and not vice versa.

---

#38. THE MAIN CRITERION FOR THE QUALITY OF THE NEXT AI

Not the number of new architectural words.

And here it is:

> After the next long period of work the user does not have to speak again  
> “I already explained that the system should sort itself out.”

That is, the goal, context, blockers, evidence and orchestration must survive:

- long work;
- new models;
- context compaction;
- provider failures;
- restart;
- change of performers;
- criticism;
- integration.

---

#39. SHORT NORTH-STAR PASSPORT

If the context is compressed again, do not lose the minimum:

```text
PROJECT:
Hermes foundation + Deep Create.

USER INTENT:
Построить не swarm-демо, а полноценную автономную систему длительной работы,
способную сама строить нужную конфигурацию работы под запрос.

CURRENT MAIN EXECUTION GOAL:
Довести Deep Create до полноценного принятого приложения через автономную
ИИ-организацию; основная сильная модель — архитектурный интегратор, не
ручной диспетчер.

TRUTH POLICY:
Artifacts/execution > reports/status/explanations.
FACT != HYPOTHESIS.
Do not sycophantically agree.

ORCHESTRATION:
Dynamic WorkGraph, capability leases, blocker routing, useful-delta loop
detection, dynamic parallelism, independent verification, staged integration.

CONTEXT:
Canonical objective/evidence/work state survives compaction.
Summary is cache, not authority.

CODE SAFETY:
Original preserved.
Workers isolated.
Semantic integration + Verify before promotion.

DEPTH MODES:
Light / Medium / High / Very High / Ultra.
Modes are ceilings, not target agent counts.

ANTI-PROXY:
PID/HTTP200/worker-count/lines/tests alone never prove outcome.

LAST INCIDENT:
Static lanes could not close cross-module blockers, repeated review without
delta, organization collapsed to one integrator. Previous narrow controller
audit was rejected by user.

NEXT:
Reality reconstruction → dynamic control-plane proof → real self-development
gate → broad autonomous completion → full candidate → user-visible acceptance.
```

---

#40. FINAL INSTRUCTIONS

Don't respond to the user with another general plan and don't stop after one local fix if the environment allows you to actually work.

First restore reality.

Then **act autonomously for this purpose**, preserving evidence and original.

When you find a new unknown, don’t automatically say “impossible”:
- determine what exactly is unknown;
- find a way to check;
- create the necessary work;
- obtain evidence;
- continue.

When a user solution is really needed, show:
- specific ambiguity;
- why automatic solutions are dangerous;
- what options exist;
- what are the consequences?

In all other cases, the organization must work itself.

**End handoff.**
