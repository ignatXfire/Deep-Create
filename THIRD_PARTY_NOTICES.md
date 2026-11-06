# Third-party material and provenance

Deep Create's [Apache-2.0 grant](LICENSE-STATUS.md) covers original project material only. It does not relicense dependencies, third-party quotations, trademarks, or other material for which the owner does not hold rights.

## Dependency inventory

The November 6, 2026 publication review enumerated **486 Rust and 121 npm package/version pairs** from the included lockfiles and direct npm declarations, including retained historical snapshots. [DEPENDENCY_LICENSES.json](publication/DEPENDENCY_LICENSES.json) records the declarations, license expressions, metadata provenance, and metadata hashes for all 607 pairs. All entries had a declared license resolved during this review. This inventory is not the resolved dependency graph for one particular target or build.

Examples include React, Tauri, serde, rusqlite, and Windows bindings. Their licenses remain their own. The inventory includes permissive licenses, Unicode/data terms, **17 MPL-2.0 entries**, and alternative-license expressions that include LGPL. An `OR` expression offers alternatives; it does not automatically impose every listed alternative. Historical slash-separated declarations are preserved verbatim rather than silently normalized into a legal conclusion.

No installed `node_modules`, Cargo registry sources, build products, or dependency binaries are bundled in this source snapshot. Lockfile references are not copies of those packages. Before distributing an executable, installer, vendored dependencies, or a derivative, review the actual selected dependency graph, the upstream license texts and notices, any license choices, and applicable source/redistribution obligations. This source inventory is not a binary-distribution compliance bundle.

## Publication media

The package includes an 88-second animated overview, its poster, the approved project logo, and the owner's original 39:30 AI-generated audio podcast under `assets`. The short video contains music by Sascha Ende under CC BY 4.0 and Kenney sound effects under CC0; preserve the attribution when redistributing it. The Apache-2.0 project grant does not replace those terms. [MEDIA_CREDITS.md](assets/MEDIA_CREDITS.md) identifies the sources and modifications. Media-production tools, font files, private recordings, and the production workspace are not bundled.

## Originality and limits

The first-party implementation code was generated entirely by AI systems under Ignat's direction. This statement does not describe the authorship of third-party dependencies. Ignat initiated and directed the project; it is not a claim that generated code is correct, safe, or independently proven original. The owner does not recall knowingly importing third-party source beyond declared dependencies, but cannot certify the provenance of every generated fragment. Review actual implementation paths, tests, and their assumptions independently before execution or reuse.

A lexical review of included source files found no explicit copied/adapted markers or third-party copyright/license headers matching the documented patterns. That negative result **does not prove absence of unmarked copying**. The review did not perform code-similarity analysis, authenticate every cached metadata record against package archives, or establish the origin of every icon, quotation, generated fragment, or historical artifact. Repository URLs in dependency metadata are upstream references, not endorsements.

If you identify misattributed or incompatible material, preserve the exact file/revision and provide the upstream source and applicable terms. A maintained fork should correct attribution, replace the affected material, or obtain permission as appropriate; it must not erase another party's notices. No continuing response or remediation service from the original owner is promised.

## Release scope

This is an unsupported research snapshot, not a completed application, legal opinion, security certification, patent clearance, or statement that every potential downstream use is compliant. See [research status](RESEARCH_STATUS.md), [validation](publication/VALIDATION.md), and [security reporting](SECURITY.md).
