# Model routing for a given set

> This is a recommended distribution of roles, and not a statement that the model is “always better” than another. Real routing should be updated according to your telemetry: first-pass acceptance, defects, wall time, retry rate.

## GPT-6.1 Sol High

Use for:

*architecture constitution;
* cross-domain design;
* complex root-cause investigations;
* review changes with a large blast radius;
* resolving conflicts between specifications;
* final release reasoning.

The reason is not that other models “can’t code,” but that the Plus makes the Sol scarce. One strong Sol run should unlock dozens of cheap parallel runs.

## GPT-6.1 Sol XHigh

Do not put XHigh on mass writing of CRUD. Use at 3–8 most expensive decision points during the project: kernel architecture, security, migrations, complex integration failure, final adversarial audit. The number 3–8 is a planning recommendation, not a product quota.

## qwen3.8-max-0902

Strong roles:

* repo cartography;
* workgraph decomposition;
* contract/spec synthesis;
* tool-heavy investigation;
* architecture/read-only review;
* UI/image-assisted tasks, if the workflow uses it.

Alibaba documentation for snapshot specifies 1M context and large max output. But exact snapshot in Frankfurt has a significantly lower TPM ceiling than DeepSeek/Kimi from this set, so you shouldn’t make Qwen the only mass writer backend.

## deepseek-v4-pro-0813

Main high-volume implementation pool:

* independent bounded features;
* unit/property tests;
*refactors;
* adapters/connectors;
* CI fixes;
* alternate-family reviewer for Qwen/Kimi-authored tasks.

Large context/output and high TPM ceiling are convenient for many parallel task lanes.

## kimi-k3

Roles:

* long-horizon multi-file implementation;
* whole-repository reasoning;
* integration diagnosis;
* large-context reviewer;
* UI/vision-oriented investigation;
* tasks with a large tool catalog, where dynamic tool loading reduces context pollution.

## Cross-model pairing

Example matrix:

| Author | Primary reviewer | Escalation |
|---|---|---|
| DeepSeek | Kimi | Sol High |
| Kimi | DeepSeek | Sol High |
| Qwen | DeepSeek/Kimi | Sol High |
| Sol | Kimi + DeepSeek independently | Sol XHigh final only if critical |

Diversity is useful as error independence, but **is not proof in itself**: the reviewer is still required to check exact files/symbols/SHA/tests.

## Adaptive router

For each `(model, task_class)` store EWMA:

* median completion latency;
* first-pass reviewer acceptance;
* post-merge defect rate;
* tokens/accepted change;
* compile/test retries;
* hallucinated-path incidence;
*revert rate.

Router does not choose the “favorite model”, but maximizes the expected `accepted_value / bottleneck_minute`.
