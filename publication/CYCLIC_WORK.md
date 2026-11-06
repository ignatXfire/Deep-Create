# The next attempt should inherit evidence

## The problem is not simply a short context window

A long-running AI task can lose progress without losing a single message. A result may depend on an obsolete specification. A summary may omit the reason a previous approach failed. A timeout may conceal an action that already happened. A confident review may accept an artifact without checking the requested criteria.

Appending more text does not by itself resolve those problems. Neither does increasing the number of agents. The system needs a way to distinguish useful evidence from an assertion, a changed dependency from an unrelated edit, and a genuinely failed operation from an unknown outcome.

Deep Create investigates a specific response: make each iteration an explicit transition over objectives, work units, evidence, dependencies, authority, and resource state. Preserve the record of what happened. Rebuild the next working context from that record. Admit a result only against its actual current basis.

This is a research design with a partial implementation, not a demonstrated completed autonomous application.

## What changes between iterations?

The intended cycle is not “ask again until the answer looks good.”

1. **Select.** Construct a bounded context for a particular work unit from relevant sources, accepted results, unresolved obligations, and current dependency versions.
2. **Propose.** A model produces a candidate artifact or action proposal. Its text does not directly become trusted system state.
3. **Check.** Evaluate the artifact against the actual criteria and dependency basis. Record missing evidence and uncertainty as such.
4. **Record.** Retain the result, observations, provenance, verification outcome, and known resource consumption. A failed attempt remains part of the record.
5. **Revise or accept.** Return specific obligations for repair, or admit the result through the controlling transition. A changed basis can invalidate an otherwise plausible result.
6. **Continue.** Recompute what the next unit needs. Do not automatically repeat independent work that remains demonstrably applicable.

This cycle changes external work state and selected context. It is not a claim that model weights improve, that every iteration is better, or that unlimited iteration converges.

## Where an efficiency advantage could come from

| Mechanism | Work it could avoid | Condition that must hold |
|---|---|---|
| Selective repair | Recreating unrelated outputs after one part fails | Dependencies and unaffectedness must be established correctly |
| Reusable accepted results | Asking another worker to regenerate an already suitable result | The result must still satisfy its scope, revision, and acceptance conditions |
| Context reconstruction with provenance | Re-reading everything or blindly trusting a lossy summary | Retrieval must retain the facts needed for the next decision; provenance alone does not guarantee relevance |
| Explicit feedback obligations | Repeating the same known mistake without carrying its cause forward | The next context and acceptance check must actually include those obligations |
| Unknown-outcome reconciliation | Repeating an external operation merely because its response was lost | The real dispatch path must preserve uncertainty and support reconciliation |

These are causal hypotheses about avoiding redundant or invalid work. They are not measured speedups. Checking, indexing, state maintenance, reconciliation, and coordination all consume resources. For short or already reliable tasks, they may cost more than they save.

The useful question is not “how many iterations ran?” It is “how much independently acceptable progress survived per unit of total cost?”

## A concrete selective-repair example from the code

The domain test `returned_publication_blocks_its_consumer_without_stopping_proven_independent_consumer` is in [owner_return.rs](../crates/dc-domain/tests/cases/owner_return.rs).

It constructs two publications and their consumers. Returning one publication creates a reason to block its dependent consumer. The other publication is supplied with evidence of being unaffected. After applying the proposed transition, the test checks that:

- the affected consumer is blocked by the unresolved publication;
- the proven-independent consumer remains unblocked;
- the original publication records, objectives, and work units are preserved;
- the affected and independent read sets both reflect the new basis;
- an unrelated obligation cannot be smuggled into the repair request.

The fourth point matters: “independent” does not mean that an old authorization or read snapshot can be reused forever. The independent branch remains admissible under the new conditions.

This is a tangible mechanism behind the efficiency hypothesis. A repair need not imply a restart of all work. However, the test applies a proposal synthetically. Its own comment explicitly says it is **not a C-03 durable commit**. It does not measure model calls, money, wall-clock time, or a real desktop workflow. The [evidence map](EVIDENCE_MAP.md) keeps those distinctions visible.

## The record survives; the working view can change

[Context and memory contracts](../docs/technical/15_CONTEXT_MEMORY_AND_INSTRUCTIONS.md) distinguish source evidence from a derived working projection. This permits a smaller task-specific view without treating the summary as authority to overwrite the original record.

The implemented [context validator](../crates/dc-domain/src/context.rs) includes tests such as `changed_dependency_basis_rejected` and `secret_fragment_never_enters_projection`. These establish inspectable local boundaries. They do not establish that live retrieval always selects the best information, that every provider adapter enforces the boundary, or that a smaller prompt preserves task quality.

The proposed benefit is continuity with traceability: retain why a conclusion was reached, which facts it used, and which changes would make it obsolete. The effectiveness of selection still requires evaluation.

## A bad loop can preserve the wrong thing

The included independent audit found that a machine-verification API could return `Passed` for the wrong unit without criterion evidence. See [finding N03](../independent-review-20261001/02_FINDINGS.md).

That is a counterexample to “verification exists, therefore the loop improves.” If a verifier accepts weak evidence, feedback can reinforce a mistake. If the dispatcher blindly retries an unknown paid operation, iteration can increase waste. If a projection omits the real objective, a perfectly recorded loop can optimize the wrong task.

The same audit records these broken boundaries instead of hiding them. This makes the repository useful as a design-and-failure corpus: a reader can compare the intended rule, the implementation, and an adversarial reproduction. It is not evidence that those defects have been fixed.

## What is worth studying here?

The reusable material is the connection between contracts, machine objects, transition tests, and negative audit cases. It offers concrete questions for researchers:

- Can selective repair preserve quality while reducing repeated model work?
- When does task-specific context beat a full transcript, and what does it lose?
- Can a failed check be carried into the next iteration without becoming an untrusted instruction?
- What is the cost of detecting stale results before acceptance rather than discovering them downstream?
- Under which failures does feedback reduce waste, and under which does it amplify it?

Persistence and multi-agent orchestration are already established areas: see [LangGraph's persistence documentation](https://github.com/langchain-ai/docs/blob/main/src/oss/langgraph/checkpointers.mdx) and [AutoGen](https://github.com/microsoft/autogen). Deep Create does not claim to have invented feedback loops or demonstrated superiority over those projects. Its proposed research contribution is this particular contract-linked implementation and failure corpus.

For the fair experiment needed to turn these hypotheses into a result, see [the evaluation protocol](EVALUATION_PROTOCOL.md). For exactly what can be inspected now, see [the evidence map](EVIDENCE_MAP.md).
