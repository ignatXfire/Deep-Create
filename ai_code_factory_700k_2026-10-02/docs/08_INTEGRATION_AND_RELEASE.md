# Integration and release pipeline

## Single integration authority

The main branch/candidate has one logical writer - integration service. It is not necessarily one physical process; but the serialized candidate head ensures that accepting one commit is based on a known previous state.

## Merge train

1. Candidate commit + worker receipt enters the queue.
2. Freshness base and authority conflicts are checked.
3. Rebase/cherry-pick into temporary integration worktree.
4. Contract/affected integration tests are launched.
5. With GREEN, the candidate head is atomically updated.
6. In case of FAIL, the package is returned with precise evidence, not with the general “try to fix it.”

## Continuous candidate gates

In parallel, constantly working:

* full workspace build/test;
*architecture lints;
* generated-artifact freshness;
* security scans;
* end-user critical journeys;
* performance smoke;
* install/upgrade/uninstall;
* crash/restart/persistence checks.

## Freeze window

The last ~2–3 hours of a one-day project cannot continue at maximum feature-generation speed. The system goes into convergence:

* almost no new features;
* only P0/P1 fixes;
* dependency/config freeze;
* full E2E is repeated on exact release SHA;
* SBOM/manifest/checksums are created;
* known limitations are fixed.

## Release judge

Release judge reads not narrative summaries, but release manifest:

* exact SHA;
* artifact hashes;
* gate matrix;
* unresolved blockers;
* skipped/ignored tests;
* test environment limitations;
*rollback evidence;
* security/performance findings.

If one mandatory gate is not confirmed, the status is not `DONE`, but `BLOCKED`/`PARTIAL` with a specific reason.

## Do not confuse local green and product ready

`cargo test`, `npm test`, 500 passing tests or even a clean full-workspace suite can only prove what the suite covers. Real product readiness requires installation/launch, external dependencies, actual user journey, restart/recovery and deployment environment.
