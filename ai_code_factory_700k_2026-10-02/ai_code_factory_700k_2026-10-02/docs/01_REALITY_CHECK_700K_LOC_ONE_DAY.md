# Reality check: 700k LOC/day

## 1. The physical limit is almost unimportant

LLM can issue thousands of tokens in one run, and dozens of parallel runs - a huge amount of text. The bottleneck comes later: compile, test discovery, flaky tests, merges, schema conflicts, runtime environments, E2E and review validity.

## 2. Law of minimum throughput

Actual factory speed:

`T = min(writer_capacity, review_capacity, build_capacity, integration_capacity, e2e_capacity)`

If writers generate 200k LOC/hour, and the integration gate is able to check changes at 20k LOC/hour, the factory output is around the second number, and the rest turns into queue/WIP and technical debt.

## 3. Why “more agents” stops helping

Concurrency scales almost linearly only while work packages:

* have independent mutable boundaries;
* rely on stable interfaces;
* launch independent environments;
* have a fast feedback loop;
* can be tested without global serialization.

If 50 agents simultaneously change one `schema.ts`, one migration chain or one state machine core, conflict grows faster than useful performance.

## 4. Correct scale unit

Not an “agent”, but an **authority lane**: one independent area that exactly one writer has the right to mutate at a given moment. Read-only scouts/reviewers around it can be scaled much more.

Examples of authority keys:

* `domain:billing/api:v3`
* `db:user-schema:v7`
* `ui:onboarding-flow`
*`runtime:job-lifecycle`
* `infra:deploy/eu-prod`

## 5. Completion Criteria

A work package is considered closed only if:

* diff refers to the anchored base SHA;
* contracts are not broken;
* mandatory tests are actually detected and launched;
* exit codes are saved raw;
* reviewer is independent;
* integration candidate contains exactly the verified commit;
* E2E/negative tests passed on candidate;
* evidence receipt has hashes and machine-readable status.

This transforms speed from "faith in agents' messages" into a measurable system.
