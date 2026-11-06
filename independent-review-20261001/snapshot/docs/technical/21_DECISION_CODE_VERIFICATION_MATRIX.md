# Deep Create: matrix “decision → code → check → beacon”

**Date:** 2026-09-06  
**Status:** design traceability; the code has not yet been created  
**Number of initial number solutions:** 51  
**Updated:** 2026-09-07

---

## 1. How to use the matrix

Each line is an implementation commitment. A future feature is not considered ready until the following is added to the line:

- exact path of the source file;
- number of the accepted code change;
- name of the automatic check that passed;
- link to the saved test result;
- an example of a mandatory event;
- total `COMPLETE`, `GAPS` or `INVALID`.

The matrix now checks the traceability of the accepted requirements. The presence of a string does not in itself prove that the project is complete or that the application works. C-01—C-16 refer to document passports 19; AC numbers - for mandatory acceptance scenarios.

Check names of the form `AC-...` are constant script numbers. Their implementation may consist of several tests, but no number can be removed without a new solution and explanation.

---

## 2. Personal decisions of the owner - 25 lines

| Solution | Fixed behavior | Code owner and main function | Mandatory Beacon | Readiness check |
|---|---|---|---|---|
| L-01 | The switch allows the system to only offer deep work; when turning off, direct selection of the owner is required | C-05, C-14 (document passport 19) | `deep_mode_proposed`, `deep_mode_changed` | `AC-L01`: the off state does not create a transition; included creates only a sentence |
| L-02 | Changing a file requires a separate preliminary parsing and permitted workspace | C-07, C-08, C-09 (document passport 19) | `file_analysis_completed`, `action_intent_committed` | `AC-L02`: editing without accepted parsing and evaluation of connections is prohibited |
| L-03 | External consequences require confirmation or a precise order issued in advance; particularly significant actions are not transmitted to the system | C-01, C-09, C-14 (document passport 19) | `permission_requested`, `permission_issued` | `AC-L03`: General switch does not allow new recipient, payment or sensitive transfer |
| L-03a | The decision is made by the owner in the native trusted window with a full description | C-02, C-14 (document passport 19) | `owner_event_received`, `permission_issued` | `AC-L03A`: Web text, file and model cannot create confirmation |
| L-04 | One right has an exact scope and does not expand after confirmation | C-01, C-09 (document passport 19) | `permission_issued` | `AC-L04`: Changing the recipient, data, number or amount destroys the applicability of the old solution |
| L-04a | The term is precise, visible and historical; perpetuity is a clear choice | C-09, C-10, C-14 (document passport 19) | `permission_issued`, `permission_expired` | `AC-L04A`: Sleeping, restarting and changing clocks do not renew eligibility |
| L-04b | One right is revoked without stopping all work; Late results will not be accepted | C-03, C-09, C-10 (document passport 19) | `permission_revoked`, `late_result_rejected` | `AC-L04B`: revoking one right preserves unrelated rights and blocks late response |
| L-05 | After an unexpected restart, only recovery and report; old units are waiting to “continue” | C-10, C-13 (document passport 19) | `recovery_report_ready`, `resume_waiting_owner`, `resume_authorized` | `AC-L05`: no old attempt starts before trusted command |
| L-05a | Missed schedule can only read, reason and check freshness | C-05, C-06, C-13 (document passport 19) | `scheduled_run_recovered` | `AC-L05A`: Missed run doesn't catch up with file change or external effect |
| L-05b | Connection replacement is only possible for pure operation and proven equal connection without data expansion and limit | C-06, C-07, C-09 (document passport 19) | `connection_selected`, `fallback_rejected` | `AC-L05B`: A similar model name via another unsafe path is not considered a replacement |
| L-06 | Prepared and immediate stop are different state machines | C-10, C-14 (document passport 19) | `prepared_stop_requested`, `emergency_stop_started`, `stop_confirmed` | `AC-L06`: immediate stop does not wait for the model; prepared reaches safe points |
| L-06a | The screen does not show the final stop until all processes are actually fenced | C-10, C-14 (document passport 19) | `process_tree_terminated`, `stop_confirmed` | `AC-L06A`: Laggard process holds "stopped" state |
| L-07 | The general limit of the owner is insurmountable until a new decision | C-01, C-03, C-09 (document passport 19) | `execution_slot_reserved`, `budget_limit_reached` | `AC-L07`: two parallel armor does not exceed the last balance; model replacement does not bypass the limit |
| L-08 | Local search can be broad, but the model only gets a narrow selection of the selected memory mode | C-04, C-07 (document passport 19) | `context_projection_created`, `data_disclosure_denied` | `AC-L08`: the found personal archive is not transferred entirely to one performer |
| L-09 | Child executor and ability do not automatically inherit anything | C-01, C-09, C-11, C-12 (document passport 19) | `permission_delegated`, `delegation_denied` | `AC-L09`: New ability without separate lease does not read file or use network |
| L-09a | Development allows only an explicit narrowing of rights with a visible recipient, duration and depth of transfer | C-01, C-09 (document passport 19) | `permission_delegated` | `AC-L09A`: each child is provably narrower than the parent along all axes |
| L-10 | Internal review creates a candidate; final completion of the main work is accepted by the owner | C-01, C-05, C-08, C-14 (document passport 19) | `completion_candidate_created`, `owner_completion_accepted` | `AC-L10`: model word "ready" does not terminate |
| L-11 | The owner is alarmed only when there is a real need, a critical threat or a ready outcome | C-05, C-06, C-14 (document passport 19) | `retry_exhausted`, `owner_notification_emitted` | `AC-L11`: one hundred repetitions of one temporary cause create one incident and not one hundred messages |
| L-12 | New connection goes through artificial tasks and work in the shadows | C-06, C-08, C-09 (document passport 19) | `connection_admission_tested`, `connection_shadow_observed` | `AC-L12`: an unverified connection does not receive real personal data and does not influence the choice |
| L-13 | The system itself can determine the adequacy of the improvement only outside the protective base and after independent tests | C-08, C-16 (document passport 19) | `version_staged`, `version_promoted` | `AC-L13`: Candidate cannot change rights, guard, own checks or enable itself |
| L-14 | Canonical data, solutions and negative results are not automatically removed | C-04, C-13 (document passport 19) | `evidence_sealed`, `storage_protection_entered` | `AC-L14`: Compressing and cleaning derived data does not remove the original source |
| L-15 | The main screen has a simple graph and peer list with full disclosure details | C-14 (Document Passport 19) | `ui_projection_built` | `AC-L15`: map and list show the same state and the same actions without losing position |
| L-15a | By default, View, Source, Destination, and Validation are visible; sensitive content opens separately | C-09, C-14 (document passport 19) | `sensitive_view_requested` | `AC-L15A`: the main screen does not reveal the contents of the sensitive fragment |
| L-15b | The full log is available to the owner and only narrowly authorized local audits | C-04, C-07, C-14 (document passport 19) | `evidence_scope_granted`, `data_disclosure_denied` | `AC-L15B`: the performer cannot request a full log of his main work |
| L-16 | The main job and the physical workplace are different entities; the location can be changed using a proven solution | C-01, C-05, C-09 (document passport 19) | `workspace_binding_changed`, `file_target_verified` | `AC-L16`: changing the visible path does not change the identity of the work and does not give access to the entire disk |

---

## 3. Engineering solutions - 26 lines

| Solution | Fixed behavior | Code owner and main function | Mandatory Beacon | Readiness check |
|---|---|---|---|---|
| I-01 | All modes use the same defense path; private instructions can only narrow | C-02, C-05, C-09, C-12 (document passport 19) | `action_preflight_passed` or failure event | `AC-I01`: ordinary conversation, deep work and ability have no second path of action |
| I-02 | The old staging inside Hermès has been cancelled; normal self-conversation application uses the same units, rights and log | C-05, C-14, C-15 (document passport 19); no Hermes change code | ordinary events units and actions | `AC-I02`: stopping Hermes does not break Deep Create's conversation; chat does not bypass the intermediary |
| I-03 | State and event are recorded by one transaction of one local database | C-03, C-05 (document passport 19) | `state_transition_committed` | `AC-I03`: failure at each point gives a completely old or new version |
| I-04 | The global epoch is stored outside the normal restored copy | C-10, C-13 (document passport 19) | `authority_epoch_advanced` | `AC-I04`: the old copy of the database does not reduce the epoch and does not revive rights |
| I-04a | After restoration, a limited mode is in effect until local verification and confirmation of the owner | C-10, C-13, C-14 (document passport 19) | `recovery_restricted`, `resume_authorized` | `AC-I04A`: Incompatible version or open action prevents restriction from being lifted automatically |
| I-05 | The intention is written before the action; unknown outcome is first checked | C-03, C-09, C-13 (document passport 19) | `action_intent_committed`, `action_outcome_unknown`, `action_reconciled` | `AC-I05`: Losing a response after sending does not cause the second effect |
| I-06 | There is a complete private directory of paths; unknown path prohibited | C-01, C-02, C-09 (document passport 19) | `action_path_denied` | `AC-I06`: New action string view does not reach files, network or process |
| I-07 | Before the operation, a real open Windows object is checked | C-09, C-12 (document passport 19) | `file_target_verified`, `file_target_denied` | `AC-I07`: Redirect, short name, network path and target spoofing do not expand scope |
| I-08 | Connection is verified by model, vendor, address, input, connector, form and rules | C-06, C-09 (document passport 19) | `connection_registered` | `AC-I08`: the same names of different connections remain different entities |
| I-09 | Automatic replacement requires maintaining quality, data, facilities, service, limits and independence | C-06, C-07, C-08, C-09 (document passport 19) | `connection_selected`, `fallback_rejected` | `AC-I09`: degraded or unequal replacement is prohibited with the exact reason |
| I-10 | Independence is determined by origin, not role title | C-01, C-08 (document passport 19) | `verification_completed`, `verification_independence_failed` | `AC-I10`: the same model, selection and method are considered a second opinion |
| I-11 | Reduction preserves purpose, prohibitions, open conditions, negative paths, rights and changes; another examiner is looking for losses | C-04, C-07, C-08 (document passport 19) | `summary_candidate_created`, `summary_candidate_verified` | `AC-I11`: artificially removed denial or revoked right is detected |
| I-11a | Continued rechecks target, files, sources, permissions and open actions | C-07, C-13 (document passport 19) | `resume_authorized` or exact failure | `AC-I11A`: A change to the source file or an expired source is blocking the old selection |
| I-12 | The branches are isolated, share a common limit, and retain negative results; combined total is checked | C-01, C-03, C-06, C-08 (document passport 19) | `hypothesis_branch_created`, `hypothesis_branch_merged` | `AC-I12`: Branch collision does not lose negative pin and does not exceed budget |
| I-13 | Instructions are versioned to suit the model and task; fixed base stays short | C-07, C-08, C-16 (document passport 19) | `prompt_bundle_compiled`, `adaptation_profile_tested` | `AC-I13`: setting up a weak model changes only the sixth layer and passes closed tasks |
| I-14 | Goal, unit, selection, right, result, check, transition and beacon have a strict form and clear description | C-01, C-02, C-05, C-14 (document passport 19) | corresponding typed event | `AC-I14`: Unknown field or state is rejected; screen explains machine failure |
| I-14a | Stopping times are measured by type of performer; review immediately prohibits late commit | C-03, C-09, C-10, C-12 (document passport 19) | `prepared_stop_fencing_started`, `late_result_rejected` | `AC-I14A`: slow and fast performer use different proven timing |
| I-14b | Warning, health check and ownerlessness have separate measured thresholds | C-06, C-10, C-11 (document passport 19) | `worker_heartbeat_observed`, `worker_orphaned` | `AC-I14B`: a normal long pause does not create a false repetition of the action |
| I-14v | Sensitive Comparisons Use Separate Purpose Secure Code | C-04, C-07, C-12 (document passport 19) | `sensitive_match_recorded` no value | `AC-I14V`: The same data for different purposes does not have a comparable open fingerprint |
| I-15 | The new version is prepared separately and included in its entirety; mix of versions is prohibited | C-13, C-16 (document passport 19) | `version_staged`, `version_promoted` | `AC-I15`: the absence of one file blocks the candidate from starting; failure leaves the old or new version entirely |
| I-16 | Integration into the current Hermes canceled | C-05, C-13 (document passport 19); no Hermes change code | `app_started` without Hermes | `AC-I16`: removed or disabled Hermes does not affect startup and recovery |
| I-16a | Support for the old and new Hermes order is simultaneously withdrawn; transitions exist only between steps Deep Create | C-16 (passport document 19); no Hermes change code | `version_promoted` | `AC-I16A`: the assembly is missing the old Hermes architecture switch |
| I-16b | Removing the old Hermes path is not Deep Create's job; the ban on silent removal of beacons remains | C-16 (passport document 19); no Hermes change code | `assurance_map_mismatch` | `AC-I16B`: Candidate who removes mandatory beacon will not be admitted |
| I-17 | Inability to securely record or lack of space blocks change and action; have a physical emergency supply | C-03, C-04, C-10, C-13 (document passport 19) | `storage_protection_entered`, `emergency_reserve_consumed` | `AC-I17`: Filling each affected volume stops effects before log loss |
| I-18 | The architecture change updates the affected checks and may not reduce their strength | C-08, C-16 (document passport 19) | `assurance_map_verified`, `version_admission_rejected` | `AC-I18`: A candidate who changes a rule without checking will be rejected |
| I-19 | The old study is retained with reasons; legacy Hermes statements do not carry over as valid architecture | C-15 (document document 19); decision history is stored in documents | documentation review record | `AC-I19`: automatic counting of 51 lines and manual verification of replaced solutions |

---

## 4. Cross-cutting mandatory prohibitions

The following prohibitions are covered by several solutions and are tested separately:

| Number | Ban | Basic solutions |
|---|---|---|
| SZ-01 | text does not create law | L-03a, I-01 |
| SZ-02 | the model does not change the state directly | L-10, I-03, I-14 |
| SZ-03 | the performer does not bypass the intermediary | L-09, I-01, I-06 |
| SZ-04 | the secret is not included in the context and journal | L-05b, L-15b, I-14v |
| SZ-05 | unknown outcome does not repeat | I-05, I-09 |
| SZ-06 | late result does not apply | L-04b, I-14a |
| SZ-07 | old copy does not return rights | I-04, I-04a |
| SZ-08 | no hard limit | L-07, I-12, I-17 |
| SZ-09 | the original source is not replaced by retelling | L-14, I-11 |
| SZ-10 | the author does not allow his own protective change | L-13, I-18 |
| SZ-11 | screen is not a source of state | L-03a, L-15, I-03 |
| SZ-12 | Hermes is not a condition of employment | I-02, I-16, I-16a |

---

## 5. Additional requirements from section 6 of document 07

The 51 numbered answer does not cover separately formulated additions. The following groups retain each item listed in the original section; they cannot be lost after successfully counting 51 lines.

| Number | Accurate Source and Coverage | Code Contracts | Check |
|---|---|---|---|
| D-01 | 07 §6.1: separate project on D, switchable Hermes source, rules for transitioning from a conversation, preliminary display of settings with a regular phrase | C-05, C-14, C-15 | AC-D01: works without Hermes; phrase changes only the sentence until confirmed |
| D-02 | 07 §6.2: purpose and dependencies of the file before editing; separate analysis if there is a lack of information; factual trace of actions | C-07, C-08, C-09 | AC-D02: incomplete disassembly blocks actual replacement; the parsing result is associated with the action |
| D-03 | 07 §6.3: multiple major works, adjacent area, rights connections, timing and history, separate permission handling, appearance preservation, meaningful animation | C-05, C-09, C-10, C-14 | AC-D03: two works per card/list; waiting for the right does not lose position and does not disclose data |
| D-04 | 07 §6.4: last confirmation, canceling prepared stop, saving forms, failure to complete with reason, post-start settings | C-05, C-10, C-13, C-14 | AC-D04: Reconstructed form is not considered confirmation; secret input is not saved |
| D-05 | 07 §6.5: long-term operation, steady state, recovery separate from continuation, independent safe branches | C-03, C-06, C-10, C-13 | AC-D05: interrupted operation is waiting; separate work saves valid path |
| D-06 | 07 §6.6: unified connection registry, all types of limits, reasonable quota checks, verified replacement, secure input of secrets | C-01, C-03, C-06, C-09, C-14 | AC-D06: unknown quota does not become unlimited; changing connection does not bypass the limit |
| D-07 | 07 §6.7: knowledge into skills, comparison of versions and scope of applicability, primary source, independent admission outside the protective framework | C-04, C-07, C-08, C-16 | AC-D07: improvement with harm or change of own rules is not included |
| D-08 | 07 §6.8: substitution/damage/rollback, emergency stock, stop without log, different fate of primary and temporary data | C-03, C-04, C-10, C-13 | AC-D08: shortened valid log is not recognized as current; filling the volume blocks the consequences |
| D-09 | 07 §6.9: a random name is not a defense; stable identity of the entity and separate verification of the real object | C-04, C-09, C-12 | AC-D09: knowing the name does not grant access; object substitution is detected regardless of name |
| D-10 | 11 §9.5: light, medium, high, very high and extreme depth; general rights, origin and limits | C-01, C-06, C-07, C-08 | AC-D10: All five settings go through the same inhibition matrix; depth does not remove mandatory verification |

###Additional edge cases after review 2026-09-07

- AC-L07-RestoreBudget: returning the old base does not reset the actual or unknown expense; merging by operation/reserve/observation numbers does not count it twice. If there is insufficient evidence, the budget is blocked and `budget_recovery_blocked` is recorded.
- AC-Prompt-Envelope: model fields of identity, expense and fingerprints do not become trusted; `provider_request_adapted` is fixed.
- AC-Dispatch-Freshness: changing the tariff, right, recipient or desired source in the queue blocks the old dispatch; `dispatch_preconditions_checked` or failure is recorded.
- AC-Verify-Bounded: insufficient independence or exhausted check limit gives `verification_independence_failed` / `verification_limit_reached`; there is no endless check of inspectors.
- AC-Parallel-ReadSet: unlinked branch does not make the result obsolete; changing the read dependency does.
- AC-Scoped-Stop: stopping one job does not increase the global epoch of the others; an effect that has already been sent after a review is not declared canceled.
- AC-Checkpoint-Ack: A failure between the master base commit and the checkpoint does not issue a success to the owner or trigger the dependent action. The receipt does not create an event of itself; the expected vertex conflict is rejected.
- AC-Restore-Lineage: a verified old copy receives a new line only after the owner has been accurately confirmed indicating the lost tail. Fixed `restore_lineage_authorized`; the previous era, consequences and reference to the checkpoint are not erased.

## 6. Automatic matrix completeness check

### Clarifications on external analysis 2026-09-08

This is a traceability of document 23's twelve comments, not new owner responses. Complete settings and experimental results are in the specified passports; All tests of the application have only been specified and not completed.

| Note | Contracts | Key checks | Beacons or the reason for the absence of a new one | Full passport |
|---|---|---|---|---|
| R-01 | C-01, C-03, C-05, C-06, C-07 | AC-XW-01, AC-XW-03, AC-XW-05, AC-XW-15, AC-XW-17 | `result_publication_committed`, `publication_eligibility_changed`, `cross_work_dependency_changed` | 24 §§3–5/10; 15 §2.5a |
| R-02 | C-01, C-03, C-05, C-06 | AC-SCH-01, AC-SCH-02, AC-SCH-03, AC-SCH-10, AC-SCH-13 | `root_turn_dispatched`, `protected_capacity_claim_created` | 25 §§3–4/10 |
| R-03 | C-03, C-05, C-09, C-10, C-12, C-13 | AC-COMMIT-01, AC-COMMIT-02, AC-COMMIT-06, AC-COMMIT-08, AC-COMMIT-10 | `commit_backpressure_entered`, `queued_command_rejected_stale` | 25 §§5–10 |
| R-04 | C-01, C-03, C-09, C-10, C-12, C-13 | AC-STATE-04, AC-STATE-06, AC-STATE-08, AC-STATE-09 | Existing transition events; test report does not create new production writer | 26 §§2–3/6.1 |
| R-05 | C-01, C-03, C-05, C-08, C-09, C-12, C-13 | AC-XW-08, AC-XW-09, AC-XW-10, AC-XW-12, AC-XW-13, AC-XW-16 | `workspace_conflict_detected`, `workspace_reconciliation_blocked` | 24 §§6–10 |
| R-06 | C-04, C-05, C-06, C-08 | AC-CAL-01, AC-CAL-02, AC-CAL-04, AC-CAL-05, AC-CAL-09 | `calibration_evidence_recorded`, `method_qualification_changed` | 26 §4/6.2 |
| R-07 | C-03, C-04, C-05, C-08, C-10, C-13, C-16 | AC-CAL-07, AC-EVO-01, AC-EVO-03, AC-EVO-05, AC-EVO-07, AC-EVO-10 | `evaluation_holdout_exposed`, `scoped_rollback_completed` | 26 §5/6.3 |
| R-08 | C-05, C-14 | AC-OV-01, AC-OV-02, AC-OV-03 | `ui_projection_refresh_required`; regular redraw does not wake up the model | 23 §4 |
| R-09 | C-01, C-05, C-06, C-14 | AC-OWNER-01, AC-XW-01 | Existing acceptance and grant events; there is no new automatic acceptance by the owner | 23 §5; 24 §3 |
| R-10 | C-03, C-04, C-05, C-07, C-09, C-13 | AC-STORE-01, AC-STORE-02, AC-STORE-03, AC-STORE-04, AC-STORE-05 | `evidence_location_committed`, `evidence_location_unavailable`, `evidence_relocation_interrupted` | 23 §6 |
| R-11 | C-02, C-12 | AC-PLATFORM-01 | Regular refusal to unknown sender; network product not added | 23 §7; border V1 v13 |
| R-12 | C-01, C-03, C-08, C-09, C-10, C-13, C-16 | AC-STATE-07, AC-STATE-09, AC-STATE-10, AC-STATE-11, AC-EVO-09 | Existing subject events plus reproducible test report | 26 §§2–3/6; 20 §19 |

### What the documentation tool checks

Before each accepted version of documentation and code, the following is checked:

1. in decision map 07 there are exactly 51 headers `Л-*` or `И-*`;
2. in this matrix, each number appears exactly once in the first column of the corresponding section;
3. no unknown number;
4. each line has non-empty code, beacon and check, or it is clearly written why there is no production code;
5. the canceled direction does not have a hidden working path;
6. all contracts C-01—C-16 are covered by a numbered solution or addendum; each significant function has a passport in such an agreement;
7. a new significant function, type of event or action requires updating the passport and affected scenarios.

The `checks/verify_design.ps1` tool checks numbers, links, the presence of contracts, the form of tables and code blocks. It does not check the meaning of all promises and does not launch an application that is not yet present. Semantic verification and testing cannot be replaced by searching for words.

Additionally, it checks the 12 note numbers between 23 and this table, the presence of scripts explicitly named here in 23–26, and the event names in catalog 18. This is a search for broken requirement references, not a proof of the sufficiency of the scripts or the correctness of all functions.

---

## 7. Honest running total

- owner decisions: **51 out of 51 closed**;
- technical purpose: **51 out of 51 compared**, additional 10 groups of requirements;
- source code: **not yet created**;
- completed application acceptance scenarios: **0**, their description is not a passed test;
- the right to say “the system works”: **absent until implementation and end-to-end evidence**.

The result of checking the documentation and the result of the working version have different reasons. The original list of unproven properties is preserved in historical document 22; clarifications and current verification are linked through document 23 and the navigator. Numbers 51/51 indicate a comparison of solutions, not the completeness of a proven implementation.
