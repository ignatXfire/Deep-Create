# Deep Create: technical constitution of the first version

**Date:** 2026-09-06  
**Updated after verification:** 2026-09-08  
**State:** Required technical background before writing working code  
**Purpose:** Turn the owner's decisions into boundaries that cannot be accidentally bypassed by a separate function, screen, model or extension.

---

## 1. What does this document support?

This document does not rehash the philosophy of Deep Create. It sets the rules by which code will be allowed to be created.

The first version is built as a standalone local application for Windows. Hermes is not its internal part, source of condition, launcher or condition of recovery. Later, Hermes can be only one of the explicitly connected readable sources.

A technical project is considered suitable for implementation only when for each requirement the following are known:

1. source of the owner's decision;
2. a machine object that stores the solution;
3. the only part of the program that has the right to change it;
4. input and output of each open function;
5. unacceptable conditions;
6. mandatory essential transition event or reference to caller event for pure computation;
7. automatic check;
8. action in case of failure;
9. Presentation that is understandable to the owner.

If the behavior contract is unknown, the working code for this feature is not accepted. A narrow prototype to test a specific engineering assumption is permitted in the test area; it does not receive real data and is not advertised as a ready-made opportunity.

---

## 2. Hierarchy of solutions

In case of discrepancies between documents, the following procedure applies:

1. exact last answer of the owner in **07_OWNER_DECISIONS.md**;
2. unchangeable protective boundaries and deviation conditions in documents **10** and **11**;
3. technical design documents **13–21, 23–26**;
4. source code;
5. Screen behavior and model text.

The lower level cannot override the upper one. If the code does not match the document, the code is incorrect. If two documents at the same level diverge, the implementation is blocked until the documentation is explicitly corrected. The designer or model does not silently choose a convenient interpretation.

Historical research materials provide evidence of origin, but do not replace recent decisions made.

Document 22 saves a report of the previous state. Document 23 explains accepted and rejected comments; The exact new contracts for work connections, resource distribution and testing are in 24–26. They refine existing C-01 through C-16 rather than creating concurrent owners of the estate. When developing a corresponding opportunity, its main contract is read along with the named addition; adding a link does not allow the conflicting old call to remain.

---

## 3. Exact boundary of the first version

### Included in the first working version

- independent launch without Hermes;
- several independent main works with limited units and explicitly specified version connections; the first end-to-end slice checks one job, the next one checks their isolation, sharing of results, and conflicts of real common objects;
- purpose, obligations, dependencies and branches of assumptions;
- local status database and immutable evidence archive;
- assembly of a working selection and instructions for the model;
- several connections of models with checking capabilities, availability and limits;
- an isolated executor without direct access to secrets and main repositories;
- independent verification of the result;
- a single permission path for files, programs, networks and external services;
- reading and changing only explicitly permitted local files;
- prepared and immediate stop;
- recovery after process completion and power loss;
- a complete factual journal without the secrets and hidden reasoning of the model;
- graph and peer list reading one state;
- final acceptance of the main work by the owner.

### Not included in the first working version

- automatic payment, publication or message to a real person;
- constant two-way exchange with Hermes;
- execution of arbitrary downloaded code;
- independent change of the protective base;
- distributed work on several computers;
- the promise of protection from the administrator of a completely hijacked Windows;
- automatic removal of canonical evidence;
- the subtle inclusion of a new model, ability or path of action.

These boundaries are not a ban forever. They prevent the first version from pretending to be more mature than it is.

---

## 4. Selected technical basis

### 4.1. Languages and shell

The first version uses:

- **Rust** for subject rules, base, work management, protective intermediary, guard and executors;
- **Tauri** as a thin shell of an independent application;
- **TypeScript and React** for state representation and command suggestions; trusted confirmation of a sensitive decision is processed by the native part;
- **SQLite in forward log mode with full synchronization** for the main local database;
- **Windows named pipes** for local exchange between processes;
- **protect Windows data for the current user** to seal keys;
- **Windows job objects** for limiting and terminating executive process trees.

The exact dependency versions are recorded in `Cargo.lock` and `pnpm-lock.yaml`, the compiler version is recorded in `rust-toolchain.toml`. All these files are included in a single list of application versions; their compatibility is checked by one assembly. Independent updates of individual libraries in a running application are not allowed.

### 4.2. Why was this basis chosen?

Rust was chosen for the carrier layer because it allows you to keep states and errors in explicit types, disallow unaccounted for branches, and isolate small, unavoidable Windows code. One language in all local executable parts reduces the number of transitions between environments and the amount of repetitive binding.

Tauri is chosen only as a shell. Its permissions are not considered the main security basis: the window does not receive direct rights to the file system, command line, network, database, or secrets at all. It refers to the narrow local protocol of the controlling process.

SQLite was chosen because the application runs on a single machine and requires indivisible changes to multiple related records. The main database is not located on a network drive. Accepted transitions are set to forward log mode and full synchronization; a setting that allows the last received recording to be lost when the power is turned off is disabled.

### 4.3. What is not consciously chosen

- separate database server: in the first local version it creates a new process, setup and failure surface without the desired benefit;
- multiple languages ​​for the carrier layer: this makes it difficult to apply rules and handle errors equally;
- direct shell access to Tauri capabilities: the convenience of the window should not become a bypass of the intermediary;
- one huge process: a frozen model or extension should not block stopping and issuing rights;
- network between local parts: it opens an extra listening port; Only local channels are used with process identity verification;
- dynamic loading of extension libraries into trusted processes: the ability works outside the root of trust;
- general catalog `utils`, `helpers`, `common` or class `Manager`: such places hide the real owner of the rule.

---

## 5. Five mandatory executable roles of a secure kernel

This is not a promise of exactly five Windows processes on any given run. There can be several executors, and the late reading bridge and the admission node of the new version are launched separately and only when necessary. Listed here are five roles without which a protected kernel cannot be considered operational; they cannot be combined across the trust boundary.

~~~mermaid
flowchart LR
    UI["Окно Deep Create\nвеб-представление и родной доверенный узел"] -->|"типизированное действие владельца"| C["Управляющий процесс\nединственный писатель главной базы"]
    C -->|"ограниченное поручение"| W["Исполнитель\nотдельный процесс на единицу работы"]
    W -->|"предложение действия"| C
    C -->|"намерение и запрошенное право"| G["Защитный посредник действий\nотдельный процесс"]
    G -->|"разрешённый результат без секрета"| C
    C -->|"снимок состояния"| UI
    S["Независимый сторож\nне планирует и не выполняет действия"] -->|"ограждение и остановка дерева"| W
    S -->|"перекрыть новые действия"| G
    UI -->|"родная аварийная команда"| S
~~~

### 5.1. Application window

Has the right:

- show snapshots of the state received from the control process;
- send normal, strictly typed owner actions;
- show sensitive confirmation through the native trusted node, and not through the content of the web page;
- store only view settings: size, position, scale, scrolling, expanded groups.

Does not have the right:

- read or change the main database;
- open an arbitrary file on behalf of the system;
- contact the model supplier;
- store keys;
- independently decide that the work is completed;
- change the state until the control process is confirmed.

The web view is considered untrusted. It can ask to show confirmation, but cannot generate an "owner confirmed" event. The native node builds the question from the canonical action, shows the real object, recipient, data, limit and deadline, and then signs the one-time response with the expected version. Changing any field shown makes the response obsolete.

### 5.2. Control process

This is the only writer of the main database. He:

- accepts the owner’s trusted events;
- creates a goal, obligations, graph and state versions;
- collects working collections;
- selects a valid connection;
- issues limited instructions to performers;
- accepts proposals for results;
- appoints an independent audit;
- requests rights from the protective process;
- accepts or rejects transitions in one transaction;
- creates pictures for the window.

It does not execute generated commands and does not reveal secrets to models.

### 5.3. Performer

Each executable unit receives a separate process or a separate process tree. The performer sees only:

- order;
- working selection;
- time domain;
- transferred funds;
- lower time, request and volume limits;
- opaque designations of allowed connections.

He does not see the main base, the full journal, other works, the original secrets and the broad right of the parent. His response is always a suggestion and not a direct change in the general state.

### 5.4. Protective Action Mediator

This process is outside the executor tree. He:

- stores and checks active limited rights;
- immediately before each consequence, receives from the guard a current snapshot of the epoch and generation of the launch;
- substitutes the secret directly into the permitted circulation;
- opens a verified real file object;
- launches only the registered type of program;
- writes intention before external action;
- does not allow blind repetition of an unknown outcome;
- accepts protection from an independent guard and does not begin new actions after it.

If the security process is not available, the new action is prohibited. The control process can only continue pure reasoning over an already issued selection, if this does not interfere with fair stopping and logging.

### 5.5. Independent watchman

The watchman does not plan the work, does not access models, does not read memory, and does not approve the result. He owns process trees and emergency fencing paths. When stopped immediately it:

- sustainably increases the security of the chosen job, unit or right; the global epoch in a protected anchor increases only when the entire installation is stopped;
- stops allowing new actions by the intermediary in this area, without stopping unrelated work;
- completes the executor trees related to this area through Windows task objects;
- informs the control process of the observed stopping facts;
- saves a minimal emergency record even when the control process is frozen.

The mediator and the gatekeeper are physically separated: a frozen network or file action should not deprive the owner of the ability to block further consequences.

The Watchman is the only writer of the protected anchor of the era and generation of launch. The mediator maintains its own database of rights and intent, but cannot overwrite the anchor. The control process and the mediator only receive a certified snapshot of the anchor through a trusted local channel.

---

## 6. Roots of truth

| Truth | Sole owner of the entry | Who can read | What not to do |
|---|---|---|---|
| goals, obligations, graph, results and decisions | control process | window through the snapshot; performer via selection | write from the window or performer |
| ordered significant events | same control process transaction | inspector and owner through sampling | keep a second divergent journal |
| canonical proofs | storage of encrypted objects via a control process | only through selection | replace the source with a retelling; reveal personal data with an open file name hash |
| Age of Empowerment, Generation Launch and Fencing | watchman, base `authority.anchor` | control process and mediator via verified snapshot | restore from a regular backup |
| current rights and consequences observations | intermediary, base `guard.db` | control process via verified snapshot | change the watchman's anchor; consider local success as evidence of external outcome |
| secrets | Windows Protective Process and Protected Storage | only the connecting part at the time of circulation | transfer to model, window or magazine |
| window position | shell | shell | take for the state of work |

If the two parts assert different values, the more restrictive condition is applied until reconciliation. The discrepancy is never resolved by choosing the more convenient version.

---

## 7. Twelve immutable rules

1. **Text does not create power.** Neither a model, nor a file, nor a website can give out power through text.
2. **One state writer.** Received subject transitions are written only by the control process.
3. **The result of the performer is a proposal.** It does not become truth without verification and acceptance.
4. **Permissions are not inherited.** The child process receives a new, explicitly narrower limit.
5. **The secret is not included in the context of the model.** It is inserted after all checks in the security process.
6. **Intention precedes consequence.** Without a stable record of intention, action does not begin.
7. **The unknown outcome is not repeated.** First, a reconciliation is performed against the stable number.
8. **Review prohibits new and later.** After the fence point, a late response is stored as a fact and taken into account in expense reconciliation, but is not applied as a meaningful result and does not trigger the next action. An external consequence already sent may be irreversible.
9. **Recovery does not mean continue.** After an unexpected restart, old units wait for an explicit "continue" command.
10. **The owner's limit is insurmountable.** The system can only request a new solution.
11. **Beacon is required.** Failure to reliably record will stop state change and action.
12. **The protective foundation does not improve itself.** The model may suggest a change, but its tolerance remains outside the area being changed.

Each rule has a violation check. Where a rule affects persisted state, a failure and recovery check is added; a pure function does not need a separate journal for the sake of formal coverage.

---

## 8. Code Existence Rule

A new open function is only allowed if you have a passport:

| Passport field | Mandatory answer |
|---|---|
| Reason | What accepted requirement is impossible to fulfill without it |
| Owner | Which module is solely responsible for the rule |
| Caller | Which specific part has the right to call the function |
| Login | Which verified type does she take |
| Exit | What type of result or error does |
| Change | What state does it change and in what transaction |
| Ban | What she fundamentally has no right to do |
| Beacon | How is a significant transition proven or in which caller is it recorded; for a pure function - “event does not create” |
| Check | Which automatic script will break if it doesn't work correctly |
| Removal | How to prove that a function is no longer needed |

A function is prohibited if its only reason is “may be useful”, “for flexibility”, “for the future”, or hiding several unrelated actions under a convenient name.

---

## 9. Rules against junk code

### 9.1. Structural prohibitions

- no common file of auxiliary functions;
- there is no type with the name only `Manager`, `Service`, `Engine` or `Processor` without one subject obligation;
- there is no open interface without a valid reason: a trust boundary, replaceable external I/O, or several true execution methods;
- there is no state field that is not involved in the rule, view or validation;
- there are no two ways to change one state;
- there is no hidden entry in the constructor, property getter, or formatting function;
- there are no logical string states where a closed enumerated type is possible;
- there is no universal map of arbitrary values ​​in the carrier layer;
- there is no fallback that weakens the check in order to continue;
- there is no suppression of an error without a recorded decision about its fate.

### 9.2. When is it permissible to highlight a general solution?

A general solution stands out when repetitions have one semantic rule and one reason to change. The sheer number of similar lines does not prove this. Two checks of the same right should be combined at once; three similar formatting with different responsibilities can be kept separate. The passport indicates the existing callers and the reason for the allocation.

The general solution preserves subject types. It is impossible to replace three clear functions with one function `execute(kind, data)` with an arbitrary action type string.

### 9.3. Automatic build bans

For Rust:

- build and style check warnings are considered errors;
- unused code is prohibited;
- native insecure Rust is prohibited in all packages except the Windows Small Adapter; this is not a statement about the absence of such code in third-party libraries, which are checked separately;
- each place of unsafe code contains a description of the security condition and separate checks;
- incomplete parsing of an enumerable state does not compile.

For TypeScript:

- strict mode is enabled: implicit `any`, unchecked casting of external data, unused variables and unparsed states are prohibited; normal type inference by the compiler is allowed;
- network and file libraries are not included in window dependencies;
- the work state is not duplicated in a separate window storage;
- an optimistic image is allowed only for a purely visual action, but not for the right, result or state of work.

### 9.4. Checking the need for each change

Each change answers four questions:

1. What requirement does it implement?
2. Why is the existing function not the correct owner of this behavior?
3. What new failure does it create?
4. What test proves the new path and what owner records its significant event?

If there are no answers, the change is not accepted.

---

## 10. Resolved dependencies between parts

~~~mermaid
flowchart TD
    D["Предметные типы и правила"]
    P["Местный протокол"] --> D
    S["Главное хранилище"] --> D
    C["Управление работой"] --> D
    C --> P
    C --> S
    X["Сборка контекста"] --> D
    C --> X
    R["Выбор подключения"] --> D
    C --> R
    V["Проверка результата"] --> D
    C --> V
    W["Исполнитель"] --> P
    G["Защита и сторож"] --> P
    G --> D
    U["Окно"] --> P
    H["Мост Hermes"] --> P
~~~

Prohibited reverse dependencies:

- domain types do not know about SQLite, Tauri, a specific provider and interface;
- the repository does not call the model and does not make product decisions;
- the executor does not depend on the main repository;
- the window does not depend on the internal tables of the database;
- the Hermes bridge does not depend on the Hermes control logic;
- the verifier does not call the function to accept its own result;
- the security broker does not import the ability code.

The dependency loop blocks the architecture from being built even when the language is technically able to bypass it.

---

## 11. Mistakes are part of the contract

All errors belong to one of the classes:

| Class | Reaction |
|---|---|
| invalid input data | reject without repeating; show exact field |
| version conflict | reread the state and reassemble the sentence |
| the right is missing or expired | prohibit an action; if necessary, create a request to the owner |
| total limit reached | stop affected branches; do not bypass another model |
| temporary unavailability | repeat on a limited schedule with the same stable number |
| the outcome of the action is unknown | prohibit repetition and start reconciliation |
| damage or rupture of integrity | go to limited mode |
| not enough space for beacon | prohibit new changes and actions |
| internal rule error | stop the affected path and create an incident |
| a person's request is really necessary | combine the same reasons and notify once |

It is not allowed to return one common `something went wrong` within the system. Owner friendly text is created from the exact machine class and does not replace it.

---

## 12. When a technical basis should be rejected

The design is returned for correction if:

- any opportunity does not have one owner of the state;
- the protective solution exists only in the text of the model instructions;
- stopping depends on the model’s response;
- the window or the performer can reveal the secret;
- event and state change are written separately;
- one logical action receives a new stable number to bypass reconciliation; own numbers of individual transport attempts are allowed and associated with the original action;
- the restored copy returns the old right;
- the technical scheme requires Hermes to start;
- added a second path of action bypassing the protective process;
- there is a public function without a caller, check and requirement;
- verification confirms only a successful path;
- documentation describes the state that program types allow to be violated.

---

## 13. Sources of technical properties

Architectural solutions remain ours, but are based on verifiable properties of the foundation:

- SQLite describes indivisible commit and power failure behavior: <https://www.sqlite.org/atomiccommit.html>;
- for the durability of the last commit in the proactive log mode, full synchronization is used: <https://sqlite.org/pragma.html#pragma_synchronous>;
- the capabilities of the Tauri window should be minimal and not overlap unnecessarily: <https://tauri.app/security/capabilities/>;
- Windows Data Protection usually associates the decryption with the same user and computer: <https://learn.microsoft.com/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata>;
- Windows job objects manage the process tree as a whole: <https://learn.microsoft.com/windows/win32/procthread/job-objects>.

These properties must be verified by testing on the target version of Windows. A link to the documentation does not replace testing the assembled application.

---

## 14. Release of this document

After the adoption of this constitution, the remaining documents are required to determine:

- exact tables, types, states and transitions;
- the exact composition of the working selection and the model’s instructions;
- the procedure for selecting connections, repetitions and independent verification;
- form of law, intention and result of action;
- list of beacons, restoration and stop;
- source code tree and passports of open functions;
- matrix “decision → code → check → beacon”;
- the procedure for creating the first end-to-end working chain.

It is dangerous to begin substantive implementation before these documents are completed: otherwise, the first convenient solutions will quietly become architecture.
