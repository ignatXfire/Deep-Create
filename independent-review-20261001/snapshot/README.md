#DeepCreate

A separate personal application for long-term work with language models. It does not integrate into Hermes and does not receive its profile, keys or operational data.

**The complete system is not ready yet.** Here are the initial contracts and the first implemented parts. The current window only shows the real version of the application; Job, dependency, permission, and resource rules are checked separately in Rust. There is no connected task runner yet.

## Where to watch

- [What is really created and what remains](docs/implementation/STATUS.md).
- [Build on Windows](docs/implementation/BUILD.md).
- [Saved architectural contracts](docs/technical/00_BRANCH_NAVIGATOR.md).
- [Work schedule, results and transitions](docs/implementation/WORK_DOMAIN.md).
- [Associated change in result and unit](docs/implementation/RESULT_TRANSITIONS.md).
- [Relationship of the result with the observed flow rate](docs/implementation/USAGE_BINDINGS.md).
- [Return of the total by the owner and influence on other works](docs/implementation/OWNER_RETURN.md).
- [Acceptance of refusal before analyzing the reason](docs/implementation/OWNER_REJECTION.md).
- [Strict form of interprocess messages ](docs/implementation/PROTOCOL_CODEC.md).
- [Analysis of external comments](docs/implementation/REVIEW_20260910.md) and [checking the general edition of the graph](docs/implementation/TOPOLOGY_REVISION.md).
- [Development delegation and measured delays](docs/implementation/DELEGATION.md).
- [Permissions and resource accounting](docs/implementation/AUTHORITY_RESOURCES.md).
- [Comparison of maintenance of different works](docs/implementation/SCHEDULING.md).
- [Window shell](docs/implementation/DESKTOP_STAGE1.md) and [test of real window](docs/implementation/NATIVE_SMOKE.md).

## Checking the current edition

After preparing the build environment in PowerShell from this folder:

```powershell
.\tools\Verify.ps1
```

The script does not change Windows settings and does not access models. It performs checks, repeats the build with dependencies attached, and saves a report with fingerprints of the checked files. A failed check leaves a failure report and stops the chain. Success applies only to existing code - not to contracts not yet implemented.

Simple check for packages and forbidden direct dependencies:

```powershell
node tools/check-architecture.mjs
```

This is an additional source check and is not a replacement for Windows process restrictions. The result of model text analysis never in itself gives the right to read files, change rules, or perform external actions.
