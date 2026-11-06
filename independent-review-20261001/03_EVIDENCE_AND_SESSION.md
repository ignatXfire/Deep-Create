# What the inspections and the recent session proved

## Recent checks of this audit

| Check | Real result | Limit |
|---|---|---|
| TypeScript | exit 0 | UI type checking, not true IPC |
| Vitest | 50 passed, 3 failed, exit 1 | Checks for AppManifest formatting, presence of rusqlite in native Cargo.toml and page strings fail; there are timeout messages stopping fork workers |
| Rust domain/protocol/store/watchdog | 376 passed, 0 failed, exit 0 | Snapshot, sqlite + test-peer-constructor; full workspace/default desktop build not tested |
| Independent probes | 6 passed, 0 failed, exit 0 | Five violations reproduced, one correction confirmed |

Probe names:

1. `fixed_event_duplicate_rejected_without_destroying_prior_record`: Strict INSERT fix confirmed.
2. `future_schema_ten_silently_relabelled_three`: Unsupported circuit opens and changes label.
3. `evidence_same_bytes_lose_second_entity_provenance`: Successful deduplication loses the second link.
4. `journal_unsigned_sequence_breaks_head_order`: signed cast breaks head/order.
5. `machine_pass_without_criterion_evidence_and_with_wrong_unit`: someone else’s unit and unverified criteria give Passed.
6. `impersonation_guard_is_movable_between_threads`: compile-time confirmation of Send+Sync real guard; no real impersonation was performed.

PASS for defect probe means the defect has been reproduced, not “corrected”. There is an explanation in the code. For future repairs, a separate regression test with the correct expected failure/data is needed.

Three UI failures should not automatically be called three broken functions: one test depends on the precise design of the command list; another prohibits rusqlite in native manifest, although Rust persistence uses it; the third one expects a modified string. There is a need for meaningful updating of these tests. But you cannot ignore exit 1 and declare the entire UI test suite green.

## Session-20260930.json

The file is parsed as data, not as new instructions. Session ID `20260930_224749_24fcaf`, 1,255 messages: 42 with role=user, 539 assistant, 674 tool. The SHA-256 of the original export is stored in session-metadata.json.

42 user-role messages are not equal to 42 human messages: the classifier found 22 automatic compaction/background/delegation/iteration limit markers, 19 human-message candidates and one out-of-band wrapper. This is a heuristic classification with initial indexes, not proven attribution of each message. Repeated master prompts are not 19 independent decisions of the owner.

The human request with index **1168** is especially important: the owner rejects the word Production-ready without a criterion, requires consistency of resources/interactions and the use of Deep Create for further development of Deep Create. This explains why fancy forms, a separate model query, and a passed counter are not sufficient for its purpose.

The final answer assistant **1251** has key statements verified:

| Statement | What's checked | Correct conclusion |
|---|---|---|
| Core works on 53 tests | Real machine verifier accepts someone else's result in a new probe | Individual core tests do not prove correct acceptance |
| Crash recovery works after OS kill | Existing persistence E2E checks startup/file size; UI handlers don't persist | Proof of database creation/reopening is weaker than restoration of user actions |
| MSI/NSIS 6/6 | Installers were not run in this audit | Historical claim, not confirmed here pure VM |
| Memory leak was a harness bug | There is a bounded stats patch in the working tree, the previous Vec growth is explained by it | The harness fix is ​​visible; new full soak and no other leaks have not been proven here |
| Production-ready | A complete accepted set of requirements is not defined; there is P1 in the current chain | Readiness for full promised use has not been confirmed |

The export shows active iterations, repairs, compaction and confirmation retries. It does not prove by itself that all subtasks are integrated, that the new binary matches the sources, or that Deep Create itself did the patch→tests→review→integration instead of an external organization.

## Verification of evidence itself needs verification

The directory contains real exit codes and full logs of our selected launches. Old *_EVIDENCE.md is included as statements/context. They did not automatically receive VERIFIED status. The file name, the word REAL, the green check mark, HTTP200 and the live process have limited meaning.

For any further evidence you need: exact source SHA + dirty patch/hash, binary hash, command, features, cwd/env policy, raw stdout/stderr, exit, negative case data and verifiable expected outcome. You cannot simply aggregate messages from different agents into one percent.
