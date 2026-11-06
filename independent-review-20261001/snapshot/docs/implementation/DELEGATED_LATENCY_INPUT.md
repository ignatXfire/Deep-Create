# Measured data for narrow analysis of delays

Date: 2026-09-11. Analysis of only the given facts, without access to keys or changes to the service.

OmniRoute 3.8.48, local Chat Completions dash route to DashScope compatible-mode/v1. The supplier has not been transferred to Responses. HTTP stream is enabled. There are no automatic replays in our client.

1. Kimi, identical order for five Rust tests, about 2390 input tokens. Default settings + max_tokens=4096: 321938 ms total, first thread 7159 ms, first content 299107 ms. The output is 14064 tokens, of which the reasoning is 12893, the answer is 1171. Normal completion.
2. The same order (input difference is only Rust formatting), reasoning_effort=low + max_completion_tokens=4096: 32985 ms total, thread 6870 ms, content 16034 ms. Output 1244, reasoning 175, answer 1069. Normal completion. TWO parameters have been changed, the effect cannot be attributed to one without additional experience.
3. DeepSeek v4 pro 0813, default code parsing, max_tokens=4096: 71586 ms, finish_reason=length, all 4096 for reasoning, no answer. Narrower analysis with enable_thinking=false: 18919 ms, 758 response tokens, no reasoning. Entry reduced from 6495 to 3093 tokens: uncontrolled comparison.
4. After Windows loaded, there was no listener 20128. The existing standard omniroute.cmd --no-open brought up the port in 4 seconds. Initialization lasted approximately one and a half minutes; During this period, health exceeded the limits of 8 and 12 s. After initialization, protected health returned 401 in 2717 ms. The reason for the termination of the previous autorun has not been precisely established: the last value is 0xC000013A, there is no fresh line in its separate log.
5. After initialization Kimi low, short JSON, total limit 512: normal connection selection 17805 ms, first response 14027 ms; previously pinned successful connection 17384 ms, first response 13191 ms. Both returned the expected content and exited. One pair is not enough to conclude that the pinning has won.
6. Queue: automatic activation for API-key providers=false, interval 350 ms, 60 requests/min, concurrent=6, maxWait=900000 ms. Waiting 900000 cannot be considered an artificial delay for each request.
7. Direct request to upstream without a key and without model generation: TLS 1.524 s, HTTP 400 after 1.846 s. This is not the equivalent of a model call.
8. The log contains errors from other providers and unavailable proxies. They don't prove that dash goes the same way. The settings of the service itself, keys and connection tables did not change.

We need: a short conclusion about which changes to our request are justified; what conclusions have not been proven; exactly one next measuring step if necessary. Do not suggest disabling global restrictions, changing keys, or bypassing vendor restrictions. The goal is to return development to useful assignments.
