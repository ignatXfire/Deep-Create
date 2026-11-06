# Publication validation — November 6, 2026

This report concerns a sanitized, English-language research snapshot. It does not certify the full application or replace the independent product audits.

R3 adds owner-approved public attribution (Ignat, @ignatXfire), Apache-2.0 licensing of original material, citation and continuation policies, and a bounded third-party dependency review. No implementation fix is claimed. Publication code and test content are compared with R2; licensing metadata and release documents are the intended changes.

| Check | Actual result | Scope |
|---|---|---|
| Original-file preservation | 1,017 inventoried source files rehashed; no changes | Compared with the private copy-time manifest; not a hash of every excluded runtime file |
| English Markdown | No Cyrillic prose found outside protected code/literals | Includes primary documents and retained audit snapshots |
| Quoted source fidelity | Fenced blocks match in 293 retained historical Markdown files | Compared with sanitized source; published filename mappings applied |
| Local Markdown links | All checked local file targets resolve | Omitted screenshots are explicitly labeled; omitted ZIPs point to included review directories; external URLs and heading anchors are not certified |
| Secret scan | Gitleaks 8.30.1: exit 0, no findings | Filesystem scan of the public tree, including retained source snapshots; original Git history is not included |
| Scanner negative control | Exit 2, one expected synthetic credential finding | Private fixture; neither fixture nor its secret is included |
| Dependency license metadata | 486 Rust and 121 npm package/version pairs; no unresolved declarations | All included lockfiles plus direct npm dependencies; not proof of source originality or binary-distribution compliance |
| English UI type check | Exit 0 | Installed snapshot dependencies reused in an isolated validation tree |
| English UI production asset build | Exit 0 | Vite build; not a native desktop release or clean-machine installation |
| UI tests before localization | Exit 1; 50 passed, 3 failed | Sanitized source baseline in an isolated tree |
| UI tests after localization | Exit 1; 50 passed, 3 failed | Same three failing test names as the baseline |
| Architecture contract checker | Exit 1 | Existing dependency-contract drift retained, not suppressed |

The three UI failures concern command exposure expectations, native dependency expectations, and a work-graph banner assertion. They were already present in the baseline. Equal failure counts alone are not the evidence: the failing test names were also compared. This is not a claim that all behavior is unchanged under translation.

Personal-path/private-name checks, JSON parsing, excluded-file checks, and packaging inventory checks are recorded in the adjacent machine report. The owner explicitly approved publishing the name Ignat and the X profile @ignatXfire; that approval does not extend to a local Windows username, workstation paths, or other personal data. A lexical scanner cannot prove the absence of every secret or vulnerability. The preparation intended to exclude private conversations, Git history, operational databases, caches, credentials, and workstation inventories. A later manual review found residual host metadata and development paths despite the earlier scan; the November 7 cleanup redacts those records. See [privacy boundaries](PRIVACY.md). Existing GitHub commit metadata and uploaded NotebookLM sources require separate cleanup; this file does not certify their current contents.

No complete Rust workspace/native release build, clean Windows VM test, live provider connection, or full bilingual engineering review was performed for this publication. See [translation limitations](TRANSLATION.md), [scope](PUBLICATION_SCOPE.md), and [research status](../RESEARCH_STATUS.md).

`PUBLIC_MANIFEST.json` records the actual public file sizes and SHA-256 hashes. It excludes itself to avoid a circular hash. Historical hashes elsewhere describe older, possibly unredacted or untranslated snapshots.
