# Deep Create: the result of technical design and the boundaries of what has been proven

> Historical audit report 2026-09-07. After an external review, the contracts were clarified 2026-09-08: [decisions and additions](./23_EXTERNAL_CRITIQUE_AND_REFINEMENTS_20260908.md). The numbers and prints of the previous check below refer to its saved version and do not certify subsequent edits. Application tests that failed did not become passed after updating the documentation.

**Date:** 2026-09-07  
**Item:** standalone Deep Create application, not a remake of the current Hermes.  
**Result:** the technical basis for consistent implementation has been prepared. A working application has not yet been created; The safety and usefulness of the future code has not yet been proven.

## 1. What exactly is there now?

Instead of one description of roles, interrelated agreements have been prepared. They answer not only the question “what do you want to get,” but also the questions “who has the right to change it,” “what data is transmitted,” “what is considered a refusal,” and “what test is used to detect a violation.”

| Document | What does it define |
|---|---|
| [13. Technical Constitution](./13_TECHNICAL_CONSTITUTION.md) | Application Boundaries, Basis Selection, State Owners, Fixed Rules, and Security Assertion Constraints |
| [14. Data, states and recovery](./14_DATA_STATE_AND_RECOVERY.md) | subject records, valid transitions, versions, consumption, recording order, recovery after failure and old copy |
| [15. Context, memory and instructions](./15_CONTEXT_MEMORY_AND_INSTRUCTIONS.md) | what does the model get, where does the information come from, what is saved during reduction, how does a model response differ from a local ID |
| [16. Execution and verification](./16_EXECUTION_MODELS_AND_VERIFICATION.md) | connection selection, work division, concurrency, retries, differing checks, acceptance and termination |
| [17. Rights and Actions](./17_PERMISSIONS_SECURITY_AND_ACTIONS.md) | precise powers, their narrowing and withdrawal, secrets, verification of the real purpose of the action and the last limit before the consequence |
| [18. Beacons and stop](./18_BEACONS_JOURNAL_STOP_AND_RECOVERY.md) | mandatory observations, silent operation without repeated messages, independent stopping, lack of space and control points |
| [19. Code Map](./19_CODE_FUNCTION_AND_DEPENDENCY_MAP.md) | 16 Contracts of Code Parts: Future Path, Caller, Input, Output, Allowed Change, Exact Failure and Validation |
| [20. Creation procedure](./20_IMPLEMENTATION_ORDER_AND_READINESS_EVIDENCE.md) | stages of implementation, prohibited premature opportunities and the necessary evidence to move on |
| [21. Responsibilities Matrix](./21_DECISION_CODE_VERIFICATION_MATRIX.md) | connection of each of the 51 number solutions with a code, verification and beacon; 10 more groups of requirements from detailed answers |

File 19 is the main entry point for future implementation. File 21 is a means to avoid losing your solution when changing the code. Files 14–18 explain the exact meaning of these treaties rather than being replaced by summary tables.

## 2. What was actually done in the check

The work was not done in one pass:

1. Technical parts were divided into areas: data and security; model path; code and creation sequence.
2. Individual agents renegotiated data/protection/stop and context/execution contracts. Their saved clarifications are summarized in documents 14–18. Map 19 was then reassembled by the main performer: without multiplying public functions by each stage name.
3. A separate review of documents 19/13/20 relative to 15/16 found five specific discrepancies. All five have been corrected and re-matched by the main artist; list below.
4. An end-to-end reconciliation of related points was carried out: consumption after the return of the old base, different owners of intentions, confirmation of a control point, stopping of individual work, a list of beacons.
5. Created and implemented a local documentation structure checker. It reads documents and produces results; it does not launch an application, models, network, or actions with user data.

These are not several independent positive conclusions about the entire system. Independent review had the specified area; The final assembly and re-verification of the rules was carried out by the main executor. There are no production code tests here.

### Five discrepancies from a separate viewing

| Found discrepancy | Why was it dangerous | Accepted fix |
|---|---|---|
| Immediate stop has always enhanced the global era | stopping one job would extinguish the rest | separate fencing of work/unit/rights; common era - for the entire installation |
| The input of the general function and the number of fixation prerequisites diverged between files | could have skipped sender ID or final verification | single chain of certified `ValidatedCommand` and three inputs `commit_transition`, including `CommitPreconditions` |
| Parsing the response had different error names and incomplete input | handlers may have understood the failure differently; service fields lacked explicit verification | full contract from 15 §2.9: expected scheme, restrictions, local wrapper and certified mediator trail |
| The request conversion and last check did not have a named owner | gap between the prepared order and the actual dispatch | internal conversion for C-07; checking the exact body and current powers inside C-09 |
| The order of creation was the names of the forms, but not the explicit creation of circuits and tests | description of the structure could be declared to be a working parser | at stage 6, real diagrams of all nested forms, test examples and exact failures are required |

## 3. What deeper errors were prevented?

### 3.1. An instruction with a ban is not equal to a valid ban

Normal conversation, deep work, tester, skill, and self-change suggestion all use the same consequence path. None of them get independent network or file crawling. The instruction tells the model the boundaries; the intermediary checks the real rights regardless of the model text.

The downloaded skill may suggest a step or material. He is not the owner of your powers. Even if the model believed the harmful text, it is required to separately prove that the proposal could not be implemented. Therefore, acceptance checks not only the model’s response, but also the actual absence of prohibited reading/sending in all modes.

### 3.2. Exact intent does not mean valid clearance

The entitlement, recipient, source file, rate, or connection may change between scheduling and sending. The last check applies to the final body after transformation and queue. An old photograph of a license and a link to a previous permit are not a multiple-use pass.

Revocation and commencement of action have a general admission procedure. If the action began before the fence, the system stores the observed or unknown outcome. It does not claim that a request that has already been sent or an external consequence disappeared when you clicked “stop.”

### 3.3. Two stories don't make two bosses

The control process stores the subject intent in the main database. The mediator maintains a stable acceptance and observation in a protective base because it must survive the failure of the control process. Records are linked by the same number, but have different purposes.

Only certified observation by the intermediary confirms the fact of dispatch and consumption available to us. Moving an observation to the main story does not trigger the action a second time. The documentation does not promise a common transaction between different databases.

### 3.4. Reducing Memory Shouldn't Reduce Responsibility

The source material is stored separately from the retrieved memory and working selection. The reduction is a candidate with origin and loss verification, not a new undisputed truth. Lost denial, prohibition, unknown outcome and changed decision are separate grounds for refusal.

Parallel work does not use the general event number as a version of all knowledge: the result depends on the specific editions read. Otherwise, an active unrelated branch would continually render the useful results of the remaining branches obsolete.

### 3.5. The model does not fill out her own certificate of entitlement and expense

She writes `ModelResult`: content, statements, hypotheses, open questions and suggestions. The local part creates `TrustedAttemptEnvelope`: attempt, connection, dispatch and flow observations. `ResultPacket` is assembled from them.

This is not a limitation on creative content. The answer contains free explanation and hypotheses. A closed form is needed where the application distinguishes a proof from a proposition and data from control fields.

### 3.6. Returning an old copy does not refund the money spent.

The resource journal has one owner. Confirmed consumption, valid reservation and withheld unknown outcome are not summed up twice. The intermediary stores evidence of the upper bounds before sending, so that the lost tail of the main base does not erase the possible flow.

When returning an old copy, the account is restored using stable numbers from the available certificates. If the safe balance cannot be determined, new consumption is prohibited. An empty line after recovery does not mean that the action did not occur.

### 3.7. Writing to disk and confirming to the owner are different boundaries

After recording the main base, the guard separately confirms the control point. Before it, you cannot show an accepted success or start a dependent action. A failure between steps leaves a verifiable unfinished tail, rather than silently wasted work.

A checkpoint receipt does not create an event that requires a subsequent receipt of itself. Returning a verified old copy creates an explicitly verified new line with a connection to the old one; does not overwrite counters and unknown consequences. Consistent rollback of the entire machine outside of this protection.

## 4. What prevents the appearance of useless code

The rule is not to write fewer lines at any cost. Every significant part must have a verifiable responsibility.

The public function from Document 19 has a caller, input, result, valid change, and failure defined. Additionally fixed:

- no function without a real caller or necessary responsibilities;
- pure computing does not create its own log and does not require a beacon for each call;
- the name of the stage does not mean a new public redirector function;
- the state is changed by a single owner, and not several identical “managers”;
- the package or process appears at its stage, and not empty “for the future”;
- changes in rights, consumption, acceptance and restoration require updating the affected checks;
- information about the impact of the change is stored next to the contract, and not just in the developer’s memory.

The requirement of “not a single unnecessary line” has not been declared already proven: there is no source code for the application. When implemented, it is checked by change, caller, scripts and responsibility, not by a fancy line count or overall coverage percentage.

## 5. How the documentation is checked

Tool: [verify_design.ps1](./checks/verify_design.ps1). Run from this directory:

```powershell
& .\checks\verify_design.ps1 -SelfTest
& .\checks\verify_design.ps1
```

The first run checks 13 test cases of the tool itself: unclosed block of code, invalid close, empty cell, escaped delimiter, missing/repeated/foreign number, local references, and invalid encoding. The second one checks the real documents.

Machine check subject:

- availability of documents 00 and 07–22, strict coding and closed code blocks;
- exact one-to-one correspondence between 51 number solutions and 51 rows of the matrix;
- 10 non-repeating additional groups, 16 non-recurring contracts and their coverage with lines;
- non-empty required columns and the existence of local files via links;
- the presence of named matrix beacons in the catalog and the absence of repeated names;
- the absence of several specific corrected discrepancies in the recording of contracts.

Saved results from the final run: [structure check](./checks/design_check_20260907.json) and [tool test cases](./checks/design_checker_self_test_20260907.json). The first contains the start time, the number of checks and SHA-256 of the documents read and the tool itself. Fingerprints associate a report with specific bytes; it is not a digital signature or proof of security.

The tool does not understand the meaning of each promise, does not open external links, does not check the transition to each header anchor, does not run Windows isolation, and does not prove the absence of vulnerabilities. The AC list is future tests of the application, not completed tests.

## 6. What else cannot be considered proven

These items are not hidden under the word “ready” and do not require a new set of general questions for the owner. For each, an engineering action and a dependent step are specified.

| Number | What has not yet been proven | How to get proof | What is prohibited before this |
|---|---|---|---|
| P-01 | real limitations of the Windows executor and descendants | attempts to read a prohibited file, establish a network, exit the tree and spoof a local channel | real personal data and registered programs; C-11/C-12 |
| P-02 | trusted input separated from page and model | substitution of a form, an old edition, message repetition, someone else’s sender; checking the declared threat model without promises against the machine administrator | real rights and entering secrets; C-02/C-14 |
| P-03 | fencing, fixation and transfer of action are ordered | controlled pauses before/after admission, death of the holder, stuck mediator, late response; evidence from both sides of the order | real consequences; C-03/C-09/C-10/C-12 |
| P-04 | specific encryption provides the stated storage | secure the library, versions, format and key recovery; check temporary, log, emergency files and decoy secrets | real personal data; steps 1/3, C-04/C-12 |
| P-05 | storage survives required power outage | separately process death and loss of power/lower recording layer; checking the main and protective bases, integrity and confirmation boundaries | promise of long-term sustainability; steps 3/4 |
| P-06 | operation operates on a resolved file object | substitution of link/path/object between steps, replacement failure, lack of space and file being locked by another process | real recording of user files; stage 8 |
| P-07 | connection has the desired shape, upper cost and verification method | boundary requests, schemes, changing the tariff in the queue, loss of response, lack of external one-time use, reconciliation of consumption | connection usage under a hard limit; stage 7 |
| P-08 | subject announcements have become executable forms | canonical types, schemas, constraints, and all closed errors; forward/backward conversion and incorrect examples | adoption of model packages; steps 2/6 |
| P-09 | reduction, model selection and verifiers are useful for our tasks | closed examples with negations/contradictions, comparison of configurations, scope of applicability and deterioration | including a customization/skill as a verified enhancement; C-07/C-08/C-16 |
| P-10 | long-term work does not repeat the consequences and does not lose restrictions | chain with crash, restart, old copy, lost tail and two independent jobs | status of the working first version; section 15 of document 20 |

The names Rust/Tauri/SQLite are not enough: you need a real list of dependencies and a reproducible build. Recording a DPAPI or Job Object in a project is also not a substitute for testing its use.

## 7. What is completed and what is not

| Subject | Honest status |
|---|---|
| Owner's decisions | represented by source map 07; are not rediscovered or replaced here |
| Purpose of future parts of code | described in 16 contracts, associated with decisions and inspections |
| Interdocumentary errors listed in this report | corrected; the order is reflected in the affected sections |
| Structural Check | reproducible; the result applies only to the specified fingerprints |
| Proof of the absence of all design errors | absent; does not follow from several views |
| Source code for standalone Deep Create | not created at this stage; check 2026-09-07 did not find the catalog `D:/Projects/DeepCreate` |
| Passed application acceptance tests | 0 |
| Current Hermes and its profile | did not change at this stage |

The result is not “the whole system is ready.” The bottom line is that “there is a technical basis on which you can start building and verify the meaning of each change without passing off platform hypotheses as guarantees.”

## 8. Next work in order

Start from stage 1 of document 20: separate workspace and minimal reproducible assembly. Then - subject types and pure transitions, an artificial chain of custody/faults, a protective boundary, and only after testing it a real connection and user files.

There is no need to re-run export studies, change the owner model, configure the current Hermes, or create all future processes in advance. When a platform limitation is discovered, the specific contract and dependent checks are changed, rather than the rule you accepted being silently weakened.

Matrix 21 is updated with each responsibilities implemented: the exact file, the accepted change, the actual test and its result. It is this connection that will subsequently allow us to say “this function fulfills its role” with a verifiable basis.
