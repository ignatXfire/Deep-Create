#Research evidence and limits

## OpenAI: Harness engineering (2026-02-11)

Public case study: agent-first internal product, approximately one million LOC after about five months, ~1,500 merged PRs, initially three engineers. App logic, tests, CI, docs, observability and tooling were agent-generated. Important mechanisms: worktree-isolated app instances, logs/metrics/traces exposed to agents, architecture linters, short AGENTS.md as navigation, agent-to-agent review, recurring cleanup agents.

**What it proves:** large real codebases can be predominantly/entirely agent-generated and used by real users.

**What it doesn't prove:** that 700k release-verified novel LOC can be reliably built in one day.

## OpenAI Symphony orchestration (2026)

Issue tracker/control plane, dependency DAG, isolated workspace per task, reconciliation loop, bounded concurrency. Open-source spec documents a default concurrency setting around ten agents; this is a safe operating default/spec, not a universal maximum.

**Lesson:** orchestration quality and workspace isolation precede high concurrency.

## DeepSWE

Long-horizon software engineering benchmark using one harness across models. Current public leaderboard illustrates that model performance differs materially even under the same agent framework. Do not use a single benchmark as a universal model ranking; use it to justify adaptive routing and local evals.

##ChainSWE

Benchmark/research on chained multi-bug maintenance shows a strong degradation when tasks require long dependency chains. Lesson: long-horizon autonomy should externalize intermediate state/verification instead of trusting one enormous agent trajectory.

## AgentCoder / MetaGPT / protocol-driven MAS literature

Repeated theme across multi-agent coding research: specialized roles, explicit protocols, tests and structured artifacts reduce cascaded errors compared with naïve free-form agent chains. Research benchmark gains do not automatically transfer to an arbitrary 700k-LOC production project.

## Evidence hierarchy used in this pack

1. Runtime/repository evidence from exact candidate.
2. Primary vendor/product documentation.
3. Reproducible benchmarks under documented harnesses.
4. Peer-reviewed/preprint research.
5. Community reports.
6. Our engineering estimates.

Capacity numbers in this pack are explicitly marked as estimates. They should be replaced with your telemetry after the first calibration wave.
