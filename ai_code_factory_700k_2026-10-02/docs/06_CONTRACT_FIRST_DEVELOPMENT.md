# Contract-first development: how to make dozens of writers independent

## Goal

Before mass generation, it is necessary to turn the system into a set of **explicit contracts**, otherwise the agents will each come up with the architecture in their own way.

## Architecture constitution

Before Wave 1 the following are recorded:

* domain map;
* layer/dependency directions;
* public API/schema versioning;
* error taxonomy;
* persistence/migration policy;
* identity/idempotency rules;
* async/concurrency model;
* observability contract;
* security boundaries;
* naming/package conventions;
* max file/module complexity rules;
* prohibited dependencies/patterns.

These rules should not live only in markdown: everything possible is encoded by architecture lints, type checks, schema validation and CI.

## Boundary contracts

For each domain:

1. Inputs and outputs.
2. Invariants.
3. Failure semantics.
4. Persistence effects.
5. Events/messages.
6. Performance budget.
7.Security/authorization requirements.
8.Backward compatibility.

The writer can then implement the internals autonomously.

##Package sizing

A good work package should usually:

* have 1 main semantic objective;
* affect 1–3 authority keys;
* have locally run proof;
* give a diff that the reviewer is able to understand in one context;
* take approximately 15–90 minutes agent wall-time if the infrastructure is already ready.

Too small - orchestration overhead eats throughput. Too large - diagnosability and independence sharply deteriorate.

## Generated surfaces

To quickly increase a large code volume without manual drift, generate from source-of-truth:

* OpenAPI/protobuf clients;
* database types;
*validators;
* mocks/fixtures;
* config schemas;
* typed event definitions;
* docs tables;
* repetitive CRUD layers.

Do not instruct dozens of agents to manually copy identical patterns - one generator + contract tests is better and faster.

## Progressive disclosure

The root `AGENTS.md` should be a short router/map, not a 1000-page prompt. For each task, context builder provides only relevant architecture docs, exact contracts, affected code map, tests and recent decisions.

## Definition of ready

Node is allowed into the writer pool only if:

* dependencies resolved;
* version contracts-pinned;
*allowed authority available;
* expected test commands known;
* success/failure observable;
* rollback path known.
