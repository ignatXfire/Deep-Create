# Numerical model throughput

## Basic formula

For a rough estimate:

`IntegratedLOC = W × H × R × A × I`

where:

* `W` - concurrent writer lanes;
* `H` - effective productive hours on lane per day;
* `R` - useful local LOC/hour before review;
* `A` — first-pass/repair acceptance fraction;
* `I` - integration yield after conflicts/regressions.

This is a capacity model, not a promise. LOC is highly dependent on language, generated fraction and complexity.

## Scripts

### Conservative

`24 × 10 × 350 × 0.55 × 0.80 = 36,960 integrated LOC`

Suitable for cohesive complex project with new harness.

### Aggressive

`48 × 12 × 600 × 0.65 × 0.85 = 190,944 integrated LOC`

You need stable boundaries, fast CI, mature test harness and good routing.

### Extreme modular/codegen-heavy

`96 × 14 × 850 × 0.70 × 0.90 = 719,712 integrated LOC`

This is where the 700k goal comes from mathematically. But **850 useful LOC/hour/writer for 14 effective hours** is realistic only with a high proportion of repetitive/generated code, very good contracts and cheap integration. For architecturally dense code this cannot be considered.

## Separately count semantic density

Add `semantic_weight`:

* generated/client/schema boilerplate: 0.1–0.3;
* repetitive application code: 0.4–0.7;
* normal business/domain code: 1.0;
* difficult concurrency/compiler/security core: 2–5.

It is better to optimize `verified_semantic_units/hour` rather than LOC.

## Bottleneck model

Let writers produce 60 accepted packages/hour, reviewers are capable of 40, integrators 25, E2E 12. The actual output is a maximum of about 12 E2E-cleared packages/hour. Adding writers is pointless; you need to scale E2E or reduce package cost.

## Queue signals

Every 10–15 minutes the control plane calculates:

* writer utilization;
* review queue age/p95;
* integration queue age;
* build queue wait;
* first-pass acceptance;
* revert/rollback rate;
* conflict rate;
* E2E cycle time;
* defect escape rate.

## Scaling rules

Example:

* review queue > 1.5× writers' median cycle → freeze +20% of new writers, transfer them to reviewers/tests;
* conflict >5% → reduce writer concurrency for the conflicting authority key;
* acceptance <50% → stop scaling and improve spec/harness;
* acceptance >80%, review queue stable, build wait <20% → increase writers by 25%;
* E2E queues dominate >2 waves → switch agents to E2E sharding/fixtures/observability.

## Provider quota planning

TPM/RPM is another resource bottleneck. The Scheduler should use a token-bucket to limit dispatch by provider/model and reserve at least 10–20% capacity for reviewers/repair so that writers do not eat up the entire quota.
