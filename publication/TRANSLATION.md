# English publication and source fidelity

The publication-facing README, documentation map, research status, build guidance, security guidance, and research overview were written in English for this release candidate. Historical Russian Markdown prose was translated with machine assistance and checked for protected code blocks and Markdown link targets.

Identifiers, source-code quotations, literal fixture values, protocol strings, and historical command output are preserved where their identity matters. Some non-English text may therefore remain inside code quotations, canonical reference labels, or test data. That is not evidence of a missing English prose section. Historical document filenames were converted to English and internal references updated. Current UI labels, their matching assertions, source comments, helper diagnostics, and contract-map descriptions were localized without changing canonical contract IDs or status fields.

Machine translation is not an independent engineering validation. The translation was spot-checked for terminology and meaning, but no complete professional bilingual review is claimed. If a normative ambiguity is discovered, compare against the private original retained by the owner before using it as an implementation authority.

Do not use original historical hashes to authenticate translated or redacted files. `PUBLIC_MANIFEST.json` identifies this public snapshot. Product behavior was not modified merely to hide failed checks. See `VALIDATION.md` for the actual checks performed during packaging.
