#Deep Create: Owner Response Card

**Date of analysis:** 2026-09-04  
**Last border closure date:** 2026-09-05  
**Initial Questionnaire:** `ВОПРОСЫ_ДЛЯ_ЗАВЕРШЕНИЯ_ПРОЕКТИРОВАНИЯ_20260902.md`  
**Purpose:** to record what the owner decided, what he changed, what he commissioned the architect to design, and how the last borders are closed.

## 1. Major change of direction

Deep Create is no longer considered part of Hermes. It should become a separate project and a separate application in a new workspace on disk `D:`. Communication with Hermes is permitted only as a separately controlled way to obtain desired conversations, memory or other data.

This solution has an advantage over the wording of the old questionnaire. Therefore, questions about embedding Deep Create into the current Hermes are not considered accepted by default: their previous formulation is canceled, and useful requirements must be transferred to the device of a stand-alone application.

## 2. How to understand grades

- **Accepted** - the boundary has been resolved quite accurately. The option you choose may differ from my previous recommendation.
- **Accepted with changes** - the basis of the answer is clear, but the owner has added mandatory conditions or replaced the original version with his own.
- **Delegated to Architect** - The owner has set the intention and constraints and commissioned the complete internal solution to be designed without new choices on his part.
- **Canceled by change of direction** - the question related to integration into Hermes and is no longer needed in its previous form.
- **Remained ambiguous** - the answer touched on a neighboring topic, but did not close exactly the border that was asked about.

The phrase “think deeper” next to an already selected option does not cancel the choice. It means that the option has been accepted, and its complete structure must be developed by an architect. A simple wish “that there will be no failures” does not negate the mandatory development of disaster recovery.

## 3. Overall comparison result

A total of 51 questions were addressed:

| State | Quantity |
|---|---:|
| Accepted | 23 |
| Accepted with modification | 20 |
| Delegated to Architect | 4 |
| Canceled by change of direction | 4 |
| Remained ambiguous | 0 |

The last two solutions - **L-05** and **L-07** - were closed by the owner on 2026-09-05. There are no new custom forks left from the 51 answer mapping.

## 4. Personal decisions of the owner

### L-01 — Accepted with changes

A permanent switch is created:

- if it is enabled, the system may offer to go to deep work, explain the reason and run the proposal through a general check;
- if it is turned off, the transition is possible only by the direct choice of the owner.

One model does not have the right to arbitrarily enable deep work. The proposal should be a decision of the management part of the system, and not of an individual performer.

### L-02 — Accepted with changes

The system can change local files, but before recording it must create a separate unit of preliminary parsing: establish the purpose of the file, connections with other parts, permissibility of the change and possible damage. Every action of the model is fully recorded in the main log.

The original restriction of "explicitly selected project only" has been expanded. The new mandatory limit is set by the permitted workplace and the result of the preliminary check. This extension cannot be interpreted as access to the entire disk.

### L-03 — Accepted with changes

Option A is taken as the basis: sending, publishing, purchasing, paying, changing an account, transferring personal data and acting in another service require confirmation.

Added an autonomy switch for pre-expected external work, such as ongoing communication with clients. Even with autonomy enabled, particularly significant cases should be addressed to the owner.

There is still a direct contradiction here, described in Section 7: the “confirm every time” rule and the right of the system itself to decide that confirmation is not needed cannot operate simultaneously. The architect must separate the permanent pre-granted right from the actions that can never be transferred to the system.

### L-03a — Accepted with changes

Confirmation is created with a separate, clear button next to the full description of the action. The card should show not only the name of the file or service, but also the purpose, connection with the main work, the expected consequence, the recipient and the information being transmitted.

Confirmations are visible both in the list and on the general work graph. The decision should be made only by the owner in the trusted window of the standalone application. The model's response, the contents of the file, site or extension are not permission.

### L-04 — Accepted with changes

One permit may cover:

- one precise action;
- clearly shown list of actions;
- the owner’s own condition with permitted and prohibited parts.

The volume is always shown before confirmation and is stored without further expansion. A separate switch could allow the system to suggest an interpretation of the scope, but not silently increase it upon confirmation.

### L-04a — Accepted with changes

The duration is specified exactly in seconds, minutes or hours. The quick selection must have at least one hour, eight hours, and a lifetime permit. If the owner instructs the system to select a period, the selected value is still shown until confirmation.

Permissions are handled in a separate service conversation so as not to clog up the meaning of the main work. Valid permits are always visible, count down, and are sorted from closest to expiring. The history of issued, rejected and expired permits is saved.

### L-04b — Accepted with changes

An individual permit can be revoked without stopping the entire work. Other safe work continues. The feedback immediately reaches the executing part and prohibits the late result from being accepted.

Details are revealed on the spot without moving to another page or losing position on the screen. A clear confirmation of intent is shown before permission or prohibition. States differ in text, color and quiet animation.

### L-05 — Adopted by an additional decision

The answer definitely requires persistence through normal restarts, sudden shutdowns, and power loss. After launch, the system must restore the goal, state, unfinished actions and evidence, carry out reconciliation and be able to continue working at the owner’s command.

The owner chose option A: before the new “continue” command, the system only restores, checks integrity, identifies unknown outcomes and prepares a report. The old performance units, including reading and reasoning according to the old plan, do not automatically resume.

This does not cancel the separate L-05a about the missed launch on schedule.

### L-05a — Accepted

Option B is selected. The missed run can itself only perform reading, reasoning, and verification if the source information is still fresh. Actions with consequences remain unconfirmed.

### L-05b — Accepted with changes

Option A is selected. Self-replacement of the connection is allowed only for work without external consequences, through a connection that has been previously recognized as equal, without expanding data transmission and exceeding the total cost limit.

Additionally, you need a single directory of connections: service, intermediary, exact model, login method, available options, types of restrictions and test results. The owner should conveniently add service, models and secrets both through settings and by direct command in a trusted conversation. The secret should not end up in ordinary conversation or the main magazine.

### L-06 — Accepted with changes

The main button opens two types of stopping the main work:

1. **Immediate stop.** Immediately prohibits new actions, revokes rights, terminates performers and does not allow them to consolidate late results.
2. **Prepared stop.** Receives owner's explanation, selects safe termination points, saves state to continue, and then terminates.

A final clear question of intent is needed before use. A prepared stop can be canceled within a short pre-determined period; An immediate stop cannot be reversed retroactively. Individual branches should have their own controls that do not change the meaning of the main button.

### L-06a — Accepted with changes

Option A is selected, but the intermediate state may be shown not as a word, but as a constantly visible sign on the button and graph. The final termination cannot be shown until all related performers have been confirmed to have stopped and their late results have been prohibited.

The interface should take into account work that lasts several days: sending a new message, changing a task, stopping a branch and stopping the entire work should not merge into one action.

### L-07 — Adopted by an additional decision

The answer defined in detail the accounting for the availability of models: unlimited, monetary, time, interest and subscription restrictions; checking quota restoration no more often than a reasonable period of time; continuation through a suitable connection instead of dropping the entire work.

The owner chose “yes”: the general limit of money, time, paid volume, number of calls or other spent quota established by him is an insurmountable prohibition until a new explicit decision of the owner. The system may suggest changing the limit, but not apply it itself.

### L-08 — Accepted with changes

Deep Create does not get one eternal memory mode. At startup, an access level is selected, and in the most independent mode, the local control layer can search through all pre-connected data and determine the usefulness of the information itself.

The knowledge found can be turned into testable skills, tools, and improvements. This does not give each external contractor the entire archive: the transfer of a specific model is still limited to a working selection of information. It is this division that eliminates the contradiction with L-09 and L-15a.

### L-09 — Accepted

Option A is selected. The child executor or extension does not automatically inherit anything. Each call receives only the minimum information and rights explicitly listed; expansion requires a new solution.

The proposal to randomly change folder and file names is recorded as a separate security hypothesis, and not as an already accepted security measure. A random name by itself does not protect data and cannot replace rights verification.

### L-09a - Delegated to the architect

The owner rejected the too narrow permanent limitation of option A and instructed to derive the rule from the general philosophy of system development.

Mandatory boundaries for the architect's decision: rights are not silently inherited; recipient and deadline are visible; the transfer cannot be broader than the right of the parent; network, file changes and external consequences require stronger checking; the first launch should not permanently deprive the system of the opportunity to develop.

### L-10 — Accepted with modification

Each major work requires a separate full review, a detailed report on what was done, the location of the results, connections and evidence. The owner then confirms or rejects the completion. If it is rejected, he indicates the reason, the system checks it and either continues the work or explains why the condition has already been met differently.

Thus, option B is actually chosen: the final completion of the main work is approved by the owner. The internal technical completion of individual safe units of work can be recorded by a machine and does not require a separate question to a person.

### L-11 — Accepted with changes

Option A is selected. The user is disturbed only when it is really impossible to continue without it, safe recovery methods have been exhausted, or the final result is ready.

An individual performer's request to “call the person” is not, in and of itself, grounds for notice. The control layer first checks to see if its authority and safe paths have been exhausted. Repeats of the same cause are combined into one incident.

### L-12 — Accepted

Option A is selected. New models and connection methods are first tested only on artificial or previously impersonal tasks without external consequences and work in observation mode.

### L-13 — Accepted

Option B was chosen, contrary to the previous recommendation: the system itself determines when there is enough evidence to enable its own improvement.

This decision is valid only together with I-01 and I-18. Self-development does not have the right to disable the general protective layer, falsify evidence, change the rules of its own verification, or expand its powers. Otherwise, a direct contradiction arises, as indicated in section 7.

### L-14 — Accepted

Option B selected: Important input data, decisions and evidence are not automatically deleted. It is permissible to create condensed representations, new connections and more convenient forms, but they must not destroy the original source.

This solution requires external or expandable storage and a stopping threshold when running out of space. Infinite storage is not possible on a finite disk; the contradiction with I-17 is discussed in section 7.

### L-15 — Accepted with changes

The main presentation should be simple, calm and understandable without learning internal terms, but all detailed information remains available upon expansion. Instead of one regular conversation, you need a graph of major work and units of work. The node opens in the adjacent area as text, check progress, visual representation, or other suitable view.

Maintaining position on the screen, smooth transitions, small meaningful animations, darkening and blurring backgrounds under important windows, clear resolution states, and a consistent visual language are a must. Beauty should not hide the source, the recipient, the consequence, the risk and the solution required.

### L-15a — Accepted

Option A is selected. By default, the information type, source, recipient, and verification result are shown. Sensitive content can only be opened by a separate action by the owner.

### L-15b — Accepted with changes

The detailed log is available to the owner and to those local parts of the system that actually need specific records for routine inspection or recovery. No one else gets full access.

Access by the system itself means issuing a limited working selection, rather than allowing any model to read the entire magazine.

### L-16 — Delegated to the architect

The owner rejected the mandatory preliminary binding of all work to one folder and instructed to design the organization taking into account the following requirements:

- the workplace can be specified with a regular phrase and later replaced;
- the system itself removes suitable work locations from the task, but does not gain hidden access to arbitrary folders;
- main works, branches, results, permissions and beacons remain visible and linked regardless of the physical location of the files;
- selected settings survive restart;
- the controls remain simple, consistent and fairly self-explanatory.

Consequently, the physical folder and the user-visible main work become different entities. The architect should provide a complete diagram of their connection.

## 5. Engineering solutions

### I-01 — Accepted

Option A is selected. One common execution protection layer is required for normal conversation, deep work, child executors, background runs, verifiers, and extensions. The instructions of individual modes can only narrow the rights.

### I-02 - Canceled by change of direction

The question was framed around the usual Hermes conversation. Deep Create is now a separate application, so the previous staging has been cancelled.

The mandatory replacement requirement remains: a normal conversation of a new application must be able to use part of the capabilities of Deep Create, and any action from the conversation still receives a common unit of account, rights, preliminary check and final record. Settings can be set both by screen elements and by the owner’s usual phrase; the system shows exactly what it is going to change.

### I-03 — Accepted with changes

Option A is chosen: one local base indivisibly changes state and adds an ordered event; the screen and board are built only from it.

A mandatory requirement for protection against substitution and damage has been added: integrity checks, access control, backups and restoration should not allow history to be rewritten unnoticed or old rights to be resurrected.

### I-04 — Accepted

Option B is selected. The valid authorization number is stored separately from the normal restored copy; reinstated permits are considered invalid until re-verification.

### I-04a — Accepted

Option A is selected. After recovery, a separate local check verifies integrity, authority, part versions, and open actions. The owner confirms the removal of restrictions.

### I-05 — Accepted

Option A is chosen. Before the external action, a stable intention and a single key of action are preserved. When the outcome is unknown, it is first determined whether the action occurred; blind repetition is prohibited.

The requirement to prevent such conditions as much as possible is accepted as a reliability goal, but not as a permission to remove unknown outcome processing.

### I-06 — Accepted

Option A is selected. A complete list of implementation methods is maintained; everyone goes through a common intermediary and verification. Unknown path is prohibited until listed and tested.

### I-07 — Accepted

Option A is selected. The actual open Windows object is checked immediately before the operation. Links, transitions, short names, network paths and target substitution cannot expand the permitted scope.

### I-08 — Adopted with changes

Option A is selected and expanded to a single machine connection identity. It should eliminate different names for the same connection and the same names for different connections. In addition to these features, any differences that may change quality, safety, tools, data transmission, restrictions or cost are taken into account.

### I-09 — Accepted

Option A is selected. Replacing the connection without a new question is permissible only if the requirements for quality, information, tools, service, costs and independence are maintained, and the previous attempt must be clean or fully clarified.

### I-10 — Accepted

Option A is selected. A different role or a new conversation does not, by itself, create independent verification. The model, service, instructions, sources, working selection and origin of the result are compared; acceptable proximity is determined on hidden test tasks.

### I-11 — Adopted with changes

Option A is selected. When pruning, the target, prohibitions, open conditions, rejected paths, negative checks, effective rights, and file changes are stored separately. Another examiner looks for losses and inverted meaning.

A requirement has been added to preserve primary sources and all information necessary for subsequent verification. Compression creates a new access layer, but does not replace the original history.

### I-11a — Accepted

Option A is selected. The current versions of files and sources, effective rights, external actions, and the last accepted target are separately checked before continuing.

### I-12 — Accepted

Option A is selected. The branches operate in separate copies, divide the total spending limit in advance, store the negative findings, and after merging, the final total result is checked.

### I-13 - Delegated to the architect

The owner did not choose one of the three options and requested that the only permanent approach not be fixed. Instructions and inputs must change with the model's capabilities, task, and test results.

The architect is tasked with creating a changing, versioned way of assembling instructions: short non-cancelable boundaries, separate knowledge for a narrow area, varying degrees of step-by-step support for different models, and evidence-based comparison of options. One huge instruction and a complete absence of rules are equally not accepted.

### I-14 — Accepted

Option A is selected. The goal, obligations, work selection, rights, unit of work, result, verification, state change, and beacon have an accurate machine verifiable appearance and a human-readable explanation.

### I-14a — Accepted

Option A is selected. Soft and forced stop times are measured separately for each type of performer. Revocation of the right immediately prohibits later consolidation of the result and does not wait for the expiration of time.

### I-14b — Accepted

Option A is selected. For each type of performer, normal pauses are measured and separate periods for warning, health checks and recognition of the process as ownerless are set.

### I-14v — Accepted

Option A is selected. For sensitive value matching, only the secure fingerprint is stored, generated by the local secret separately for each comparison target. The value itself does not appear in the general log.

### I-15 — Accepted

Option A is selected. The associated new version is built separately and included in its entirety only after verification. After a failure, the entire old or entirely new version is launched, but not a mixture of both.

### I-16 - Canceled by change of direction

Deep Create is not being introduced as part of the current Hermes. A standalone application is created. The old question about replacing the Hermes operating procedure is closed as no longer valid.

The useful requirement remains in a new form: Deep Create's own parts are still introduced in stages - surveillance, artificial tasks, limited local actions and only then expansion of powers.

### I-16a - Canceled due to change of direction

Simultaneous support of the old and new Hermès order is no longer the chosen transition. Instead, a stand-alone application needs verifiable transition conditions between its own stages and versions.

### I-16b - Canceled by change of direction

Disabling the old Hermes path is no longer part of the Deep Create launch. At the same time, the ban on quietly removing mandatory beacons remains a general rule: a self-developing part cannot remove evidence of its own security.

### I-17 — Adopted with changes

Option A is chosen, but the reaction is increased. If there is not enough space or it is impossible to reliably record a mandatory beacon, the state change operation stops and a clear requirement to free up space appears. You cannot continue protected actions with the promise to complete the journal later.

An emergency reserve of space is needed in advance for critical entries and completion of the stop. Simple conversation without action can only explain the problem and help prepare for safe cleaning; Continuing the main work until the log is restored is prohibited.

### I-18 — Adopted with changes

Option A is selected. Each architecture change updates the affected checks or proves that the previous checks completely cover it. A mismatch blocks switching on.

Added responsibility for the system to develop its own verification schemes, tools, recovery scripts, compressed views and versions of solutions. No such improvement can itself reduce the force of mandatory verification.

### I-19 - Delegated to the architect

The owner did not formally choose A or B, but ordered a deep revision of the previous materials after a change in direction. The architect must correct three known documented inconsistencies, test the legacy beacon map for applicability to a stand-alone application, retain useful findings, and explicitly remove only outdated statements about the Hermes device.

This is not permission to throw away old research. Every decision saved, changed and rejected must have a reason.

## 6. New mandatory requirements that were not in the original questionnaire

### 6.1. Self-service application and connection with Hermes

1. Create a new project on disk `D:` and do not consider it part of the Hermes development.
2. Formalize the connection with Hermes as a separate, disconnectable and verifiable source of data.
3. Link a normal conversation of a new application with Deep Create according to custom rules.
4. Allow the owner to specify settings with a common phrase, but show the exact change before applying.

### 6.2. Preliminary analysis of each change

1. Before changing the file, find out the purpose, dependencies, scope of use and possible damage.
2. If there is insufficient information, create a separate unit of analysis rather than allowing one conversation to guess.
3. Record each actual action of the model and its basis in the general journal.

### 6.3. Work schedule and human-readable permissions

1. Show several main works on a common graph.
2. Open each node in the adjacent area without losing the location where the user was.
3. Show the relationship of the authorization to the main work, unit of work, recipient, information and consequence.
4. Support exact deadline, countdown, sorting, history and immediate revocation of permits.
5. Separate permission processing from the semantic conversation of the main work.
6. Maintain scroll position, expanded nodes, and selected view.
7. Use calm animations and color only as an additional explanation, and not as the only carrier of meaning.

### 6.4. Accounting for human error

1. Dangerous decisions require explicit final confirmation.
2. A prepared stop can be canceled within a limited visible period.
3. An erroneous transition on the screen does not reset the position and filled data.
4. Failure to complete returns the work for review with the owner's reason.
5. Settings and incomplete forms are restarted.

### 6.5. Long-term work and recovery

1. Maintain basic work lasting several days.
2. Save the state often enough so that sudden shutdown does not destroy the meaning of the task.
3. After launch, distinguish between state restoration, freshness check and actual continuation of actions.
4. Don't block independent secure branches while one branch is waiting for owner permission.

### 6.6. Connections, restrictions and model replacement

1. Store a single editable directory of services, models, intermediaries and login methods.
2. Consider monetary, time, interest, subscription and unlimited restrictions.
3. After the confirmed exhaustion of the time quota, check its restoration at a reasonable interval, and not every second.
4. Replace the connection only with a previously tested equal one and do not ruin the entire main work due to one failure.
5. Add secrets only through secure input; the regular model and the main log do not receive their plaintext.

### 6.7. Developing memory and self-improvement

1. Convert re-validated knowledge into versioned skills, tools and tips.
2. Check the improvement, degradation and scope of each new version.
3. Do not delete the original source after compression.
4. Allow the system to suggest and, to the extent permitted, introduce improvements on its own after sufficient evidence.
5. Do not allow the improved part to change its own irrevocable boundaries, rights and rules of evidence.

### 6.8. Storage protection and space pressure

1. Detect substitution, damage and rollback of the main database.
2. Have an emergency reserve of space to record a failure and safely stop.
3. When the threshold is reached, show a clear requirement to free up space and not continue actions without a log.
4. Separate the indefinite storage of important data and temporary technical details; if the owner prohibits deletion at all, consider transferring it to expandable storage.

### 6.9. Random names as a hypothesis, not a defense

Changing the names of temporary folders can make primitive substitution more difficult, but it does not replace rights, real object verification, and protected internal numbers. Persistent entities must have persistent internal numbers; only temporary paths that are not shown as a source of truth can be accidental. This idea needs to be tested separately before application.

## 7. Direct contradictions and a mandatory way to remove them

### 7.1. Confirm every external action or let the system decide for itself

In L-03, mandatory confirmation of each external action and a switch are simultaneously selected, allowing the system to decide for itself whether it can act.

**A way to resolve the contradiction:** the switch does not retroactively cancel confirmations. It creates a pre-limited type of permanent right. Payments, purchases, account changes, transfer of new types of personal data, changes in the right itself and actions above a certain threshold remain non-transferable to the system.

### 7.2. Wide memory search or minimal data transfer

L-08 asks for almost no search restrictions, and L-09, L-12 and L-15a require minimal transmission and depersonalization.

**A way to resolve the controversy:** the broad search is performed by a trusted local part. The external model receives only the selected working selection. The right to find information and the right to convey it are different rights.

### 7.3. Self-improvement or unchangeable protection

L-13 allows the system to introduce improvements on its own, while I-01 and I-18 prohibit bypassing the protection and enabling changes without proof.

**The way to resolve the contradiction is** to separate the variable area of ​​skills, instructions and work tools from the protected basis of authority, journal and checks. The system can independently upgrade the version only in the modified area; changing the protected basis requires the owner.

### 7.4. Never delete data or continue working on the destination drive

L-14 prohibits automatic removal, and I-17 requires stopping if there is not enough space. On a finite disk, both requirements cannot be met indefinitely.

**Way to resolve the contradiction:** primary sources are not deleted until they are transferred and checked on another repository. If there is no location or destination, the system stops working. Compression helps search, but does not count as freeing up space if the source is stored nearby.

### 7.5. Free change of working folder or exact rights on the path

L-16 allows you to change the workplace with a simple phrase, and L-02 and I-07 require a precise access boundary.

**Way to resolve the contradiction:** the owner’s phrase creates an offer for a new workplace. Before the first change, the system shows the real path and consequences, after which a new exact law is created. The old law is not carried over in silence.

### 7.6. Random names or sustainable recovery

The name change proposed in L-09 could destroy the links, log, and restoration required by I-03, I-11, and L-05.

**Way to resolve the contradiction:** the magazine refers to stable internal numbers and secure prints. A random name is only allowed as a temporary external representation and is never the identity of the object.

### 7.7. The system configures itself or the owner manages dangerous settings

I-02 asks to allow artificial intelligence to set settings by phrase, while L-03, L-04 and I-01 require explicit permissions.

**A way to resolve the contradiction:** the model only prepares an exact set of changes. Without confirmation, only pre-listed harmless display settings can be applied. Rights, data, costs, external actions, storage and self-improvement are changed only through a secure procedure.

## 8. What can now be considered closed

The following architectural directions can be considered closed:

- a separate application instead of the mode inside Hermes;
- a general protective layer below all conversations and performers;
- a single source of wealth and separate protection against the return of old rights;
- precise rights, log, safe work with paths and unknown outcome;
- independent verification, verifiable history reduction and re-verification of freshness;
- isolated parallel branches and checking the overall result;
- two types of stops and a ban on late results;
- artificial testing of new connections;
- detailed, but disclosed upon request, picture of the work;
- mandatory update of the checklist when the system changes.

## 9. What remains engineering work

User boundaries are closed, but this does not prove the application is ready. The next stage remains:

1. detailed formats of status, events, rights, beacons and results;
2. precise separation of processes and protective boundaries of the first assembly;
3. a complete list of paths to files, programs, networks and external services;
4. tests of crashes, disk full, rights revocation, unknown outcomes and recovery;
5. First self-assembly and end-to-end inspection.

These items are an engineering responsibility and should not be developed into new issues without a true change in ownership, risk to the data, or allowable expense.

## 10. Rule for the next design branch

The next branch begins not with a correction of the old Hermes plan, but with a new system boundary:

> A self-contained local application, Deep Create, manages long-running core jobs, models, memory, permissions, checks, and recovery. Hermes is one possible data source, but is not a state master and is not a required runtime environment.

Each further decision must be checked by four questions:

1. What right of the owner does it preserve or limit?
2. Which local part makes the decision and which only proposes it?
3. What record proves that the decision actually happened?
4. How does the system return to an understandable state after an error, incorrect human action or sudden shutdown?

If at least one answer is missing, the part has not yet been designed and should not proceed to creation.
