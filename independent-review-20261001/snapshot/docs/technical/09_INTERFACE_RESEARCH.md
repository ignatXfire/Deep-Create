# Study of the interface of a separate Deep Create mode

**Date of source verification:** September 4, 2026  
**Purpose:** Define rules for time map, permit center, shutdown, state recovery, availability, and human error protection.  
**Document Boundary:** is a study and set of design requirements. It does not prove that the interface has already been created or tested in a running application.

## 1. How the material was read

Only primary sources were used:

- Apple's official interface design guidelines;
- official Microsoft Fluent design system and Windows documentation;
- Official Material Design 3 guide and Android documentation;
- current WCAG 2.2 standard and W3C official clarifications.

The style of any one system is not suggested to be copied. Rules of behavior, understandability, accessibility, and error protection are extracted from the sources. The look and feel of Deep Create should remain native and consistent with the design of the application in which it will run.

The document distinguishes between three types of statements:

- **standard requirement** - necessary to comply with the specified WCAG level;
- **official recommendation** - advice from the owner of the design system;
- **conclusion for Deep Create** - an architectural consequence from several sources and features of long-term agent work.

## 2. Brief summary

A separate Deep Create mode should be built not as one beautiful graph, but as four consistent representations of one actual state:

1. **Map of work** shows the whole: main works, dependencies, branches, expectations and places where a person is needed.
2. **Work List** gives the same information without having to read the spatial diagram.
3. **Details area** explains the selected node: why it exists, what it received, what it does, what it changed, how the result is confirmed.
4. **Permission Center** separately shows pending, active, expired and revoked permissions.

The map and the list should not have their own divergent states. They are two ways to see one job entry. If the graph says "executing" and the main log says "awaiting resolution", the graph is wrong, not the log.

Basic interface security rule:

> No color, movement, button label, or saved window position constitutes permission or evidence of operating status. The resolution and state are determined only by the machine entry that the interface displays.

## 3. Long-term work map

**Clarification 2026-09-08:** Above the map of the selected work, an overview of all main works from the same confirmed image is entered. Exact composition, page-by-page reading, reasons for waiting and testing - [23, §4–5](./23_EXTERNAL_CRITIQUE_AND_REFINEMENTS_20260908.md); the only construction contract is C-14 of document 19. The review does not compute its own truth, does not turn a verified internal result into final owner acceptance, and does not wake up the model on every redraw.

### 3.1. What each node must show

The map node should have a short visible part:

- a clear name for the result, not the name of the internal performer;
- type of work: study, creation, verification, coordination or correction;
- current state in text and sign;
- a known share of completion or an honest mark “unknown share”;
- time of the last confirmed event;
- number of dependencies, results and open obstacles;
- a noticeable sign if a user decision is required.

When you open a node, the detail pane should show:

- initial purpose and connection with the main work;
- why this unit of work was created;
- what information was transmitted to her;
- what rights were issued to her and until when;
- model connection used or local performer;
- input dependencies and who needs its result;
- current step, previous confirmed step and next valid step;
- created materials and their location;
- changes made;
- checks, negative results and unresolved doubts;
- consumption of time and money;
- available user actions.

This is in line with Apple's general recommendation to give clear feedback and be transparent about a program's actions, but the specific composition of the fields is a takeaway for Deep Create from its traceability model: [design principles Apple](https://developer.apple.com/design/human-interface-guidelines/design-principles).

### 3.2. The final list of states

The interface should not come up with arbitrary words for states. For the first version, the following closed list is sufficient:

1. Not started.
2. Ready to start.
3. Waiting for another job.
4. Waits for user permission.
5. In progress.
6. Stops safely.
7. Suspended.
8. Stopped safely.
9. Forced termination requested.
10. The outcome of an external action is unknown.
11. Reconciliation required.
12. Completed and verified.
13. Completed with spaces.
14. Error, may be repeated.
15. Error, we need a person.
16. Canceled.

The state must always have:

- verbal name;
- sign or form;
- color as an additional, but not the only sign;
- time to enter the state;
- reason;
- allowed transitions to other states.

[WCAG 2.2, use of color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color) prohibits the conveying of significant meaning by color alone. [WCAG 2.2, Contrast of Non-Text Elements](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast) requires a contrast ratio of at least 3:1 for significant controls and portions of graphics.

### 3.3. Actual progress and evaluation

Apple and Fluent recommend showing definite progress when the quantity is actually known, and not presenting an undefined process as measured: [progress indicators Apple](https://developer.apple.com/design/human-interface-guidelines/progress-indicators), [progress bar Fluent 2](https://fluent2.microsoft.design/components/web/react/core/progressbar/usage).

For Deep Create the rules follow:

- percentage is allowed only if there is a measurable denominator: for example, 38 out of 120 files were checked;
- you cannot calculate the percentage from the number of “thoughts” of the model, the number of messages or elapsed time;
- if the volume is unknown, the current step, the time of the last confirmation and a sign of live work are shown;
- the expected time can only be shown as a range with a calculation basis; Fluent specifically warns that time estimates are often not accurate enough;
- the percentage should not go backwards. When changing the plan, you should explicitly write “plan recalculated”, and not silently reduce the band;
- when the indicator stops moving, the text should appear: work is waiting, stopped, lost connection or requires checking;
- completion is displayed only after the provided check, and not after the last response of the model.

### 3.4. The graph cannot be the only way to control

The user's desire to move icons on the map is acceptable, but dragging and dropping cannot be the only way to perform the action. [WCAG 2.2, drag-and-drop ](https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements) requires simple replacement performed by individual clicks.

Required replacements for drag and drop:

- select a node and click “Move”;
- select a new place from the list;
- buttons “above”, “below”, “left”, “right”, if this only changes the location;
- keyboard control;
- a separate list of works with the same actions.

If moving a node changes not only the appearance, but also the execution order or dependency, the exact change must be shown before application: “Check B will now begin after result A.” A simple drag and drop should not silently change the executable plan.

For the hierarchical part, you can rely on [tree sample W3C](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/): expanding and collapsing, predictable keyboard arrows, distinguishing between a selected node and a node with a keyboard pointer. However, a map with arbitrary connections is more complex than a tree, so a list remains a mandatory equivalent representation.

### 3.5. Updates without stealing attention

The change of state, the appearance of the result and the progress of the work must be announced to the auxiliary means without arbitrarily moving the keyboard pointer. This follows directly from [WCAG 2.2, status message](https://www.w3.org/WAI/WCAG22/Understanding/status-messages).

For Deep Create:

- the normal course of work updates the status bar quietly;
- the error of a specific node is displayed near the node and in the general message bar;
- only the threat of loss of data, funds, rights or an unknown outcome of an external action can interrupt the user;
- the new permission appears in the permission center and on the site, but does not automatically open a window on top of the current work unless the decision stops all the main work;
- the user can disable optional pop-up messages.

Fluent also advises using persistent declarations only for errors and warnings to avoid overloading the stream: [Fluent message bar 2](https://fluent2.microsoft.design/components/web/react/core/messagebar/usage). W3C recommends allowing all but emergency interrupts to be deferred or suppressed: [WCAG 2.2, interrupts](https://www.w3.org/WAI/WCAG22/Understanding/interruptions).

## 4. Permission Center

### 4.1. Permission is a separate subject

Permission should not be stored as a vague “trust the system” checkbox. Each entry should answer the questions:

- who asks for the right;
- for what main work and unit of work;
- what exact action is permitted;
- over which object or in which service;
- what data will be read, modified or transmitted;
- to whom the data will go;
- upper limit on the number of actions or amount;
- is it possible to reverse the consequences;
- when the right begins to operate;
- when it stops;
- is replay allowed?
- what will continue to work in case of failure;
- what new request will be required if the conditions change.

Apple and Android advise asking for access only when you really need it, in a clear context, with a specific explanation of the reason, and with normal refusal behavior: [Apple privacy](https://developer.apple.com/design/human-interface-guidelines/privacy), [Android](https://developer.android.com/training/permissions/requesting) permission request.

### 4.2. Types of term

The interface may offer:

- once;
- until the end of the selected unit of work;
- one hour;
- eight hours;
- until the specified moment;
- before manual review.

The last option cannot be called “forever”: the user must see that the right remains valid until revoked, and does not become irrevocable.

Basic time rules:

- the moment of completion is stored as an accurate machine record;
- the exact moment and remaining time are shown on the screen;
- after the computer sleeps or restarts, the remaining time is calculated again, and does not continue from the saved number;
- the end of the term always transforms the right into a prohibition;
- time is not extended silently;
- the extension button creates a new conscious decision;
- an unobtrusive warning appears a reasonable time before the end;
- if the deadline is not an essential protection, the user is given enough time to read the question and extend it with a simple action.

[WCAG 2.2, Time Limit](https://www.w3.org/WAI/WCAG22/Understanding/timing-adjustable) requires the ability to disable, change, or extend most time limits; Exceptions are allowed for significant protective periods. Therefore, the resolution deadline may be tight, but reading the question itself should not turn into a race against time.

### 4.3. Expiration and revocation

The permission center should have four separate groups:

- awaiting a decision;
- are acting now;
- expired or completely used up;
- withdrawn or rejected.

Review means:

1. new protected actions no longer begin;
2. pending retries are cancelled;
3. an activity already in progress receives a safe termination request;
4. if the external action could have happened in time, its state becomes “outcome unknown” until reconciliation;
5. the fact of the recall and its area are recorded in the main journal.

W3C describes permission as a changeable user decision and specifically provides for response to revocation: [permissions specification W3C](https://www.w3.org/TR/permissions/). Android provides one-time access and revocation of unused rights, which confirms the appropriateness of temporary and revocable rights: [permission request Android](https://developer.android.com/training/permissions/requesting).

### 4.4. What happens to the rest of the work?

The user's desire not to stop all the main work because of one question is correct, but requires a precise boundary:

- a unit directly dependent on permission goes into “awaiting permission”;
- its subsequent dependent units do not start;
- independent branches continue within the limits of their own rights;
- the merging of results is not declared complete while the mandatory branch is awaiting decision;
- the user sees what exactly continues to work and what is blocked.

This should not be decided by the color of the connection in the graph; need text reasons on both ends of the dependency.

## 5. Two types of stops

### 5.1. "Stop Safely" - Normal Action

This action must be primary and available on the node and primary job.

After pressing the system:

1. prohibits the creation of new units of work;
2. does not begin new external actions;
3. asks ongoing activities to reach a predetermined safe point;
4. saves the confirmed result and unfinished information;
5. releases borrowed funds;
6. records the reason and the last completed step;
7. shows whether and from where work can be resumed.

The button should be named after the result - “Stop Safely”, and not just “Stop”.

### 5.2. “Stop immediately” - emergency action

This action should be visually separated from the normal stop and not accessible by accidental double-tapping.

After confirmation, the system:

1. immediately revokes the right to begin new actions;
2. sends a cancellation request to local processes and external calls;
3. does not assert that an external action already dispatched has been cancelled;
4. marks questionable actions as having an unknown outcome;
5. runs a reconciliation before any renewal;
6. saves everything that she managed to confirm, without passing off a partial result as complete.

The wording “cease immediately” describes the intention to cease further activity, but does not promise the impossible instantaneous reversal of an external action that has already occurred.

### 5.3. Transition from a safe stop to a forced stop

For each type of performer, the normal time to reach the safety point can be measured. After this time, the interface shows:

- what exactly still works;
- when was the last sign of life;
- what can be lost;
- what could have already gone outside;
- options “continue waiting” and “stop immediately”.

The transition should not happen automatically just because the average period has passed, unless there is a pre-written safety rule.

Apple recommends canceling, and if a move is lost, informing the consequence and offering to continue; Microsoft recommends canceling for long-running jobs: [Apple progress indicators](https://developer.apple.com/design/human-interface-guidelines/progress-indicators), [Windows progress management](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/progress-controls).

### 5.4. Closing a window is not the same as stopping

Closing a detail pane, a tab, or an entire window should not silently change the performance. For long-running background jobs, Microsoft recommends letting them continue unattended and not tying the job's life to a window: [Windows Guide to Long Running Jobs](https://learn.microsoft.com/en-us/windows/win32/uxguide/win-dialog-box#progress-dialogs).

Before closing the application, it is enough to show a clear choice only when the closing mode is not yet set:

- continue working in the background;
- stop safely;
- return to the application.

The selected permanent rule should be visible and editable in the settings.

## 6. Confirmation of risky actions

### 6.1. When a separate window is really needed

Apple, Fluent, Material, and Windows agree on one thing: a break window is needed for an important decision, especially when an action is irreversible, but it cannot be shown for every normal and undoable step: [warnings Apple](https://developer.apple.com/design/human-interface-guidelines/alerts), [Fluent decision windows 2](https://fluent2.microsoft.design/components/web/react/core/dialog/usage), [Material Design decision windows 3](https://m3.material.io/components/dialogs/guidelines), [confirmations Windows](https://learn.microsoft.com/en-us/windows/win32/uxguide/mess-confirm).

Scale for Deep Create:

- **reversible local action:** execute, show the result and cancel;
- **significant but verifiable change:** show preview and apply after one confirmation;
- **irreversible or external action:** separate window with object, consequence and safe failure;
- **large payment, publication, transfer of personal data or account change:** re-verification of identity or a second independent confirmation after reviewing the exact data.

[WCAG 2.2, Error Prevention](https://www.w3.org/WAI/WCAG22/Understanding/error-prevention-legal-financial-data.html) requires at least one of three for critical operations: reversibility, error checking, or the ability to review, validate, and correct data before completion.

### 6.2. Solution Window Contents

One window asks one question. It should contain:

- title in the form of a specific decision;
- brief description of the result;
- exact object of action;
- recipient of data or funds;
- amount and limit, if applicable;
- reversibility and method of correction;
- period of permission;
- what happens in case of failure;
- safe exit button;
- a button for a specific action.

An example of a good headline: “Send report to organization A?”  
An example of a bad headline: “Permission Required.”

Buttons: “Send report” and “Do not send”, not “Yes” and “No”.

The detailed history, full details, and solution origin open side by side in the Details pane, but do not create a second window on top of the first. Fluent explicitly discourages nesting windows and advises limiting the number of actions: [Fluent solution windows 2](https://fluent2.microsoft.design/components/web/react/core/dialog/usage). Microsoft requires at least one secure button: [window managementWindows](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs).

### 6.3. Stale Confirmation Protection

After the question is displayed, the amount, addressee, data composition, file, deadline or performer may change. Therefore, the confirmation must be associated with a secure fingerprint of the conditions shown.

If the significant field has changed:

- the old confirmation button no longer works;
- the window shows what exactly has changed;
- a new solution is required;
- both versions remain in the log without recording the secret value.

This is the conclusion for Deep Create from the principle of conscious confirmation: the user confirms the action he sees, not the name of the card.

### 6.4. Cancellation is preferable to constant questions

For frequent reversible actions, reliable cancellation with a clear name of the result is better than constant questions. This is directly recommended by [Apple for cancel and return](https://developer.apple.com/design/human-interface-guidelines/undo-and-redo) and [Windows for commands](https://learn.microsoft.com/en-us/windows/apps/design/basics/commanding-basics).

Cancel in Deep Create should show:

- what action will be canceled;
- what related changes are included in the cancellation;
- that it is no longer possible to cancel;
- the result after performing the cancellation.

## 7. Saving position and state

### 7.1. You need to save two different layers

**Interface Position:**

- selected main work and unit;
- open groups;
- position and scale of the map;
- scrolling the list and details area;
- selected tab;
- filters;
- area sizes;
- unfinished local user draft.

**Actual operating status:**

- state of work units;
- addictions;
- permits and deadlines;
- confirmed results;
- unknown outcomes;
- change log;
- points of safe continuation.

The first layer is for convenience. The second defines truth. They cannot be saved in one opaque snapshot.

Apple explicitly links the restoration of the selected location, tabs, editor, and windows to the continuity of the user experience: [app state restoration AppKit](https://developer.apple.com/documentation/AppKit/restoring-your-app-s-state-with-appkit). Android also separates short-lived screen state from long-term local storage: [saving screen state Android](https://developer.android.com/topic/libraries/architecture/saving-states).

### 7.2. Recovery procedure

After launch:

1. the current actual state is loaded from the main source;
2. permit timing and unknown external outcomes are checked;
3. only then the position of the interface is restored;
4. links to disappeared nodes are replaced by the nearest existing level;
5. The user is shown the time of the last successful reconciliation;
6. execution does not continue until mandatory reconciliation if the stored information differs from the current one.

You cannot revive the work just because in the saved window the node was green and moving.

### 7.3. Keyboard position

After closing the solution window, the keyboard pointer returns to the element that called it or to the next logical location. This is required by [sample window W3C](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/) and Fluent. The general jump order should remain meaningful: [WCAG 2.2, keyboard jump order](https://www.w3.org/WAI/WCAG22/Understanding/focus-order).

## 8. Movement, color and accessibility

### 8.1. Movement should explain, not decorate

Valid motion applications:

- briefly show the emergence of a new branch;
- show the connection of the selected node with dependencies;
- indicate the transition between states;
- show that the uncertain work still shows signs of life;
- gently lead the gaze to the place that the user has opened.

Unacceptable uses:

- endlessly moving lines throughout the graph;
- error flickering;
- pulsation of each running node;
- movement as the only sign of the state;
- animation that delays access to the button;
- automatic movement of the map without direct user action.

Apple recommends that motion be used in a targeted, brief manner, with the ability to cancel it, and with systemic motion reduction in mind: [motion Apple](https://developer.apple.com/design/human-interface-guidelines/motion), [availability Apple](https://developer.apple.com/design/human-interface-guidelines/accessibility).

### 8.2. Reduced Motion Mode

Deep Create should automatically take into account the system motion reduction setting and have its own setting:

- “as in the system”;
- "decrease";
- “without movement.”

In reduced mode:

- movements are replaced by a short change of transparency or an instant change;
- pulsation, scaling, depth and background movement disappear;
- the state remains fully understandable in text, form and sign;
- graph updates do not transfer the visible area.

[WCAG 2.2, movement after action](https://www.w3.org/TR/WCAG22/#animation-from-interactions) requires the ability to disable non-essential movement caused by an action. [WCAG 2.2, Pause and Hide](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide) requires control of automatically moving or updating content when it is running in parallel with other content.

### 8.3. Color, contrast and size

Mandatory rules:

- one color does not receive two opposite meanings;
- the state always has text and a sign, color only helps;
- regular text has a contrast of at least 4.5:1, large text has a contrast of at least 3:1;
- significant boundaries of control elements and graphic connections have a contrast of at least 3:1;
- Windows high contrast mode is checked;
- color values ​​are taken from customized system tools, and are not hardwired;
- the important target of the click has a size of at least 24 by 24 conventional pixels and a sufficient gap; for the main buttons, more is better;
- the position of the keyboard pointer is visible without relying on a faint shadow.

Sources: [Windows Accessibility](https://learn.microsoft.com/en-us/windows/apps/develop/accessibility), [Windows Accessibility Design](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/designing-inclusive-software), [WCAG 2.2 Minimum Target Size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum)

### 8.4. Custom map elements

Each non-standard node, link, and button must have a machine-definable name, purpose, state, and meaning. This requirement is [WCAG 2.2, name, purpose and meaning](https://www.w3.org/WAI/WCAG22/Understanding/name-role-value). Using a visually beautiful canvas without an accessible meaning tree is unacceptable.

## 9. Preventing human errors

### 9.1. Protection is located before, during and after the action

**Before action:**

- safe initial values;
- dangerous commands are separated from frequent ones;
- specific name of the action;
- preview;
- checking the amount, recipient, path and area;
- inability to apply an outdated resolution.

**During action:**

- one question at a time;
- the safe button is always available;
- the dangerous button is not selected by default;
- there is no requirement to press quickly;
- exact conditions are visible until confirmation;
- pressing again does not create a second action.

**After action:**

- visible result;
- cancellation, if possible;
- clear magazine;
- reconciliation of external results;
- an unknown outcome does not automatically turn into success or failure;
- repetition is possible only after proof that it will not double the effect.

Windows recommends preventing the error first, physically separating dangerous commands, providing cancellation and clear feedback, and leaving acknowledgment for meaningful consequences: [acknowledgementsWindows](https://learn.microsoft.com/en-us/windows/win32/uxguide/mess-confirm).

### 9.2. Protection against accidental repetition

The submit, payment, publish, and apply buttons must have a single intent number. After the first acceptance:

- the button immediately shows “accepted”;
- pressing again does not create a new action;
- after the connection is lost, the interface first finds out the outcome;
- a restart is created only as a new clearly visible solution or as a safe repetition of the same number if the recipient supports it.

This is the architectural output for Deep Create. Locking a button alone is not enough: it only protects the current screen, but not replay after restart.

### 9.3. Question overload protection

If several permissions appear at the same time:

- the graph marks all waiting places;
- the permission center shows the queue;
- no more than one question is brought to the fore;
- similar actions cannot be thoughtlessly combined into one broad right;
- a repeated safe action can receive a limited rule only after a separate user decision;
- a critical question can interrupt the user, a normal one cannot.

## 10. Contradictions and necessary clarifications of the user’s wishes

Below is not a refusal of wishes, but boundaries, without which they will create a false sense of control.

| Wish | What is the contradiction | Safe solution |
|---|---|---|
| Make a beautiful graph for the main screen | The spatial diagram is difficult to read from the keyboard, magnification and voiceover means; connections may overlap | Leave the graph as the main visual view, but give an equivalent list and detail area from the same state |
| Allow icons to move | Drag and drop is not available to some people and may accidentally change the meaning of the plan | Store position and meaning separately; give movement commands without dragging; confirm semantic change with text |
| Use a lot of nice animations | Constant movement is distracting and can cause physical discomfort | The movement is only to explain the transition; take into account systemic reduction of movement; give a complete shutdown |
| Show all additional information in a question at once | A long break window hides the main question and increases the likelihood of error | Leave the decision and consequences in the window; Expand full data in one linked area without nested window |
| Give permission for one hour, eight hours or indefinitely | Long-term wide law increases the consequences of error or control seizure | Name "until manual revocation", hide among additional options, limit to scope, always allow immediate revocation |
| Accurately show the time period by second | The visible counter may diverge from the actual time after sleep or failure | The decision is made by a machine recording of the exact moment; the screen recalculates the balance and never renews the right itself |
| Continue the rest of the work while we wait for permission | A dependent branch may use a result that has not yet been resolved | Continue only proven independent branches; leave dependents on hold; do not declare the overall result complete |
| Stop work immediately | An external action that has already been sent may be irreversible or end after clicking | Immediately prohibit new actions, request cancellation, send unknown outcome for reconciliation; do not promise a rollback |
| Save exact position and entire state | An old screenshot may conflict with current work and rights terms | First load and verify the actual state, then restore only the view position |
| Completely store all details in a log | Secret keys and personal data in the log increase the damage if leaked | Store fact, area, protected fingerprint and result; do not write down or show the secret meanings themselves |
| All pending questions are immediately highlighted and opened | Several interruptions steal attention and provoke automatic acknowledgment | Highlight nodes and queue, but open one question at user's choice; automatically interrupt only when there is a threat of loss |

## 11. Proposed composition of a separate screen

### Top band

- title of the main work;
- general condition in words;
- confirmed move;
- time of last reconciliation;
- expenses and established limit;
- number of pending decisions;
- “Stop safely”;
- menu with emergency action “Stop immediately”.

### Left area

- list of main works and folders;
- search;
- filters;
- switching “map / list”;
- saved views.

### Middle area

- map or list;
- unified states;
- dependency relationships;
- scale and transition to the selected node;
- fixed legend of signs;
- a button to stop automatic image updating without stopping the work itself.

### Right detail area

- purpose;
- input information;
- current and confirmed step;
- rights and term;
- results;
- checks;
- magazine;
- permitted actions.

### Permission Center

- queue of pending decisions;
- valid licenses with countdown and exact end time;
- review button;
- history of expired, used and rejected rights;
- filters by main job, action, recipient and risk level.

## 12. Checks without which the interface cannot be considered ready

### Map and list

- [ ] Any state is read without color or movement.
- [ ] Each map node is in the list with the same state.
- [ ] The percentage appears only when the volume is measurable.
- [ ] A node that has not been updated for a long time does not continue to depict live work.
- [ ] All drag-and-drop actions are replaced by separate clicks.
- [ ] The map is completely controlled by the keyboard.
- [ ] Increasing to 400 percent does not hide the function; for a two-dimensional map, an equivalent list remains.

### Permissions

- [ ] The question shows the action, object, recipient, data, limit and deadline.
- [ ] Expiration always means prohibition.
- [ ] After sleep and restart, the deadline is recalculated based on the exact moment.
- [ ] Revocation immediately prohibits new protected activities.
- [ ] Changing an essential condition makes the old confirmation invalid.
- [ ] Failure does not stop independent branches.
- [ ] Secret values ​​are not included in the visible log.

### Stop

- [ ] “Stop Safely” and “Stop Immediately” cannot be confused.
- [ ] Closing a window does not trigger any kind of stop silently.
- [ ] After forced termination, unknown external outcomes require reconciliation.
- [ ] Resume starts only from a confirmed safe point.
- [ ] The repetition of the external action does not occur until the previous outcome is established.

### Confirmations

- [ ] A separate window asks one question.
- [ ] There is a clear safe button.
- [ ] Danger button is not selected by default.
- [ ] Button names describe specific actions.
- [ ] Reversible frequent action uses cancel instead of permanent question.
- [ ] Large payments, publication or transfer of personal data require enhanced verification.

### Recovery

- [ ] The actual state is loaded before the interface position.
- [ ] The selected node, expansion, scale, scrolling and filters are restored.
- [ ] A missing node does not cause a recovery error.
- [ ] The time of the last successful reconciliation is visible.
- [ ] An expired right does not come to life from the saved snapshot.

### Availability

- [ ] Keyboard and voiceover support.
- [ ] After closing the solution window, the pointer returns to its logical place.
- [ ] Windows high contrast is supported.
- [ ] Regular text has a contrast of 4.5:1, significant borders and graphics have a contrast of 3:1.
- [ ] There is a mode of reduced and completely disabled movement.
- [ ] Automatic image updates can be stopped without stopping the calculation.
- [ ] The main buttons are of sufficient size and distance.

## 13. Final conclusion

For Deep Create, you cannot choose between “beautiful” and “reliable”. A beautiful graph is acceptable precisely as an explanation of the actual state, and not as its source. The permission is permissible precisely as a narrow, expiring and revocable right, and not as a trust in the model in general. A forced stop is permissible precisely as an emergency prohibition of further actions with subsequent reconciliation, and not as a promise to cancel the past.

If these boundaries are accepted, the user's desires for a visual graph, live animations, independent background work, granular permissions, and position saving are not inconsistent with security. They form a single interface in which a person sees the whole, but every significant action remains explainable, verifiable and correctable.

## 14. List of primary sources

### Apple

- [Design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles)
- [Warnings](https://developer.apple.com/design/human-interface-guidelines/alerts)
- [Progress indicators](https://developer.apple.com/design/human-interface-guidelines/progress-indicators)
- [Privacy and Rights Request](https://developer.apple.com/design/human-interface-guidelines/privacy)
- [Availability](https://developer.apple.com/design/human-interface-guidelines/accessibility)
- [Movement](https://developer.apple.com/design/human-interface-guidelines/motion)
- [Cancellation and Refund](https://developer.apple.com/design/human-interface-guidelines/undo-and-redo)
- [Disclosure](https://developer.apple.com/design/human-interface-guidelines/disclosure-controls)
- [Restoring the application state AppKit](https://developer.apple.com/documentation/AppKit/restoring-your-app-s-state-with-appkit)

###Microsoft and Fluent

- [Windows solution Fluent 2](https://fluent2.microsoft.design/components/web/react/core/dialog/usage)
- [Stroke Fluent 2](https://fluent2.microsoft.design/components/web/react/core/progressbar/usage)
- [Fluent Message Strip 2](https://fluent2.microsoft.design/components/web/react/core/messagebar/usage)
- [Windows and pop-up areas Windows](https://learn.microsoft.com/en-us/windows/apps/design/controls/dialogs-and-flyouts/)
- [Window Management](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs)
- [Windows Progress Management](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/progress-controls)
- [Windows Confirmations](https://learn.microsoft.com/en-us/windows/win32/uxguide/mess-confirm)
- [Windows Availability](https://learn.microsoft.com/en-us/windows/apps/develop/accessibility)
- [Designing Accessible Windows Applications](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/designing-inclusive-software)

### Material Design and Android

- [Windows Solution Material Design 3](https://m3.material.io/components/dialogs/guidelines)
- [Stroke indicators Material Design 3](https://m3.material.io/components/progress-indicators/guidelines)
- [Windows solution in official Android documentation](https://developer.android.com/develop/ui/compose/components/dialog)
- [Permission request Android](https://developer.android.com/training/permissions/requesting)
- [Saving screen state Android](https://developer.android.com/topic/libraries/architecture/saving-states)

### W3C

- [WCAG Standard 2.2](https://www.w3.org/TR/WCAG22/)
- [Status messages](https://www.w3.org/WAI/WCAG22/Understanding/status-messages)
- [Color Usage](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color)
- [Contrast of non-text elements](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast)
- [Minimum target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum)
- [Drag](https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements)
- [Time limit](https://www.w3.org/WAI/WCAG22/Understanding/timing-adjustable)
- [Pause, stop and hide movement](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide)
- [Preventing important errors](https://www.w3.org/WAI/WCAG22/Understanding/error-prevention-legal-financial-data.html)
- [Name, purpose and meaning of the element](https://www.w3.org/WAI/WCAG22/Understanding/name-role-value)
- [Keyboard transition order](https://www.w3.org/WAI/WCAG22/Understanding/focus-order)
- [Sample window](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)
- [Wood sample](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/)
- [Permission Specification](https://www.w3.org/TR/permissions/)
