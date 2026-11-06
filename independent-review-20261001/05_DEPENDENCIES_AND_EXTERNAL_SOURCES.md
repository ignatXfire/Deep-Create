# Dependencies and external engineering foundations

Local manifests/lockfiles are saved. A full CVE/license/SBOM audit of all transitive packages was not performed; The addiction is not declared safe due to the lack of advisory known to me. I don’t suggest updating the entire stack for the sake of these defects.

| Technology | My assessment in relation to the mechanisms found |
|---|---|
| Rust | Suitable for the owner of the state and closed refusals. Type safety does not prevent false Qualified/Passed; Send guard requires correct thread-bound type |
| Tauri/React | Suitable for thin UI. The current errors are in DTO/parser/caller paths; Tauri as a library has not been proven to be defective |
| SQLite/rusqlite | Smart local store with one writer. TEXT schema comparison, u64 casts and evidence provenance errors - usage/contract errors |
| Tokyo/HTTP | Implements concurrency and transport. Job admission, unknown and budget remain the responsibility of the application |
| windows-sys | Provides the necessary OS APIs. The created profile and Job do not automatically apply the AppContainer token; thread impersonation requires its own lifecycle |

Primary sources reviewed in the new audit 10/01/2026:

- [SQLite: Datatypes](https://www.sqlite.org/datatype3.html) - TEXT affinity and comparison rules. Validates the reason schema '10' < '3'; The defect is reproduced by this SQLiteStore separately.
- [Microsoft: RevertToSelf](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-reverttoself) - stopping impersonation and the need to close the process if it fails. Supports guard error handling, does not prove actual operation of the current application.
- [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html#section-9.2.2) - non-idempotent automatic retries require justification; [Retry-After](https://www.rfc-editor.org/rfc/rfc9110.html#section-10.2.3) allows seconds/date.
- [SQLite developer forum](https://sqlite.org/forum/info/5d26a13fcb4c7013e2b619a98c486e51dfe02d133e7914f5140683ecc20e17dd) - an example of a discussion of TEXT comparison with reference to the official affinity rules. Used as context; The output is based on the specification and our probe.

There are no statements “the developers consider the entire Deep Create production-ready”: the development sources found describe the mechanism of the library, and not a review of our project.

Next dependency audit: exact resolved versions of runtime/build/dev, RustSec/GHSA/OSV and reachable API, supply chain/licenses, separate candidate for each required migration, real regression tests. Lockfile versions and successful compilation do not in themselves prove the absence of vulnerabilities.
