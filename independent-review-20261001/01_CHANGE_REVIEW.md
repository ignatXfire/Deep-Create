# What has changed since the September 29 audit

Reason: source code of the saved image `f7444d158fb9900217e8a62d0d0c53cbff2f5dd3` plus two existing uncommitted edits. Their patch is saved separately. Old reports are used as a list of issues to be verified, not as evidence.

Between the previous `b194df4` and the current HEAD in crates/ui/tools/Cargo.toml, 81 files were changed: 10,015 added and 1,126 deleted lines. This is the diff volume; it does not measure useful work, readiness, or the number of requirements accepted.

| Previous question | What's Really Changed | What remains |
|---|---|---|
| F01: UI UUID and IPC | 32hex and snake_case are consistent | Projection issues Debug-ID; start_attempt parses ID as decimal |
| F02: persistence panic | Singleton ID is now UUIDv7, init error is processed | Changing window commands bypass persistence |
| F03: success panic/verification | Invalid numeric IDs are replaced, hash is calculated by content | A non-zero hash remains the basis of a false Passed; someone else's unit is not rejected |
| F04: task/dependencies | There is a separate executor with topological levels | The goal/criteria are not fully delivered; parent's refusal does not block the next level |
| F05: nested retry | Outer loop of three attempts removed | Broker makes up to four POSTs without proven permission unknown |
| F06: Retry-After | The numeric header is now seconds | HTTP date is not parsed, fallback is applied |
| F07: AppContainer | Added other Windows/pipe primitives | Adopt does not create the required token; is_process_appcontainer remains stub |
| F08: CPU limit | There is no significant correction in the reviewed path | Limit field does not apply to Job Object |
| F09: replace history | Event/evidence is now a strict INSERT; probe confirmed saving of the first record | ID from milliseconds may conflict; checkpoint remains REPLACE |
| F10: schema gate | Added new tables and label 3 | Future schema is accepted; version 10 is mistakenly changed to 3 |
| F11: exact authenticated bytes | No meaningful fix found | Parsed envelope is serialized again |
| F12: live tests | Some new live-tests have ignore | Old broker/worker paths remain automatic |

Real new basis: journal, ledger and evidence archive with SHA-256; additional error-injection/stability checks; IPC round-trip tests; new parallel executor; tracing/health; named pipe API; deployment tools These are useful implementations. Their connections and limits are checked separately in the following files.

You cannot declare all previous comments closed by the name commits or the word REAL in the comment. It is also impossible to continue to claim that already fixed old panics exist: these specific causes have been eliminated.
