# Deep Create — Forensic Defect Re-Verification 2026-09-30

Branch: `codex/deep-create-foundation` @ `1dc45a8`
Method: verified each defect from `DeepCreate_Forensic_Handoff_2026-09-23.zip` (04_DEFECT_REGISTER.md) against LIVE source on current HEAD. Test counts are NOT treated as readiness proof; each item below cites the actual file/line evidence.

## P0 defects

### P0 — ARCH-CHECKER-UNREGISTERED — FIXED
`tools/check-architecture.mjs` now registers all 9 workspace crates:
dc-domain, dc-protocol, dc-store, dc-windows, dc-watchdog, dc-desktop, dc-broker, dc-worker, **dc-core** (with per-dependency justification strings).

### P0 — BROKER-STUB — FIXED
`crates/dc-broker/src/dispatch.rs` now performs real HTTP dispatch:
- `dispatch_work_unit()` uses `reqwest::Client` with timeout from `endpoint.config()`
- Exponential backoff retry (1s, 2s, 4s…), 429 `Retry-After` honoring, 400 → `Refused`
- Parses OpenAI-compatible `ChatCompletionResponse`, extracts content/model/tokens
- `stream: false` explicitly set (OmniRoute defaults to SSE otherwise)
- Separate `dispatch_work_unit_mock()` retained ONLY for tests
- Comment at top: `P0-BROKER-STUB fix: Real HTTP dispatch using reqwest`

### P0 — WORKER-BROKER-EDGE — FIXED
`crates/dc-worker/Cargo.toml`: `dc-broker` moved to `[dev-dependencies]` only.
Comment: "Production code never links dc-broker — the WorkDispatcher trait is the only interface."
Worker→broker edge exists only for the live_e2e test proving real dispatch works.

## P1 defects

### P1 — ADMISSION-LIVENESS-TYPE — FIXED
`crates/dc-windows/src/admission.rs:62` — `HeldAdmission<'a>` is a real RAII guard
holding `&'a mut AdmissionGate`. Boundary verification takes `&HeldAdmission`,
not a clonable receipt.

### P1 — ADMISSION-TOKEN-COPY — FIXED
`AdmissionToken` (line 41): `#[derive(Debug, PartialEq, Eq)]` — deliberately
NOT Clone, NOT Copy. Private `fn new` — only the gate can mint. Consumed by
`leave_admission`.

### P1 — AUTH-TIMESTAMP-OVERFLOW — FIXED
`crates/dc-protocol/src/auth.rs:118,125`:
- `proof.timestamp > context.current_time.saturating_add(MAX_FUTURE_SKEW)` (future-skew bound)
- `.checked_add(context.max_proof_age)` (overflow-safe expiry)
- Regression tests at lines 415, 434, 445 exercise `u64::MAX` boundary cases.

### P1 — CORE-MISSING — FIXED
`crates/dc-core/` exists on this branch (added in `20f52a1`). Contains
command/scheduler/context/dispatch/persistence/verification/self_development
modules. Single-writer controller implemented.

### P1 — DC-BROKER-CONTRACT-LABEL — FIXED
`crates/dc-broker/src/dispatch.rs:3` — doc comment: "The broker (C-09)…" Correct C-09 label.

### P1 — DC-WORKER-CONTRACT-LABEL — FIXED
Worker Cargo.toml comment references C-11 correctly.

### P1 — UI-FIXTURE — FIXED
`crates/dc-desktop/src/main.rs:63` — `CoreStateSource` reads live from
`CoreState.graph`. Lines 143-144, 277, 441: "no fake fixture data",
`workUnitsCount: 0` until a real graph is accepted. Fixture path retained only
as an explicitly-marked empty fallback.

### P1 — STORE-DURABILITY-GAPS — FIXED
`crates/dc-store/src/sqlite.rs:191-204`: `PRAGMA journal_mode = WAL`,
`synchronous = FULL`, `foreign_keys = ON`, `trusted_schema = OFF`,
`busy_timeout = 5000`, `wal_autocheckpoint = 1000`. Line 448: `PRAGMA
integrity_check` exposed. In-memory path skips WAL (unsupported) but keeps
FULL+FK.

## Verified on 2026-09-30 via full workspace gate

- `cargo test --workspace --offline --no-fail-fast`: **40 test binaries, all `test result: ok`, 0 FAILED / 0 panicked** (~427 tests passed; grep-based counting, exact per-binary totals in raw log).
- `cargo check --workspace --offline`: zero warnings, zero errors.
- Release artifacts present:
  - `.local/.../target/release/dc-desktop.exe` (12,518,912 B)
  - `bundle/msi/Deep Create_0.1.0_x64_en-US.msi` (5,025,792 B)
  - `bundle/nsis/Deep Create_0.1.0_x64-setup.exe` (3,623,995 B)

## What is STILL NOT PROVEN (honest limits)

These were NOT claimed fixed and remain open gaps vs the forensic verdict:

1. **Real E2E UI↔Core round-trip** — no automated WebView test drives the actual
   running desktop app through submit → plan → dispatch → verify → accept.
   Statically the 9 Tauri commands are registered; runtime invocation is proven
   only by unit/integration tests, not a live GUI session.
2. **dc-windows native enforcement** — Job Objects / AppContainer / named-pipe
   ACL / DPAPI are still typed scaffolding, not opened OS handles. Windows
   security runtime readiness stays near the forensic 0–10% estimate.
3. **Real model dispatch to a live endpoint** — broker code is real HTTP now,
   but no test in the workspace calls a live external model; `live_e2e` tests
   use the dev-only broker path against a local/mock endpoint.
4. **Stage 3 canonical schema** — durability pragmas are in, but the full
   canonical event journal / resource ledger / evidence archive / encryption /
   sealed store key from the C-03 contract is not complete.
5. **Installer validation** — MSI/NSIS exist but were not installed on a clean
   machine, not uninstalled, not launched post-install.
6. **Crash recovery** — persistence round-trip tests pass, but no test performs
   a real OS-level process kill of a running desktop instance followed by
   restart-and-recover verification.

## Bottom line

The forensic handoff's P0 and P1 register is genuinely closed on this branch —
verified against live source, not status reports. The previous "~99% ready"
claims in `CURRENT_STATUS_2026-09-30.md` were still inflated because they
counted test totals and `.exe` existence. The honest position: **the reviewed
engineering-implementation layer is now solid; the *proven-at-runtime* layer
(live E2E, native Windows enforcement, real model dispatch, installer
validation) is the remaining frontier.**
