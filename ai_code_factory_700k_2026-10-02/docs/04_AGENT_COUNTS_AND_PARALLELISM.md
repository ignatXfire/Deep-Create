# How many agents can you realistically run?

## Recommended one-day organization

For an already prepared repository with good contracts:

| Pool | Concurrent lanes |
|---|---:|
| Chief / release reasoning | 1–2 |
| Domain architects | 3–6 |
| Spec/decomposition | 8–16 |
| Writers | 32–64 |
| Test authors / mutation / property | 24–48 |
| Independent reviewers | 24–48 |
| Integrators / rebase / conflict resolution | 8–16 |
| E2E/reliability/security | 8–16 |
| Read-only scouts | 20–100 ephemeral |

Not all pools are required to reach the upper limit at the same time. Over the course of a day, approximately **100–220 logical agents/runs** are obtained, with **40–80 simultaneously mutating lanes**.

## Extreme 700k stress topology

To mathematically approach 700k integrated LOC at very high performance, you need approximately:

* 96 writer lanes;
* 72–96 reviewer/test lanes;
* 16–24 integration lanes;
* 12–20 E2E/reliability lanes;
* 10–30 spec/scout/architecture lanes.

Total is about **200–260 logical parallel functions**. This does not mean that 260 processes should always be running simultaneously: backpressure must stop upstream pools.

## Why not 1000 writers

With 1000 writers without thousands of truly independent authority shards, the following arises:

* merge conflicts;
* duplicate features;
* contract drift;
* build queue explosion;
* review backlog;
* false greens on stale base;
* expensive re-reading of the context;
* WIP is too high.

1000 read-only scouts can be useful for searching/sampling. 1000 concurrent writers in one codebase almost always require artificially created independent shards.

## Calibration rule

Start with 8–16 writers for 30–60 minutes. Increase concurrency by ~25–50% only if:

* first-pass review acceptance >70%;
* merge/conflict failure <5%;
* review queue age does not increase;
* build wait <30% wall time;
* integration queue < 1–2 waves;
* E2E capacity is not saturated.

If acceptance <50%, **do not add agents**. Fix spec/harness/boundaries.

## Little's Law intuition

`WIP ~= throughput * cycle_time`.

Doubling the number of writers with the same integration capacity simply doubles the work in progress and makes feedback older. Therefore, it is more important to reduce cycle time and increase downstream capacity.
