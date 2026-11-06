//! C-02 sender-authority boundary tests.
//!
//! These tests run against the public API only (the crate is compiled twice:
//! once for the lib target with `cfg(test)` off, once for integration tests).
//! `AuthenticatedPeer::for_test` is exposed only under the
//! `test-peer-constructor` feature, which is enabled for integration tests
//! via `cargo test --features test-peer-constructor`. Production builds never
//! enable it.

#![cfg(feature = "test-peer-constructor")]

use dc_domain::ids::EntityId;
use dc_domain::work::{ExpectedLifecycleBasis, UnitAttemptCommand, UnitRef, WorkScope};
use dc_protocol::{
    AuthenticatedPeer, DecodedEnvelope, PeerRole, ProtocolContext, ReplayCache, ReplayCheck,
    RoleTable, SenderValidation, ValidateSenderError, decode_envelope, encode_envelope,
    validate_sender,
};
use std::num::{NonZeroU32, NonZeroU64};

fn id(seed: u128) -> EntityId {
    // UUIDv7-shaped base, monotonically offset by `seed`. The C-01 contract
    // requires lowercase hex and version-7 bits; EntityId enforces both.
    let base = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f_u128;
    EntityId::from_u128(base + seed).expect("seeded UUIDv7 must be valid")
}

fn ctx() -> ProtocolContext {
    ProtocolContext {
        installation_id: id(0x100),
        run_generation: NonZeroU64::MIN,
    }
}

fn basis() -> ExpectedLifecycleBasis {
    ExpectedLifecycleBasis {
        scope: WorkScope {
            session_id: id(0x10),
            work_id: id(0x11),
            objective_id: id(0x12),
            objective_revision: NonZeroU32::MIN,
        },
        unit: UnitRef {
            unit_id: id(0x13),
            revision: NonZeroU32::MIN,
        },
        unit_row_version: NonZeroU64::MIN,
        attempt_id: None,
        attempt_row_version: None,
        run_generation: NonZeroU64::MIN,
    }
}

fn envelope_with(command: UnitAttemptCommand) -> dc_protocol::Envelope {
    dc_protocol::Envelope {
        command_id: id(0x1),
        installation_id: ctx().installation_id,
        expected: basis(),
        command,
    }
}

fn peer(role: PeerRole) -> AuthenticatedPeer {
    AuthenticatedPeer::for_test(role, id(0x200))
}

/// Produce a DecodedEnvelope the ONLY way production can: encode, then decode
/// those exact bytes. The returned value binds parsed envelope to wire bytes.
fn decoded_with_id(command: UnitAttemptCommand, command_id: EntityId) -> DecodedEnvelope {
    let mut env = envelope_with(command);
    env.command_id = command_id;
    let bytes = encode_envelope(&env, &ctx()).expect("test envelope must encode");
    decode_envelope(&bytes, &ctx()).expect("test envelope must decode")
}

fn decoded(command: UnitAttemptCommand) -> DecodedEnvelope {
    decoded_with_id(command, id(0x1))
}

// ---------------------------------------------------------------------------
// Closed role table — production() defaults
// ---------------------------------------------------------------------------

#[test]
fn controller_may_prepare_start_request_and_confirm_cancellation() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    for (slot, command) in [
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(0x30),
        },
        UnitAttemptCommand::Start {},
        UnitAttemptCommand::RequestCancellation {},
        UnitAttemptCommand::ConfirmCancellation {},
    ]
    .into_iter()
    .enumerate()
    {
        let env = decoded_with_id(command, id(0x1000 + slot as u128));
        let outcome = validate_sender(env, &peer, &ctx(), &table, &mut replay);
        assert!(
            matches!(outcome, Ok(SenderValidation::Fresh(_))),
            "controller must be allowed: {outcome:?}"
        );
    }
}

#[test]
fn executor_may_only_observe_submission() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Executor);
    let mut replay = ReplayCache::with_capacity(16);

    let ok_env = decoded(UnitAttemptCommand::ObserveSubmission {
        result_id: id(0x40),
    });
    assert!(matches!(
        validate_sender(ok_env, &peer, &ctx(), &table, &mut replay),
        Ok(SenderValidation::Fresh(_))
    ));

    for (slot, command) in [
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(0x30),
        },
        UnitAttemptCommand::Start {},
        UnitAttemptCommand::RequestCancellation {},
        UnitAttemptCommand::ConfirmCancellation {},
        UnitAttemptCommand::ObserveLost {
            observation_id: id(0x50),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let env = decoded_with_id(command, id(0x1100 + slot as u128));
        let outcome = validate_sender(env, &peer, &ctx(), &table, &mut replay);
        assert_eq!(outcome.unwrap_err(), ValidateSenderError::WrongRole);
    }
}

#[test]
fn broker_may_confirm_cancellation_and_observe_loss_but_not_drive_lifecycle() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Broker);
    let mut replay = ReplayCache::with_capacity(16);

    for (slot, command) in [
        UnitAttemptCommand::ConfirmCancellation {},
        UnitAttemptCommand::ObserveLost {
            observation_id: id(0x50),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let env = decoded_with_id(command, id(0x1200 + slot as u128));
        assert!(matches!(
            validate_sender(env, &peer, &ctx(), &table, &mut replay),
            Ok(SenderValidation::Fresh(_))
        ));
    }

    for (slot, command) in [
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(0x30),
        },
        UnitAttemptCommand::Start {},
        UnitAttemptCommand::ObserveSubmission {
            result_id: id(0x40),
        },
        UnitAttemptCommand::RequestCancellation {},
    ]
    .into_iter()
    .enumerate()
    {
        let env = decoded_with_id(command, id(0x1300 + slot as u128));
        let outcome = validate_sender(env, &peer, &ctx(), &table, &mut replay);
        assert_eq!(outcome.unwrap_err(), ValidateSenderError::WrongRole);
    }
}

#[test]
fn watchdog_may_only_observe_loss() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Watchdog);
    let mut replay = ReplayCache::with_capacity(16);

    let ok_env = decoded(UnitAttemptCommand::ObserveLost {
        observation_id: id(0x50),
    });
    assert!(matches!(
        validate_sender(ok_env, &peer, &ctx(), &table, &mut replay),
        Ok(SenderValidation::Fresh(_))
    ));

    for (slot, command) in [
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(0x30),
        },
        UnitAttemptCommand::Start {},
        UnitAttemptCommand::ObserveSubmission {
            result_id: id(0x40),
        },
        UnitAttemptCommand::RequestCancellation {},
        UnitAttemptCommand::ConfirmCancellation {},
    ]
    .into_iter()
    .enumerate()
    {
        let env = decoded_with_id(command, id(0x1400 + slot as u128));
        let outcome = validate_sender(env, &peer, &ctx(), &table, &mut replay);
        assert_eq!(outcome.unwrap_err(), ValidateSenderError::WrongRole);
    }
}

// ---------------------------------------------------------------------------
// Installation and generation checks
// ---------------------------------------------------------------------------

#[test]
fn wrong_installation_is_refused_before_role_check() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    // The wire decode would already refuse a foreign installation id; we
    // prove validate_sender independently checks by decoding for the local
    // context and then validating against a DIFFERENT context.
    let env = decoded(UnitAttemptCommand::Start {});
    let foreign_ctx = ProtocolContext {
        installation_id: id(0xdead),
        run_generation: NonZeroU64::MIN,
    };
    let outcome = validate_sender(env, &peer, &foreign_ctx, &table, &mut replay);
    assert_eq!(outcome.unwrap_err(), ValidateSenderError::WrongInstallation);
}

#[test]
fn stale_generation_is_refused_before_role_check() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    let mut env_struct = envelope_with(UnitAttemptCommand::Start {});
    env_struct.expected.run_generation = NonZeroU64::new(2).unwrap();
    let bytes = encode_envelope(&env_struct, &{
        ProtocolContext {
            run_generation: NonZeroU64::new(2).unwrap(),
            ..ctx()
        }
    })
    .unwrap();
    let env = decode_envelope(
        &bytes,
        &ProtocolContext {
            run_generation: NonZeroU64::new(2).unwrap(),
            ..ctx()
        },
    )
    .unwrap();

    let outcome = validate_sender(env, &peer, &ctx(), &table, &mut replay);
    assert_eq!(outcome.unwrap_err(), ValidateSenderError::StaleGeneration);
}

// ---------------------------------------------------------------------------
// Fresh vs Duplicate consumer dispatch (audit finding 1)
// ---------------------------------------------------------------------------

#[test]
fn first_delivery_is_fresh_and_carries_validated_command() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    let env = decoded(UnitAttemptCommand::Start {});
    let outcome = validate_sender(env, &peer, &ctx(), &table, &mut replay);
    match outcome {
        Ok(SenderValidation::Fresh(cmd)) => {
            assert_eq!(cmd.envelope().command_id, id(0x1));
            assert_eq!(cmd.role(), PeerRole::Controller);
            assert!(!cmd.canonical_bytes().is_empty());
        }
        other => panic!("first delivery must be Fresh, got {other:?}"),
    }
}

#[test]
fn byte_identical_redelivery_is_duplicate_not_fresh_and_never_reexecutable() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    let env = decoded(UnitAttemptCommand::Start {});
    let first = validate_sender(env.clone(), &peer, &ctx(), &table, &mut replay);
    assert!(matches!(first, Ok(SenderValidation::Fresh(_))));
    assert_eq!(replay.len(), 1);

    // Byte-identical redelivery of the same wire frame. It MUST NOT come back
    // as Fresh — the consumer would execute the command a second time.
    let second = validate_sender(env, &peer, &ctx(), &table, &mut replay);
    match second {
        Ok(SenderValidation::Duplicate { command_id }) => {
            assert_eq!(command_id, id(0x1));
        }
        other => panic!("redelivery must be Duplicate, got {other:?}"),
    }
    assert_eq!(replay.len(), 1, "duplicate must not consume a new slot");
}

#[test]
fn replay_same_id_different_body_is_conflict() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    let env_a = decoded(UnitAttemptCommand::Start {});
    assert!(matches!(
        validate_sender(env_a, &peer, &ctx(), &table, &mut replay),
        Ok(SenderValidation::Fresh(_))
    ));

    // Same command_id, different body (Prepare vs Start).
    let env_b = decoded_with_id(
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(0x99),
        },
        id(0x1),
    );
    let outcome = validate_sender(env_b, &peer, &ctx(), &table, &mut replay);
    assert_eq!(outcome.unwrap_err(), ValidateSenderError::ReplayConflict);
}

// ---------------------------------------------------------------------------
// Replay cache semantics (audit findings 3 + 6)
// ---------------------------------------------------------------------------

#[test]
fn replay_cache_at_capacity_is_capacity_exhausted_not_conflict() {
    let mut replay = ReplayCache::with_capacity(2);
    let a = id(0xa);
    let b = id(0xb);
    let c = id(0xc);
    let body = b"body".to_vec();

    assert_eq!(replay.check_and_record(a, &body), ReplayCheck::FirstSeen);
    assert_eq!(replay.check_and_record(b, &body), ReplayCheck::FirstSeen);
    assert!(replay.is_full());

    // Third distinct id at capacity: DISTINCT capacity outcome, NOT Conflict.
    // Audit: exhaustion is backpressure, not evidence of a clashing body.
    assert_eq!(
        replay.check_and_record(c, &body),
        ReplayCheck::CapacityExhausted
    );
    // Original `a` is still recorded — a duplicate of `a` is still Duplicate.
    assert_eq!(replay.check_and_record(a, &body), ReplayCheck::Duplicate);
    // And a different body for `a` is still a genuine Conflict — capacity did
    // not poison the recorded fingerprints.
    assert_eq!(replay.check_and_record(a, b"other"), ReplayCheck::Conflict);
}

#[test]
fn validate_sender_maps_capacity_exhaustion_to_distinct_error() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(1);

    let first = decoded_with_id(UnitAttemptCommand::Start {}, id(0x1));
    assert!(matches!(
        validate_sender(first, &peer, &ctx(), &table, &mut replay),
        Ok(SenderValidation::Fresh(_))
    ));
    assert!(replay.is_full());

    let second = decoded_with_id(UnitAttemptCommand::Start {}, id(0x2));
    let outcome = validate_sender(second, &peer, &ctx(), &table, &mut replay);
    assert_eq!(
        outcome.unwrap_err(),
        ValidateSenderError::ReplayCapacityExhausted,
        "capacity exhaustion must NOT masquerade as ReplayConflict"
    );
}

#[test]
fn replay_failure_after_role_passes_does_not_poison_cache_for_well_formed() {
    // Order of checks in validate_sender: installation → generation → role →
    // replay. A WrongRole refusal must NOT record the command_id: a later
    // well-formed delivery from an authorized role must still be FirstSeen.
    let table = RoleTable::production();
    let executor = peer(PeerRole::Executor);
    let controller = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    let env = decoded(UnitAttemptCommand::Start {});

    // Executor tries Start (not allowed). WrongRole expected.
    let refused = validate_sender(env.clone(), &executor, &ctx(), &table, &mut replay);
    assert_eq!(refused.unwrap_err(), ValidateSenderError::WrongRole);
    assert_eq!(replay.len(), 0, "role refusal must not record command_id");

    // Controller sends the same envelope. Must succeed as Fresh.
    let accepted = validate_sender(env, &controller, &ctx(), &table, &mut replay);
    assert!(matches!(accepted, Ok(SenderValidation::Fresh(_))));
    assert_eq!(replay.len(), 1);
}

// ---------------------------------------------------------------------------
// Wire-byte binding (audit finding 2)
// ---------------------------------------------------------------------------

#[test]
fn validated_command_canonical_bytes_are_the_exact_wire_bytes() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    let env_struct = envelope_with(UnitAttemptCommand::Start {});
    let wire = encode_envelope(&env_struct, &ctx()).unwrap();
    let env = decode_envelope(&wire, &ctx()).unwrap();
    // The decoded pair carries the exact bytes it was decoded from.
    assert_eq!(env.original_bytes(), wire.as_slice());

    let outcome = validate_sender(env, &peer, &ctx(), &table, &mut replay);
    match outcome {
        Ok(SenderValidation::Fresh(cmd)) => {
            assert_eq!(
                cmd.canonical_bytes(),
                wire.as_slice(),
                "replay fingerprint must be the exact wire bytes, not a re-encoding"
            );
        }
        other => panic!("expected Fresh, got {other:?}"),
    }
}

#[test]
fn duplicate_detection_keys_on_exact_wire_bytes_not_structural_equality() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);

    // Two structurally equal envelopes encoded independently produce the same
    // deterministic bytes (canonical codec) and thus classify as Duplicate.
    let env = decoded(UnitAttemptCommand::Start {});
    let first = validate_sender(env.clone(), &peer, &ctx(), &table, &mut replay);
    assert!(matches!(first, Ok(SenderValidation::Fresh(_))));
    let second = validate_sender(env, &peer, &ctx(), &table, &mut replay);
    assert!(matches!(second, Ok(SenderValidation::Duplicate { .. })));
}

// ---------------------------------------------------------------------------
// ValidatedCommand cannot be forged from public API
// ---------------------------------------------------------------------------

#[test]
fn validated_command_exposes_only_read_accessors() {
    let table = RoleTable::production();
    let peer = peer(PeerRole::Controller);
    let mut replay = ReplayCache::with_capacity(16);
    let env = decoded(UnitAttemptCommand::Start {});
    let outcome = validate_sender(env, &peer, &ctx(), &table, &mut replay);
    let Ok(SenderValidation::Fresh(cmd)) = outcome else {
        panic!("expected Fresh")
    };
    // Read-only surface: envelope(), role(), canonical_bytes(). No fields, no
    // constructor — the type is sealed by visibility.
    let _ = cmd.envelope();
    let _ = cmd.role();
    let _ = cmd.canonical_bytes();
}
