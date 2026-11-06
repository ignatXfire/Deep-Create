# Deep Create: analysis of external criticism and clarification of the project

**Date:** 2026-09-08.  
**Final reconciliation after continuation:** 2026-09-09. File names preserve the start date of parsing; The machine scan time is indicated in the report.  
**Subject:** notes from owner-submitted text, not new channel research.  
**Work Boundary:** Single application documentation. The current Hermes remains unchanged. The features described here have yet to be implemented and tested.

## 1. Main conclusion

Criticism is useful, but does not mean that Deep Create should be turned into a simple chain of executioners. Several lengthy works, independent choice of solution methods, branches of hypotheses, transfer of results, selection of instructions for the task and model, memory, testing and development of the variable part of the system are saved.

The discovered flaw is different: the individual parts were described in more detail than their **simultaneous operation**. What happens when a study has updated the output, another work is already using the old output, a third is changing the same file, and the testing attempt is waiting for the last free connection? This is not a reason to remove opportunities. This is a reason to accurately define input versions, change owners, maintenance order, and testing event combinations.

Complexity is acceptable where it brings opportunity or resolves a particular ambiguity. The name of a new entity does not in itself make it necessary. The new rules are placed within the existing sixteen code contracts rather than being turned into sixteen additional services.

The opposite extreme is also not accepted: considering threats as insignificant because the project is personal. An error in the writing order or file selection is dangerous even without an attacker. We preserve the power of the system by making the consequences of its actions visible, verifiable and manageable.

## 2. What is accepted for each comment

The first column contains the stable number of the comment from the external text. These are not new owner answers or a replacement for the 51 solutions from document 07.

| Note | Solution | Why and where included |
|---|---|---|
| R-01 | Accepted | The connection between the main works should indicate the provided revision of the result, the verification conditions and the reaction to the change. Detailed agreement - document 24; data - C-01/C-03, acceptance - C-05, readiness - C-06. Does not transfer rights between jobs. |
| R-02 | Accepted with change of method | We need fairness of service, but the sum “importance + age + term” without units and restrictions does not set it. Document 25 defines the admissibility, maintenance and protection of long-awaited work separately. |
| R-03 | Accepted with significant reservation | Limited queue and timing are needed. The sole owner of the state remains. Stop does not become the usual high-priority command behind a busy queue: its independent path through the guard is preserved. Document 25. |
| R-04 | Accepted as strengthening evidence | Manual cases are needed, but do not cover combinations. Document 26 specifies the final test model, sequence generation, reproducibility and verification against the real implementation. Doesn't declare brute force proof of all Windows. |
| R-05 | Accepted | Isolating job numbers does not eliminate the common real file. Document 24 clarifies workspace identity, shared read, write conflict, and separate join. Employment registration is not a new permission. |
| R-06 | Accepted as an extension to existing review | There have already been tests of the ability. Paper 26 adds a comparable assessment of the verification method by task type, reference origin, and errors. Different model names do not prove independence. |
| R-07 | Accepted without limiting self-development with manual approval of each step | Testing with familiar examples does not equal improvement. Paper 26 adds hidden sampling, real errors, post-admission observation, and version reversion. The admission rules are not edited by the candidate being verified. |
| R-08 | Accepted as a refinement of the representation | Need an overview of several works before diving into a large graph. This is another level of the same confirmed state, not a new control system. Agreement - §4 of this document and C-14. |
| R-09 | The risk is recognized; no automatic final acceptance added | L-10 leaves final acceptance of the main work to the owner. There is no need to ask it after each internal unit. The procedure for accumulating results and further independent work is §5. |
| R-10 | Accepted | We need media availability states and verifiable transfer without losing canonical materials. Agreement - §6, C-04/C-05/C-09/C-13. Archiving is not considered a backup. |
| R-11 | This is not a disadvantage of the first version | Local Windows application is the accepted direction. Several machines will require a new authority agreement, shutdown and restoration. The boundaries of future expansion are fixed in §7; Empty network layers are not currently created. |
| R-12 | Accepted as end-to-end requirement | The combination of rules and the useful outcome are checked, not the number of beacons and tests. Document 26 and the admission procedure in document 20. Opportunities are not cut out to reduce the number of states. |

## 3. How to read changes as one system

1. [24. Work connections and common work areas](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md): **which edition of someone else’s result became the input and who has the right to change the common object**.
2. [25. Resource Allocation and Commit Queue](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md): **when a valid job receives a resource and when its result is actually accepted**.
3. [26. Checking combinations of states and development](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md): **how to detect an error in these rules and distinguish an improvement from a successful report**.
4. Sections below: **how the owner will see what is happening, how materials will be saved and where the next expansion boundary is**.

Example: Job A examines memory and provides verified output. Work B designs a new set of context based on a specific edition of this conclusion. Job B independently evaluates the candidate on tasks that the candidate has not seen. At this time, A is revising the output, and B and other work want to replace one file.

Without new contracts, you can seamlessly connect the old input, the new file, and the third version check. Now the link commits the revision, the try stores the read dependencies, the record is compared to the real current object, the change results in an explicit revision or merge. At the same time, the scheduler prevents the flow of fast new tasks from forever crowding out the check. The screen shows the reason for the wait and the impact on other work. The owner is not involved in every internal step; he receives a clear, verified result of the main work.

## 4. Review of work: not another panel, but an exact answer to “what’s going on”

### 4.1. Data and origin

The existing `UiProjection` C-14 is being expanded. Its root contains `installation_id`, `journal_id`, `confirmed_event_seq`, the version of the presentation form and the limited recovery flag. All counters of one answer are built from one agreed snapshot, not from independently surveyed performers. An unacknowledged write before the checkpoint is not shown as an accepted result.

First, the main jobs are displayed, then the selected job with its branches, then the unit and the specific attempt. For each main job the following are available:

- purpose, current edition and exact status;
- verified result or expected result, not a stream of messages;
- reason for waiting: input, resource, right, reconciliation of consequences, final acceptance by the owner;
- the age of waiting for the resource and the basis for the service order from document 25;
- used and retained resources separately, without reporting unknown expenses as zero;
- incoming and outgoing communications indicating revisions and consequences of the change;
- information about the availability of necessary materials and media;
- open decisions of the owner, grouped by reason;
- time of the last confirmed progress. It is not equal to a sign of a living process.

The map, list, and detail pane use the same values. Grouping and hiding nodes only changes the view, not the layout. The overall percentage of success of different tasks is not calculated from the average percentage or number of units completed.

### 4.2. Functions, scope and failure

`build_ui_projection(StateView, ViewQuery) → UiProjectionPage` remains a function of C-14; it is a refinement of her argument and result, not a new source of power. `ViewQuery` contains an area, a filter, a valid sort, a page limit, and an optional next page pointer. The pointer is associated with the matched snapshot number and the request. If the snapshot becomes unavailable, `ProjectionRefreshRequired` is returned; you can't quietly continue the list from another point and lose or double lines.

The presentation is limited by volume. The graph is loaded in parts with an explicit number of undisclosed neighbors. The first version does not require keeping SQLite snapshots indefinitely: when a snapshot changes, the window prompts for an update and saves the selection by persistent number. The full log is not copied to the window memory to build counters. Authenticated commands still re-check the current conditions - the screenshot number does not provide permission.

Checks: **AC-OV-01** - list, map and overview are consistent in one image; **AC-OV-02** - during page changes, jobs change, an explicit update is obtained without false acceptance; **AC-OV-03** - many nodes and events do not hide the pending decision, keyboard controls and the selected item are preserved. The scope of the test and the measured latency are recorded in a report, rather than replaced by a promise of “no lag.”

The `ui_projection_refresh_required` beacon indicates the need for a new snapshot without task content. A normal redraw does not generate a canonical log event and does not wake up the LLM. Consistency violation - test failure, infrequent notification to owner.

## 5. Final acceptance by the owner without a queue of questions on trifles

The main job is a long-term goal with its own contract, not any model team. Researching part of the sources, comparing options, correcting the candidate and checking are units within it. Their suitability takes the provided machine and semantic verification path.

`awaiting_owner_acceptance` means that the result of the main work has been prepared. The verified provided result can be used by another work under the conditions of document 24, if the connection does not specifically require final acceptance by the owner. `owner_accepted` is a separate fact of L-10. Using a result does not falsify that fact or declare the owner to agree with the conclusion.

Several results are compiled into a list: what is ready, what has been proven, where are the gaps, and what will change after the decision. The impersonal “agree to everything, including future results” button is not introduced. Independent work continues; dependents wait for exactly the required condition. Repeated display of the same unchanged total does not create a new request to the owner.

If automatic final acceptance of certain classes of targets is required in the future, this will be an explicit change to the L-10 decision with a precise list of classes and consequences. Now criticism is not such a permission.

Validation **AC-OWNER-01**: A awaits final acceptance, B consumes the verified result without such a condition, C requires acceptance, D is independent. B and D are not blocked by A alone; B does not start before the solution. All internal units do not turn into four new permission requests. Existing result, grant, and acceptance events are used for observation; no additional copy of the event is needed.

## 6. Life of materials: memory grows, but the source does not disappear

### 6.1. Share meaning, place and accessibility

`EvidenceId` and canonical bytes preserve identity regardless of medium. C-03 adds accounting `EvidenceLocation`: location number, `evidence_id`, expected encrypted byte fingerprint, volume number, relative opaque path, location revision, size, last checked time, and status `copying / verified / offline / missing / damaged / retired`. Paths and personal information are protected according to the existing data class; The path with the person's name is not displayed in the general log.

These are not new states of meaning `evidence_objects.storage_state`. `canonical` can only have a disabled location now. `offline` means “known media is not available”, `missing` means “the expected bytes are not on the available media”, `damaged` means “they are there, but the check failed.” An access error or temporary unavailability is not replaced by a finding of loss. The derived search index does not become a replacement for the original evidence.

Losing one location does not transfer the entire item to `missing` if another valid copy is verified. The reading path evaluates allowed locations rather than generalizing the first failure to the entire archive.

The designations “frequently used,” “infrequently used,” and “archived” are placement policies, not new permission levels or different truths. In the first version, one explicit managed transfer is sufficient; Separate three storage engines are not needed.

### 6.2. Migration and Failure Procedure

1. C-05 accepts a transfer order to an authorized location with a volume limit. The source remains valid. C-09 checks path and write permission, C-13 checks for space and critical reserve.
2. C-04 copies the **already encrypted** canonical bytes to a temporary destination object. Changing the key is another explicit operation, not a side effect of the transfer.
3. The complete copy print, size, decipherability and identity of the destination are checked. A substitution of the path or a break gives a failure; the old copy remains unchanged.
4. C-05/C-03 commits the new checked location and event in a normal checkpoint transaction. Until there is confirmation, readers will use the previous arrangement. An orphaned new copy is parsed by restoration and does not automatically turn into the only truth.
5. Switching the preferred reading is not the same as deleting the source. Removing a redundant old copy is only possible with a separate exact right and after re-checking the new copy and current dependencies. Canonical content is not removed. Without this right, the transfer remains a copy and does not promise to free up space.

The location pointer and its revision are part of the delete step's validation; a parallel reader with an open object should not lose data. The deletion itself goes through the usual intent and C-09 reconciliation. If the outcome is already unknown, it is not repeated blindly. It is prohibited to declare the transfer successful if only the start of the copy is confirmed.

### 6.3. Challenges and consumers

Inside C-04, `stage_evidence_copy(EvidenceRef, DestinationGrant, StorageBudget) → StagedLocationReceipt` is introduced: stream copy and check, but not change the canonical link. Errors: `InsufficientStorage`, `DestinationChanged`, `ScopeDenied`, `IntegrityFailure`, `StorageUnavailable`. The return contains the imprint, the size, the verified identity of the destination, the source edition and the fact that the transcript has been verified; private keys are not returned.

`DestinationGrant` is a narrow representation of the already verified C-09 right for a specific purpose, not a new type of permit. `StorageBudget` is a representation of the existing confirmed C-03 reserve, not a second location counter. None of these are a substitute for re-examining current conditions before a consequence. Their form is not accepted from the model text as trusted evidence.

In C-05, the existing command type is given the variant `AcceptEvidenceLocation`; `decide_transition` verifies the certificate, current edition and authority, C-03 single-handedly records the location. No independent writer or public "assign any path as evidence" feature is added.

`open_scoped_evidence` distinguishes between `EvidenceOffline`, `MissingEvidence`, `IntegrityFailure`, `StorageUnavailable`. If a required input is unavailable, the required unit is suspended; independent parts continue. If the contract permits the work based on the available information, a new, explicitly limited edition of the entry is created. One cannot silently call it a verification of an inaccessible primary source. Disabling the archive does not in itself permit a new cloud address or disclosure of materials.

Derived pointers can be rebuilt from stored sources in a separate bounded task. Resetting them should not erase negative findings, decisions or origins. The backup copy remains a separate C-13 contract with a trial recovery: two copies on one failed physical disk do not prove safety.

### 6.4. Mandatory checks and beacons

| Check | What happens intentionally | Acceptable outcome |
|---|---|---|
| AC-STORE-01 | Break at every step of copy, check, accept and cleanup | There is an old or new verified copy; no false canonical reference; unknown consequence described |
| AC-STORE-02 | The archive disk is disconnected, the file is deleted, the file is corrupted - three separate experiences | Discernible reasons; unavailability is not passed off as empty memory; unconfirmed conclusion not accepted |
| AC-STORE-03 | Free space runs out during copying; source on another volume | The copy does not damage the source and does not consume the emergency reserve by normal operation |
| AC-STORE-04 | Between checking and deleting the old copy, the assignment or right changes | Removal rejected; the current edition has been verified; copying did not expand resolution |
| AC-STORE-05 | The search index is lost; source restored, source unavailable | In the first case, a reproducible restoration, in the second, an honest gap, not a fictitious memory |

Beacons: `evidence_location_verified`, `evidence_location_committed`, `evidence_location_unavailable`, `evidence_relocation_interrupted`. Significant change in location - Event C-03; copy checking and interruption before acceptance - local observation followed by normal admission. Re-discovery of an offline volume is combined into one incident. The content, keys, and full private path are not written to the message.

## 7. What we plan for the future without creating empty code

Consistent numbers, editions, origins, accurate messages, no hidden entry from the window and the C-12 border are already helping to transfer individual parts. But this is not readiness to work on multiple machines.

Such expansion will require new solutions: where the sole owner of the state is located, how the remote executor verifies who stops him when the connection is lost, what the right to expire means on different clocks, how the identity of the shared file is established, and who proves the expense after the break. Before these solutions, the remote executor is not represented as a local one through an empty network adapter. The **AC-PLATFORM-01** check should reject an unknown remote participant in the current local protocol.

Likewise, a future change in storage, testing method or planning method requires a new edition of the relevant contract, a list of affected consumers and a repetition of its boundary tests. This is an opportunity for development with an understandable price for change, and not a ban on development.

## 8. The limit of confidence and the next step

This analysis provides solutions to twelve observations, but does not promise the absence of all errors. Meaningful contract testing, documentation reference testing, model testing, and real application testing are four different types of evidence.

Papers 24–26 set forth future challenges. As long as there is no executable application, they cannot be marked as completed. The stored structure check checks the documentation and associates the result with the fingerprints of the input files. The old report 22 is saved as a history of the previous state rather than automatically becoming a new confirmation.

After making the agreed clarifications, the next step is the implementation of document 20 step by step. You can start with self-assembly, types and a test model. Resources of real works, arbitrary user files and self-updating are connected only at their proven stages. This is the order of creating a complete system, not replacing Deep Create with a stripped-down final product.

## 9. What has been done and what the re-check yielded

The analysis was carried out in parts: connections and general areas; resources and fixation; combinations of conditions and development. After the contracts were prepared, three separate reviewers received the relevant materials and associated master map locations. They did not re-analyze the channel body or change the current Hermes. A general collection of clarifications has been included in documents 09, 13–21 and the navigator; 22 is marked historical. Sixteen code owners have been retained, with new responsibilities distributed among them.

A re-reading revealed six inconsistencies specifically in the new treaties. These were corrected before the documentation was transferred:

| Inconsistency found | Correction and Verifiable Consequence |
|---|---|
| Waiting for the release required an exact edition of the unit, but the form only stored its number | `AwaitOutput` and `ResultPublication` received `producer_unit_revision`; the release of another edition does not satisfy the previous expectation |
| The variable validity of the result was in the explanation, but did not have a separate appearance in the canonical read set | Added `DependencyKind::PublicationState` with `publication_id` and `row_version`; pinning an old version does not bypass revoking its validity |
| The text distinguished between the absence and disabling of an archive copy, but the list of states did not contain `missing` | State added; the loss of one location does not make the entire item lost if another copy is verified |
| The queue required the command to be re-evaluated, but the closed form contained only the fingerprint, no body | Added own limited typed body; his memory is taken into account, large materials remain encrypted links |
| Fair service conditions did not take into account the continuous automatic change of inputs | The condition for a stable candidate prior to admission has been clarified; AC-SCH-13 checks for volatile `FollowHead` without inhibiting this capability |
| The example transferred the new pending move number from the fact of sending to the completion of the result | The example has been corrected: the number changes once per certified shipment; completion does not re-rotate it |

The finite numerical domains of the test model have been additionally clarified: without them, a systematic traversal is not called finite or complete. No extra wording of the purpose was added to the provision: `objective_id` already denotes the exact unchangeable wording under document 14. This is an example where a longer form would not provide a new guarantee.

Structural check after mixing: **57 out of 57 conditions passed**. 51 number solutions, 10 additional groups, 16 code agreements and 12 comments were compared; local file links, table forms, presence of named acceptance scenarios and corresponding events were checked. [Full result with fingerprints of input files](./checks/design_check_20260908.json). Product self-test: **16 out of 16**, [result](./checks/design_checker_self_test_20260908.json).

These numbers do not mean that future AC scenarios have been completed. **The application was not tested during this phase; product code was not created at this stage.** The reviewers did not find any other proven contradictions in the places assigned to them, but this is not a statement that all possible defects are absent. The next source of confidence is working code and reproducible testing on document 20, not yet another increase in page count.
