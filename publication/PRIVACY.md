# Publication privacy boundary

Updated November 7, 2026. This document concerns publication data, not certification of the experimental application.

## Deliberately public identity

The owner approved publishing the name **Ignat** and X profile **[@ignatXfire](https://x.com/ignatXfire)**. This does not authorize publishing a personal email, workstation profile, credentials or private session data.

## Privacy redactions

The publication copy removes owner-specific OS/build/locale, firmware and virtualization observations, installed-software names, runtime file inventory/timestamps, and original development paths. Historical duplicate snapshots receive the same redactions. Host-specific sections are explicitly marked as redacted; they must not be treated as verbatim forensic evidence after publication processing.

Historical findings, test failures, missing evidence and the lack of a completed fresh-VM deployment test remain visible. Redaction does not turn a failed, disputed or unperformed check into a PASS. Technical contracts and application implementation are not being redesigned by this cleanup. Historical hashes refer to their original dated artifacts, whereas `PUBLIC_MANIFEST.json` records this publication tree.

Build helpers use a worktree-relative or explicitly configurable toolchain location instead of the owner's original location. They remain historical utilities, not a newly certified portable build system. Inspect them before execution.

## Git metadata is separate

Clean source files do not sanitize an existing Git history. Before public release, check all reachable commits, tags and branches, including both author and committer fields. Use the owner's actual GitHub no-reply address for new commits and enable private email for GitHub web edits. Do not copy the development project's `.git` directory. A future no-reply commit does not remove an email in an earlier commit.

## NotebookLM is a separate copy

Replace the notebook's old project-source documents with the regenerated privacy-cleaned source set; uploading a summary does not remove private text from older sources. Verify the actual uploaded source list and sharing permissions. A chat-only interface is not a redaction boundary.

The Markdown corpus quotes code, tests, scripts, configuration and documentation with explicit original paths. Small binary assets are losslessly encoded and labeled as such. Audio/video are supplied separately, with their original project paths and hashes recorded in the export; their bytes are not explanatory prose. Do not upload private preparation/audit directories or old source sets alongside the cleaned set.

## Limits

Manual inspection and secret scans reduce accidental disclosure; they cannot guarantee absence of every secret, vulnerability or hidden media payload. No credential-validation calls, live provider requests or account-setting changes are part of this privacy cleanup. Published research code must still be independently reviewed before execution.
