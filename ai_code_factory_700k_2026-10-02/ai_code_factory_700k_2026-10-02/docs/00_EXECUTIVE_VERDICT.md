# Executive verdict: is it possible to make 700k+ rows in a day?

## One sentence answer

**Generating** 700k rows is easy; **assembling them into a fully working, integrated, proven top-level product in a day** is possible only for an extremely favorable project structure and does not have publicly confirmed practice as a reliable general mode.

## Why the number of lines is deceiving

The same 700k LOC can mean:

1. generated DTO/API clients/schemas/tests/docs - high speed, low semantic density;
2. independent CRUD/service modules - moderately high parallelism;
3. core of the compiler, database, distributed runtime, security or complex UI state - high coupling, low secure parallelism;
4. migration of an existing system - speed is limited by integration, regression and hidden contracts.

Therefore, the factory must maintain five counters:

*`generated_loc`
* `compiling_loc`
* `module_verified_loc`
* `integrated_loc`
* `release_verified_loc`

The core business metric is the latter, not the former.

## Real operating ranges for one day

These are **engineering estimates**, and not the results of a specific benchmark:

| Level | Aggressive range in 24 hours | What you need |
|---|---:|---|
| Raw generated | 0.7M–3M+ LOC | tens/hundreds of agent runs |
| Compiling | 0.25M–0.8M LOC | good contracts, modularity, generated code |
| Module-verified | 0.12M–0.4M LOC | tests for each shard, CI farm |
| Integrated + meaningful E2E | ~40k–150k novel LOC | realistic for a complex new product with a strong harness |
| Extreme integrated | ~700k LOC | ~96 writer lanes and almost ideal conditions; perceived as a capacity stress target, not a promise |

## What public practice shows

OpenAI described a real agent-first project of about a million lines, where all the code, tests, CI, docs and tooling were written by Codex. This was a real application in use, but scale was achieved in **months** rather than a day. Conclusion: the millionth agent-generated codebase is real; a million checked lines/day has not been publicly proven.

## Solution for your resource set

Instead of “100 identical coders”, a hierarchy is built:

* GPT-6.1 Sol High - chief architect / cross-domain judge / hard integration.
* GPT-6.1 Sol XHigh - only the most expensive solutions and final adversarial release review.
* Qwen 3.8 Max 0902 - planning, repo mapping, contract decomposition, tool-heavy investigation.
* DeepSeek V4 Pro 0813 - massive implementation/test/refactor lanes.
* Kimi K3 - long-context integration, whole-repo reasoning, hard multi-file tasks, multimodal/UI analysis.
* independent reviewer lanes, if possible, of a different model family than the author.

For Plus, do not plan mass Sol swarm: local Work/Codex usage is limited by quotas and is not equivalent to the unlimited API.
