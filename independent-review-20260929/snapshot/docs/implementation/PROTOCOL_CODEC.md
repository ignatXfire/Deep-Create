# Strict form of messages between processes

Date: 2026-09-11. Restricted section C-02 from document 19. This is a pure check of bytes and types in memory, not a ready channel between processes and does not jump to the execution of real commands.

## Why do we need a separate border?

The message may be syntactically correct but contain a duplicate field, an incorrect setting, an old run, or a rounded version of the string. An intermediate conversion to a custom dictionary can hide duplicate keys. Therefore, the parsing immediately goes to closed types with a ban on unknown fields.

Added architecture-specific package `crates/dc-protocol`. It uses the existing `WorkScope`, `UnitRef`, `ExpectedLifecycleBasis` and six variants of `UnitAttemptCommand`; a second system of commands or transitions was not created. No new external libraries have been added: only a local package entry has been added to `Cargo.lock`.

## What is transmitted

The message contains the form version, command number, installation number, expected unit/attempt base, and closed command. The basis includes the work area, unit revision, line versions, matched attempt number-attempt version pair, and run generation.

Only existing forms are available: preparing an attempt, starting, observing the presented result, requesting cancellation, confirming cancellation, observing the loss of the process. These are **actions not allowed for each sender**. The resolution depends on the authenticated role and state verified by subsequent layers.

There are no fields for the sender's self-declared identity, credentials, or arbitrary executable text. The extra field `sender` will be rejected rather than accepted as evidence.

## Data path

1. The future receiving channel limits the frame size even when reading. The current `decode_envelope` takes the already received byte array and re-checks its size **before parsing**. This is not a memory limitation of the not-yet-implemented network/interprocess reader.
2. Direct typed parsing validates the form at all levels without losing duplicate fields in the intermediate dictionary.
3. The protocol version should be `1`. The pair of attempts must be consistent. The installation number and start generation must exactly match the local `ProtocolContext`; both the old and future generations are rejected. The command number cannot reuse the number of an object already named in this message; This is a local part of the document 14 ID reuse ban, not a global history check.
4. Only `Envelope` is returned - the command proposal. It does not have an authenticated sender status, an allowed action, or a recorded command.
5. Future C-12 and C-05 should check the current channel, role, action validity and status. C-03 must recognize the repetition of the command number and store the result permanently. None of these points are replaced by the success of the analysis.

`encode_envelope` does the reverse conversion with the same setup, startup, matched try pair, and size limit checks. After encoding and parsing back, all supported values ​​are the same.

## Number precision

| Meaning | Transfer form | Reason |
|---|---|---|
| UUIDv7 | String of 32 lowercase hexadecimal characters | Already existing contract C-01; the number is not rounded |
| Line version and run generation (`u64`) | Non-empty decimal string without leading zero | JavaScript cannot accurately represent the entire range of `u64` with a regular number |
| Target/unit revision (`u32`) | Integer | Full range represented accurately |

For `u64`, JSON numeric values, zero, sign, spaces, fraction, exponential, non-numeric characters, and overflow are rejected. Tested values ​​on both sides of the JavaScript precision boundary and `18446744073709551615`.

Version 1 limits the message to 65,536 bytes, including spaces. This is clearly a chosen form limit of current small teams, not a right, a spending limit, or a reduction in Deep Create's capabilities. Longer content in future posts should be linked to as provided; expanding the form requires a separate contract and checks, rather than a tacit increase in the limit.

## Errors and no leaks

`ProtocolError` distinguishes between oversize, unsupported version, corrupted message, duplicate field, unknown field/command, invalid structure, try, set or run pair mismatch, object number repeated as command number. The original bytes, names of unknown fields, or detailed parse error text are not included: they may contain sensitive data.

The separation of some errors relies on messages from the pinned version `serde`; it's covered in tests. If the unknown error is not recognized after updating the library, the result remains a failure `InvalidStructure` rather than resolution.

## Dependency features and limitations

- `decode_envelope` - bytes and local context → sentence or closed error. No state change.
- `encode_envelope` - sentence and local context → limited bytes or closed error.
- Internal `WireExpected`, `WireEnvelope`, `TextCounter` belong only to the transmission form. C-01 subject types are not repurposed for JSON.
- `check_context` has one install/run check owner and is used in both directions.

The architecture check only allows for exact local dependency on `dc-domain` and form/bug libraries already in use. Path spoofing, online libraries, aliases, or optional dependencies are rejected. The sources of this package also check for the absence of files, network, clock, environment and system calls. Source analysis is not considered proof of Windows process isolation.

## Evidence and next step

`crates/dc-protocol/tests/envelopes.rs` has nine groups of challenges with multiple entry options: all commands, counter accuracy, pair attempts, foreign setup and generations, repeats at each level, unknown fields/commands and safe errors, version/required fields/identity, object number reuse prohibition, damage/tail/exact size limit. Three new boundary checks have been added to the architecture checker. The overall result and prints are in [STATUS.md](STATUS.md).

The base was suggested by Kimi. The core agent corrected the use of a private module, reduced repetitive transformations, added safe bug classification, wrote tests, and logged the exact package dependency. The DeepSeek review confirmed the usefulness of checking for matching IDs by first obtaining a failing test on the old code, then fixing the common testing path. Claims about skipping overflow and garbage after JSON are unproven; existing failure at such inputs is verified. The risk of changing error statements in a future library update is recorded as a re-validation condition, rather than as a current defect.

The next layer should give C-02 an authenticated channel identity and a private role-command table. While there is no C-12, the dummy type `AuthenticatedPeer` or the public "trusted sender" constructor is not added. Long-lasting repetition recognition remains with the owner of the state in C-03, not in the memory of this parsing. The truth of the observation, the admissibility of the transition, the rights, the expense, the delivery, the order of messages and the restoration have not yet been proven by this site.
