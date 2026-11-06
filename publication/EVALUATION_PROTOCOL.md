# How to test whether the loop earns its cost

**Status: proposed experimental protocol, not benchmark results.** The published application is not yet a validated runner for this entire experiment.

## Hypothesis

Explicit feedback, versioned dependencies, and selective reuse can reduce repeated or invalid work at a given independently evaluated quality level. The hypothesis can fail: bookkeeping, verification, context construction, and coordination may exceed the avoided work.

## Baselines

Use the same task set, model/provider versions, tools, external inputs, output requirements, and total compute or monetary budget. Include:

1. A strong single-pass or best-of-N strategy within the same total budget.
2. Restart after rejection, retaining the original request but not intermediate work.
3. Iteration using an appended transcript and explicit feedback.
4. A structured cycle with evidence, current dependency basis, and selective repair.

The third baseline matters: a weak “forget everything” comparison would not isolate the value of the structured loop. Run ablations removing selective reuse, dependency invalidation, or context reconstruction. Do not silently give one strategy a stronger verifier, more attempts, or extra hidden context.

## Task families

Use multi-stage tasks with independently checkable artifacts. Include clean runs and predetermined perturbations: one incorrect branch, an unrelated edit, a changed shared dependency, a revised criterion, a lost response, and an ambiguous verification outcome. Add short tasks where coordination overhead is likely to dominate.

Keep fault timing and task variants matched across strategies. Distinguish ordinary negative feedback from adversarial data. Do not represent process termination as power-loss testing or a simulated provider as a real paid service.

## Primary measurements

- Independently accepted complete tasks within the budget, with incorrect acceptance counted separately.
- Total input/output tokens and monetary cost, including planning, retrieval, critique, verification, repair, and failed or unknown attempts.
- End-to-end wall-clock latency and critical-path delay; report concurrency and queue time separately.
- Model calls and completed sub-artifacts recomputed despite remaining applicable.
- Incorrect reuse of stale artifacts, dropped obligations, repeated known failures, and duplicated external effects.
- Human interventions and unresolved outcomes.

Do not compare tokens as though they were the same unit as money or latency. Do not condition the main result only on successful runs. Report the cost of failures, exhausted budgets, and unresolved outcomes.

## Evaluation and controls

Freeze acceptance criteria before running. Evaluate actual artifacts with an independent deterministic checker where possible, otherwise blinded reviewers with a documented rubric. Check the checker against deliberately wrong, stale, incomplete, and wrong-unit artifacts. A verifier that accepts these can make a broken strategy look efficient.

Use paired task variants, multiple runs, documented randomization, and provider seeds where supported. Predefine the stopping rule. Publish per-task outcomes and uncertainty intervals appropriate to the sample rather than only an average or the best run. Separate exploratory tuning from held-out evaluation.

## Minimum result bundle

Publish the exact source revision, task data and provenance, prompts, model identifiers, route configuration without credentials, tool versions, budgets, fault schedule, commands, exit codes, and sanitized event/artifact traces. Include failures and exclusions. Connect every headline number to a machine-readable row and the underlying artifact.

A defensible result would be narrower than “loops are better”: for example, a measured reduction in repeated sub-artifact work on a specified perturbation family, at matched independently assessed quality and total cost. Only report the actual effect and coverage obtained.
