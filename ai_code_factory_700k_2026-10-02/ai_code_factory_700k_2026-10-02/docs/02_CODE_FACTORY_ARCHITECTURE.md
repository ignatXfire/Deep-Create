# AI Code Factory Architecture

## Control plane

Control plane doesn't write core code. It stores mission, dependency DAG, authority ownership, queues, budgets and gate state.

Package states:

`DISCOVERED -> SPEC_READY -> ADMITTED -> IMPLEMENTING -> LOCAL_GREEN -> REVIEWING -> REVIEW_GREEN -> INTEGRATING -> CANDIDATE_GREEN -> E2E_GREEN -> RELEASED`

Any state transition must have an evidence artifact.

## Six planes

### 1. Intent plane
PRD, architecture constitution, non-goals, acceptance criteria, threat model.

### 2. Decomposition plane
Graph planner turns the goal into a DAG of small work packages. The problem must be large enough to justify the agent context, but small enough that diff and proof remain observable.

### 3. Execution plane
Each writer gets a separate git worktree/container, separate ports, temp DB, logs and cache namespace. No common mutable filesystem other than explicitly declared coordination stores.

### 4. Verification plane
Test authors, adversarial reviewers, architecture/static checks, dependency checks, provenance validator. Reviewer should not accept text `PASS` without checking exact commit/diff/artifacts.

### 5. Integration plane
One integration authority per candidate branch. Writers do not push directly to release candidates. Merge train rebases/cherry picks only proof-backed commits.

### 6. Evidence plane
Append-only receipts: base/head SHA, model/run ID, commands, exit codes, tests discovered/passed/skipped, touched authority keys, artifacts + SHA256, review result, integration result.

## Main principle

**Enforce boundaries centrally; allow autonomy locally.**

An agent can choose an implementation within its work package. But dependency rules, schemas, public contracts, security invariants, observability and release gates are mechanical.

## Dynamic workgraph

Do not fix the organization as “always 20 frontend + 20 backend”. The control plane should create new prerequisite tasks when a blocker is detected, split nodes that are too large, merge duplicates, and reduce concurrency when saturation downstream queues.

## Hierarchy

*Mission controller
* Architecture council (3–6 domains)
* Workgraph planners/spec miners
* Implementation pools
*Test factory
* Independent review pool
* Integrators
* Reliability/security/E2E pool
* Release judge

In a small project, several roles can be logically different agent runs on the same model. In extreme factory these are separate process pools.
