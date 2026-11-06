# Deep Create: Work Links and Shared Workspaces

**Date:** 2026-09-08  
**Status:** clarification of the technical project to the application code; the listed checks have not yet been performed  
**Area:** points 1 and 5 of external criticism; linking the main work through verified results and agreeing access to one real work object  
**Base:** documents 14, 16, 17, passports C-01—C-16 of document 19, matrix 21; exact form `DependencySnapshot` verified with 15 §2.5a

## 1. What was really missing and what has already been decided

The criticism is not accepted as a proposal to rebuild the system, but as a demand to close two specific gaps in its existing contract.

| Verified site | What has already been determined | What this doesn't prove yet |
|---|---|---|
| 14 §4.3 and §4.5 | Several `root_works`, each with one active edition of the goal and its own history | There is no form of the promised output of one main work for another, rules for its provision and handling of change of source |
| 14 §4.9; 16 §3 | `work_dependencies` links units; the cycle is prohibited until the graph is accepted | There is no interworking agreement that distinguishes the expectation of a future result from reading an already saved edition; there is no express condition of final acceptance by the owner |
| 14 §4.12—4.15; 16 §13; 15 §2.5a | Accepted packages, checks, proofs and an immutable set of read dependencies; extraneous `EventSeq` does not make the result obsolete | The named output of the main work, the choice of its editor and the exact consumers affected by the change of this output have not been determined |
| 16 §4 and §12 | Separate unit of union and verification of the combined result | The recipient of the merged branch and the collision order of two main works on the same user file are not defined |
| 17 §10 and §12.1 | Separate Copy, File Real Identity, Original Content, Revocation Fencing, Intent and Unknown Outcome Reconciliation | There is no common consideration for the main works of intersecting applications and the rules for its restoration |
| 19 C-03/C-05/C-09/C-13 | One Subject Writer, Individual Consequence Mediator, Controlled Recovery | You cannot add another writer, a second money magazine, or a second independent right called `WorkspaceLease` |
| 21 D-03 and I-12 | Several works on the map; isolation and merging of branches | Checking two jobs in the interface does not prove the correctness of the transfer of the result or the simultaneous change of the common object |

Thus, separate communication data is needed, but a new management service is not needed. The agreement continues on the path of `предложение модели → проверка цели и границ → C-05 → C-03 → проверяемое событие`. The rights and all real consequences remain with the C-09 with the C-10/C-12 guardrail. LLM saves the work of meaningfully selecting connections, parsing influence, finding compatible associations, and testing meaning; the machine part does not guess these decisions, but checks their form, basis and limits.

## 2. Three differences without which communication is dangerous

### 2.1. Package accepted, exit provided, and final acceptance of work

1. `result_packets.status = accepted` means that a particular package has passed the required check and has been accepted by the control part for its unit. This is not the final acceptance of the main work.
2. `ResultPublication` means that the exact accepted package or specified materials from its list are registered as the output of the main work available under the contract. Here, "giving" is an internal fact of the Deep Create directory. It doesn't send content out, publish the page, shut down, or grant read permissions to the consumer.
3. `root_works.stage = owner_accepted` arises only after the owner's trusted decision on the exact final candidate. The state before it is called `awaiting_owner_acceptance` according to document 14. The unequal abbreviated names in 16 do not create two more states.

Job B can, by accepted contract, use A's verified intermediate output while A is still running or awaiting final acceptance by the owner. If it is A's final acceptance that is essential to B, this is a separate explicit condition of connection. Replacing “verified” with “accepted by the owner” is prohibited in the database, readiness and interface.

### 2.2. Result binding is not a transfer of authority

Communication tells **what** the consumer needs. `CapabilityLease` tells **what action is allowed** for a specific job, on what data, and within what limits. The second does not follow from the first.

Creating a link does not copy A's rights to B, turn read rights into writes, allow model expansion, or spend A's limit on behalf of B. To read the result of A, work B gets its own valid right to the exact area/fragments issued by the existing trusted path. Having A or General Installation Owner rights does not replace this step. If the area is already explicitly authorized by B, reconfirmation of the unchanged permission is not required, but the suitability of the entitlement is checked again.

Attempt, check and merge limits are taken into account in the existing `resource_ledger` for actual responsible work and general limits. A link to a result does not automatically transfer the manufacturer's remaining budget. Changing the distribution of the limit is a separate decision made.

### 2.3. Origin story does not equal fulfillment expectation

Reading stored `A.report@8` does not mean waiting for all of A to complete. The reference "derived from A" does not mean disabling B. Prohibiting B must have the exact output required and the exact dependent unit or obligation. Therefore, a universal edge is not introduced, the meaning of which is determined by a free phrase of the model.

## 3. Minimal canonical entries

All entries listed are in `deepcreate.db`, are modified only through C-05/C-03, and have single transaction events. Immutable revision fields are never corrected in place. The new edition keeps a link to the previous one; the current state and index of the current edition are protected by `RowVersion`.

### 3.1. Providing the result

```rust
struct ResultPublication {
    publication_id: EntityId,
    producer_work_id: EntityId,
    producer_objective_id: EntityId,
    producer_unit_id: EntityId,
    producer_unit_revision: u32,
    export_name: ExportName,
    export_revision: u32,
    result_id: EntityId,
    result_fingerprint: ContentFingerprint,
    output_manifest_evidence_id: EvidenceId,
    verification_id: EntityId,
    verification_policy_ref: VersionedPolicyRef,
    predecessor_publication_id: Option<EntityId>,
    created_event_id: EntityId,
}

struct PublicationState {
    publication_id: EntityId,
    eligibility: PublicationEligibility, // Valid | Unresolved | Withdrawn
    reason_evidence_id: Option<EvidenceId>,
    row_version: RowVersion,
}

struct PublicationBinding {
    binding_id: EntityId,
    producer_work_id: EntityId,
    export_name: ExportName,
    head_publication_id: EntityId,
    row_version: RowVersion,
}
```

`ExportName` is the qualified exit name being verified, not an object identity or path. The `(producer_work_id, export_name, export_revision)` pair is unique; the editorial is monotonous and not reused. The package and list of outputs define the exact materials and limitations, not the entire contents of work A. Changing `PublicationBinding` does not change the bytes of the previous result.

Submission is allowed after acceptance of the exact package, sufficient validation, verification of its current dependencies, and a durable checkpoint. Having an arbitrary `verification_id` is not enough: the check must relate specifically to the candidate's fingerprint and a pre-adopted policy. Open outcome restrictions remain; an assumption cannot be presented as an established fact. Testing level `pass` for the question “explore options” does not, in itself, prove that the conclusion is sufficient for subsequent dangerous action.

`PublicationState.Unresolved` means that the validity of the result is affected but not yet established; `Withdrawn` - The result was withdrawn as a valid basis for a specific reason. Old bytes, checks and the fact of provision remain in history. The usual revocation of the right to read is not replaced by the revocation of the truth of the result: these are different reasons for refusal.

### 3.2. Interworking Communications Editorial

```rust
enum RootWorkRelation { RequiresResult, ConsumesArtifact, Informs }
enum PublicationSelector {
    Pinned { publication_id: EntityId },
    FollowHead { binding_id: EntityId },
    AwaitOutput { producer_unit_id: EntityId, producer_unit_revision: u32, export_name: ExportName },
}
enum OwnerAcceptanceRequirement {
    NotRequired,
    RequiredForPublicationObjective,
}
enum SourceChangePolicy { KeepPinned, RequireRevalidation }
enum LinkCondition {
    Always,
    All { atoms: NonEmptyVec<ConditionAtom> },
}
enum ConditionAtom {
    ObligationHasStatus { obligation_id: EntityId, status: ObligationStatus },
    AcceptedFieldEquals { publication_id: EntityId, field: SchemaFieldId, value: CanonicalScalar },
}

struct RootWorkLinkRevision {
    link_id: EntityId,
    revision_id: EntityId,
    predecessor_revision_id: Option<EntityId>,
    producer_work_id: EntityId,
    consumer_work_id: EntityId,
    consumer_objective_id: EntityId,
    consumer_targets: NonEmptyVec<ConsumerTargetRef>,
    relation: RootWorkRelation,
    selector: PublicationSelector,
    required_output_schema: SchemaVersion,
    required_verification_level: VerificationLevel,
    owner_acceptance: OwnerAcceptanceRequirement,
    condition: LinkCondition,
    freshness_policy_ref: VersionedPolicyRef,
    on_change: SourceChangePolicy,
    accepted_basis_ref: EntityId,
}
```

`ConsumerTargetRef` has closed options for the exact edition of the unit or the obligation of the accepted goal. The words "phase X" must be resolved to these numbers when accepting a connection; a free stage mark does not block execution. `accepted_basis_ref` refers to the graph/goal's accepted decision or owner's trusted event, but not to the model's self-assertion.

`root_work_links` stores `link_id`, `active_revision_id`, `status = active|cancelled|superseded`, `row_version`; `root_work_link_revisions` stores an immutable form. Cancel does not remove the link from history. The interworker communication and its consuming units are accepted consistently, with the expected versions of the affected targets and the version of the graph region; two simultaneously accepted edges do not have to pass the test separately, but together form a cycle.

`RequiresResult` expresses the required proven base, `ConsumesArtifact` additionally specifies the exact materials of the required shape; both participate in the readiness of said consumers. `Informs` - optional information basis: its absence does not delay the launch. But if the material is actually included in the assignment or used as a result, it falls into the dependency snapshot and deprecation rules. The name of an optional relationship does not give the right to hide the actual reading.

`derived_from` is stored as the origin of the result; `supersedes` - as the attitude of the editors; `blocks_until` is expressed by a mandatory connection with the readiness condition. These meanings do not require three more competing types of planning dependence.

`Pinned` allows `KeepPinned` or the stricter `RequireRevalidation`; `FollowHead` requires `RequireRevalidation`. `AwaitOutput` indicates a specific accepted pair `(producer_unit_id, producer_unit_revision)`, rather than an infinite promise of "someday A will give something away." The result provided must belong to this pair; a new result from another edition of the same unit does not satisfy the old expectation. `producer_objective_id` already denotes a separate, unchangeable edition of the target under 14 §4.5 and does not need a second person. After the output appears, the choice is fixed in the snapshot of a specific consumer; a future change of source does not correct the old snapshot.

`RequiredForPublicationObjective` requires a trusted record of acceptance of the final candidate **of the edition of Objective A to which the submission relates**, including this output in the accepted list. Historical adoption of the former goal A does not satisfy the new requirement. A link to the exact record of acceptance is kept separately; stage A is not sufficient on its own. A contract requiring a permanent final outcome additionally reads the binding of the current goal A and therefore reacts to its change.

### 3.3. Terms and snapshot of what you read

`ConditionAtom` is a closed machine form and not a code or model instruction. The number of atoms is limited by the circuit. The field value is taken from the accepted schema and verified result, not from arbitrary text. Condition checking has three outcomes: `True`, `False`, `Unknown`.

- `True`: Mandatory communication must be satisfied.
- `False`: communication is not required for this consumer edition; The calculation result and the read conditions are still saved.
- `Unknown`: Selection dependent branch does not receive positive tolerance. Missing data is not considered a lie.

The validity of the links is appended to the existing `DependencySnapshot.read_set` rather than creating a second snapshot view. `DependencyKind` adds `RootWorkLink`, `ResultPublication`, `PublicationState`, `PublicationBinding`, `OwnerAcceptanceBinding`. The same `entity_id`, `expected_revision` and protected `content_hash` are used. The unchangeable `ResultPublication` and the changeable `PublicationState` are different read items: for the second `entity_id = publication_id`, and the expected revision is `PublicationState.row_version`. It is required even with `Pinned`; the validity change is checked by the canonical read_set, not just the explanatory `publication_state_version` below. `PublicationBinding` separately validates the head of the named exit under `FollowHead` and does not replace the validity state of the old grant.

The encrypted explanation of the snapshot contains:

```rust
struct CrossWorkDependencyRead {
    link_revision_id: EntityId,
    selected_publication_id: Option<EntityId>,
    condition_result: ConditionResult, // True | False | Unknown
    condition_read_refs: Vec<VersionedDependency>,
    publication_state_version: Option<RowVersion>,
    owner_acceptance_ref: Option<EntityId>,
    selected_output_refs: Vec<ScopedDataRef>,
    checked_policy_refs: NonEmptyVec<VersionedPolicyRef>,
}
```

This is an explanation of an existing snapshot, not a pass on its own to read or apply. For `FollowHead` the set includes a version `PublicationBinding`; for `Pinned + KeepPinned` the new edition of the head is not there. In both cases, the admissibility of the selected publication, its significant grounds, deadline policy and current rights are checked. Changing the conditions from `False` to `True` makes the old branch selection unusable, even if the material was not read at the time.

## 4. End-to-end history A → B → C

Let A research the market, B create a product, C prepares its release. The example describes the allowed work steps, but does not state that external publishing or trading actions are already included in the first action catalog.

1. For A, the purpose, unit of study and criteria are accepted. The contractor creates the package `A.report@8`; independent review confirms its suitability as a study within the stated limitations. C-05 accepts the result and after the checkpoint provides `publication_id=P8`.
2. B has previously accepted the relationship `RequiresResult`, the goal is a specific requirement selection unit; selection `Pinned(P8)`, final acceptance A not required. C-05 verifies compliance with Objective B and Level of Inspection. C-09 separately checks B's right to read allowed P8 fragments and transmit them to the selected connection. B starts only after both checks. A can continue with another independent unit.
3. If decision B has consequences, checking the research P8 is not sufficient by default. For a decisive approval, the check required in 16 is not lower than `DiverseChallenge`; the absence of such verification blocks the appropriate transition rather than erasing an already useful research result.
4. B creates and reviews product `B.product@4`. C can read this output to prepare draft materials. If the connection for authorized action C requires final acceptance by B, that unit remains locked to the trusted `owner_accepted` exact candidate B. The right to perform the action is still required separately.
5. The owner rejects the final completion of A, for example because the second region has not been explored. This does not rewrite P8 and does not prove all the data in the first region to be false. C-05 saves the reason; Affected statements and consumers are subject to impact testing. If the reason directly refutes the evaluation used by B or its scope is unknown, P8 receives `Unresolved` and B's dependent transitions are blocked until parsed. If it is machine proven that the cause relates only to another output, the independent branch B is not stopped. An owner's refusal does not result in either an automatic global revocation or an automatic disregard.
6. A provides P9. B with `Pinned(P8) + KeepPinned` receives an explanatory notice of the new revision, but the material is not replaced. If P8 is valid under the accepted policy, B continues reproducibly on P8. For B with `FollowHead`, the read binding changes: dependent units that have not yet been started are reassembled, the launched attempt does not quietly receive new inputs, its late candidate is not applied as relevant.
7. The LLM breaks down the P8/P9 differences, provides a list of affected requirements and minimal additional work. A separate check confirms the statement “output B remains true at P9” or requires correction. This is a new verified revision of base B, and not a replacement of the dependency number of the old result without verification.

Changing Vendor A to D is a new communication revision with the same checks for Goal B, Output Design, Independence, Conditions, Confidentiality, and Permissions. One matching report title does not make the sources interchangeable. The approved rights to A do not extend to D.

## 5. Cycles, cancellation and propagation of obsolescence

### 5.1. What exactly is checked for the cycle

The expectation graph is constructed based on unit revisions and the exact expected outputs, including the necessary final acceptance nodes. It cannot be replaced only by a map of arrows between the names of the main works.

If A1 is waiting for the future output of B1, and B1 is waiting for the future output of A1, the acceptance of the graph is rejected by `Cycle` with a minimum proven contour. Conditional binding links are checked as potentially valid; The current `False` does not allow saving the cycle that will occur when the observation changes. To exclude an edge, we need the recorded impossibility of activating it in this edition of the graph, and not the opinion of the model. Reactivation requires a new acceptance of the graph.

Mutual references to already existing P8 and Q4 are not in themselves a mutual expectation: publication is a ready-made immutable input. Information links may form an origin loop, but do not trigger recursive copying, rescheduling, or positive judgment. An intentional iteration A → B → A is described by finite generations of units, a finite number of revisions, and a general limit, such as A1 → B1 → A2. A closed graph of infinite waiting is not called depth of exploration.

### 5.2. Cancellation does not extend as someone else's power

Canceling A that has not yet produced a result removes the promise of a future output: the exact consumers receive `UpstreamCancelled`. C-05 does not cancel all of B or stop independent branches. B may suggest a different connection, a different method, or a new owner's decision about the goal; tacit withdrawal of obligation is prohibited.

Canceling B does not cancel A, other consuming jobs, or effects that have already taken place. Only B's proven unused reserves are released. If A continued to operate solely for that customer, C-05 may accept a separate justified termination of her unnecessary unit within the accepted count; the presence of an edge does not itself give B the right to stop A.

Stopping A does not destroy previously accepted results. Their continued use depends on the preserved basis, policy and applicable consumer rights. A revoked right or enclosed area does not require reading the “old copy.” Unlinking preserves previously used inputs in snapshots; You cannot retroactively remove a dependency to pass validation.

### 5.3. Accurate, long-lasting impact analysis

According to the adopted change, C-05 records the changed binding and the work to recalculate affected consumers. The backlink index is a derived accelerator and is reconstructed from canonical snapshots/links. Even if the notification or recalculation has not yet been completed, C-06, C-09 and acceptance of the result reconcile the current significant versions; losing the event by the interface does not resolve the old action.

Propagation takes into account actual dependencies read, rather than all work in a single session. Changing the head of A does not make the fixed historical P8 false; revocation of P8 validity affects everyone who actually used P8. The freshness period is also checked against the current time, even if a new source record has not appeared.

The candidate with the old base gets `DependencyChanged`/`StaleInput` with the exact source. The result of a previously accepted unit is not erased; its current suitability is subject to review. Already `owner_accepted` the work does not secretly go into an unfinished state and does not lose the historical decision of the owner: the impact and need for a new edition/subsequent work is recorded, and dependent future actions are blocked. A detected inaccuracy does not automatically roll back actual files or external effects.

## 6. Shared workspace is an object of approval, not a second right

### 6.1. Three different actions

| Accounting mode | What does | What existing law is needed |
|---|---|---|
| `ReadSnapshot` | Read the installed edition of the source files; long read is performed on a consistent snapshot | `ReadWorkspaceFile` or `ReadEvidenceFragment` for exact data |
| `BranchWrite` | Change a separate copy with one responsible job and unit | `WriteWorkerScratch` / `PrepareWorkspaceChange` in a permitted separate area; the right to read the source separately |
| `Apply` | Transfer checked difference to exact custom object when base matches | `CommitWorkspaceChange` for this object and all necessary current rights to sources/data |

Registration of the mode does not issue the corresponding right. The name `WorkspaceLease` suggested by critics is not used here so as not to confuse the resource order with `CapabilityLease`. The required entry is called `WorkspaceUseRecord`; it is planned use and mutual exclusion, not ability. Even a fake "free area" without permission prevents the man in the middle from changing the file.

### 6.2. Real personality and editorial

`WorkspaceRef` is already used in 16. It gets the exact contract: the random `workspace_id` refers to a registered binding to the authenticated root directory; the displayed path is not a person. In the original Windows implementation, the binding includes the volume number, directory number, valid redirection point policy, and `binding_version`. Changing the binding does not reuse the old base.

```rust
struct WorkspaceRevision {
    revision_id: EntityId,
    workspace_id: EntityId,
    binding_version: RowVersion,
    manifest_evidence_id: EvidenceId,
    manifest_fingerprint: ContentFingerprint,
    observed_by_intent_id: EntityId,
    parent_revision_id: Option<EntityId>,
}

struct WorkspaceUseRecord {
    use_id: EntityId,
    work_id: EntityId,
    unit_id: EntityId,
    mode: WorkspaceUseMode, // ReadSnapshot | BranchWrite | Apply
    source_revision_id: EntityId,
    target_manifest_evidence_id: EvidenceId,
    private_workspace_ref: Option<WorkspaceRef>,
    intent_id: Option<EntityId>,
    run_generation: RunGeneration,
    state: WorkspaceUseState, // Prepared | Held | Observed | Unresolved | Released
    row_version: RowVersion,
}
```

The revision list contains, for each object read or changed, a verified binding to a parent directory, an existing object or a verified absence of a name, the type of operation, and a fingerprint of the exact contents. The file number does not replace the fingerprint, and the fingerprint does not replace the authorized location. Deleting/creating, listing a directory, and essentially missing a file are also dependencies.

Two names, case, short name, directory renaming and nested working roots should not hide the intersection of real objects. Failure to prove non-overlapping means a wider conflicting area or failure rather than optimistic parallel application. Unexpected hard links, extra flows, network/device paths, and redirection points are rejected by 17; this document does not declare them supported. The Git snapshot itself does not cover uncommitted and untracked files, so the commit ID does not replace the complete list required.

A live tree read is not called a consistent snapshot just because the files are read quickly. The handler must prove the stability of the readable set and absences in the chosen method. Otherwise, `StaleInput` or incomplete is returned, and the subsequent result does not receive a false reproducible reason.

### 6.3. Who can write and how

An individual working copy has one responsible `work_id` and one current writer unit; the transfer of this role goes through C-05 with the termination/fencing of the former executor. The reviewer reads the specific image and does not add corrections to the copy being reviewed.

Two works can simultaneously have `BranchWrite` from the same source X, because they are physically written to different copies. `Apply` reserves the entire declared set of targets in a single transaction C-03. Overlapping withholding applications are not permitted regardless of `work_id`; non-overlapping changes are allowed only if their read dependencies and write areas match. A directory operation holds the corresponding subtree, not just the name string. Blindly blocking the entire project forever is not required, but reducing the scope must be proven by the operation list.

The subject register of working bindings, editions, uses and conflicts has one writer C-03. The proxy security log stores evidence of the exact intent and authenticated `use_id`, rather than a second mutable register of operational locks. C-09 does not trust one field `Held`: just before the consequence, it checks the accepted record through the current admission order, rights, fences, identity of files and original content.

The `Apply` hold does not automatically expire after a timeout if the action could have started. With `outcome_unknown`, the affected targets remain unavailable for new use until reconciliation. After a proven outcome, the observation and the actual binding of the target state are recorded by the protective transition C-05/C-03, even when the semantic result is already prohibited by the recall. Release of the account does not mean acceptance of the result.

The regular user editor does not participate in this registry. Therefore, internal mutual exclusion protects against collision of Deep Create works, but does not prove the absence of someone else's change. Protection against last-minute substitution and modification between check and write remains the responsibility of a proven file handler 17 §10. If it cannot be proven on the selected local volume, failure `file_identity_race_unproven`; the document does not promise universal interception of other people's records.

## 7. Example of collision and separate join

Job A reworks the structure `project/src/app.rs`, Job B corrects the error. Both have legitimately read the original X and are changing different separate copies.

1. A creates candidate Y and undergoes its own verification. Application A obtains a hold on target X, a formalized intent, and a current right. At point `D` the mediator performs a valid substitution X → Y; the observation, the actual tree edit, and the reconciliation result are then saved.
2. B completes candidate Z from X. A positive test of Z does not prove the suitability of Z over Y. `Apply` B detects a change in the true target base; returns `WorkspaceConflict` with a specific target, expected X, and observed Y. Child reason remains a type of subject dependency mismatch, but is different from a normal source document change.
3. B does not write Z over Y and does not trigger the live tree's automatic `rebase`. C-05 saves the existing incident with typed conflict details and blocks only the affected application. The main job status is not replaced by the new global `NeedsMerge`; the need for unification is the reason for the boundary of a particular unit.
4. Within the already accepted goal B and the final budget, an existing unit of the type “union” is proposed. In her contract: single responsible work B, reasons X/Y/Z, current revision of goal B, maintenance of confirmed conditions A and B in the affected area, prohibited losses, finite resource, separate copy of M and required independent verification. If maintaining outcome A extends goal B or requires a new scope, an appropriate solution is needed first; the word "union" is not permission.
5. LLM examines the differences and creates sentence M in a separate copy. A suitable automatic merge tool can only be used as a registered authorized program in this copy; its null return code is not a meaningful check. Testing M involves contradictions, negative results, and changed expectations of both works. It is the M bytes that are checked, and not the old separate verdicts Y and Z. The minimum is the rule `Synthesis`/`LocalChangeProposal` of 16, and not two agreeing authors.
6. The user tree remains Y until a new permitted use. Before M → living object, the current Y, all significant sources, criteria, rights to both source areas, write rights to the target area and fences are checked again. If Y2 appears, M is not “corrected along the way”: new unit or association edition, new check, new applicable treaty.
7. A successful M is the result of B, with clear provenance from A and B. It does not give B ownership of all of A, rewrite the result of A, or assert fulfillment of unaffected obligations of A. The shared work object registry stores the actual revision and provenance of the application; the user project itself does not become the property of one `RootWork` in perpetuity.

```rust
struct WorkspaceConflict {
    conflict_id: EntityId,
    incident_id: EntityId,
    blocked_unit_id: EntityId,
    workspace_id: EntityId,
    expected_revision_id: EntityId,
    observed_revision_id: Option<EntityId>,
    affected_targets_evidence_id: EvidenceId,
    competing_intent_refs: Vec<EntityId>,
    merge_unit_id: Option<EntityId>,
    resolution: ConflictResolution, // Open | MergeProposed | Revalidated | Abandoned
}
```

These are typed details `incidents` and not a new independent incident mechanism. `observed_revision_id = None` means that a reliable edition has not yet been installed; an undefined tree does not become a suitable basis for a union. The conflict can be resolved by abandonment or a new justification for non-intersection, and not necessarily by expensive unification.

## 8. Rights to both areas and revocation race

The unifying unit B must have its own valid path to read the source area A, its area B, and write the separate M. It is not legal to write to A. Subsequent application requires `CommitWorkspaceChange` on each affected target. If there is no single right over the entire target set, a single compound operation does not extend partial permissions: you need precisely verifiable rights over all scopes and a handler with explicit partial outcome semantics, or separate operations.

The existing form `ActionIntent` retains the basic `lease_id`/`lease_version`. For an action that depends on multiple protected areas, it is supplemented by `supporting_lease_refs: Vec<RequiredAuthorityRef>`, and `CommitPreconditions` by the same exact set of currently required permissions. `RequiredAuthorityRef` contains `lease_id`, `lease_version`, `work_id`, `action_kind`, certified area and fencing. The list is collected by the control part from real dependencies and exact actions; The model does not assign itself supporting rights. The main right and supporting links do not duplicate expenses or extend each other.

All required rights must apply to responsible work B: you cannot add someone else's lease A to the list and thereby gain access. The required read permissions are checked when reading, when expanding data, and when accepting a derived result; right of recording - immediately before a specific use. The expiration of the supporting right does not dispense with the fact that the material already exists in a separate copy.

The existing general tolerance boundary checks the entire set: source A/B regions, target region, candidate version, hold `Apply`, and desired dependency versions. One common short order of the first version allows the revocation of either of these areas to be serialized with `D` and `C`; The portable "all rights once verified" pass does not occur.

- If the A read or write recall of the target is pinned at point `F` before `D`, the application is rejected by `Revoked`, even if the M check just gave `pass`.
- If `D < F`, the file may have changed; observation is necessarily maintained. After `F`, you cannot accept the late semantic result or begin the next action. Y does not automatically return.
- If `C < F`, the decision already made is not declared late; the following actions after `F` require a new valid base.

Neither NTFS and SQLite nor multiple files are declared as one atomic transaction. For a set of substitutions, the order, observed partial outcomes, and reconciliation are predefined; failure after some replacements blocks dependent actions. Until the atomicity of a particular set is proven, the interface does not promise “all or nothing.” Returning is a new action that also cannot erase someone else’s record.

## 9. Function contracts without a new control layer

The forms above are a clarification of the inputs of existing functions, not a basis for creating new public writers. The commands are listed as options `ValidatedCommand`. Pure failures do not write the event themselves: it is stored by C-05 through C-03; protective observations are received in the existing certified way.

| Agreement | Refined entry and exit | Mandatory Conduct and Waivers |
|---|---|---|
| C-01 `validate_work_graph` | `GraphCandidate`, which includes editions of interworker communications; `ObjectiveContract`, `LimitView`, consistent view of the affected graph area → `ValidatedGraph` | Checks the purpose of each consumer, producers' revisions, closed conditions, future cycles and the finiteness of iterations. Saves `Cycle`, `UnboundObligation`, `UnboundedUnit`; detailed reasons `UnknownPublication`, `InvalidLinkCondition`, `OutputSchemaMismatch`, `UnsupportedLinkPolicy` |
| C-01 `derive_completion_candidate` | Previous commitments/results/checks plus the validity of actually used interworking grounds → candidate or gaps | `PublicationUnresolved`, `PublicationWithdrawn`, `OwnerAcceptancePending`, `StaleInput` do not turn into completion. Doesn't accept job as owner |
| C-03 `commit_transition` | Previous `ExpectedRows + TransitionPlan + CommitPreconditions` with exact changed connections, publications, working bindings and holds → `CommitReceipt` | One writer and one subject data/event transaction. Checking intersections of `Apply` and common graph versions in the same transaction. Previous `VersionConflict`, `FenceChanged`, `LimitReached`, `StorageUnavailable`; detail `WorkspaceBusy` does not turn into an issued right |
| C-05 `handle_command` / `decide_transition` | Options `AcceptRootWorkLink`, `CancelRootWorkLink`, `PublishVerifiedResult`, `SetPublicationEligibility`, `RecordDependencyImpact`, `PrepareWorkspaceUse`, `RecordWorkspaceConflict`; former `AcceptGraph`, `ApplyVerifiedResult`, intent/observation commands | The only acceptance. Checks the original trusted/planned path and target limits. `MissingPermission`, `DependencyChanged`, `InvariantViolation` are saved; details of reasons do not bypass basic prohibitions |
| C-06 `select_ready_units` | Coordinated `StateView` with connections, publications, conditions and working deductions → `ReadySelection` | Explains `AwaitingPublication`, `OwnerAcceptancePending`, `ConditionUnknown`, `UpstreamCancelled`, `WorkspaceBusy`, `WorkspaceConflict`, without stopping extraneous branches. Selection does not reserve or write |
| C-09 `canonicalize_action` | Typed set of real files and required data → old `CanonicalAction` | Normalizes scope through real objects rather than names; `TargetChanged`, `ScopeDenied`, `file_identity_race_unproven`. You cannot declare two aliases to be disjoint just by line |
| C-09 `prepare_intent` / `dispatch_verified_intent` | Previous intention plus confirmed `use_id`, exact target list and all `supporting_lease_refs` → previous `PreparedIntent` / `ActionObservation` | Before `D` checks all rights/fencing/dependencies, original content and suitability of the hold. `Revoked`, `TargetChanged`, `StaleInput`, `ScopeDenied`; after start distinguishes between `ConfirmedFailure` and `OutcomeUnknown` |
| C-13 `inspect_startup` / `prepare_resume` | Previous state plus unclosed uses, conflicts, publications and connections → `RecoveryReport` / `ResumeProposal` | Restores the derived influence index, checks the intent and actual trees, does not release the undefined `Apply`, does not revive the old attempt. `UnresolvedOutcome`, `StaleInput`, `StorageUnavailable`; lack of evidence obscures consequences |

The semantic merging checks are performed by the existing C-08; limited access to evidence - C-04/C-07; true identity and secure order - C-10/C-12. This is a clarification of passport dependencies, not new C-17s and beyond. A pure graph check can call private auxiliary functions, but you cannot add separate services, logs, or acceptance rights to them.

When migrating, existing packages do not automatically become provisioned, former internal dependencies do not become cross-production rights, and discovered file paths are not registered as already resolved scopes. Empty new tables are acceptable; completion requires verified origin and customary acceptance.

## 10. Events and testable scenarios

Events use the legacy wrapper `state_events`, single `transaction_id`, string versions, `rule_id`, and a secure link to details. The secure part contains the numbers and reason code, but not the contents of reports, the full personal path, or data from other inaccessible work. The interface even filters the names of unavailable works; the presence of a link does not allow disclosure of their metadata.

| Event | Minimum safe meaning |
|---|---|
| `root_work_link_committed`, `root_work_link_cancelled` | Which communication revision was accepted/cancelled, which permitted areas are affected |
| `result_publication_committed`, `publication_eligibility_changed` | Exact grant number, previous/new eligibility, reason |
| `cross_work_dependency_blocked`, `cross_work_dependency_revalidated` | Consumer, exact reason/edition, link to review; removing one reason does not remove the others |
| `cross_work_dependency_changed` | Which read binding has changed, list of affected consumers in secure evidence |
| `workspace_use_reserved`, `workspace_use_released` | Usage number, mode and proven reason for detention/release; non-issuance of rights |
| `workspace_conflict_detected`, `workspace_merge_proposed` | Unchangeable grounds for conflict and the number of a separate unit of association |
| `workspace_revision_observed`, `workspace_reconciliation_blocked` | Actual sighting of a tree or the unknown, connection with intention; not false acceptance of the result |

Events `result_committed`, `result_rejected`, `action_intent_committed`, `action_observed`, recall/restore and check events are saved. New events add missing causation rather than replace existing evidence of consequences.

| Number | Experiment setup and required observed outcome | Treaties and events |
|---|---|---|
| `AC-XW-01` | A provides verified P8, but not yet `owner_accepted`; B with `NotRequired` and a separate valid right begins the exact unit. B with the requirement of acceptance by the owner is waiting. The substitution of one stage for another is rejected. While waiting for revision 2, the output of revision 3 of the same unit does not satisfy the connection | C-01/03/05/06; `result_publication_committed`, `cross_work_dependency_blocked` |
| `AC-XW-02` | There is a connection and a valid right for A, but no right for B to read A. Neither bytes nor protected metadata of A get to B/its model; writing to A is also prohibited | C-04/05/07/09; existing access denial, `cross_work_dependency_blocked` |
| `AC-XW-03` | Two consumers read P8: fixed and following head. After P9, the first one remains at the valid P8, the second one does not accept the old attempt; revoking P8's validity blocks both. The extraneous result of A does not invalidate them. Removing PublicationState from read_set of deliberately corrupted variant is detected even with lost reverse index | C-03/05/06; `cross_work_dependency_changed`, `publication_eligibility_changed` |
| `AC-XW-04` | Condition first `False`, then `True`; the consumer with the old photo does not go through. An unknown field gives `ConditionUnknown`, not permission; free software predicate is rejected by the circuit | C-01/05/06; `cross_work_dependency_blocked` |
| `AC-XW-05` | A1 waits for B1 and B1 waits for A1, including via conditional edge/future final acceptance: the entire conflicting graph is rejected. Accepting two edges at the same time does not bypass the check. Historical P8/Q4 and final A1→B1→A2 are valid without infinite running | C-01/03/05; graph failure with `Cycle` |
| `AC-XW-06` | Canceling a manufacturer before exit only blocks exact consumers. Cancellation of B does not cancel A. The previously granted ground does not disappear from history; unknown effect of canceled unit is not repeated | C-03/05/06/09; `root_work_link_cancelled`, `cross_work_dependency_blocked` |
| `AC-XW-07` | The link source has been changed from A to D with the same header. Old rights, snapshot and verification are not transferred; new form/area/level is checked. The previously accepted result remains a historical fact with a note of influence | C-01/05/06/09; `root_work_link_committed`, `cross_work_dependency_changed` |
| `AC-XW-08` | The two jobs have different copies from X. A applies Y. B's attempt to apply Z from X returns a conflict and does not change Y by a single byte | C-03/05/09/12; `workspace_conflict_detected` |
| `AC-XW-09` | Two path strings, different case/short name, and nested working roots indicate intersecting real objects. Simultaneous use does not work as independent; binding/content substitution is detected. Unsupported links are rejected | C-03/09/12; `workspace_conflict_detected`, existing `file_target_denied` |
| `AC-XW-10` | The Y/Z merge creates a separate M with a single responsible B. Old Y/Z checks are not counted as a check for M. A negative A result is not lost; living Y is unchanged until a new action | C-01/05/08/09; `workspace_merge_proposed`, `verification_completed` |
| `AC-XW-11` | After checking M, B's right to read A's source area is separately revoked, then in another run of the experiment, the right to write the target is revoked. There are no changes with `F < D`; with `D < F` observation is maintained, late adoption and new effect are prohibited | C-03/05/09/10/12; revocation events, `action_observed`, `result_rejected` |
| `AC-XW-12` | After checking M, the user editor creates Y2. The application does not overwrite Y2, does not automatically merge the living tree, and does not substitute a new basis for the old verdict | C-09/12; `workspace_conflict_detected` / `file_identity_race_unproven` |
| `AC-XW-13` | Failure before `D`, after physical replacement before observation, and after observation before semantic acceptance. Reconstruction distinguishes cases by evidence; the undefined area remains held, the entry is not repeated, the old copy does not revive the right | C-03/09/13; `workspace_revision_observed`, `workspace_reconciliation_blocked` |
| `AC-XW-14` | Two non-overlapping applications with independent read sets can proceed sequentially without conflicting the overall `EventSeq`. Overall readable configuration changed - dependent second result becomes obsolete despite different write files | C-01/03/05/06/09; `workspace_use_reserved`, `cross_work_dependency_changed` or `result_rejected` |
| `AC-XW-15` | Feedback index deleted/recalculation delayed/notification lost. The current check before sending and accepting still rejects the old dependency; C-13 recovers index from canonical data | C-03/05/06/09/13; `cross_work_dependency_blocked` |
| `AC-XW-16` | A compound change crosses two permitted areas, but only one is allowed; the consequence does not begin. If dialing is allowed and fails after the first replacement, a partial/unknown outcome is retained rather than a false “all cancelled” | C-03/09/10/12/13; access denial or `action_observed`, `workspace_reconciliation_blocked` |
| `AC-XW-17` | The P8 expires without a new source entry; the ban is carried out on time. Granting to the required checkpoint does not trigger B. Owner failure on an irrelevant exit does not stop a proven independent branch | C-01/03/05/06/09; `cross_work_dependency_blocked`, `cross_work_dependency_revalidated` |

## 11. What the document promises to check, but does not yet implement

Promises tested: The missing right does not arise from a relationship or resource record; each consumer has an exact edition of the base; cyclic wait is not accepted; the conflicting entry is not silently applied; the combined candidate is checked separately; recall of any required area is ordered with application; an undefined file outcome does not free up the retry area.

Implementation and stored evidence are also required: records migration/schemes; machine graph verification with competitive acceptance; durable impact treatment; calculation of intersections of real objects; proven Windows snapshot and replacement primitives; faulty reconciliation; time-limited critical areas; behavior of compound replacement and external editor. The semantic check of influence and association requires a qualified route of 16 and does not receive the guarantee of error-freeness from the presence of this document.

This is not a report on the successful testing of Deep Create, nor is it a permission to change the current Hermes. Implementation is allowed in a future separate application through the same sixteen passports. The ambition remains: several long-term interrelated works, studies, conditional plans and complex associations are supported by explicit contracts, rather than being cut out for the sake of simplicity.

## 12. Exact points of integration with main documentation

- **14 §4.9:** explicitly limit the existing `work_dependencies` to internal units of one main work; interworking edges are not created by the implicit reference `depends_on_unit_id`. After the section, add a regulatory link to 24 §§3-5: `result_publications`/their admissibility and bindings, `root_work_links`/editions, connection with the existing image. In §4.19 and §8 add working bindings/editions/uses and typed versions of commands from 24 §9; the only former writer. In §7, keep the semantic stages: conflict/waiting for publication - reasons for boundaries, not a new stage. In §9, add reconciliation of open `Apply` and restoration of the influence index.
- **16 §3:** add inter-working required outputs, closed conditions and check for future cycles on 24 §§3-5. In §4, associate the union with a single responsible work and a separate copy; prohibit the transfer of the old verdict to M. In §13, separate internal grant from final acceptance; use canonical `awaiting_owner_acceptance`/`owner_accepted` and check the validity of interworking grounds. Readiness takes into account precise consumers, not all session work.
- **17 §3, §10 and §12:** resource record `WorkspaceUseRecord` is not right; clarify reading/separate copy/application, lists of present objects, consumer rights to both source areas, exact set of supporting rights and revocation between review and `D`/`C`. The previous actions and the only mediator remain; external publication from `ResultPublication` does not occur. Failure/return is not considered atomic between files and SQLite.
- **19 C-01/C-03/C-05/C-06/C-09/C-13:** supplement the inputs, detailed reasons for refusal and evidence strictly according to table 24 §9. C-08/C-04/C-07/C-10/C-12 remain existing verification, read, and security dependencies; no new public writers and C-17s are added.
- **21:** after additional boundary scenarios, add a separate group `AC-XW-01`—`AC-XW-17` with a normative reference to 24 §10; connect it with D-03, I-03, I-05, I-07, I-11a, I-12, I-14/I-14a. Do not change the number 51 of the original decisions: this is a detailing of the evidence, not a retroactive new response from the owner. The presence of scenarios is noted as design traceability, not `COMPLETE`.
- **15 §2.5a - only necessary matching link:** add five options `DependencyKind` from 24 §3.3 or directly declare this section as an addition to the enumeration. The rest of the memory and instruction architecture does not change; the second type of image is not entered.
