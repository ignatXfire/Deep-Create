# External Model Assignments - September 12, 2026

This is the Deep Create development process via Hermes/OmniRoute, not the in-app model selection mechanism.

The owner allowed the use of dash/qwen3.8-max-0902, as well as Claude and Google through the existing Antigravity subscription with the use of its limits. The basis of preference is a comparative document provided by the owner; its numbers have not been verified and do not represent our test results. The actual compatibility of each route is checked separately. The name from the rating table does not replace the exact ID of the available route.

## Next assignments

- dash/qwen3.8-max-0902: Next limited implementation/fix package after checking tool call. Evaluate actual corrections, delivery time and quality; the status of the main performer does not mean proven superiority on our project.
- dash/kimi-k3: analysis of requirements, research and continuation of packages already started by him; alternative artist.
- dash/deepseek-v4-pro-0813: independent review of code and negative scenarios.
- Antigravity: allowed for complex controversial areas and additional review. From the submitted list of candidates antigravity/claude-sonnet-5, antigravity/claude-opus-4-6-thinking, antigravity/gemini-3.1-pro-high. Please check the exact route before making your appointment. Presence in the UI does not prove a successful call. Do not replace them with the names Fable/Opus 5/Gemini 3.8 from the comparison document.

Antigravity limits are allowed to be spent; use one reasonable order at a time, avoid duplicate runs and automatic fanning of repetitions. Do not bypass quota limitation/exhaustion. This document does not establish actual current balances.

The current package 20260912-retry keeps the running sequence Kimi → DeepSeek → Kimi → DeepSeek. At the time of changing assignments, the first three stages have been passed, the fourth is being completed. The new model does not replace a working process in the middle of a stage.

Qwen 0902 added to limited external helper with provider_default. The reasoning modes of the old qwen3.8-max are not carried over automatically. Protocol probe - an artificial call to probe_echo(7) without executing the tool and without project sources; its result is not equal to the stand-alone development test.

## Actual connection check

Four local profile checks are successful; Helper syntax checked. The first request adf8aa72-ea5f-4c1e-af30-630668d85715 received a 400 error inside the thread: the provider prohibits required/object tool_choice in thinking mode. This is a sample parameter incompatibility, not evidence of model inaccessibility. For repeated testing, an explicit tool_probe_choice=auto has been added; other existing probes retain the forced call.

Replay 342b81a9-4420-41e4-b651-5badd826c665 completed successfully: model_reported=qwen3.8-max-0902, finish_reason=tool_calls, exactly probe_echo with argument value=7. The instrument was not performed. Total time 72.760s, headers 44.064s, first event 68.783s. This is one compatibility test; speed benefits have not been established. Reports are saved in .local/external/<id>/report.json. The model has been approved for the following limited coding package with independent review.

During the final integration of the current candidate copy, save the new changes to tools/external-request.mjs, tools/external-model.mjs, tools/tests/external-request.test.mjs, the link in DELEGATION.md and this document: they were made by the main agent after baseline.json. The old full review report describes the previous revision; This pass only needs to check the modified helper.
