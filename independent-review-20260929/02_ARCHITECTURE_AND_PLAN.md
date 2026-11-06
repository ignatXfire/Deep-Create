# State of the architecture and future work

This is an engineering opinion on proven critical paths; not fully certified C-01 to C-16.

## What is actually collected

| Layer | Observable implementation | What prevents recognition as ready |
|---|---|---|
| dc-domain | Identities, goal/work contracts, rights, resources, transitions | The full requirements map has not been verified; callers bypass some of the rules |
| dc-protocol | Limited codec, types outcomes, replay, auth interface | Production ProofVerifier was not found in the crates examined; different byte semantics |
| dc-store | SQLite backend, CAS, stage3 tables | There is no proven single commit state+event+expenditure; replace history; schema gate |
| dc-core | Command handlers, scheduler, context/scheduler/verification modules | Simple root planner; execution does not use a full-fledged scheduler; false machine pass |
| dc-broker/worker | Real HTTP transport and try adapter | Retry of unknown outcome, no proven end-to-end admission/reserve |
| dc-watchdog | Admission/recovery model, anchor persistence | The independent process and end-to-end stop before the irreversible effect have not been proven |
| dc-windows | Real Job/DPAPI calls, AppContainer profile | Profile does not mean token isolation; CPU limit not applied |
| dc-desktop/UI | Tauri commands, forms, projection, execution entry | Incompatible IPC/ID, persistence crash, loss of response content |
| Self-development | Bookkeeping proposals and work-unit constructors | Actuator not found in this runtime edit→test→independent review→integrate |

The key issue is not choosing Rust versus another language. This is the gap between the contract and the actual execution route. Rust helps types and memory, but `Qualified`, created without proof, remains a false solution in safe Rust.

Division into crates is not division into processes. The flow of models in the local gateway does not prove that the worker is isolated. The SQL transaction API does not prove that state, log, and expenses are atomic at the same time. Updating the JSON field of a graph is not a complete crash recovery. The number of tests passed does not cover these transitions.

## About percentages

The measured percentage of readiness cannot now be calculated: there is no independently accepted full denominator of requirements, weights and evidence for each item. Numbers from previous reports are not reused.

My preliminary subjective assessment of the readiness of the **full promised product** is about **20-35%**, with low confidence. This is not statistics, not the share of files read, and not the fact that such a volume has been delivered. Reason: Essential implementation of the building blocks is there, but the core user journey, correct acceptance, durable recording, and boundary of external consequences have blocking gaps. The rating may change after checking other circuits. To connect a valuable account, the current criterion is binary: the necessary access chain has not been proven.

Architectural documents are much more detailed than the code, but the percentage of “architecture ready” by number of pages is meaningless. In particular, the most important thing now is to check the feasibility of joint invariants: stop+send+commit, change target+old result, restart+unknown, budget+repeat, two long-running jobs+shared object.

## Repair sequence

1. **First, the reliability of verification and isolation of tests.** Remove the unconditional success of verification, do the unknown Indeterminate; live tests separate. Otherwise, the next edits will be accepted by an unreliable mechanism.
2. **Close the vertical chain on the artificial executor.** Unified DTO/ID, real IPC, goal/criteria → context → result → evidence → core acceptance. Preserve branching and multiple works; this cut is a test of the supporting chain, not a replacement of the complete design with a simple chat.
3. **The only durable writer.** The command checks versions/authorities, atomically writes status, event, expense posting and receipt. Until commit only queued; after restart, the unknown commit is found out by identity. Append-only evidence, schema compatibility.
4. **Execution and external consequences.** Worker AppContainer/token, Job and failure checks; one retry owner; persistence unknown; real limits and cancellation before dispatch. Then a local fake provider with lost responses/429/partial response.
5. **Normal scheduler and long work.** Tolerance for dependencies and current goal, resource armor and generations; checking fairness, cancellation, stale results, common object of two works.
6. **Context and self-development.** Maintain an unchanged history, build projections with origins; independent verifier, isolated patch candidate, tests, separate acceptance and only then integration. Having the approve_proposal function is not merge authority.
7. **Final operational check.** Clean VM, install/update/rollback, migrations, backup restore, long runs, load and storage failure. Testing a process and turning off the power are different tests.

Work 2/3 and the study of the OS boundary can be carried out in parallel in separate candidates. Changes to the shared DTO, authority and commit agreement require one owner. Integration is consistent; an independent reviewer is not the author of his edit. Each card requires a baseline SHA, pre-fix test, diff, negative tests, new SHA and integration check.

## Timing and actual speed

The latest commits confirm activity, but not acceptance of functions. For example, adding retry and graph persistence on the same day coexists with F02/F05. A stable rate of “closed functions per day” cannot be deduced from this.

According to available data, there is no honest calendar date. My planning assumption is that we're talking about weeks for the first reliable synthetic end-to-end script and months for the full set of promises, rather than a few final fixes. This is an opinion, not a calculation based on the accepted throughput.

To obtain a forecast, conduct two weekly cycles with independent acceptance. Count only integrated requirements that pass negative checks on the exact SHA; separately consider re-openings and integration times. Break down the remainder according to dependencies; forecast = critical path + integration/research risks. Do not divide the number of files or lines by the model generation speed.

## Accounts, antidetect and proxies

For the current state, my conclusion is: do not connect valuable accounts and user work files. The reasons are specific: F05, F07, lack of proven single acceptance/record. A separate test account with a small limit reduces the consequences; At first, an artificial server without an account is enough.

The anti-detect browser and proxy do not correct the logic of permissions, replays, recording and checking of Deep Create results. If a tool has access to cookies/files/network, changing your browser fingerprint does not change those rights. For the next stage, a separate VM/profile without valuable data, limited rights and secrets, an allowlist endpoint and a budget are more important. Verifying a real account is a separate permissible step after this evidence.

The ha-otc-scanner files mentioned in the original quote belong to another project. They were not read or changed in this audit; the error of blocking them is not a defect in Deep Create. For your own running SQLite, you need a consistent backup, and not a blind copy of a busy db file.
