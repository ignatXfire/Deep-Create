# Dependencies: what you should and shouldn't change

Checked local manifests/lockfiles and official docs below, accessed 2026-09-29. A full CVE/SBOM/license audit of all transitive dependencies has not been performed. This file does not claim that all current versions are safe, nor does it recommend updating everything at the same time.

| Choice | My assessment of Deep Create |
|---|---|
| Rust | Suitable for state owner and obvious failures. Does not prevent logically false acceptance and does not create an OS sandbox on its own |
| Tauri + React | Reasonable thin shell. The problem now is the consistency of the IPC DTO and credentials, not a proven defect Tauri |
| SQLite/rusqlite bundled | Suitable for local application with one writer. CAS and transactions are useful; REPLACE for immutable proofs is a usage error, not a reason to change SQLite |
| Tokyo/HTTP client | Suitable for the transport, but the unknown/idempotency boundaries belong to the application. Repetitions cannot be cured by just replacing the library |
| windows-sys | Provides the necessary OS APIs, but the AppContainer/Job profile must be used correctly. We need real token checks and negative tests |

## Primary sources and inference boundaries

- [Tauri v2: Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/#passing-arguments): default IPC keys camelCase, snake_case require corresponding rename_all. Confirms F01; does not prove other aspects of the application's security.
- [SQLite: ON CONFLICT](https://www.sqlite.org/lang_conflict.html): REPLACE removes the conflicting old row before the new entry. Confirms mechanism F09; this is the expected behavior of the library.
- [Microsoft: Launch an AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer): describes the security capabilities when creating a process. Confirms why one profile and Job are not enough for F07.
- [RFC 9110 §9.2.2](https://www.rfc-editor.org/rfc/rfc9110.html#section-9.2.2): automatic retry of a non-idempotent request requires knowledge of idempotency or proof of non-use of the previous one. Supports F05, does not prove specific provider write-off.
- [RFC 9110 §10.2.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-10.2.3): Retry-After allows seconds/date. Confirms F06.

Forum opinions are not used as evidence. There are primary specifications for the specific mechanisms found. There are no statements here: “the developers consider the entire project bad/good”: they did not review this project.

Next dependency audit: extract exact versions from Cargo.lock/pnpm-lock, build runtime/build/dev SBOM, check RustSec/GHSA/OSV for specific versions and reachable API; evaluate the update in a separate branch with regressions. The absence of an advisory found does not prove the absence of vulnerabilities.
