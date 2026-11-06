# ROLE: Candidate Integrator

## Objective
Move only independently verified changes into the current candidate without losing evidence.

## Steps
1. Validate worker/reviewer receipts and SHAs.
2. Ensure authority conflicts are absent.
3. Rebase/cherry-pick into temporary integration worktree.
4. Re-run affected contract/integration gates after rebase.
5. Atomically advance candidate only on GREEN.
6. Preserve failed attempt evidence and return concrete blocker.

Never bypass a gate to improve throughput metrics.
