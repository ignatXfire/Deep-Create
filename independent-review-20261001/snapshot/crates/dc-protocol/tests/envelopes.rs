use dc_domain::ids::{EntityId, RowVersion, RunGeneration, UnitRevision};
use dc_domain::work::{ExpectedLifecycleBasis, UnitAttemptCommand, UnitRef, WorkScope};
use dc_protocol::*;
use serde_json::{Value, json};

fn id(n: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | n).unwrap()
}
fn fixture(n: u64) -> (Envelope, ProtocolContext) {
    let context = ProtocolContext {
        installation_id: id(1),
        run_generation: RunGeneration::new(n).unwrap(),
    };
    let envelope = Envelope {
        command_id: id(2),
        installation_id: context.installation_id,
        expected: ExpectedLifecycleBasis {
            scope: WorkScope {
                session_id: id(3),
                work_id: id(4),
                objective_id: id(5),
                objective_revision: UnitRevision::new(u32::MAX).unwrap(),
            },
            unit: UnitRef {
                unit_id: id(6),
                revision: UnitRevision::new(1).unwrap(),
            },
            unit_row_version: RowVersion::new(n).unwrap(),
            attempt_id: Some(id(7)),
            attempt_row_version: Some(RowVersion::new(n).unwrap()),
            run_generation: context.run_generation,
        },
        command: UnitAttemptCommand::RequestCancellation {},
    };
    (envelope, context)
}
fn raw() -> (String, ProtocolContext) {
    let (envelope, context) = fixture(1);
    (
        String::from_utf8(encode_envelope(&envelope, &context).unwrap()).unwrap(),
        context,
    )
}
fn value() -> (Value, ProtocolContext) {
    let (raw, context) = raw();
    (serde_json::from_str(&raw).unwrap(), context)
}
fn decode(
    value: &Value,
    context: &ProtocolContext,
) -> Result<dc_protocol::DecodedEnvelope, ProtocolError> {
    decode_envelope(&serde_json::to_vec(value).unwrap(), context)
}

#[test]
fn round_trip_preserves_all_supported_commands_and_exact_u64_counters() {
    for n in [1, 9_007_199_254_740_991, 9_007_199_254_740_993, u64::MAX] {
        let (mut envelope, context) = fixture(n);
        for command in [
            UnitAttemptCommand::Prepare {
                new_attempt_id: id(8),
            },
            UnitAttemptCommand::Start {},
            UnitAttemptCommand::ObserveSubmission { result_id: id(9) },
            UnitAttemptCommand::RequestCancellation {},
            UnitAttemptCommand::ConfirmCancellation {},
            UnitAttemptCommand::ObserveLost {
                observation_id: id(10),
            },
        ] {
            envelope.command = command;
            let bytes = encode_envelope(&envelope, &context).unwrap();
            let wire: Value = serde_json::from_slice(&bytes).unwrap();
            for field in ["unit_row_version", "attempt_row_version", "run_generation"] {
                assert_eq!(wire["expected"][field], json!(n.to_string()));
            }
            assert_eq!(
                wire["expected"]["scope"]["objective_revision"],
                json!(u32::MAX)
            );
            assert_eq!(
                decode_envelope(&bytes, &context).unwrap().envelope(),
                &envelope
            );
        }
    }
}

#[test]
fn counter_spellings_and_numeric_coercion_are_rejected() {
    for field in ["unit_row_version", "attempt_row_version", "run_generation"] {
        for invalid in [
            json!(0),
            json!(1),
            json!(9007199254740993u64),
            json!(1.0),
            json!(true),
            json!(""),
            json!("0"),
            json!("01"),
            json!("+1"),
            json!("-1"),
            json!(" 1"),
            json!("1 "),
            json!("1e0"),
            json!("1.0"),
            json!("١"),
            json!("18446744073709551616"),
        ] {
            let (mut wire, context) = value();
            wire["expected"][field] = invalid.clone();
            assert!(decode(&wire, &context).is_err(), "{field}: {invalid}");
        }
    }
}

#[test]
fn attempt_pair_is_checked_on_both_decode_and_encode() {
    let (mut envelope, context) = fixture(1);
    envelope.expected.attempt_id = None;
    assert_eq!(
        encode_envelope(&envelope, &context),
        Err(ProtocolError::MismatchedAttemptBasis)
    );
    envelope.expected.attempt_row_version = None;
    let bytes = encode_envelope(&envelope, &context).unwrap();
    assert_eq!(
        decode_envelope(&bytes, &context).unwrap().envelope(),
        &envelope
    );
    for field in ["attempt_id", "attempt_row_version"] {
        let (mut wire, context) = value();
        wire["expected"][field] = Value::Null;
        assert_eq!(
            decode(&wire, &context),
            Err(ProtocolError::MismatchedAttemptBasis)
        );
    }
}

#[test]
fn installation_and_old_or_future_generation_are_checked_in_both_directions() {
    let (envelope, mut context) = fixture(2);
    let bytes = encode_envelope(&envelope, &context).unwrap();
    context.installation_id = id(99);
    assert_eq!(
        decode_envelope(&bytes, &context),
        Err(ProtocolError::InstallationMismatch)
    );
    assert_eq!(
        encode_envelope(&envelope, &context),
        Err(ProtocolError::InstallationMismatch)
    );
    context.installation_id = envelope.installation_id;
    for n in [1, 3] {
        context.run_generation = RunGeneration::new(n).unwrap();
        assert_eq!(
            decode_envelope(&bytes, &context),
            Err(ProtocolError::GenerationMismatch)
        );
        assert_eq!(
            encode_envelope(&envelope, &context),
            Err(ProtocolError::GenerationMismatch)
        );
    }
}

#[test]
fn duplicates_survive_raw_parsing_and_are_rejected_at_every_level() {
    let (wire, context) = raw();
    for (needle, replacement) in [
        (
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
        ),
        (
            "\"unit_row_version\":\"1\"",
            "\"unit_row_version\":\"1\",\"unit_row_version\":\"1\"",
        ),
        (
            "\"objective_revision\":4294967295",
            "\"objective_revision\":4294967295,\"objective_revision\":1",
        ),
        ("\"revision\":1", "\"revision\":1,\"revision\":1"),
        (
            "\"kind\":\"request_cancellation\"",
            "\"kind\":\"request_cancellation\",\"kind\":\"request_cancellation\"",
        ),
    ] {
        assert!(wire.contains(needle));
        assert_eq!(
            decode_envelope(wire.replace(needle, replacement).as_bytes(), &context),
            Err(ProtocolError::DuplicateField),
            "{needle}"
        );
    }
}

#[test]
fn unknown_fields_and_fake_sender_claims_are_not_authority() {
    for pointer in [
        "",
        "/expected",
        "/expected/scope",
        "/expected/unit",
        "/command",
    ] {
        let (mut wire, context) = value();
        wire.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("sensitive_sender_claim_do_not_log".into(), json!("owner"));
        let error = decode(&wire, &context).unwrap_err();
        assert_eq!(error, ProtocolError::UnknownField);
        assert!(!error.to_string().contains("sensitive_sender_claim"));
        assert!(!format!("{error:?}").contains("owner"));
    }
    let (mut wire, context) = value();
    wire["command"]["kind"] = json!("arbitrary_code_execution");
    assert_eq!(decode(&wire, &context), Err(ProtocolError::UnknownCommand));
}

#[test]
fn version_shape_required_fields_and_identity_are_strict() {
    for version in [0, 2, u32::MAX] {
        let (mut wire, context) = value();
        wire["schema_version"] = json!(version);
        assert_eq!(
            decode(&wire, &context),
            Err(ProtocolError::UnsupportedVersion)
        );
    }
    for field in [
        "schema_version",
        "command_id",
        "installation_id",
        "expected",
        "command",
    ] {
        let (mut wire, context) = value();
        wire.as_object_mut().unwrap().remove(field);
        assert!(decode(&wire, &context).is_err(), "{field}");
    }
    for invalid in [
        json!(1),
        json!("00000000000000000000000000000000"),
        json!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
    ] {
        let (mut wire, context) = value();
        wire["command_id"] = invalid;
        assert!(decode(&wire, &context).is_err());
    }
}

#[test]
fn command_number_cannot_reuse_an_entity_already_named_in_the_frame() {
    let (original, context) = fixture(1);
    for reused in [id(1), id(3), id(4), id(5), id(6), id(7), id(8)] {
        let mut envelope = original.clone();
        envelope.command = UnitAttemptCommand::Prepare {
            new_attempt_id: id(8),
        };
        envelope.command_id = reused;
        assert_eq!(
            encode_envelope(&envelope, &context),
            Err(ProtocolError::IdentifierReuse),
            "encode: {reused}"
        );
        let (mut wire, context) = value();
        wire["command_id"] = json!(reused.to_string());
        wire["command"] = json!({"kind":"prepare","new_attempt_id":id(8).to_string()});
        assert_eq!(
            decode(&wire, &context),
            Err(ProtocolError::IdentifierReuse),
            "decode: {reused}"
        );
    }
    for command in [
        UnitAttemptCommand::ObserveSubmission { result_id: id(2) },
        UnitAttemptCommand::ObserveLost {
            observation_id: id(2),
        },
    ] {
        let mut envelope = original.clone();
        envelope.command = command;
        assert_eq!(
            encode_envelope(&envelope, &context),
            Err(ProtocolError::IdentifierReuse)
        );
        let (mut wire, context) = value();
        wire["command"] = serde_json::to_value(&envelope.command).unwrap();
        assert_eq!(decode(&wire, &context), Err(ProtocolError::IdentifierReuse));
    }
}

#[test]
fn malformed_trailing_and_oversized_frames_never_return_a_partial_candidate() {
    let (wire, context) = raw();
    assert_eq!(
        decode_envelope(format!("{wire} extra").as_bytes(), &context),
        Err(ProtocolError::Malformed)
    );
    assert_eq!(
        decode_envelope(&wire.as_bytes()[..wire.len() - 1], &context),
        Err(ProtocolError::Malformed)
    );
    for invalid in [
        Vec::new(),
        b"null".to_vec(),
        vec![0xff],
        wire.as_bytes()[..wire.len() - 1].to_vec(),
        format!("{wire} {{}}").into_bytes(),
        format!("{wire}secret").into_bytes(),
    ] {
        assert!(decode_envelope(&invalid, &context).is_err());
    }
    let mut boundary = wire.into_bytes();
    boundary.resize(MAX_ENVELOPE_BYTES, b' ');
    assert!(decode_envelope(&boundary, &context).is_ok());
    boundary.push(b' ');
    assert_eq!(
        decode_envelope(&boundary, &context),
        Err(ProtocolError::Oversize)
    );
    assert_eq!(
        decode_envelope(&vec![0xff; MAX_ENVELOPE_BYTES + 1], &context),
        Err(ProtocolError::Oversize)
    );
}
