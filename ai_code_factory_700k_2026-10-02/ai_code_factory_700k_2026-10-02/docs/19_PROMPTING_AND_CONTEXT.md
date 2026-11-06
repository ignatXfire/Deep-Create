# Prompting and context architecture

## Stop writing mega-prompts

A high-throughput agent factory needs **context assembly**, not one enormous universal instruction. The root prompt should specify invariant rules and where authoritative information lives.

## Context packet per task

1. mission excerpt;
2. task objective/non-goals;
3. base SHA;
4. relevant contracts;
5. code map/symbols;
6. allowed authority/paths;
7. acceptance and negative tests;
8. commands/tools available;
9. expected receipt schema;
10. links to 1–5 relevant design docs.

## Role prompt shape

Use concise sections:

`ROLE -> OBJECTIVE -> OWNERSHIP -> MUST PRESERVE -> EVIDENCE -> STOP CONDITIONS`.

Do not ask an implementer to be planner, architect, reviewer and release manager simultaneously.

## Tool-first instructions

Tell agents to inspect actual repo/status/tests before proposing code. Claims about files/functions must be grounded by tool read/search. After patch, agent runs verification itself and outputs machine receipt path rather than a persuasive essay.

## Context freshness

Packets include doc/version hashes. If candidate SHA or relevant contract changes, task becomes stale and scheduler must rebase/rebuild context.

## Kimi dynamic tool loading

For very large tool catalogs, Kimi K3's dynamic tool-loading capability can reduce tool-definition context bloat. Still keep role toolsets minimal: writer needs code/test tools; reviewer needs read/test; planner usually should not have production mutation rights.

## Prompt evolution

Every recurring failure should lead to one of:

* deterministic lint/check;
* better architecture doc;
* better context retrieval;
* refined task schema;
* new regression test.

Do not endlessly grow the system prompt. Promote repeated guidance into executable constraints.
