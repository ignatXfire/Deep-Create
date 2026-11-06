# Delegation of development and delay of requests

Current expansion of appointments from September 12: [Qwen 0902 and authorized use of Antigravity](MODEL_ASSIGNMENTS_20260912.md). The self-recording prohibition described below applies to the old text assistant. A separate Hermes package, on the new instructions of the owner, works with tools in a dedicated candidate copy; acceptance into the main project remains with the main agent.

Updated: 2026-09-11. This is the working order of development, not the implementation of model selection within the Deep Create application.

## Who does what

| Performer | Limited order | What is not left to him for independent decision |
|---|---|---|
| `dash/kimi-k3` | Proposal of a small function or tests based on exact sources and requirements | Changes in accepted requirements, rights, spending restrictions; independent entry into a project or execution of commands |
| `dash/deepseek-v4-pro-0813` | Checking a specific proposal and analyzing already collected measurements | Approval of the readiness of the entire system using several files; general service settings without confirmed reason |
| Principal Agent | Choosing an Assignment Boundary, Reviewing Proposals, Combining Code, Independent Tests, and Accepting Changes | Passing off the opinion of an external model as a proven fact |

Qwen Max previously successfully received one artificial tool call without executing it. This is not a complete tool test. There is no completed development test here for Qwen A95B. They are not automatically assigned as the "best" for the role by title or general description of the model. The failure of one other model with the prefix `dash/` does not prove the inoperability of the entire route.

Only the specified lines of the required files are transferred. The history of all correspondence, research uploads, keys, user profile and external sources are not included in the instructions. Currently, no more than two external orders are launched simultaneously; internal Codex subagents have not been launched.

## What has already been changed in requests

Development assistant `tools/external-model.mjs` uses verified profiles from `external-request.mjs`:

- Kimi: for small orders explicit `reasoning_effort=low`; disabling reflection is not sent.
- DeepSeek: For limited viewing, you can explicitly select `direct`, corresponding to `enable_thinking=false` of this route. This is not a general prohibition against thinking in complex problems.
- The old order file field `max_tokens` is converted on the wire to `max_completion_tokens`: the limit includes both reflection and response. Limiting only the visible part was not enough.
- Unsupported combinations are rejected before the request. There are four automatic checks on profiles.

The assistant does not repeat the request on its own, does not execute the suggested tools, and does not apply the suggested code. The beginning and the total are saved in `.local/external/<номер>/report.json`, the response text is stored separately. The staging status `running` is not evidence of a hang: live staging recordings have not yet been implemented. Timing distinguishes between headings, the first event, the beginning of reflection, and the beginning of a visible response. The text of the reflection is not saved.

This is a separate local development tool that uses an existing connection on behalf of the owner. It does not give the application keys or replace future C-03/C-09, Total Flow Accounting, or Execution Boundary.

## What the measurements showed

| Experience | Full Time | Essential Observation |
|---|---:|---|
| Kimi, proposal of five tests, previous profile | 321,938 s | 12,893 reflection tokens and 1,171 response tokens; first visible reply in 299.107 s |
| Same order, `low` and total limit 4096 | 32.985s | 175 reflection tokens and 1,069 response tokens |
| DeepSeek, previous order and limit 4096 | 71,586 s | The entire limit was spent on thinking, there was no meaningful answer |
| DeepSeek, short order without thinking | 18,919 s | Response received, 758 tokens |
| Kimi, short answer after starting the service | 17.805 s | Normal connection selection |
| Kimi, the same short request with connection pinned | 17,384 s | One pair does not prove the usefulness of pinning |

In the Kimi experiment, two parameters were changed simultaneously. The usefulness of the combination for a specific order has been verified, but not the individual contribution of each parameter and not the equality of quality in terms of the number of tokens. DeepSeek also has a reduced input: it is not a pure comparison of the speed of the models. The original trial numbers and restrictions are stored in [DELEGATED_LATENCY_INPUT.md](DELEGATED_LATENCY_INPUT.md) and local reports.

New assignments: Kimi's initial plan completed in 77.444 s (`60145776-f463-441d-8e90-2856a53fde7a`), analysis of DeepSeek measurements in 26.246 s (`9369a4e9-7a2d-40ae-8aa7-e1d73b141e28`). The code is not automatically accepted: the verification procedure is described in [RESULT_TRANSITIONS.md](RESULT_TRANSITIONS.md).

## What about OmniRoute

Acceptance of refusal before parsing the reason: Kimi proposal completed in 76.083 s (`c7d66233-fe38-428b-92ce-42e0dcdf4533`), DeepSeek review in 33.765 s (`8a3f6130-877d-4b52-9aff-c6bfb26b3ccb`). External models received only the strings of the required modules and the associated contract. The proposal is corrected and joined by the main agent; The reviewer's comments were not confirmed by the code and reproducible tests. Detailed analysis - [OWNER_REJECTION.md](OWNER_REJECTION.md). The new OmniRoute general settings and internal subagents were not used.

Return total by owner: Kimi prepared a limited offer for 81,940 s (`6648b31a-3b55-4783-adde-911408cce105`, 18,036 entry characters); DeepSeek scanned the integrated version in 11.642 s (`24493c68-c2ee-4ac9-84a5-0ce3f8a258d4`, 16,018 input characters), response `NO_FINDINGS`. Omissions in Kimi's original proposal have been corrected by the main agent; independent negative tests and associated consumer example are described in [OWNER_RETURN.md](OWNER_RETURN.md). The absence of DeepSeek comments does not replace these checks. The router configuration has not changed; internal subagents were not launched.

Pass C-02: Kimi prepared the message parsing code in 86.630 s (`de58d0ef-6cd6-4274-ac5d-67f2189597c7`), DeepSeek scanned the intermediate version in 36.150 s (`8050ca44-773b-45d9-b526-089a0ada1493`). Some of the inspector's comments were not confirmed; the match between the command number and the object number has been reproduced and corrected. The final boundary and checks are [PROTOCOL_CODEC.md](PROTOCOL_CODEC.md). New router settings and internal subagents were not used.

The following development pass is performed without a new router configuration: Kimi prepared the flow binding in 83.161 s (`603a1a78-c8d9-4d13-9be9-4f099bf0d998`); DeepSeek dissected the impact of the commitment record in 19.516 s (`883d8352-9dcc-4bdf-abd6-be33893866e5`) and scanned the intermediate code in 15.297 s (`9bfb095e-004f-4080-ac16-2075839de9f8`). Changes are reviewed by the primary agent and not automatically applied. The resulting differences from the proposals and the verified boundaries are listed in [USAGE_BINDINGS.md](USAGE_BINDINGS.md).

After restarting the computer there was no listener; An existing service is started without changing its configuration. The cold initialization took about a minute and a half. The last scheduler job code and old log do not establish why the service was missing.

`maxWaitMs=900000` refers to the queue wait limit, not the required delay of each request. Errors from other vendors have no proven connection to the current route `dash`. A direct request to a provider without a key that ends with HTTP 400 is not a comparison between generating through a router and directly.

Therefore, the general settings file, keys and connection tables did not change. It was found that a significant part of the delay of a particular long experience was due to reflection before the visible answer. **It has not been determined** that this explains any remaining delays or that OmniRoute is fully operational.

An external reviewer proposed five iterations of Kimi, but this is on hold: additional requests are not needed at this time to continue development. Even five repetitions of just the new profile will not separate the effect of the two changed parameters. If the delay again interferes with performance, what is needed is a limited comparable experience with the same input and a pre-specified testable difference, rather than another broad reconfiguration.
