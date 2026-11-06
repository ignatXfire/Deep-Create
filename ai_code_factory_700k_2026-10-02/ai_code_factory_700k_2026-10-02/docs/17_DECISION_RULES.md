# Decision rules for the orchestrator

1. **Never optimize raw LOC.** Optimize accepted semantic value per bottleneck minute.
2. **One active mutator per authority key.** Read-only concurrency is cheap; write concurrency is scarce.
3. **Scale only after measurement.** 8–16 writer calibration wave before 32–64.
4. **If downstream queue grows, stop upstream.** Do not hide backlog with more agents.
5. **Critical-path work gets the best model, not the largest task.** One Sol decision should unlock many DashScope tasks.
6. **No task without observable acceptance.** If success cannot be mechanically observed, spec is incomplete.
7. **No reviewer PASS from prose alone.** Exact SHA/files/tests/evidence required.
8. **Retry changes attempt, not history.** Preserve failure evidence.
9. **Discover new prerequisites dynamically.** Block + create node instead of hallucinating a workaround.
10. **Treat historical docs as maps, not runtime truth.** Code/current candidate/evidence wins.
11. **Generated code comes from generators.** Do not make 50 agents manually replicate a pattern.
12. **Prompts point to sources of truth.** Avoid mega-context.
13. **Provider quotas are scheduled resources.** Reserve review/repair bandwidth.
14. **Feature cutoff is mandatory.** Convergence and release proof need protected time.
15. **Every global architecture exception is expensive.** Require architect/release authority.
16. **If acceptance <50%, fix the environment.** “Try harder” is not a scaling strategy.
17. **If a worker can run the app, let it observe the app.** UI/logs/traces/metrics should be agent-legible.
18. **Garbage collection is continuous.** Recurring agents remove drift/bad patterns.
19. **Use model diversity for criticism, not as a substitute for evidence.**
20. **Release status is computed, never narrated.**
