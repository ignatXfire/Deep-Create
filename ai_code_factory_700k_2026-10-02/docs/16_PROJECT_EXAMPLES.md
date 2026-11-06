# Project examples

## Example A — SaaS web platform, greenfield

Domains: auth, users, billing, files, search, notifications, admin, analytics.

Suggested peak:

* 4 architects/spec planners;
* 32 writers;
* 24 tests/reviews;
* 8 integrators/E2E.

High parallelism because domains and vertical slices can be isolated. 100k+ integrated LOC/day is plausible if much UI/API/client/test code follows patterns; 700k would require a large generated component.

## Example B — Local desktop/Rust/Tauri-like autonomous system

Shared runtime state, persistence, process management and recovery have high semantic coupling.

Suggested peak:

* 4–6 architecture/reliability agents;
* 16–32 writers;
* 24–40 test/reviewer lanes;
* 8 integration/recovery/E2E.

Lower writer count, higher proof ratio. Aiming for 700k novel core LOC would be counterproductive; verified state transitions matter more.

## Example C — Enterprise CRUD/API surface from schemas

Hundreds of resources/endpoints with standardized patterns.

Architecture:

* schema/IDL generator is the main source of truth;
* generated clients/models/validators/docs/tests;
* human-like agent work focuses on exceptions/business rules.

This is the project class where 700k compiling/module-tested LOC/day can be genuinely attainable, because much of the volume is deterministic generation rather than 700k independent design decisions.

## Example D — Existing 700k LOC repo needing large feature set

Do **not** create 100 writers immediately. First 20–50 read-only scouts map domains/ownership/hidden contracts/tests. Then workgraph partitions changes by authority. Existing architecture may be the limiting factor; investing the first 1–2 hours in maps/lints can save more than maximum generation.
