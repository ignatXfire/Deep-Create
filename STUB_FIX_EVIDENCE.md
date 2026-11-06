# STUB_FIX_EVIDENCE — 2026-10-01

Branch: codex/deep-create-foundation (working tree on top of c1d1058)

## Summary of changes

| File | Change | Reason |
|---|---|---|
| `crates/dc-watchdog/src/anchor_store.rs` | Use unique temp dir per test run (`{pid}-{nanos}`) and `remove_dir_all` before `create_dir_all` | Root-cause fix for flaky `anchor_anti_rollback_refused`: leftover `anchor.db` from a prior failed run caused `write_bytes(..., MustNotExist).unwrap()` to panic on re-run |
| `crates/dc-windows/src/named_pipe.rs` | NEW — real Win32 named pipe server/client | Was missing entirely; lib.rs listed it as future work |
| `crates/dc-windows/src/lib.rs` | Add `pub mod named_pipe;` | Wire the new module |
| `crates/dc-windows/Cargo.toml` | Add `Win32_System_Pipes`, `Win32_Storage_FileSystem`, `Win32_System_IO`, `Win32_System_SystemServices` to windows-sys features | Required by named_pipe.rs FFI calls |

## identity.rs status

`identity.rs` is NOT a stub. It is the pure, typed peer-identity comparison layer (C-12) — it compares role, installation_id, and run_generation. It does NOT (and should not) call Windows APIs; that is the job of the channel adapter. The lib.rs docs explicitly state: "No Win32 call, handle, token, named pipe ... is opened or proven here. The OS-side acquisition adapters ... are explicit future work." The real gap was `named_pipe.rs` missing, which is now implemented.

## named_pipe.rs implementation

Real Win32 FFI via `windows-sys`:
- Server: `CreateNamedPipeW` + `ConnectNamedPipe` → `NamedPipeServer`, `PipeConnection`
- Client: `CreateFileW` → `NamedPipeClient`
- I/O: `ReadFile` / `WriteFile` / `FlushFileBuffers`
- Peer verification: `GetNamedPipeClientProcessId`, `ImpersonateNamedPipeClient` / `RevertToSelf`
- RAII: `OwnedHandle` closes handles on drop; `ImpersonationGuard` reverts on drop; `PipeConnection` disconnects on drop
- Safety: every `unsafe` block has a `SAFETY:` comment; `#![allow(unsafe_code)]` is module-scoped (same pattern as `job_os.rs` and `dpapi.rs`)

## Flaky test root cause

`anchor_anti_rollback_refused` used `std::env::temp_dir().join("dc-watchdog-anti-rollback-test-{pid}")`. When a prior run failed before `remove_dir_all`, the SQLite `anchor.db` remained. On the next run, `store.initialize` succeeded but `write_bytes(iid, ..., ExpectedVersion::MustNotExist)` returned `RowVersionMustNotExist`, causing `.unwrap()` to panic. This manifested under parallel load because repeated test binaries (different PIDs) could still collide if a previous process with the same PID left debris (PID reuse on Windows is common under load).

Fix: append `SystemTime::now().as_nanos()` to the directory name and `remove_dir_all` before `create_dir_all`. Same fix applied to `anchor_persist_load_round_trip` for consistency.

## Test results (final)

```
cargo test -p dc-windows -p dc-watchdog
```

| Crate | Suite | Passed | Failed |
|---|---|---|---|
| dc-watchdog | lib (anchor_store) | 4 | 0 |
| dc-watchdog | tests/admission_generation | 7 | 0 |
| dc-watchdog | tests/crash_restart | 1 | 0 |
| dc-watchdog | tests/recovery_evidence | 13 | 0 |
| dc-windows | lib (incl. 2 new named_pipe) | 14 | 0 |
| dc-windows | tests/identity_admission | 3 | 0 |
| dc-windows | tests/job_object_runtime | 0 (3 ignored — runtime-only) | 0 |
| **Total** | | **42** | **0** |

New named_pipe tests:
- `named_pipe::tests::server_client_round_trip` — ping/pong over a real `\.\pipe\dc-windows-test-*` pipe
- `named_pipe::tests::server_reports_real_client_pid` — server calls `GetNamedPipeClientProcessId` and asserts the returned PID equals `std::process::id()`

## Verification commands

```bash
source .local/hermes-organization-20260916/build-env.sh
cargo build -p dc-windows
cargo test -p dc-windows
cargo test -p dc-watchdog
```
