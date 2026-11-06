use dc_domain::ids::{EntityId, RunGeneration};
use dc_protocol::{
    CommandOutcome, DenialReason, OutcomeEnvelope, ProtocolContext, ProtocolError, decode_outcome,
    encode_outcome,
};
use serde_json::{Value, json};

fn id(n: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | n).unwrap()
}

fn fixture(outcome: CommandOutcome, generation: u64) -> (OutcomeEnvelope, ProtocolContext) {
    let context = ProtocolContext {
        installation_id: id(1),
        run_generation: RunGeneration::new(generation).unwrap(),
    };
    let envelope = OutcomeEnvelope {
        command_id: id(2),
        installation_id: context.installation_id,
        run_generation: context.run_generation,
        outcome,
    };
    (envelope, context)
}

fn raw(outcome: CommandOutcome) -> (Value, ProtocolContext) {
    let (envelope, context) = fixture(outcome, 1);
    let bytes = encode_outcome(&envelope, &context).unwrap();
    (serde_json::from_slice(&bytes).unwrap(), context)
}

fn decode(value: &Value, context: &ProtocolContext) -> Result<OutcomeEnvelope, ProtocolError> {
    decode_outcome(&serde_json::to_vec(value).unwrap(), context)
}

fn all_outcomes() -> Vec<CommandOutcome> {
    let mut outcomes = vec![CommandOutcome::Accepted];
    outcomes.extend(
        [
            DenialReason::WrongRole,
            DenialReason::WrongInstallation,
            DenialReason::StaleGeneration,
            DenialReason::ReplayConflict,
        ]
        .into_iter()
        .map(CommandOutcome::Denied),
    );
    outcomes.push(CommandOutcome::UnknownOutcome { intent_id: id(9) });
    outcomes
}

#[test]
fn round_trip_preserves_every_outcome_and_exact_schema_shape() {
    for outcome in all_outcomes() {
        let (envelope, context) = fixture(outcome, 1);
        let bytes = encode_outcome(&envelope, &context).unwrap();
        let wire: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(wire["schema_version"], json!(1));
        assert_eq!(wire["command_id"], json!(id(2).to_string()));
        assert_eq!(wire["installation_id"], json!(id(1).to_string()));
        assert_eq!(wire["run_generation"], json!("1"));
        assert_eq!(decode_outcome(&bytes, &context).unwrap(), envelope);
    }
}

#[test]
fn denial_and_unknown_outcome_are_distinct_and_distinct_from_acceptance() {
    let (accepted, context) = raw(CommandOutcome::Accepted);
    let (denied, _) = raw(CommandOutcome::Denied(DenialReason::ReplayConflict));
    let (unknown, _) = raw(CommandOutcome::UnknownOutcome { intent_id: id(9) });

    assert_eq!(accepted["outcome"]["kind"], json!("accepted"));
    assert_eq!(denied["outcome"]["kind"], json!("denied"));
    assert_eq!(unknown["outcome"]["kind"], json!("unknown_outcome"));

    assert_ne!(accepted["outcome"], denied["outcome"]);
    assert_ne!(denied["outcome"], unknown["outcome"]);
    assert_ne!(unknown["outcome"], accepted["outcome"]);

    // A denial is a definite refusal carrying only a closed code; an unknown
    // outcome carries the intent a reconciliation step must resolve.
    assert_eq!(denied["outcome"]["reason"], json!("replay_conflict"));
    assert_eq!(unknown["outcome"]["intent_id"], json!(id(9).to_string()));
    assert!(accepted["outcome"].as_object().unwrap().len() == 1);
    assert!(denied["outcome"].as_object().unwrap().len() == 2);
    assert!(unknown["outcome"].as_object().unwrap().len() == 2);

    assert_eq!(
        decode(&denied, &context),
        Ok(OutcomeEnvelope {
            command_id: id(2),
            installation_id: id(1),
            run_generation: RunGeneration::new(1).unwrap(),
            outcome: CommandOutcome::Denied(DenialReason::ReplayConflict),
        })
    );
}

#[test]
fn unknown_denial_reason_and_unknown_outcome_kind_fail_closed() {
    let (mut wire, context) = raw(CommandOutcome::Denied(DenialReason::WrongRole));
    wire["outcome"] = json!({"kind": "denied", "reason": "owner_is_angry"});
    assert_eq!(decode(&wire, &context), Err(ProtocolError::UnknownCommand));

    let (mut wire, context) = raw(CommandOutcome::Accepted);
    wire["outcome"] = json!({"kind": "arbitrary_accepted"});
    assert_eq!(decode(&wire, &context), Err(ProtocolError::UnknownCommand));

    // Missing kind tag is an invalid structure, never a silent acceptance.
    let (mut wire, context) = raw(CommandOutcome::Accepted);
    wire["outcome"] = json!({});
    assert_eq!(
        decode(&wire, &context),
        Err(ProtocolError::InvalidStructure)
    );
}

#[test]
fn unknown_and_duplicate_fields_are_rejected_at_every_level() {
    for pointer in ["", "/outcome"] {
        let (mut wire, context) = raw(CommandOutcome::Denied(DenialReason::StaleGeneration));
        wire.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("sender_role_claim_do_not_log".into(), json!("owner"));
        let error = decode(&wire, &context).unwrap_err();
        assert_eq!(error, ProtocolError::UnknownField);
        assert!(!error.to_string().contains("sender_role_claim"));
        assert!(!format!("{error:?}").contains("owner"));
    }

    let (envelope, context) = fixture(CommandOutcome::Denied(DenialReason::WrongRole), 1);
    let encoded = String::from_utf8(encode_outcome(&envelope, &context).unwrap()).unwrap();
    let needle = "\"kind\":\"denied\"";
    assert!(encoded.contains(needle));
    assert_eq!(
        decode_outcome(
            encoded
                .replace(needle, "\"kind\":\"denied\",\"kind\":\"denied\"")
                .as_bytes(),
            &context,
        ),
        Err(ProtocolError::DuplicateField)
    );
}

#[test]
fn installation_and_generation_are_checked_in_both_directions() {
    let (envelope, mut context) = fixture(CommandOutcome::UnknownOutcome { intent_id: id(7) }, 2);
    let bytes = encode_outcome(&envelope, &context).unwrap();
    context.installation_id = id(99);
    assert_eq!(
        decode_outcome(&bytes, &context),
        Err(ProtocolError::InstallationMismatch)
    );
    assert_eq!(
        encode_outcome(&envelope, &context),
        Err(ProtocolError::InstallationMismatch)
    );
    context.installation_id = envelope.installation_id;
    for n in [1, 3] {
        context.run_generation = RunGeneration::new(n).unwrap();
        assert_eq!(
            decode_outcome(&bytes, &context),
            Err(ProtocolError::GenerationMismatch)
        );
        assert_eq!(
            encode_outcome(&envelope, &context),
            Err(ProtocolError::GenerationMismatch)
        );
    }
}

#[test]
fn command_identity_cannot_reuse_installation_or_intent_id() {
    let (original, context) = fixture(CommandOutcome::UnknownOutcome { intent_id: id(9) }, 1);

    let mut envelope = original.clone();
    envelope.command_id = id(1);
    assert_eq!(
        encode_outcome(&envelope, &context),
        Err(ProtocolError::IdentifierReuse)
    );

    let mut envelope = original.clone();
    envelope.command_id = id(9);
    assert_eq!(
        encode_outcome(&envelope, &context),
        Err(ProtocolError::IdentifierReuse)
    );

    let mut envelope = original.clone();
    envelope.command_id = id(8);
    assert!(encode_outcome(&envelope, &context).is_ok());

    let (mut wire, context) = raw(CommandOutcome::UnknownOutcome { intent_id: id(9) });
    wire["command_id"] = json!(id(1).to_string());
    assert_eq!(decode(&wire, &context), Err(ProtocolError::IdentifierReuse));
    wire["command_id"] = json!(id(9).to_string());
    assert_eq!(decode(&wire, &context), Err(ProtocolError::IdentifierReuse));
}

#[test]
fn version_required_fields_and_counter_spellings_are_strict() {
    for version in [0u32, 2, u32::MAX] {
        let (mut wire, context) = raw(CommandOutcome::Accepted);
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
        "run_generation",
        "outcome",
    ] {
        let (mut wire, context) = raw(CommandOutcome::Accepted);
        wire.as_object_mut().unwrap().remove(field);
        assert!(decode(&wire, &context).is_err(), "{field}");
    }
    for invalid in [
        json!(0),
        json!(1),
        json!(1.0),
        json!(""),
        json!("01"),
        json!(" 1"),
    ] {
        let (mut wire, context) = raw(CommandOutcome::Accepted);
        wire["run_generation"] = invalid.clone();
        assert!(
            decode(&wire, &context).is_err(),
            "run_generation: {invalid}"
        );
    }
}

#[test]
fn malformed_truncated_and_oversized_frames_never_return_a_partial_candidate() {
    let (wire, context) = raw(CommandOutcome::Denied(DenialReason::ReplayConflict));
    let encoded = serde_json::to_string(&wire).unwrap();

    assert_eq!(
        decode_outcome(format!("{encoded} extra").as_bytes(), &context),
        Err(ProtocolError::Malformed)
    );
    assert_eq!(
        decode_outcome(&encoded.as_bytes()[..encoded.len() - 1], &context),
        Err(ProtocolError::Malformed)
    );
    for invalid in [
        Vec::new(),
        b"null".to_vec(),
        vec![0xff],
        encoded.as_bytes()[..encoded.len() - 1].to_vec(),
        format!("{encoded} {{}}").into_bytes(),
    ] {
        assert!(decode_outcome(&invalid, &context).is_err());
    }

    let mut boundary = encoded.into_bytes();
    boundary.resize(65_536, b' ');
    assert!(decode_outcome(&boundary, &context).is_ok());
    boundary.push(b' ');
    assert_eq!(
        decode_outcome(&boundary, &context),
        Err(ProtocolError::Oversize)
    );
    assert_eq!(
        decode_outcome(&vec![0xff; 65_537], &context),
        Err(ProtocolError::Oversize)
    );
}
