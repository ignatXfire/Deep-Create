//! Error-injection tests for dc-core command failure paths.
//!
//! Covers: invalid objective JSON, malformed graph, verification failure.

use dc_core::CoreState;
use dc_core::command::{CoreCommand, decide_transition, handle_command};
use dc_core::error::CoreError;
use dc_domain::ids::EntityId;

/// Build a valid UUIDv7 EntityId for tests.
fn test_id_n(n: u128) -> EntityId {
    let timestamp = 0x123456789abc_u128;
    let rand_a = 0x123_u128;
    let rand_b = (0x123456789abcdef_u128 + n) & ((1u128 << 62) - 1);
    let value = (timestamp << 80) | (7 << 76) | (rand_a << 64) | (2 << 62) | rand_b;
    EntityId::from_u128(value).expect("constructed UUIDv7 must be valid")
}

/// Minimal valid ObjectiveContract JSON (matches dc-domain serde).
fn valid_objective_json() -> String {
    // The ObjectiveContract shape is complex; we build it via serde_json from
    // a minimal skeleton with all required fields.  Field names follow the
    // dc-domain definitions.
    format!(
        r#"{{
  "scope": {{
    "session_id": "{}",
    "work_id": "{}",
    "objective_id": "{}",
    "objective_revision": 1
  }},
  "statement": "Test objective",
  "owner_event_ids": [],
  "obligations": [],
  "completion_criteria": ["{}"],
  "row_version": 1
}}"#,
        test_id_n(1),
        test_id_n(2),
        test_id_n(3),
        test_id_n(4)
    )
}

/// 1. Invalid objective JSON — syntactically malformed.
#[test]
fn accept_objective_invalid_json_syntax() {
    // dc-desktop main.rs does: serde_json::from_str::<ObjectiveContract>(&objective_json)
    // We test the same parse path via the domain type.
    let bad = "this is not json";
    let result: Result<dc_domain::work::ObjectiveContract, _> = serde_json::from_str(bad);
    assert!(result.is_err(), "expected JSON parse error");
}

/// 2. Invalid objective — valid JSON but wrong shape (missing required field).
#[test]
fn accept_objective_wrong_shape() {
    let bad = r#"{"statement": "missing everything else"}"#;
    let result: Result<dc_domain::work::ObjectiveContract, _> = serde_json::from_str(bad);
    assert!(result.is_err(), "expected missing-field error");
}

/// 3. Objective with empty completion criteria → CoreError::InvalidCommand.
#[test]
fn accept_objective_empty_criteria() {
    let json = format!(
        r#"{{
  "scope": {{
    "session_id": "{}",
    "work_id": "{}",
    "objective_id": "{}",
    "objective_revision": 1
  }},
  "statement": "No criteria",
  "owner_event_ids": [],
  "obligations": [],
  "completion_criteria": [],
  "row_version": 1
}}"#,
        test_id_n(1),
        test_id_n(2),
        test_id_n(3)
    );
    let objective: dc_domain::work::ObjectiveContract =
        serde_json::from_str(&json).expect("parse should succeed");

    let mut state = CoreState::empty();
    let result = handle_command(&mut state, CoreCommand::AcceptObjective(objective));
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), CoreError::InvalidCommand(_)),
        "expected InvalidCommand for empty criteria"
    );
}

/// 4. Malformed graph JSON — AcceptGraphRaw with invalid JSON.
#[test]
fn accept_graph_raw_invalid_json() {
    let mut state = CoreState::empty();
    // Must have an objective first for graph acceptance to reach validation,
    // but invalid JSON should fail before that check.
    let result = handle_command(
        &mut state,
        CoreCommand::AcceptGraphRaw("not json".to_string()),
    );
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), CoreError::InvalidCommand(_)),
        "expected InvalidCommand for malformed graph JSON"
    );
}

/// 5. Malformed graph — self-dependency (cycle of length 1).
#[test]
fn accept_graph_raw_self_dependency() {
    let unit_id = test_id_n(10);
    let graph_json = format!(
        r#"{{
  "expected_graph_version": 1,
  "current_graph_version": 1,
  "roots": [],
  "objectives": [],
  "units": [{{
    "unit": {{"unit_id": "{}", "revision": 1}},
    "scope": {{
      "session_id": "{}",
      "work_id": "{}",
      "objective_id": "{}",
      "objective_revision": 1
    }},
    "parent": null,
    "kind": "research",
    "expected_output": {{
      "export_name": "test",
      "schema_version": 1,
      "fields": []
    }},
    "obligation_ids": [],
    "acceptance_criteria": [],
    "bounds": {{
      "max_attempts": 3,
      "max_steps": 10,
      "stop_conditions": ["output_produced"]
    }},
    "exploration": null,
    "stage": "proposed",
    "accepted_result": null,
    "row_version": 1
  }}],
  "dependencies": [{{
    "consumer": {{"unit_id": "{}", "revision": 1}},
    "producer": {{"unit_id": "{}", "revision": 1}},
    "relation": "must_finish"
  }}],
  "links": [],
  "publications": [],
  "publication_states": [],
  "publication_bindings": [],
  "owner_acceptances": []
}}"#,
        unit_id,
        test_id_n(1),
        test_id_n(2),
        test_id_n(3),
        unit_id,
        unit_id
    );

    // Must accept an objective first
    let objective: dc_domain::work::ObjectiveContract =
        serde_json::from_str(&valid_objective_json()).expect("parse objective");
    let mut state = CoreState::empty();
    handle_command(&mut state, CoreCommand::AcceptObjective(objective))
        .expect("objective should be accepted");

    let result = handle_command(&mut state, CoreCommand::AcceptGraphRaw(graph_json));
    assert!(result.is_err(), "expected self-dependency error");
    let err = result.unwrap_err();
    assert!(
        matches!(err, CoreError::InvalidCommand(_)),
        "expected InvalidCommand for self-dependency, got {:?}",
        err
    );
}

/// 6. Malformed graph — dependency references a non-existent unit.
#[test]
fn accept_graph_raw_dangling_dependency() {
    let unit_id = test_id_n(10);
    let ghost_id = test_id_n(99);
    let graph_json = format!(
        r#"{{
  "expected_graph_version": 1,
  "current_graph_version": 1,
  "roots": [],
  "objectives": [],
  "units": [{{
    "unit": {{"unit_id": "{}", "revision": 1}},
    "scope": {{
      "session_id": "{}",
      "work_id": "{}",
      "objective_id": "{}",
      "objective_revision": 1
    }},
    "parent": null,
    "kind": "research",
    "expected_output": {{
      "export_name": "test",
      "schema_version": 1,
      "fields": []
    }},
    "obligation_ids": [],
    "acceptance_criteria": [],
    "bounds": {{
      "max_attempts": 3,
      "max_steps": 10,
      "stop_conditions": ["output_produced"]
    }},
    "exploration": null,
    "stage": "proposed",
    "accepted_result": null,
    "row_version": 1
  }}],
  "dependencies": [{{
    "consumer": {{"unit_id": "{}", "revision": 1}},
    "producer": {{"unit_id": "{}", "revision": 1}},
    "relation": "must_finish"
  }}],
  "links": [],
  "publications": [],
  "publication_states": [],
  "publication_bindings": [],
  "owner_acceptances": []
}}"#,
        unit_id,
        test_id_n(1),
        test_id_n(2),
        test_id_n(3),
        unit_id,
        ghost_id
    );

    let objective: dc_domain::work::ObjectiveContract =
        serde_json::from_str(&valid_objective_json()).expect("parse objective");
    let mut state = CoreState::empty();
    handle_command(&mut state, CoreCommand::AcceptObjective(objective))
        .expect("objective should be accepted");

    let result = handle_command(&mut state, CoreCommand::AcceptGraphRaw(graph_json));
    assert!(result.is_err(), "expected dangling dependency error");
    let err = result.unwrap_err();
    assert!(
        matches!(err, CoreError::InvalidCommand(_)),
        "expected InvalidCommand for dangling dependency, got {:?}",
        err
    );
}

/// 7. Graph without objective → CoreError::NoObjective.
#[test]
fn accept_graph_raw_no_objective() {
    let graph_json = r#"{
  "expected_graph_version": 1,
  "current_graph_version": 1,
  "roots": [],
  "objectives": [],
  "units": [],
  "dependencies": [],
  "links": [],
  "publications": [],
  "publication_states": [],
  "publication_bindings": [],
  "owner_acceptances": []
}"#;

    let mut state = CoreState::empty();
    let result = handle_command(
        &mut state,
        CoreCommand::AcceptGraphRaw(graph_json.to_string()),
    );
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), CoreError::NoObjective),
        "expected NoObjective"
    );
}

/// 8. Verification failure — ApplyVerifiedResult with a non-Pass verdict.
#[test]
fn apply_verified_result_rejected_verdict() {
    use dc_domain::work::{
        DependencyKey, DependencyKind, ObligationFinding, VerificationEvidence, VerificationLevel,
        VerificationRecord, VerificationVerdict, VersionedDependency,
    };

    let unit_id = test_id_n(10);
    let unit_ref = dc_domain::work::UnitRef {
        unit_id,
        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
    };

    // Build a minimal verification record with verdict = Reject
    let record = VerificationRecord {
        verification_id: test_id_n(20),
        result_id: test_id_n(21),
        result_fingerprint: [0u8; 32],
        unit: unit_ref,
        snapshot_id: test_id_n(22),
        verification_policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::Policy,
                entity_id: test_id_n(23),
            },
            expected_revision: dc_domain::ids::RowVersion::new(1).unwrap(),
            fingerprint: None,
        },
        verifier_attempt_id: test_id_n(24),
        verdict: VerificationVerdict::Reject,
        findings: vec![],
        evidence: VerificationEvidence {
            verification_plan_id: test_id_n(25),
            method_revision_id: test_id_n(26),
            scope: dc_domain::work::WorkScope {
                session_id: test_id_n(1),
                work_id: test_id_n(2),
                objective_id: test_id_n(3),
                objective_revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
            },
            work_kind: dc_domain::work::WorkKind::Research,
            criterion_ids: vec![],
            level: VerificationLevel::MachineOnly,
            required_level: VerificationLevel::MachineOnly,
            machine_findings_id: test_id_n(27),
            policy: VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::Policy,
                    entity_id: test_id_n(28),
                },
                expected_revision: dc_domain::ids::RowVersion::new(1).unwrap(),
                fingerprint: None,
            },
            qualification: None,
            independence_assessment_id: None,
            protected_admission_evidence_id: None,
        },
    };

    let mut state = CoreState::empty();
    state.verifications.push(record);

    let result = handle_command(&mut state, CoreCommand::ApplyVerifiedResult(unit_ref));
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), CoreError::VerificationFailed(_)),
        "expected VerificationFailed for rejected verdict"
    );
}

/// 9. Verification failure — ApplyVerifiedResult with no verification at all.
#[test]
fn apply_verified_result_missing_verification() {
    let unit_ref = dc_domain::work::UnitRef {
        unit_id: test_id_n(10),
        revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
    };

    let mut state = CoreState::empty();
    let result = handle_command(&mut state, CoreCommand::ApplyVerifiedResult(unit_ref));
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), CoreError::InvalidTransition(_)),
        "expected InvalidTransition for missing verification"
    );
}

/// 10. Double objective — accept second objective while first is active.
#[test]
fn accept_objective_double_objective() {
    let objective: dc_domain::work::ObjectiveContract =
        serde_json::from_str(&valid_objective_json()).expect("parse objective");
    let mut state = CoreState::empty();

    handle_command(&mut state, CoreCommand::AcceptObjective(objective.clone()))
        .expect("first objective should be accepted");

    let result = handle_command(&mut state, CoreCommand::AcceptObjective(objective));
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), CoreError::InvalidTransition(_)),
        "expected InvalidTransition for double objective"
    );
}
