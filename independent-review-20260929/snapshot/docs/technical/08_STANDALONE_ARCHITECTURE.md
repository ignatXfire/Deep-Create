# Deep Create Single Application Architecture

**Date:** 2026-09-04  
**Status:** design solution for the next thread of discussion; a product code is not created by this document.  
**Basis:** Owner responses to questionnaire, design completion questionnaire, and `DEEP_CREATE_BEACON_AND_ASSURANCE_SPEC_20260829.md`.  

## 1. The main decision made

Deep Create is being created as a **standalone application and standalone project**.

Hermes:

- is not the basis of Deep Create;
- does not store the current state of Deep Create;
- does not issue Deep Create permissions;
- does not launch internal parts of Deep Create;
- does not decide whether Deep Create is completed;
- does not update Deep Create;
- can only be one of the sources of previous conversations, files and other information through a separate limited bridge.

This solution replaces the previous branch, in which Deep Create was introduced gradually inside Hermes. Now, a safe transition does not mean replacing Hermes parts, but creating a new application, testing it separately and only then enabling the optional Hermes data reader.

## 2. Why simply separating the folder is not enough

Independence is not determined by the project name or by a separate window. It only exists when Deep Create has its own:

1. main control process;
2. local status base;
3. main journal of actions and decisions;
4. storage of evidence and working materials;
5. storage of secrets;
6. procedure for issuing and revoking permits;
7. list of model connections;
8. a protective layer in front of files, networks, programs and external services;
9. regular chat;
10. procedure for recovery after a failure;
11. update procedure;
12. own control screen.

If at least the main state, permissions, or completion confirmation are still taken from Hermes, the application remains a dependent add-on. Therefore, communication with Hermes is only permitted at the input level.

## 3. Fixed rules of the new border

These rules must remain true in any future development of the application.

### 3.1. Deep Create continues to work without Hermes

Once the main work has started, closing, deleting or breaking Hermes should not stop Deep Create. Already accepted information is stored in Deep Create's own storage along with its provenance.

### 3.2. Hermes is not the source of rights

The phrase "permit" found in an old Hermes conversation is a historical text. It does not create a valid permission in Deep Create.

Records of previous permissions, schedules, running processes and connections may be shown to the owner as a reference, but are always accepted as inactive.

### 3.3. The bridge does not get the right to act

The first and main version of the bridge only has the right:

- read Hermes data explicitly selected by the owner;
- describe the data found;
- create a secure copy for verification;
- transfer the resolved part to the Deep Create input storage.

The bridge cannot:

- change the Hermes base;
- delete or rename Hermes materials;
- run Hermes tasks;
- read the secrets of Hermes;
- issue permissions to Deep Create;
- perform external actions;
- quietly transfer new information after completing one transfer.

### 3.4. No shared mutable base

Hermes and Deep Create should not simultaneously write to the same database or consider the same file as their current state. The common mutable base creates an unanswerable question: who was the last legal author of the state.

Only the transfer of an immutable set of information with an exact origin and a secure fingerprint is acceptable.

### 3.5. Bridge failure does not equal Deep Create failure

If the Hermes data format has changed, the path is no longer accessible, or the integrity check fails, only the next transfer is blocked. Regular chat, ongoing core work, and Deep Create's local content continue to operate within the rights already granted.

## 4. Parts of a separate application

The general chain looks like this:

```
Команда владельца
  → точная запись цели и ограничений
  → подбор нужных сведений
  → разделение на единицы работы
  → выбор подключения модели
  → ограниченное поручение исполнителю
  → результат и доказательства
  → независимая проверка
  → единственное принятие состояния
  → обновление графа, списка и отчёта

Любое действие с последствиями
  → проверка действующего разрешения
  → запись устойчивого намерения
  → защитный посредник
  → наблюдение настоящего исхода
  → сверка либо состояние «исход неизвестен»

Выбранные данные Hermes
  → ограниченный мост
  → изолированная входная копия
  → проверка и отбор
  → хранилище доказательств Deep Create
  → нужная рабочая подборка
```

| Part | Problem | What she has no right to do |
|---|---|---|
| Receiving commands | Accepts owner actions from the chat and control screen, creates one accounted event. | Does not change state directly. |
| Primary Operations Manager | Stores the purpose, prohibitions, obligations, branches and state of all main work. | Does not bypass the protective layer and declare completion without checking. |
| Compiler of the working selection | Selects only the necessary information for a specific performer and task. | Does not give out the full magazine, secrets or other people's materials. |
| Permissions Service | Creates, limits, times, revokes and checks permissions. | Does not accept the model's words as the owner's permission. |
| Model connection selection | Selects the exact working connection based on task, quality, data, price and service status. | Does not replace connection only by a similar model name. |
| Protective Mediator | Checks every call to files, network, programs and external services before execution. | Does not allow unknown execution path. |
| Results Collector | Accepts the performer's result along with the origin and links to evidence. | Does not turn the performer's statement into an established fact. |
| Inspector | Reviews commitments, resulting files, implications, and independence of review. | Does not extend rights or assert anything that has not been verified. |
| The only point of state acceptance | Accepts or rejects a state change in one ordered entry. | Does not trust a screen, a model, or a single branch as the final truth. |
| Screen Builder | Shows the main work graph, list, permits, status, costs and evidence. | Does not store its own different truth. |
| Bridge to Hermes | Transports explicitly selected information through a validated input. | Does not gain power over Deep Create or act on its behalf. |
| Updating part | Prepares a new whole version, checks and turns it on. | Does not leave a mixture of old and new parts. |

All parts that can lead to action are below one general protective order of Deep Create. Normal chat, deep work, child executor, background work, reviewer, and extension do not receive separate workarounds.

## 5. Boundaries of trust

### 5.1. Owner

The owner is the only source of initial permission and the only person to whom the complete log is available. Protected local parts of the system can read from it only what is needed to verify, restore, or build the owner's chosen view. The model and extension do not receive full access.

The owner can set permanent rules in advance, but such rules still have a precise scope, duration, limit of consequences, and the ability to be immediately revoked.

### 5.2. Secure Local Core Deep Create

Only this part has the right:

- issue status version numbers;
- record received events;
- issue internal limited rights based on the owner’s decision;
- accept the final change of state;
- confirm that the stop is actually completed;
- transfer the work to the states “completed”, “partially completed”, “requires improvement” or “blocked”.

Even the application's own screen sends a request to the kernel rather than changing state directly.

### 5.3. Models and child performers

Any model is considered capable of making a mistake, misunderstanding the text, or trying to go beyond the assigned area. Therefore she can:

- propose a goal, plan, action or new resolution;
- get a limited working selection;
- return the result and explanation;
- request extension of rights.

It cannot itself create a right, forge a confirmation of an action, record a final state, or declare a job complete.

### 5.4. Extensions and external programs

The extension only receives the intersection of three constraints:

1. what the owner has authorized in advance;
2. what the current unit of work is allowed to do;
3. what is stated and verified for the extension itself.

Unknown nested access to a file, network, or program is prohibited. The extension's signature or known name does not give it additional rights.

### 5.5. Hermes

Hermes is considered an external source of information and not a trusted part of the core. The reason is not that Hermes is necessarily dangerous, but that Deep Create's independence requires that all external origins be vetted equally.

Hermes history may contain:

- outdated solution;
- previous permission;
- erroneous model output;
- a hidden command inside someone else's text;
- a file that changed after the conversation;
- sensitive data.

Therefore, everything received through the bridge undergoes the test of purpose, freshness, origin, sensitivity and contradictions.

### 5.6. Model providers and external services

A successful service response is not proof of which model was running, what data it received, or whether the external action was executed. These facts are confirmed by the local connection module and, where possible, by a separate verification of the result.

## 6. Deep Create storages

Vaults are separated because state, evidence, secrets, and permissions have different threats and different recovery rules.

| Storage | What's inside | Basic rule |
|---|---|---|
| Home local base | Main activities, units of work, branches, goal versions, commitments, states, ordered events, and evidence links. | One state change and its event are taken into one indivisible record. |
| Main Magazine | An unchanging sequence of critical decisions, attempted actions, results, failures, stops and recovery. | The recording is created by the local performer, not the model. |
| Evidence Vault | Complete files, test results, transferred materials and confirmation of external consequences. | Access is granted separately for a specific purpose. |
| Vault of Secrets | Keys, entry tokens and other secret meanings. | Models and the general log only receive an opaque secret designation. |
| Current Government Journal | Only increasing number of authorities, reviews and protective recovery marks. | A normal backup cannot reduce this number. |
| Bridge Entrance Storage | Unmodified copies of selected Hermes data prior to acceptance. | Everything new is initially isolated and does not affect existing tasks. |
| Working copies of branches | Separate copies of files for parallel work. | Branches do not write to the same work object at the same time. |
| Emergency Records Stock | A small pre-saved space for critical status and failure messages. | Not used for normal details. |

### 6.1. Where should this data not be stored?

A valid Deep Create state cannot be placed:

- inside the Hermes data folder;
- inside the Deep Create source code;
- in the conversation history;
- only in the memory of the running model;
- only on the screen;
- only in regular text reports.

The application's source code, its production data, user projects, and backups should be four different areas.

### 6.2. Choosing precise paths

The application project is created separately under `D:/PUBLIC_DATA`, as the owner decided. However, the exact project folder name, permanent data folder, and backup location must be approved before creation.

Recommended limit:

- project code - a separate folder under `D:/PUBLIC_DATA`;
- permanent application data - a separate folder, not located inside the code and Hermes;
- temporary working copies - a separate managed area;
- backup copies - a separate medium or a separate folder that does not coincide with the current data.

### 6.3. Storage without automatic deletion

At the owner's discretion, important information is not automatically deleted. This applies to:

- direct decisions of the owner;
- critical beacons;
- evidence of completion;
- negative checks;
- rejected paths;
- the origin of the data;
- history of issuance and revocation of rights;
- history of changes to files and system versions.

Condensation, consolidation of repetitive technical details, and re-archiving of older material are permitted, but only if the original important content remains accessible and verification ensures that no meaning is lost. Lack of space is not a permission to silently delete history: the system warns the owner in advance, and when a dangerous limit is reached, it stops new actions.

## 7. The only truth and the main magazine

### 7.1. What is the truth

The final state is constructed only from the accepted sequence of events of the main local base. The graph, task list, regular chat and reports are reflections of this truth.

If the screen lags, it must show which version of the state it has reached. It cannot show the old "completed" as the current state.

### 7.2. What is recorded for each important action

The entry must link:

- owner's original command;
- main work;
- exact version of the target;
- unit of work and attempt;
- the issued right and its validity period;
- working selection of information;
- precise connection of the model;
- preliminary solution of the protective layer;
- intention to perform an action;
- observed outcome or “outcome unknown” state;
- the result obtained;
- checking the result;
- accepted or rejected state change.

Time by itself is not enough. You need a sequence number, the immediate cause of the event, the overall number of the associated chain, the attempt number and one stable key for the repeatable action.

### 7.3. What is prohibited from writing in a general journal

The following are not included in the general journal:

- complete requests and responses from models;
- contents of personal files;
- keys and entry tokens;
- full network addresses with secret parameters;
- sensitive parts of messages;
- names of private objects, if an opaque reference is sufficient for verification.

Matching uses secure fingerprints generated by the local secret separately for each comparison target. The complete details remain in the evidence store and are opened by the owner in a separate action.

### 7.4. If the log is not available

Until the intention is permanently recorded, the protected action does not begin. If a state change cannot be saved, it is not accepted.

Special case of lack of space:

1. new actions and changes to files stop;
2. the minimum emergency level is maintained if the reserve is still available;
3. The screen shows a direct message “Not enough disk space. Make room";
4. the work does not continue to create large answers and temporary materials;
5. after freeing up space, integrity checks, re-reading of the log and reconciliation of unfinished actions are performed;
6. Only after successful reconciliation can work continue.

If the refusal is not about a location, but about an additional detailed record, a normal conversation can remain available without action. If it is not possible to reliably store even the conversation state and the user's decisions, new work does not start.

## 8. Permissions

### 8.1. Resolution is a separate machine entry

The permission is not stored by the phrase alone. It contains at least:

- who gave him away;
- for what main job;
- for which unit of work;
- what action is allowed;
- with what data;
- for which recipient or path;
- to what extent are the consequences and costs;
- is it possible to transfer part of the right to a subsidiary contractor;
- exact start and end time;
- number of the current government;
- condition for immediate revocation.

### 8.2. The selfie switch is not a full resolution

The “system can decide itself” switch enables the right to make decisions only within a predefined area. It does not provide an unlimited right to send any data, pay any amount, or change any account.

For constant independent work, for example communicating with clients, a long-term rule is created with precise:

- types of messages;
- named recipients;
- valid data;
- limit of amount or consequences;
- duration of action;
- cases of mandatory contact with the owner.

An unusual amount, a new recipient, a new type of personal information, an unclear purpose, or a noticeable departure from previous work requires a separate decision by the owner.

### 8.3. Confirmation is performed by a separate part

The conversation that proposed the action should not itself interpret the owner's response as a permission favorable to him. Separate part:

1. shows the owner the action, purpose, recipient, information, consequences and connection to the main work;
2. accepts permission, refusal or the owner's own text;
3. turns it into an accurate recording;
4. transmits only the final limited document to the executing part;
5. records the decision in the general journal.

### 8.4. Deadline and review

The period is counted by the local service in seconds. An expired or revoked permit ceases to be valid immediately. An already working contractor cannot complete a later action under the old permit.

The screen shows active permissions at the nearest end time, and open-ended ones at the end. The owner can open a permit in place, revoke it, and see which attempts are stopped by the revocation.

Waiting for one permission does not stop independent units of work that do not require it.

### 8.5. What cannot be transferred from Hermes

From Hermes are never automatically transferred as valid:

- permissions;
- self-sending settings;
- schedules;
- unfinished external actions;
- rights to folders;
- extension rights;
- saved keys;
- trust in the supplier or model.

## 9. Regular chat inside Deep Create

Regular chat is created as part of a new application, and not as a Hermes window.

It is needed for:

- quick questions without starting a lot of main work;
- discussion of the project and results;
- natural control of settings;
- preparing a new main job;
- viewing the status and explanation of errors;
- transfer of comments to the current main work.

### 9.1. Transition to deep work

If the corresponding switch is turned on, the system may offer to turn the conversation into the main job. The decision should not be based on the opinion of one answer from the model, but on verifiable features: duration, number of obligations, the need for several performers, external consequences, the need for restoration and long-term storage of the state.

The system shows the reason for the transition and the boundaries of the new main job. If the switch is turned off, the transition occurs only upon direct command from the owner.

### 9.2. Actions from regular chat

Any change to a file, launch of a program, access to a network or external service creates a lightweight unit of work. It goes through the same permissions, path, data, and outcome checks as deep work.

Regular chat without actions is not required to create a complete main work.

### 9.3. Settings in natural language

The user can write: “Set up the launch yourself” or “Add a new model connection.” The system should:

1. understand the desired outcome;
2. show exactly which settings will change;
3. separate benign screen preferences from settings that affect rights, data, costs and external actions;
4. apply harmless changes within the limits of a pre-given rule;
5. request separate confirmation for sensitive changes;
6. write down the old and new values;
7. allow the previous state to return.

This way, conversation makes it easier to manage without becoming a bypass of the exact rules.

### 9.4. Order from any text source

Deep Create can have single sign-on for text from native chat, a selected file, another local application, or an external model response. This allows you to transfer the order to the desired main job without manually switching multiple settings.

But the origin of the text is always preserved. Only a message acknowledged as an owner command changes the target or settings. Text from another model, from Hermes, a file or a network is considered an offer or input material and cannot itself issue a right, initiate a payment, open a new folder or change a security rule.

## 10. Models and working connections

### 10.1. Not only the model name is taken into account

One working connection is determined by the combination of:

- exact model name;
- supplier;
- service addresses;
- login method;
- connecting module and its version;
- method of transferring tools;
- settings for the depth of reasoning;
- permissible size of the working selection;
- type of task;
- rules for storing and transmitting data;
- observed limitations and errors.

Two identical names with different connection paths are considered different connections.

### 10.2. Connection secrets

The key is entered by the owner or accepted by a special local part and immediately placed in the secret store. The model receives only the connection designation and the permitted operation. The key is not included in her query, master log, graph screen, or transfer from Hermes.

### 10.3. Model selection

The system selects the connection based on the type of unit of work and pre-established requirements:

- quality;
- necessary capabilities;
- permitted tools;
- sensitivity of information;
- supplier;
- price and type of restriction;
- independence from other performers;
- current health history.

The choice is recorded with a reason. The role name itself does not define the model.

### 10.4. Restrictions and temporary unavailability

For each connection the following are stored separately:

- money price;
- general limit;
- time limit;
- the share of the remaining subscription, if it can be observed;
- time of the last successful call;
- type of last failure;
- the nearest permitted time for a careful re-inspection.

Once the time limit has been reached, the service is not checked every second. It receives a deferred minor check after a set interval. An unexpected single failure does not destroy all major work.

### 10.5. Replacing the connection

Automatic substitution is only permitted when the new connection maintains the requirements for quality, data, tools, supplier, costs and independence. The status of the previous attempt must be clean or completely clear.

If equality is not proven, the unit of work waits, returns a partial result, or asks the owner to change the requirements. The system doesn't pretend that a similar title means equal work.

### 10.6. What information does the model receive?

The model does not receive the entire application memory, but a specially assembled working selection:

- the purpose of this particular unit of work;
- mandatory restrictions;
- necessary initial information;
- exact rights;
- expected type of result;
- links to available tools;
- only necessary previous decisions and negative conclusions.

The strength of the model affects the amount of explanation and step-by-step support, but does not override machine boundaries. General, unchanging instructions remain short; niche knowledge and requirements are added only by task.

This does not mean that Deep Create's capabilities will be permanently limited. The control system can search all allowed memory and request new sources if the current information is insufficient. It simply does not transfer everything accumulated to each individual model for no reason. The expansion of the working set is recorded, explained by purpose, and subject to data sensitivity testing.

## 11. Transferring data from Hermes

### 11.1. Recommended first order

The first version uses an explicit one-way wrap:

```
Выбор владельца
  → чтение выбранного источника Hermes
  → неизменяемая копия во входном хранилище
  → проверка вида, размера, происхождения и целостности
  → поиск секретов и скрытых команд в содержимом
  → предложение состава переноса
  → принятие разрешённой части
  → запись происхождения в Deep Create
  → использование только через рабочую подборку сведений
```

Permanent shared changes to data are not permitted. Retransfer creates a new version of the source, rather than quietly rewriting material that has already been used.

### 11.2. What can be carried

When explicitly selected, the following are allowed:

- messages from selected conversations;
- attached files;
- names and dates of sessions;
- user-defined preferences;
- design solutions;
- verification reports;
- links to source local files;
- information about the previous performance of models as an observation, but not as a valid health assessment.

### 11.3. What requires separate processing

The following data cannot be accepted in plain text without special verification:

- personal data;
- large investments;
- executable files;
- command line scripts;
- archives;
- texts with instructions for the model;
- links to the network;
- data whose origin cannot be confirmed.

The material being performed is transferred only as a stationary object for study. It doesn't run, install, or get permissions because it was in a trusted conversation.

### 11.4. What is not transferred

Not transferred automatically:

- keys and entry tokens;
- valid permits;
- running processes;
- planned launches;
- internal current state of Hermes;
- signs of “completed” without Deep Create’s own evidence;
- hidden official rights;
- settings that can expand data transfer.

### 11.5. Origin of transferred information

Each accepted object receives:

- Hermes source designation;
- session or file designation;
- reading time;
- time of the initial event, if known;
- secure fingerprint of the original bytes;
- version of the reading method;
- the result of checking the secrets;
- a mark on fullness or circumcision;
- a link to an unchangeable input copy;
- deciding what purposes it can be used for.

If the original object later changes, the previous proof is not rewritten. A new version is created and the discrepancy is noted.

### 11.6. Controversies

The owner's new message in Deep Create takes precedence over the old message carried over from Hermes, but the contradiction cannot be silently hidden. It is recorded and shown when a goal is formed.

The model's decision in Hermes cannot override the owner's direct decision in either Hermes or Deep Create.

## 12. Working directories and files

### 12.1. The user is not required to create “one project - one folder” in advance

The main work is a semantic unit. It can read multiple authorized sources and have multiple workspaces. At the same time, each file permission remains accurate.

The owner can write: “Work in this folder” or change the working folder later. The system turns this into a separate scope change and shows the consequences for the existing branches.

### 12.2. Four different types of path

Do not mix:

1. persistent data Deep Create;
2. source code of the application itself;
3. custom source materials;
4. a working copy of a specific branch.

The performer usually changes only the fourth area. The transfer of the result to the user source materials is carried out by a separate accepted change.

### 12.3. Verifying a Real Windows Object

Before each read and modification, not only the path string is checked, but also the actual open object. The following are taken into account separately:

- transitions through links and connection points;
- short names;
- register difference;
- network paths;
- hard links;
- transition between disks;
- change of purpose between check and write;
- files occupied by another process;
- change of owner or access rights.

The unknown case is closed by a prohibition rather than by a model assumption.

### 12.4. Changing a file

Before changing, the system creates a unit of work, determines the purpose of the file, its connections and possible consequences. Then:

1. retains the original protected fingerprint;
2. works in a separate copy;
3. checks the result;
4. verifies that the source file has not been modified by anyone else during this time;
5. shows or automatically accepts the change according to the current rules;
6. replaces the state entirely or does not change it;
7. records the total and the possibility of a return.

Random names are acceptable for temporary internal objects, but are not a primary security feature. Renaming persistent entities should not destroy their machine designations and relationships.

## 13. Stopping and human error

### 13.1. Two types of stop

**Immediate stop** immediately revokes rights, protects all ongoing attempts from late acceptance of the result, and terminates the work's own processes. It is always available and may leave an unfinished state that will need to be disassembled later.

**Prepared Stop** prohibits starting new actions, only allows each current unit to reach a pre-safe boundary, saves materials for continuation, and explains what is left open.

### 13.2. Stop states

After the command, “The stop has begun” immediately appears. The "Stopped" state is allowed only after confirmation:

- all rights are revoked or reserved;
- new actions do not start;
- child processes are completed or recognized as external and dangerous;
- a late result cannot be accepted;
- the state is recorded stable;
- actions with an unknown outcome are listed.

### 13.3. Fixing an erroneous prepared stop

A prepared stop can be canceled before its final state if a contradiction has not yet arisen after rechecking. After the final stop, normal continuation with full reconciliation is used, rather than “backdated cancellation.”

An immediate stop cannot be allowed to fail. Its consequences are only being restored and verified.

### 13.4. Confirmation screen

Before stopping, the affected main activity, child units, current external activities and possible losses are shown. The user selects the type of stop, writes an explanation and confirms the choice. The position in the column or list is saved after closing the window.

## 14. Recovery after a failure

### 14.1. Normal unexpected restart

Once launched, Deep Create does not continue immediately. First it:

1. checks the integrity of the main database and journal;
2. finds the last fully accepted version of the state;
3. identifies actions that have been started but not completed;
4. checks the external outcome where it could occur;
5. closes late attempts with the old authorization number;
6. checks current files, sources, model connections and target;
7. reassembles working collections;
8. shows the owner the restored state;
9. does not resume previous executive units without a new command from the owner to “continue”;
10. actions with consequences are left unconfirmed until a separate admission.

A separate accepted rule L-05a relates to a missed scheduled launch: after switching on, you can read, reason and check the latest data, but not actions with consequences. It cannot be silently extended to work that was already running at the time of the failure.

### 14.2. Sudden power outage

You can't count on the program to save anything at the last moment. Therefore, security is built on records **before** the action and on the indivisible acceptance of the state.

After switching on, only three fair outcomes of each attempt are possible:

- the action has definitely not started;
- the action is definitely completed and confirmed;
- the outcome is unknown and requires verification.

The “probably completed” state does not give the right to blindly repeat the action.

### 14.3. Restoring a backup

The backup is first opened in an isolated state. All permits, schedules and working attempts transferred from it are considered ineffective.

A separate local audit verifies:

- data integrity;
- number of the current government;
- versions of application parts;
- open external actions;
- current files;
- current goal;
- condition of the bridge to Hermes;
- availability of mandatory evidence.

After this, the owner confirms the removal of restrictions. Successful opening of the window does not mean successful recovery.

### 14.4. Renewable parts

Each long-term stage must have its own sustainable result and transfer point. The continuation reads this result rather than trying to reconstruct the thoughts of the previous model.

Stored separately:

- goal;
- prohibitions;
- open conditions;
- rejected paths;
- negative checks;
- valid rights;
- file changes;
- outcomes of external actions;
- unclosed obligations;
- origin of information.

Another reviewer looks for lost negations, substitution of solutions, and outdated sources before being allowed to continue.

## 15. Backups

### 15.1. What is copied separately

Separate copies needed:

- main database and journal;
- evidence storage;
- user settings;
- a list of connections without the secrets themselves;
- working materials that have not yet been transferred to the user project.

The current government's journal and secrets have a separate protection procedure. They cannot be thoughtlessly rolled back along with the old task.

### 15.2. What should a copy check prove?

It is not enough to successfully create an archive. A trial recovery is periodically performed to a separate location, checking:

- readability;
- integrity;
- restoration of connections;
- absence of open secrets;
- impossibility of reviving the revoked right;
- complete old or complete new data device;
- proper isolation from the operating system.

## 16. Updates and independent development

### 16.1. Whole version

The new version is compiled separately. It contains an exact list of all related parts and their secure fingerprints. Enabling occurs through one operation of selecting an already fully prepared set.

After a failure, either a completely old or completely new version is launched. Mixing parts is prohibited.

### 16.2. What the system can do on its own

The system can:

- collect suggestions for improvements;
- create new ways of working and checking;
- prepare a new version;
- run it on artificial and impersonal tasks;
- compare with the current version;
- determine that sufficient evidence required in advance has been accumulated;
- include an acceptable improvement if it is expressly permitted by the owner's standing rule and all non-changeable checks are passed.

The author of the change is not the only reviewer. The change itself cannot remove the check, expand its own rights, or rewrite the criterion for its success.

### 16.3. What is always updated along with the system device

Any change to a new mode, action, connection, extension, store or bridge must:

1. name the affected boundaries;
2. add the necessary beacons;
3. add positive, prohibitive, bypass, emergency, recovery and verification tests;
4. prove the confidentiality of the data;
5. determine the return method;
6. get a new whole version number.

If the check map does not match the application version, activation is blocked.

### 16.4. Renewal of the bridge to Hermes

The bridge has a separate version. Changing the Hermes format does not force an update to the Deep Create core and does not give the bridge new rights.

The new version of the bridge is first tested against immutable copies. Until the check is completed, the old bridge either continues to read only the known format, or the transfer is temporarily disabled. Current core work of Deep Create continues.

### 16.5. Deep Create does not update Hermes

In the first architecture, the bridge is one-way and does not have the right to change Hermes. Therefore, a Deep Create update is not a Hermes update, and a Deep Create error should not trigger a Hermes recovery.

## 17. Checking the result and completing the work

The main work is not completed according to the model's phrase. For each goal, a list of obligations is built. Every important commitment is associated with:

- acceptance condition;
- result or file;
- proof of origin;
- verification;
- an exact version of the state;
- the accepted result.

After an internal audit, the owner receives a clear report:

- what was required;
- what has been done;
- where is the result;
- what and how was checked;
- what restrictions remain;
- what external consequences occurred;
- what owner solutions were used;
- what remains unknown.

The owner confirms completion or explains the refusal. Failure creates a new version of open commitment rather than erasing the previous outcome.

The transferred "done" check from Hermes never replaces Deep Create's own check.

## 18. Border checks with Hermes

At a minimum, the following tests must be completed before the bridge is switched on.

| Case | Expected result |
|---|---|
| In an old conversation it says “permission to send” | The text is transferred as history and the current resolution does not appear. |
| The attachment contains a hidden command for the model | The command is not executed; the object remains isolated and marked for inspection. |
| Key found in Hermes data | The key does not go to the main log, model query or regular screen. |
| Hermes modified the file after creating the input copy | The previous copy remains unchanged; a new version is created and the discrepancy is shown. |
| The bridge broke in the middle | Incomplete set will not be accepted; Deep Create continues the rest of the work. |
| One set sent by bridge twice | It is recognized as a repeat and does not create two identical entries. |
| Hermes closed or removed | Data already accepted and Deep Create's ongoing work remain available. |
| Old copy of Hermes contains canceled permission | The resolution doesn't come to life. |
| The Hermes path leads through the link beyond the permitted area | Reading is prohibited until the new fine resolution. |
| Hermes format has changed | Only the transfer is blocked; The Deep Create core does not substitute guesswork for the meaning of fields. |
| Owner selected one session | Adjacent sessions and the general profile are not read. |
| The model asks to open the full story “for quality” | The request is treated as a proposal to expand the scope, rather than being executed automatically. |
| The transfer contains the result of the previous model | It is considered a statement with provenance rather than a proven fact. |
| Restored old copy of Deep Create | The bridge, schedules and rights remain stopped until reconciliation. |

## 19. How the previous architectural document changes

The main rules `DEEP_CREATE_BEACON_AND_ASSURANCE_SPEC_20260829.md` are retained, but their scope is being clarified.

### 19.1. What is preserved without changing the meaning

- safety is below all methods of work;
- the model statement is separated from the observed fact;
- action without prior verification is prohibited;
- the state has one acceptance point;
- permits have a period, scope and immediate revocation;
- the unknown outcome of an external action requires reconciliation, not blind repetition;
- completion requires full coverage of obligations;
- restoration does not resurrect old rights;
- the update includes a completely old or completely new version;
- the map of beacons and tests changes along with the architecture.

### 19.2. What is being replaced

| Previous understanding | New understanding |
|---|---|
| The overall protective layer is located under the Hermes modes. | The general protective layer is located inside the independent Deep Create and covers all its ways of working. Hermes is located outside the outer border. |
| Normal conversation means normal Hermes chat. | Normal conversation means Deep Create's own normal chat. Actions from Hermes do not count as Deep Create actions. |
| The general base of Hermes can become the basis of the condition and the screen. | Deep Create has its own main base; The Hermes database is a limited reading source only. |
| The task board and Hermes screen can become the main screen. | Deep Create builds its own graph, list and solution screen from its own database. |
| The order of introduction is structured around the old and new paths within Hermès. | The creation order is built around a single application; Only the one-way bridge is checked separately. |
| Parts of Hermes suppliers and extensions can be reused directly. | Any such part is considered as an external candidate and undergoes a new listing, restriction of rights and testing. |
| Switching off the old Hermes way is a release phase. | Hermes is not shut down or replaced. Only the read bridge is enabled or disabled. |

### 19.3. What is added

- new trust boundary “Hermes → Deep Create input storage”;
- prohibition of a common variable base;
- prohibition of transfer of permissions, schedules, secrets and completion sign;
- separate check of malicious or control text inside the history;
- stability of Deep Create in the complete absence of Hermes;
- separate version and separate bridge shutdown;
- own regular chat and control settings in natural language;
- own updates, backups and restores.

## 20. The order of further creation

### Step 1: Close the boundaries of the standalone application

You need to approve the exact folders for code, persistent data, temporary working copies, and backups. Then identify the owners of each vault and disable shared entry with Hermes.

### Step 2: Create local support layer

First, the main base, main journal, authority journal, evidence and secrets storage are designed. Until they are verified, external actions remain artificial.

### Step 3: Create a common defense path

A complete list of calls to files, networks, programs, providers and extensions is compiled. Unknown paths are prohibited.

### Step 4. Create a regular chat and the basis of the main work

Regular chat receives lightweight action units. Deep work receives purpose, commitments, branches, working selections, verification, and a single state acceptance.

### Step 5. Create a list of models and connections

The exact nature of the connection, secure key storage, health history, restrictions, deferred checks and equal replacement rules are introduced.

### Step 6. Create file work and stop

Checks real Windows paths, separate working copies, merging branches, immediate and prepared stops, late results, and reverting after human error.

### Step 7. Prove recovery

Process crash, hard shutdown, out of space, log corruption, unknown external outcome, and restoring an old copy are run through.

### Step 8. Create a limited bridge to Hermes

Only after the kernel operates independently is a one-way transfer of selected data introduced. The bridge should not be a requirement for running Deep Create.

### Step 9: Create your own control screen

The screen is built from the accepted state: graph of main works, list, permissions, timers, branches, expenses, checks, warnings and clickable details. All sensitive content is hidden by default.

### Step 10. Enable supervised self-development

At first, the system only suggests and tests improvements. Automatic inclusion is allowed only after a complete review of the entire version, independent evaluation, proven return and a pre-issued owner rule.

## 21. Solutions after complete comparison of answers

The last two external borders are closed by the owner 2026-09-05:

1. after an unexpected restart, the previous execution units do not continue until the owner's command; Only recovery and status check are performed automatically;
2. the general limit of the expended resource established by the owner is insurmountable until a new explicit decision of the owner.

Code and data paths, manual reading bridge of the first version, saving the selected source with origin, fixed root of self-development and initial place thresholds are adopted as engineering values ​​of the first version in the document **11_DESIGN_DIRECTION_AND_PLAN.md**. They are not external action permissions and are checked before the build.

The arrangement of sequential numbers, fingerprint protection, indivisible record order, path verification, unknown outcome verification and mapping restoration are also engineering responsibilities and should not be left to the owner.

## 22. An honest summary of this branch

With this decision made, Deep Create can no longer be designed as "Hermes mode". Its correct form is an independent local management system that uses models as executors of reasoning, but itself machine-manages rights, data, actions, evidence, restoration and development.

Hermes remains a useful source of history. It is not the root of trust, does not own the current state, and is not needed to continue work already started.

This document covers the architectural implications of the separation. It does not prove that the application, bridge, or security checks have already been implemented. The summary plan for creation and verification with closed decisions of the owner is located in the document **11_DESIGN_DIRECTION_AND_PLAN.md**.
