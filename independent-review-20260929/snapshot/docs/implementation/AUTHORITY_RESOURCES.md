#C-01: pure powers and resource calculations

Review date: 2026-09-10. Implemented in `crates/dc-domain/src/permission.rs`, `resource.rs`; artificial checks - `crates/dc-domain/tests/authority_resources.rs`.

These are working pure functions and types, and not a ready-made Deep Create security system. They do not read the network, files, clock, environment or database, do not call Windows, do not contain `unsafe`, do not create rights from the model response and do not record consumption. Original agreements: 19 C-01, 17 §§4/12/13, 14 §§4.17/5/7.5, 25 §3.2.

## Permissions: Inputs and APIs

`CapabilityLease` - native value for comparing trusted data. It does not have `Deserialize`, an issuing operation, or an executable token. Creating a Rust framework is also not the owner's decision. `owner_event_id`, the conditions shown fingerprint and the snapshot link are not authenticated here: this is the responsibility of the future trusted path C-09/C-10.

`LeaseScope` contains the exact work, optional narrowing to one, closed `ActionKind`, `CanonicalResource`, exact `CanonicalRecipient`, set `DataClass`, explicit number/byte restrictions, and resource dimension restrictions map. `None` recipient means no recipient, not any recipient. Absence of one means all units of that work only.

`DataClass` uses the only canonical sensitivity set from 15 §4.2: `Public`, `Local`, `Personal`, `Restricted`, `Secret`. The set of allowed classes is compared as an explicit set, rather than as automatically resolved "lesser" enum elements. Old `OwnerPrivate`, `Confidential` and `SecretReference` are rejected, aliases are not left. Presenting a link to a secret is not a level of sensitivity; a separate fictitious handle/representation API without a caller has not been added to this slice.

The existing scope check does not allow `Secret` among the data `CallModelConnection`: the failure `SecretDisclosureDenied` applies to both the lease check and the scope comparison. The service substitution of credential inside the future C-09 is not the transfer of Secret to the model prompt. `ContextProjection` is not yet here, so this ban does not prove the completeness of its filtering; permitted local work with a marked Secret also does not become the right to reveal its models.

Instead of arbitrary path, URL and wildcard strings, typed registered `EntityId` and exact editions are used. This is a conservative comparison of authenticated identities: a subset of the file directory is not guessed from the prefix. Creation of a new file/canonicalization of a missing name is not implemented by this slice.

| API | Result and closed borders |
|---|---|
| `is_scope_subset(parent, requested)` | Compares work/unit, activity type, object, recipient, classes and limits. The undescribed relation “already” is not inferred between different action kinds. Omission of a cost dimension is rejected. |
| `is_lease_subset(parent, requested)` | Additionally checks installation identity, epoch and generation, separate ID of child, parent, transfer depth, both terms and preservation of all support fences of parent. Does not produce a descendant. |
| `verify_lease(lease, expected_version, authority, clock)` | Checks for activity, exact version, installation, era/generation equality, current time and each applicable fence. Returns only `Result<(), PermissionError>`. |
| `validate_lease_transition(from, to)` | Checks for a valid state edge. No final state is returned in `Active`; the same state is acceptable as no change. `Requested → Active` is not owner authentication. |
| `next_fence_version(version)` | Verified increase in non-zero version; overflow - failure, not zero crossing. Does not write an anchor and does not provide a review. |

Time is transmitted explicitly `ClockObservation`: UTC in milliseconds, monotonic milliseconds, `RunGeneration`, observation proxy. `LeaseExpiry::Never` - explicit indefiniteness. The deadline includes UTC and the monotonic deadline of the same launch; meeting the deadline already means expiration. `RolledBack` and `Uncertain` close access. Independent detection of clock rollback and obtaining readings is a future external adapter, not this module.

`AuthoritySnapshot` contains precision fences `RootWork`, `Unit`, `Lease`. All supports are checked, including supports for other works; a descendant cannot remove them. Closing one support revokes a dependent tolerance, but changing an unrelated fence does not cause a global false rejection. The global epoch is not replaced by `max()` of two numbers.

Testing outside of held `AdmissionFence` does not prove that there is no revocation race. Future C-09/C-10s must repeat the check at a valid D/C boundary and hold gate until the OS/COMMIT operation is issued. There is no gate itself, a stable point F, snapshot authentication or a full check of the current chain of parental rights. Enumerations do not prove ACL, AppContainer, handle identity, file tampering protection, or fully trusted Windows process isolation.

## Resources: inputs and APIs

`ExplicitLimit<T>` only has `Finite(T)` and `Unlimited`. `UpperBound` distinguishes between `Known(u64)` and `Unknown`; the unknown is not zero. Money is defined by whole minimum units of registered currency; time, requests, tokens, location, slots and service are different `ResourceKind` and do not translate into each other.

`ResourceLimit` captures ID, area, window ID, dimension, explicit limit, half-open time interval and state. `ReservationRequest` captures `(logical_operation_id, limit_id, window_id)`, separate armor/attempt/intent IDs and top flow. `LedgerPosition` has private fields and is created by calculation, not JSON deserialization; getters of original request, status, version, exceed and current contribution are available.

`ResourceObservation` now contains `observation_id`, `reservation_id`, exact `ledger_key: LedgerKey`, `resource_kind: ResourceKind`, and closed `ObservationKind`. The currency is included in `MoneyMinorUnits { currency }`. A general reservation ID does not allow you to apply tokens as money, transfer the amount to another currency, another limit, window or logical operation. All bindings are checked **before** the ID repeat path and before the state changes; a mismatch returns `ObservationMismatch` without changing the position. Within a position, full observation is stored, and not just the amount/type of outcome.

| API | Duty |
|---|---|
| `evaluate_reservation(limit, current, request, now_utc_millis)` | Checks the exact dimension and window, uniqueness of keys, repeat request, limit activity, saved excess and the sum of occupied deposits with the new U. Returns the estimated position and `total_after`. |
| `evaluate_reservations(applicable_limits, current, requests, now_utc_millis)` | Builds a complete plan for the provided set of applicable ancestors: exactly one request for each limit, one operation/attempt/reservation/intention, the same physical U of the same dimension for the ancestors. If there is an error, the partial plan is not returned. |
| `apply_observation(position, observation)` | First it checks the exact reservation/key/dimension, then returns the replacement of one item while maintaining the full observation by its ID. Exact repetition does not create a second debit; changed payload with the same correct binding - `ReplayConflict`, someone else's binding - `ObservationMismatch`. |
| `upper_service_cost(parallel_slots, enforced_wall_time_millis)` | Calculates the tested integer product. This is not proof of the actual application of the wall-time restriction. |

Current contribution of one position:

- `Reserved`: U.
- `Unknown`: C + R; without partial counting C = 0, R = U.
- `Settled`: C.
- `Released`: 0.

The partial account is an accumulated C fact, not an incremental write-off. Decreasing R requires a trusted proof of the remaining upper bound. The new accumulated C cannot reduce the already confirmed consumption; returns or downward adjustments are not masked by a change in an old fact.

`OutcomeUnknown` maintains the top estimate. Timeout, recall, restart and process loss do not have a "free money" option. `NotDispatchedConfirmed` allows the release of a reserved position or an unknown position with C = 0 with a new trusted proof of non-sending. An unknown outcome after the loss of a message is not a definitive prohibition on establishing the fact: a verified intermediary reconciliation can prove the absence of D. For C > 0, such a statement conflicts with an already confirmed expense and is rejected. `ProcessTerminationConfirmed` only exempts `ConcurrentSlots`, not the non-refundable expense.

The new independent observation of the end of the process after `Released` is valid with the new `observation_id`: it is saved, the row version is promoted, but the contribution remains zero. Repeating the previous ID does not change either the contribution or the version. Here, the ID and origin of the observation are assumed to be trusted by C-03/C-09; the model should not create new observations to inflate the story.

A late score above U is retained in full and sets `bound_breached`, even if the total limit has not yet been reached. The next expense within this limit is blocked. An unexpected positive score after `Released` is also saved and marks a violation. If C + R does not fit into `u64`, both original integer facts are retained, the arithmetic total is not available, and further calculation is closed. The count is not cut to U or `u64::MAX`.

Repeating an existing reservation returns `already_exists = true` without a second deposit, even when the subsequent new admission has already been closed. This is a read of the previous result, not permission to resend the operation. A new potentially paid attempt requires a separate position while the old one is retained unknown.

The full set of applicable limits removes C-03 from the trusted domain graph. This pure API cannot detect an intentionally unpassed ancestor and is not an atomic transaction. Two calculations based on one old snapshot are not safe for parallel recording. The future C-03 must, in one transaction, check for fresh versions and record the entire plan, including common ancestors; the middleman does not become a second ledger writer.

If one outcome applies to multiple applicable items, the C-03 must explicitly prepare properly linked observations for each. Copying one `ResourceObservation` in a position with a general reservation is prohibited. The single position function is not a global store of observation IDs and does not authenticate the origin of an account; unified deduplication, holistic distribution, and atomic write remain C-03/C-09. Typed dimensions do not automatically convert units or check the truth of an external count.

## Checks performed

Locally performed on artificial identities, time and expense:

```powershell
. .\tools\Enter-BuildEnvironment.ps1
cargo test -p dc-domain --test authority_resources --locked --offline
cargo clippy -p dc-domain --test authority_resources --locked --offline -- -D warnings
cargo test -p dc-domain --doc --locked --offline
```

Actual result of the latest edition: 29 PASS integration tests; target clippy with `-D warnings` PASS; 5 package `compile_fail` doctests PASS, including own test `CapabilityLease`. All inputs are artificial, fixture IDs are in fixed form UUIDv7, system time/external connections are not used. The previous common pass confirmed 23 original tests, including `Unknown → Released` for proven absence of D. The native doctest confirms that `serde_json::from_str::<CapabilityLease>` does not compile, not that the existing IPC boundary already authenticates the owner.

Points 2 and 5 of the external criticism are first tested by regressions on the previous code. Both regressions did fall: the observation of 73000 output units was taken into the money position with U = 500 as `Settled(73000)`; the string `Local` was rejected by the old enum. After the fix, the same scenarios go away. These are confirmed local defects and fixes, and do not accept the entire external review as instructions.

Verified:

- small exhaustive ranges of limits and precise numerical boundaries;
- prohibition of expansion of work/unit, recipient, type of action, object/edition, classes, term and resources;
- depth of descendants, preservation of several supports, duplicate fences;
- withdrawal of A while maintaining unbound B; changing the version of even an open support fence;
- time recheck, UTC/monotonic bound, untrusted clock, generation and version;
- final states without animation, unknown action kind and absence of an explicit limit;
- transitions U → C + R → C, repeat observation without a second write-off, content conflict;
- common parental reservation of two branches and prohibition of underestimation of U in the ancestor;
- unknown previous attempt, late overrun, arithmetic overflow;
- the difference between a confirmed slot release and the release of money/quota.
- precise linking of each type of observation, including on the replay path: reservation, logical operation, limit, window, resource kind;
- prohibition of mixing output/input units, money and currencies with common armor; explicit application to another position requires its own key/kind;
- 1024 pairs of sets of five sensitivity classes, canonical wire names and refusal of old aliases/representations;
- `Secret` does not pass the existing model scope, even if it is incorrectly specified in the parent lease.

Additional scripts after the fix also went through a general launch: `reserved → unknown(0,U) → trusted NotDispatchedConfirmed → released`; conflict `NotDispatchedConfirmed` with C > 0; a new independent observation of the end of the process does not re-release the slot.

Not performed by this slice: real processes or power outage; C-03 transactions and durable deduplication; C-09 issue/dispatch/secrets; C-10 AdmissionFence and protected anchor; Windows isolation; real supplier and proven external one-time use; fair scheduler, queue, anti-starvation or throughput measurement. The presence of `upper_service_cost` does not mean the implementation of `FairSchedulingV1`.
