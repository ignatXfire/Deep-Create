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










Ready. I've compiled the research and work package into one ZIP:

**AI_Code_Factory_700k_2026-10-02.zip[Download AI Code Factory 700k - full ZIP](sandbox:/mnt/data/AI_Code_Factory_700k_2026-10-02.zip)**  
:chatgpt-content-reference{index="9"}[Open README separately](sandbox:/mnt/data/ai_code_factory_700k_2026-10-02/README.md)

Archive checked: **44 files**, about **7,400 words of engineering documentation**, JSON files are valid, Python-skeleton compiles and demo DAG runs, `unzip -t` - no errors. SHA-256 ZIP: `809c54204ded89035e145d7f0cecd4bc874da50bca8de195014b18918f7af393`.

### Main result of the study

**It’s possible to generate 700,000 rows per day with your resources. 700,000 lines of completely new, integrated, E2E-tested complex production code is a completely different task.** OpenAI's public experience confirms that the agent-first team was indeed able to build a usable product for approximately **1 million LOC**, with agents writing application code, tests, CI, docs, observability and tooling. But this million accumulated in about five months; about 1,500 PRs were captured by a small team. Their key enablers were isolated worktrees, mechanical architectural constraints, observability for agents and agent-to-agent review. [OpenAI](https://openai.com/index/harness-engineering/)

For one extremely aggressive day I got these **engineering** rather than marketing ranges:

- **0.7–3+ million raw LOC/day** is a completely achievable generation volume; **250k–800k compiling LOC** are possible with high modularity and a large share of generated/repetitive code; **120k–400k module-verified LOC** — with powerful CI; for a truly new complex application, I would target around **40k–150k integrated + meaningful-E2E LOC** as an aggressive operating range. To mathematically break **~700k integrated LOC**, the model from ZIP requires order `96 writers × 14 productive hours × 850 useful LOC/h × 70% acceptance × 90% integration yield = 719,712 LOC`. This is only possible with almost perfect decomposition, a huge share of independent code and a very powerful verification/integration factory.

This is why my working architecture is not “run 500 coders.” For the first calibration hour - **8–16 writers**. If first-pass acceptance >70%, conflicts <5% and downstream is not full, raise to **32–64 writers**. For a very large one-day launch - approximately **40–80 simultaneously mutating lanes** and **100–220 logical agents/runs per day**. Extreme 700k topology - about **96 writers + 72–96 test/review lanes + 16–24 integration + 12–20 E2E/reliability + architects/scouts**, that is, approximately **200–260 logical functions**. This is significantly more reasonable than 500 agents rewriting the same contracts at the same time.

OpenAI Symphony confirms the principle itself: tasks are built as a dependency DAG, blocked tasks are not launched, and each issue receives a separate workspace; in the published specification, the base `max_concurrent_agents` is generally equal to 10 and changes dynamically. That is, even with modern agent infrastructure, concurrency is considered a controlled resource, and not a “more is better” rule. [OpenAI](https://openai.com/index/open-source-codex-orchestration-symphony/)

### How I distributed exactly your models

**GPT-6.1 Sol High** I put chief architect / difficult root-cause / cross-domain integration / release judge. **XHigh** - only for several of the most expensive architectural and final solutions. The reason is especially important with Plus: OpenAI currently estimates GPT-6.1 Sol to be around **15–160 local messages per 5 hours** for Plus, with possible weekly limits; API is paid separately. Therefore, it is impossible to build 50 Sol-workers only from Plus - Sol must make solutions with maximum multiplier effect. [OpenAI Help Center](https://help.openai.com/en/articles/20001516-managing-usage-with-gpt-6-astra-in-work-and-codex)

I transferred the bulk code to DashScope. `deepseek-v4-pro-0813` - main writer/test/refactor pool; for Frankfurt the documentation shows up to **1.2M TPM / 15k RPM**, 1M context and up to 393,216 output tokens. `kimi-k3` - long-context/multi-file/integration/reviewer pool; it has ~1,048,576 context and Frankfurt **1.2M TPM / 15k RPM**. `qwen3.8-max-0902` - architecture decomposition, repo mapping, planning, contract work and tool-heavy investigation; snapshot has 1M context, but it is for snapshot 0902 that the documentation shows Frankfurt **150k TPM / 30k RPM**, so I deliberately did not make it the main mass writer pool. [AlibabaCloud](https://www.alibabacloud.com/help/en/model-studio/deepseek-v4-pro)

At the same time, the benchmark data also says not to do everything with one model: on the current DeepSWE with a single `mini-swe-agent` harness Kimi K3, DeepSeek V4 Pro and Qwen3.8 Max show noticeably different results. This is not a universal rating of models, but a good argument for **adaptive routing by type of task + own telemetry**, rather than rigidly assigning one model as “the best forever.” [DeepSWE](https://deepswe.datacurve.ai/)

### What exactly did I take from your development Deep Create

Your session yesterday was very useful as a practical case study. The correct principle has already emerged there: the old readiness percentages are no longer considered current acceptance evidence; real RED/GREEN counts and hashes were taken from the gate receipt, independent review remained a separate gate, and for the existing WIP **one writer** was specially kept so as not to get a double mutation. session-20261001(4)

More importantly, a separate model reviewer once produced a plausible PASS, but the source check found **11 non-existent paths, 3 missing symbols and an incorrect git delta**, so the PASS was rejected. This is an almost perfect example of why “let’s put another strong model to test the first” is not enough. Therefore, in the new architecture, the reviewer is required to prove exact SHA/files/symbols/tests, and his own receipt is also machine verified. session-20261001(4)

In ZIP I have already turned this into a concrete system: `workgraph → authority locks → isolated worktrees → implementation → evidence receipt → independent review → serialized integration → candidate gates → E2E → release`. There are **22 detailed MD chapters**, `models.yaml`, `roles.yaml`, JSON Schema tasks and evidence receipt, ready-made prompts for architect/implementer/reviewer/integrator/test/release judge, PowerShell worktree launcher, receipt verifier and **executable `orchestrator_skeleton.py`**.

The most important change regarding a simple “swarm of agents”: **any number of read-only scouts, planners and critics can exist at the same time, but for one semantic authority key there is only one active mutator**. This is what allows you to try to scale to dozens and then to hundreds of writers without the generation speed destroying the integration speed.

**AI_Code_Factory_700k_2026-10-02.zip[Download the entire ready-made package](sandbox:/mnt/data/AI_Code_Factory_700k_2026-10-02.zip)**

If we focus specifically on your task, I would consider **32–64 writer lanes + about the same verification/review capacity** the main combat configuration, and **~96 writers / ~200–260 total agent roles** as an extreme mode, which is turned on only after successful calibration. This is already a scale at which 700k lines cease to be fiction - but it is the architecture of proofs, and not text generation, that decides how many of these lines will actually become a working application by the end of the day.