// Developer-only request profiles, not a production capability/permission catalog.
// The exact dash route is Chat Completions; parameters differ from vendor-direct APIs.
const profiles = new Map([
  ["dash/kimi-k3", { efforts: ["low", "high", "max"], direct: false }],
  ["dash/deepseek-v4-pro-0813", { efforts: ["low", "high", "max"], direct: true }],
  ["dash/qwen3.8-max", { efforts: ["low", "medium", "xhigh"], direct: true }],
  // New version: use provider defaults until its explicit reasoning modes are tested.
  ["dash/qwen3.8-max-0902", { efforts: [], direct: false }],
  ["dash/qwen3.8-2.4t-a95b", { efforts: ["low", "medium", "xhigh"], direct: false }],
]);

export function requestParameters(job) {
  const profile = profiles.get(job.model);
  if (!profile) throw new Error("Unverified model route");
  if (!Number.isInteger(job.max_tokens) || job.max_tokens < 32 || job.max_tokens > 8192) throw new Error("Invalid output budget");
  // A total completion limit is essential: on Kimi, max_tokens only bounds
  // visible answer tokens and leaves reasoning outside that limit.
  const result = { model: job.model, max_completion_tokens: job.max_tokens, stream: true };
  if (job.reasoning === undefined || job.reasoning === "provider_default") return result;
  if (job.reasoning === "direct") {
    if (!profile.direct) throw new Error("This route cannot disable thinking");
    result.enable_thinking = false;
  } else {
    if (!profile.efforts.includes(job.reasoning)) throw new Error("Unsupported reasoning effort for this route");
    result.reasoning_effort = job.reasoning;
  }
  return result;
}
