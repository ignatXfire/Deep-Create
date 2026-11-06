import { test } from "node:test";
import assert from "node:assert/strict";
import { requestParameters } from "../external-request.mjs";

const job = (model, reasoning) => ({ model, reasoning, max_tokens: 1024 });
test("thinking-only routes never silently receive disable-thinking", () => {
  for (const model of ["dash/kimi-k3", "dash/qwen3.8-2.4t-a95b"]) {
    assert.throws(() => requestParameters(job(model, "direct")));
    assert.equal(requestParameters(job(model, "low")).reasoning_effort, "low");
  }
});
test("hybrid direct mode uses the DashScope field, not vendor-direct thinking syntax", () => {
  for (const model of ["dash/deepseek-v4-pro-0813", "dash/qwen3.8-max"]) {
    assert.deepEqual(requestParameters(job(model, "direct")), { model, max_completion_tokens: 1024, stream: true, enable_thinking: false });
  }
});
test("exact per-route effort names cannot inherit a universal Ultra or Max override", () => {
  assert.throws(() => requestParameters(job("dash/qwen3.8-max", "max")));
  assert.throws(() => requestParameters(job("dash/kimi-k3", "xhigh")));
  assert.throws(() => requestParameters(job("dash/deepseek-v4-pro-0813", "ultra")));
  assert.equal(requestParameters(job("dash/qwen3.8-max", "medium")).reasoning_effort, "medium");
});
test("provider default remains explicit absence, unknown models and excessive output fail locally", () => {
  assert.deepEqual(requestParameters(job("dash/qwen3.8-max-0902", "provider_default")), { model: "dash/qwen3.8-max-0902", max_completion_tokens: 1024, stream: true });
  for (const reasoning of ["direct", "low", "medium", "xhigh"]) assert.throws(() => requestParameters(job("dash/qwen3.8-max-0902", reasoning)));
  assert.deepEqual(requestParameters(job("dash/kimi-k3", "provider_default")), { model: "dash/kimi-k3", max_completion_tokens: 1024, stream: true });
  assert.throws(() => requestParameters(job("dash/ZHIPU/GLM-5.3", "low")));
  for (const max_tokens of [0, 8193, 0.5, "512"]) assert.throws(() => requestParameters({ ...job("dash/kimi-k3"), max_tokens }));
});
