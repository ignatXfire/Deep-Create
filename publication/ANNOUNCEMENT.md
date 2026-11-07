# English launch copy

## Suggested title

**Deep Create: what should an AI workflow remember after a mistake?**

## Short description

An unfinished research prototype for evidence-carrying AI work: feedback loops, versioned context, selective repair, Rust transition tests, and an independent audit of where the design still breaks.

## Announcement

Repeating a prompt is easy. Carrying useful progress through the next correction is harder.

Deep Create explores a workflow in which an AI result is not simply appended to a chat. It is tied to a work unit, dependency versions, evidence, and acceptance criteria. A failed check becomes explicit repair work. The next context is rebuilt from the record rather than treated as a fresh start or an ever-growing transcript.

The repository contains technical contracts, an experimental Rust/Tauri/React implementation, component tests, and independent audit probes. Its first-party implementation code was generated entirely by AI under my direction: inspect it critically, and independently check the code, tests, and reported results before running or reusing them. One inspectable example distinguishes a consumer affected by a returned result from a proven-independent consumer that can remain unblocked. Another rejects context based on changed dependencies.

It also exposes counterexamples: a historical machine-verification path could return `Passed` without criterion evidence for the expected work unit. Keeping that failure visible is part of the point—a feedback loop is only as useful as the state and verification it carries forward.

This is not a finished application or a claim of measured speedup, guaranteed convergence, or fully autonomous self-development. The research question is whether selective reuse and evidence-based feedback can earn their coordination cost. The repository includes an evidence map and a proposed fair evaluation protocol.

Useful criticism is welcome: which boundaries are missing, which component claims fail under reproduction, and which experiment would most clearly confirm or reject the efficiency hypothesis?

## Posting notes for the owner

After publishing the repository, use its actual URL; none is invented in this local package. Link directly to [the explanation](CYCLIC_WORK.md), [the evidence](EVIDENCE_MAP.md), and [the open findings](../independent-review-20261001/02_FINDINGS.md). Original project material is licensed under Apache-2.0; retain its scope and third-party qualifications. Attribute Ignat, [@ignatXfire](https://x.com/ignatXfire), without implying a promise of ongoing maintenance or a completed product.

The README links the bundled [research trailer — 1:00](../assets/deep-create-overview.mp4), unchanged [AI-generated audio podcast — 39:30](../assets/deep-create-podcast.m4a), and companion NotebookLM notebook. The trailer connects a historical verification counterexample, selective repair and real source excerpts; it is not application-execution footage. Preserve the music attribution when sharing it. Add an X post link only after the owner supplies its actual URL. Lead with the problem and one inspectable example; readers should not have to study the full source tree before deciding whether the idea is relevant. Do not add unmeasured multiplier claims, invented usage numbers, fake endorsements, or “world first” language.
