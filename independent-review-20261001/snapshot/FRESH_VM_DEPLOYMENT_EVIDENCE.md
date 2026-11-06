# Historical deployment evidence — publication-redacted copy

**Original report date:** 2026-10-01. **Publication redaction:** 2026-11-07.

Owner-specific machine identity, OS build/locale, firmware capability, installed-software inventory, process-directory names and local run locations have been removed. This is a redacted historical report, not a fresh validation run.

## What the historical report claimed

A true fresh Windows VM test could not be completed in the available environment. An isolated on-machine procedure was used instead: stage the release binary, start with a constrained environment and new database, stop and relaunch, then inspect database persistence and uninstall residue. This does not establish a clean-VM deployment or an OS sandbox.

The report described two on-machine runs and claimed successful binary launch, a SQLite header/schema, persistence across relaunch, staging cleanup with database preservation, and no detected Deep Create residue in its targeted comparisons. It also acknowledged that GUI rendering was inferred from liveness/runtime presence rather than screenshots and that one shell capture ended before its final status lines. These remain historical claims, not re-executed acceptance evidence.

## Retained audit corrections

[ADVERSARIAL_AUDIT_WAVE4.md](ADVERSARIAL_AUDIT_WAVE4.md) challenges stronger claims in the original report: registry byte equality was overstated in some runs, and the environment-isolation assertion accepted a zero-byte dump. A claimed PASS must not be inferred from those checks alone. The reported VM test was never completed.

See [DEPLOYMENT_EVIDENCE.md](DEPLOYMENT_EVIDENCE.md) for the separate historical deployment procedure. The publication privacy pass did not run an installer, a live provider call, a desktop build or a fresh-VM test. Host metadata redaction is not a product fix or a new deployment certification.
