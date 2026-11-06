# AI Code Factory 700k - engineering pack

Study date: **2026-10-02**. The purpose of this package is not to promise the “magic 700,000 lines”, but to provide a development factory architecture that can safely scale many coding agents, measure real throughput, and not confuse generation volume with product readiness.

## Short output

* 700k **raw** code per day for a large pool of agents is technically not a difficult goal.
* 700k **compiled** code is only possible with strong modularity, a large amount of template/generated code and pre-fixed contracts.
* 700k **new, integrated, E2E-tested production code** per day is not confirmed by known practice. This is an extreme scenario, requiring about a hundred independent writer-lanes, a comparable testing/review pool, a fully sharded CI and almost perfect decomposition.
* Practical start for a powerful one-day factory: **40–80 simultaneously mutating agent lanes**, **100–220 logical agents per day**, many read-only scout/reviewer jobs and one controlled integration/release path.
* GPT-6.1 Sol High/XHigh with Plus should be used as a rare architectural and forensic resource. Bulk writing and testing is moving to DashScope.

## What's inside

`docs/00_EXECUTIVE_VERDICT.md` - hard answer at 700k LOC.

`docs/01_REALITY_CHECK_700K_LOC_ONE_DAY.md` - what is physically possible and what quality levels mean.

`docs/02_CODE_FACTORY_ARCHITECTURE.md` - control plane, workgraph, worktrees, evidence plane and integration queue.

`docs/03_MODEL_ROUTING.md` - roles GPT-6.1 Sol, Qwen 3.8 Max 0902, DeepSeek V4 Pro 0813 and Kimi K3.

`docs/04_AGENT_COUNTS_AND_PARALLELISM.md` - specific numbers of agents and scaling formulas.

`docs/05_WORKGRAPH_AND_WORKTREES.md` — task DAG, authority keys, worktree isolation.

`docs/06_CONTRACT_FIRST_DEVELOPMENT.md` - how to cut a project so that the agents actually work in parallel.

`docs/07_TESTING_AND_VERIFICATION.md` — RED/GREEN, receipts, independent review, false-green defense.

`docs/08_INTEGRATION_AND_RELEASE.md` - merge train, candidate branch, E2E and release gates.

`docs/09_THROUGHPUT_MODEL.md` - numerical throughput model with conservative/aggressive/extreme scenarios.

`docs/10_24_HOUR_RUNBOOK.md` - daily schedule by waves.

`docs/11_LESSONS_FROM_YOUR_SESSION.md` - lessons from a busy Deep Create session.

`docs/12_FAILURE_MODES.md` - how exactly swarm falls apart if it is not organized correctly.

`docs/13_RESEARCH_EVIDENCE.md` - sources and what exactly they prove/do not prove.

`docs/14_IMPLEMENTATION_BLUEPRINT.md` is a minimal real orchestrator.

`docs/15_METRICS_AND_CONTROL_PLANE.md` - metrics, backpressure and auto-scaling rules.

`docs/16_PROJECT_EXAMPLES.md` - three types of project and different distribution of agents.

`docs/17_DECISION_RULES.md` - rules for the dispatcher.

`docs/18_SOURCE_BIBLIOGRAPHY.md` - links.

`docs/19_PROMPTING_AND_CONTEXT.md`, `20_CI_BUILD_FARM.md`, `21_SECURITY_AND_TRUST.md` - practical details.

There are also YAML configs, JSON Schemas, roles/prompts, an example of a task receipt and an orchestrator skeleton.

## How to use

1. First, fix the architecture constitution and boundary contracts.
2. Build a workgraph where each task has **one mutable authority** owner.
3. Launch a small calibration wave (8–16 writers), measure acceptance/conflict/build latency.
4. Scale writers only if review/integration queues are not saturated.
5. Not a single worker `DONE` is ready: readiness is only indicated by the evidence-gated integration pipeline.
6. At the end of the day, count not LOC, but `verified_value / wall_clock_hour`, saving LOC as a diagnostic metric.
