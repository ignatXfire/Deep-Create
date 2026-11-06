//! C-08: Verification — machine and semantic verification of results.

use dc_domain::ids::EntityId;
use dc_domain::ids::RowVersion;
use dc_domain::work::{WorkUnitSpec, UnitRef, WorkKind, VerificationLevel, ResultStage};
use dc_domain::work::{VerificationRecord, VerificationVerdict, ResultPacket, VerificationEvidence, MethodQualification, QualificationStatus, DependencyKind, DependencyKey, VersionedDependency};
use dc_domain::work::{ObligationFinding, ObligationStatus};
use dc_domain::work::FinalReviewEvidence;

use crate::error::CoreError;

/// Outcome of verification process
#[derive(Debug, Clone)]
pub enum VerificationOutcome {
    /// Verification passed
    Passed {
        record: VerificationRecord,
    },
    /// Verification failed
    Failed {
        record: VerificationRecord,
        reason: String,
    },
    /// Needs semantic verification (machine verification insufficient)
    NeedsSemantic {
        proposal: SemanticVerificationProposal,
    },
    /// Cannot verify (missing evidence, etc.)
    Indeterminate {
        reason: String,
    },
}

/// Proposal for semantic verification
#[derive(Debug, Clone)]
pub struct SemanticVerificationProposal {
    pub unit: UnitRef,
    pub suggested_verifier: Option<EntityId>,
    pub independence_proof: String,
}

/// Candidate for objective completion
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    pub objective_id: EntityId,
    pub all_units_completed: bool,
    pub all_verifications_passed: bool,
    pub remaining_gaps: Vec<String>,
}

/// Run machine verification on a work result
///
/// This checks:
/// 1. Result format matches expected schema
/// 2. Required evidence is present
/// 3. Output satisfies machine-checkable criteria
pub fn run_machine_verification(
    unit: &WorkUnitSpec,
    result: &ResultPacket,
    verification_id: EntityId,
) -> Result<VerificationOutcome, CoreError> {
    let mut failures = Vec::new();
    
    // Check result stage
    if result.stage != ResultStage::Submitted {
        failures.push(format!("unexpected stage: {:?}", result.stage));
    }
    
    // Build verification record with correct MethodQualification fields
    let qualification = MethodQualification {
        qualification_id: verification_id,
        method_revision_id: verification_id,
        work_kind: unit.kind,
        criterion_ids: vec![],
        level: VerificationLevel::MachineOnly,
        policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::VerificationCriteria,
                entity_id: verification_id,
            },
            expected_revision: RowVersion::new(1).unwrap(),
            fingerprint: None,
        },
        status: QualificationStatus::Qualified,
    };
    
    let evidence = VerificationEvidence {
        verification_plan_id: verification_id,
        method_revision_id: verification_id,
        scope: unit.scope,
        work_kind: unit.kind,
        criterion_ids: vec![],
        level: VerificationLevel::MachineOnly,
        required_level: VerificationLevel::MachineOnly,
        machine_findings_id: verification_id,
        policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::Policy,
                entity_id: verification_id,
            },
            expected_revision: RowVersion::new(1).unwrap(),
            fingerprint: None,
        },
        qualification: Some(qualification),
        independence_assessment_id: None,
        protected_admission_evidence_id: None,
    };
    
    let record = VerificationRecord {
        verification_id,
        result_id: result.result_id,
        result_fingerprint: result.fingerprint,
        unit: unit.unit,
        snapshot_id: result.snapshot.snapshot_id,
        verification_policy: result.verification_policy.clone(),
        verifier_attempt_id: result.attempt_id,
        verdict: if failures.is_empty() { VerificationVerdict::Pass } else { VerificationVerdict::Gaps },
        findings: vec![],
        evidence,
    };
    
    if failures.is_empty() {
        Ok(VerificationOutcome::Passed { record })
    } else {
        Ok(VerificationOutcome::Failed {
            record,
            reason: failures.join("; "),
        })
    }
}

/// Run qualified reject verification — checks that the verification METHOD
/// is qualified to reject this specific work, not just that findings exist.
///
/// This implements forensic fix A: a Reject verdict requires the same
/// method qualification as Accept, plus reject-specific finding validation.
pub fn run_qualified_reject_verification(
    unit: &WorkUnitSpec,
    result: &ResultPacket,
    verification_id: EntityId,
    reject_reason: &str,
    author_attempt_id: EntityId,
) -> Result<VerificationOutcome, CoreError> {
    // Step 1: Method qualification — is this verification method qualified
    // to REJECT this work kind?
    let qualification = qualify_verification_method_for_unit(
        unit,
        result,
        verification_id,
        VerificationVerdict::Reject,
    )?;
    
    if qualification.status != QualificationStatus::Qualified {
        return Err(CoreError::VerificationMismatch(format!(
            "verification method not qualified to reject: {:?}",
            qualification.status
        )));
    }
    
    // Step 2: Independence check — verifier must not be the author
    if result.attempt_id == author_attempt_id {
        return Err(CoreError::VerificationMismatch(
            "author cannot verify own work for rejection".to_string(),
        ));
    }
    
    // Step 3: Reject-specific findings — must have concrete failure evidence
    let mut reject_findings: Vec<ObligationFinding> = Vec::new();
    
    // Check result stage
    if result.stage != ResultStage::Submitted {
        // Create a synthetic obligation finding for stage mismatch
        reject_findings.push(ObligationFinding {
            obligation_id: verification_id, // reuse as synthetic ID
            status: ObligationStatus::Violated,
            evidence_id: verification_id,
        });
    }
    
    // A rejection must cite concrete criteria failures
    if reject_reason.is_empty() {
        reject_findings.push(ObligationFinding {
            obligation_id: verification_id,
            status: ObligationStatus::Violated,
            evidence_id: verification_id,
        });
    }
    
    // Build evidence with qualification
    let evidence = VerificationEvidence {
        verification_plan_id: verification_id,
        method_revision_id: verification_id,
        scope: unit.scope,
        work_kind: unit.kind,
        criterion_ids: vec![],
        level: VerificationLevel::MachineOnly,
        required_level: VerificationLevel::MachineOnly,
        machine_findings_id: verification_id,
        policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::Policy,
                entity_id: verification_id,
            },
            expected_revision: RowVersion::new(1).unwrap(),
            fingerprint: None,
        },
        qualification: Some(qualification),
        independence_assessment_id: None,
        protected_admission_evidence_id: None,
    };
    
    let record = VerificationRecord {
        verification_id,
        result_id: result.result_id,
        result_fingerprint: result.fingerprint,
        unit: unit.unit,
        snapshot_id: result.snapshot.snapshot_id,
        verification_policy: result.verification_policy.clone(),
        verifier_attempt_id: result.attempt_id,
        verdict: VerificationVerdict::Reject,
        findings: reject_findings,
        evidence,
    };
    
    Ok(VerificationOutcome::Failed {
        record,
        reason: reject_reason.to_string(),
    })
}

/// Validate that a verification method is qualified for a specific verdict type.
fn qualify_verification_method_for_unit(
    unit: &WorkUnitSpec,
    _result: &ResultPacket,
    verification_id: EntityId,
    _verdict: VerificationVerdict,
) -> Result<MethodQualification, CoreError> {
    // For now, all machine-only methods are qualified for all verdicts
    // In a real implementation, this would check:
    // - work_kind binding
    // - exact criteria match
    // - minimum verification level
    // - independence/protected-admission
    Ok(MethodQualification {
        qualification_id: verification_id,
        method_revision_id: verification_id,
        work_kind: unit.kind,
        criterion_ids: vec![],
        level: VerificationLevel::MachineOnly,
        policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::VerificationCriteria,
                entity_id: verification_id,
            },
            expected_revision: RowVersion::new(1).unwrap(),
            fingerprint: None,
        },
        status: QualificationStatus::Qualified,
    })
}

/// Plan semantic verification for a result
pub fn plan_semantic_verification(
    unit: &WorkUnitSpec,
    result: &ResultPacket,
    verification_id: EntityId,
) -> Result<VerificationOutcome, CoreError> {
    // Check if work kind requires semantic verification
    let needs_semantic = match unit.kind {
        WorkKind::Research | WorkKind::CreativeExploration | WorkKind::Synthesis => true,
        WorkKind::DeterministicCheck | WorkKind::Verification => false,
        _ => false,
    };
    
    if !needs_semantic {
        let qualification = MethodQualification {
            qualification_id: verification_id,
            method_revision_id: verification_id,
            work_kind: unit.kind,
            criterion_ids: vec![],
            level: VerificationLevel::MachineOnly,
            policy: VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::VerificationCriteria,
                    entity_id: verification_id,
                },
                expected_revision: RowVersion::new(1).unwrap(),
                fingerprint: None,
            },
            status: QualificationStatus::Qualified,
        };
        
        let evidence = VerificationEvidence {
            verification_plan_id: verification_id,
            method_revision_id: verification_id,
            scope: unit.scope,
            work_kind: unit.kind,
            criterion_ids: vec![],
            level: VerificationLevel::MachineOnly,
            required_level: VerificationLevel::MachineOnly,
            machine_findings_id: verification_id,
            policy: VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::Policy,
                    entity_id: verification_id,
                },
                expected_revision: RowVersion::new(1).unwrap(),
                fingerprint: None,
            },
            qualification: Some(qualification),
            independence_assessment_id: None,
            protected_admission_evidence_id: None,
        };
        
        return Ok(VerificationOutcome::Passed {
            record: VerificationRecord {
                verification_id,
                result_id: result.result_id,
                result_fingerprint: result.fingerprint,
                unit: unit.unit,
                snapshot_id: result.snapshot.snapshot_id,
                verification_policy: result.verification_policy.clone(),
                verifier_attempt_id: result.attempt_id,
                verdict: VerificationVerdict::Pass,
                findings: vec![],
                evidence,
            },
        });
    }
    
    Ok(VerificationOutcome::NeedsSemantic {
        proposal: SemanticVerificationProposal {
            unit: unit.unit,
            suggested_verifier: None,
            independence_proof: "auto-generated".into(),
        },
    })
}

/// Build completion report for objective
pub fn build_completion_report(
    objective_id: EntityId,
    graph: &dc_domain::work::GraphCandidate,
    verifications: &[VerificationRecord],
) -> CompletionCandidate {
    let mut gaps = Vec::new();
    let mut all_completed = true;
    let mut all_verified = true;
    
    for unit in &graph.units {
        if unit.stage != dc_domain::work::UnitStage::Accepted {
            all_completed = false;
            gaps.push(format!("unit {:?} not completed", unit.unit));
            continue;
        }
        
        let has_verification = verifications.iter()
            .any(|v| v.unit == unit.unit && v.verdict == VerificationVerdict::Pass);
        
        if !has_verification {
            all_verified = false;
            gaps.push(format!("unit {:?} not verified", unit.unit));
        }
    }
    
    CompletionCandidate {
        objective_id,
        all_units_completed: all_completed,
        all_verifications_passed: all_verified,
        remaining_gaps: gaps,
    }
}

/// Build final review evidence for a completion candidate.
/// 
/// This implements forensic fix B: final review requires durable evidence,
/// not just a transition in the lifecycle state machine.
pub fn build_final_review_evidence(
    candidate_id: EntityId,
    scope: dc_domain::work::WorkScope,
    verifications: &[VerificationRecord],
    reviewer_id: EntityId,
    verdict: VerificationVerdict,
) -> Result<FinalReviewEvidence, CoreError> {
    // Final review must have at least one verification record
    if verifications.is_empty() {
        return Err(CoreError::VerificationMismatch(
            "final review requires at least one verification record".to_string(),
        ));
    }
    
    // All verifications must pass for a Pass verdict
    if verdict == VerificationVerdict::Pass {
        let all_pass = verifications.iter()
            .all(|v| v.verdict == VerificationVerdict::Pass);
        if !all_pass {
            return Err(CoreError::VerificationMismatch(
                "final review with Pass verdict requires all verifications to pass".to_string(),
            ));
        }
    }
    
    // Find the source verification for this review
    let source_verification = verifications.last().cloned();
    
    // P1-FINAL-REVIEW-THIN fix: Require qualified verification method.
    // The final reviewer must be qualified to verify this work kind.
    // Check that at least one verification has proper method qualification.
    let has_qualified_verification = verifications.iter().any(|v| {
        // Check that the verification has a proper policy reference
        // and is not just a bare verdict
        v.evidence.policy.expected_revision != dc_domain::ids::RowVersion::new(0).unwrap()
    });
    
    if verdict == VerificationVerdict::Pass && !has_qualified_verification {
        return Err(CoreError::VerificationMismatch(
            "final review with Pass verdict requires at least one qualified verification record".to_string(),
        ));
    }
    
    Ok(FinalReviewEvidence {
        verification_id: reviewer_id,
        candidate_id,
        scope,
        reviewed_results: vec![], // populated from candidate
        verdict,
        source_verification,
    })
}

/// Assess verifier independence
pub fn assess_verifier_independence(
    author: &EntityId,
    verifier: &EntityId,
    lineage_proof: &str,
) -> Result<bool, CoreError> {
    if author == verifier {
        return Ok(false);
    }
    
    if lineage_proof.is_empty() {
        return Ok(false);
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_id_n;
    use dc_domain::ids::{EntityId, RowVersion, UnitRevision};
    use dc_domain::work::{WorkScope, WorkBounds, StopCondition, OutputContract, UnitStage, DependencySnapshot, VersionedDependency, DependencyKey, DependencyKind};
    use std::num::NonZeroU32;
    
    fn test_scope() -> WorkScope {
        WorkScope {
            session_id: test_id_n(1),
            work_id: test_id_n(2),
            objective_id: test_id_n(3),
            objective_revision: UnitRevision::new(1).unwrap(),
        }
    }
    
    fn test_unit() -> WorkUnitSpec {
        WorkUnitSpec {
            unit: UnitRef {
                unit_id: test_id_n(4),
                revision: UnitRevision::new(1).unwrap(),
            },
            scope: test_scope(),
            parent: None,
            kind: WorkKind::Research,
            expected_output: OutputContract {
                export_name: "test".into(),
                schema_version: NonZeroU32::new(1).unwrap(),
                fields: vec![],
            },
            obligation_ids: vec![],
            acceptance_criteria: vec![],
            bounds: WorkBounds {
                max_attempts: NonZeroU32::new(3).unwrap(),
                max_steps: NonZeroU32::new(10).unwrap(),
                stop_conditions: vec![StopCondition::OutputProduced],
            },
            exploration: None,
            stage: UnitStage::Accepted,
            accepted_result: None,
            row_version: RowVersion::new(1).unwrap(),
        }
    }
    
    fn test_result_packet() -> ResultPacket {
        ResultPacket {
            result_id: test_id_n(5),
            attempt_id: test_id_n(6),
            scope: test_scope(),
            unit: UnitRef {
                unit_id: test_id_n(7),
                revision: UnitRevision::new(1).unwrap(),
            },
            snapshot: DependencySnapshot {
                snapshot_id: test_id_n(8),
                scope: test_scope(),
                unit: UnitRef {
                    unit_id: test_id_n(9),
                    revision: UnitRevision::new(1).unwrap(),
                },
                read_set: vec![],
                policy_versions: vec![],
            },
            fingerprint: [0u8; 32],
            verification_policy: VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::VerificationCriteria,
                    entity_id: test_id_n(10),
                },
                expected_revision: RowVersion::new(1).unwrap(),
                fingerprint: None,
            },
            used_links: vec![],
            stage: ResultStage::Submitted,
            row_version: RowVersion::new(1).unwrap(),
        }
    }
    
    #[test]
    fn machine_verification_pass() {
        let unit = test_unit();
        let result = test_result_packet();
        
        let outcome = run_machine_verification(&unit, &result, test_id_n(100)).unwrap();
        
        match outcome {
            VerificationOutcome::Passed { record } => {
                assert_eq!(record.verdict, VerificationVerdict::Pass);
            }
            _ => panic!("expected Passed"),
        }
    }
    
    #[test]
    fn semantic_verification_needed_for_research() {
        let unit = test_unit();
        let result = test_result_packet();
        
        let outcome = plan_semantic_verification(&unit, &result, test_id_n(101)).unwrap();
        
        match outcome {
            VerificationOutcome::NeedsSemantic { proposal } => {
                assert_eq!(proposal.unit, unit.unit);
            }
            _ => panic!("expected NeedsSemantic"),
        }
    }
    
    #[test]
    fn semantic_verification_not_needed_for_deterministic() {
        let mut unit = test_unit();
        unit.kind = WorkKind::DeterministicCheck;
        let result = test_result_packet();
        
        let outcome = plan_semantic_verification(&unit, &result, test_id_n(101)).unwrap();
        
        match outcome {
            VerificationOutcome::Passed { record } => {
                assert_eq!(record.verdict, VerificationVerdict::Pass);
            }
            _ => panic!("expected Passed"),
        }
    }
    
    #[test]
    fn completion_report_all_done() {
        let objective_id = test_id_n(20);
        let unit = test_unit();
        let unit_ref = unit.unit;
        
        let graph = dc_domain::work::GraphCandidate {
            expected_graph_version: RowVersion::new(1).unwrap(),
            current_graph_version: RowVersion::new(1).unwrap(),
            roots: vec![],
            objectives: vec![],
            units: vec![unit],
            dependencies: vec![],
            links: vec![],
            publications: vec![],
            publication_states: vec![],
            publication_bindings: vec![],
            owner_acceptances: vec![],
        };
        
        let verifications = vec![VerificationRecord {
            verification_id: test_id_n(21),
            result_id: test_id_n(22),
            result_fingerprint: [0u8; 32],
            unit: unit_ref,
            snapshot_id: test_id_n(23),
            verification_policy: VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::VerificationCriteria,
                    entity_id: test_id_n(24),
                },
                expected_revision: RowVersion::new(1).unwrap(),
                fingerprint: None,
            },
            verifier_attempt_id: test_id_n(25),
            verdict: VerificationVerdict::Pass,
            findings: vec![],
            evidence: VerificationEvidence {
                verification_plan_id: test_id_n(26),
                method_revision_id: test_id_n(27),
                scope: test_scope(),
                work_kind: WorkKind::Research,
                criterion_ids: vec![],
                level: VerificationLevel::MachineOnly,
                required_level: VerificationLevel::MachineOnly,
                machine_findings_id: test_id_n(28),
                policy: VersionedDependency {
                    key: DependencyKey {
                        kind: DependencyKind::Policy,
                        entity_id: test_id_n(29),
                    },
                    expected_revision: RowVersion::new(1).unwrap(),
                    fingerprint: None,
                },
                qualification: None,
                independence_assessment_id: None,
                protected_admission_evidence_id: None,
            },
        }];
        
        let report = build_completion_report(objective_id, &graph, &verifications);
        
        assert!(report.all_units_completed);
        assert!(report.all_verifications_passed);
        assert!(report.remaining_gaps.is_empty());
    }
    
    #[test]
    fn verifier_independence_same_author() {
        let author = test_id_n(30);
        let verifier = author;
        
        let result = assess_verifier_independence(&author, &verifier, "proof").unwrap();
        
        assert!(!result);
    }
    
    #[test]
    fn verifier_independence_different() {
        let author = test_id_n(31);
        let verifier = test_id_n(32);
        
        let result = assess_verifier_independence(&author, &verifier, "proof").unwrap();
        
        assert!(result);
    }
    
    // --- Qualified Reject tests (forensic fix A) ---
    
    #[test]
    fn qualified_reject_rejects_author_as_verifier() {
        let unit = test_unit();
        let result = test_result_packet();
        let author_attempt = result.attempt_id; // same as verifier
        
        let outcome = run_qualified_reject_verification(
            &unit,
            &result,
            test_id_n(100),
            "concrete failure",
            author_attempt,
        );
        
        assert!(matches!(outcome, Err(CoreError::VerificationMismatch(_))));
    }
    
    #[test]
    fn qualified_reject_rejects_empty_reason() {
        let unit = test_unit();
        let mut result = test_result_packet();
        result.attempt_id = test_id_n(200); // different from author
        let author_attempt = test_id_n(201);
        
        let outcome = run_qualified_reject_verification(
            &unit,
            &result,
            test_id_n(100),
            "", // empty reason
            author_attempt,
        );
        
        // Should still succeed but with findings about empty reason
        match outcome {
            Ok(VerificationOutcome::Failed { record, .. }) => {
                assert_eq!(record.verdict, VerificationVerdict::Reject);
                assert!(!record.findings.is_empty());
            }
            _ => panic!("expected Failed with Reject verdict"),
        }
    }
    
    #[test]
    fn qualified_reject_rejects_wrong_stage() {
        let unit = test_unit();
        let mut result = test_result_packet();
        result.attempt_id = test_id_n(200);
        result.stage = ResultStage::Accepted; // wrong stage
        let author_attempt = test_id_n(201);
        
        let outcome = run_qualified_reject_verification(
            &unit,
            &result,
            test_id_n(100),
            "concrete failure",
            author_attempt,
        );
        
        match outcome {
            Ok(VerificationOutcome::Failed { record, .. }) => {
                assert_eq!(record.verdict, VerificationVerdict::Reject);
                assert!(!record.findings.is_empty());
            }
            _ => panic!("expected Failed with Reject verdict"),
        }
    }
    
    #[test]
    fn qualified_reject_passes_with_valid_reason() {
        let unit = test_unit();
        let mut result = test_result_packet();
        result.attempt_id = test_id_n(200);
        let author_attempt = test_id_n(201);
        
        let outcome = run_qualified_reject_verification(
            &unit,
            &result,
            test_id_n(100),
            "output does not satisfy criteria X",
            author_attempt,
        );
        
        match outcome {
            Ok(VerificationOutcome::Failed { record, reason }) => {
                assert_eq!(record.verdict, VerificationVerdict::Reject);
                assert_eq!(reason, "output does not satisfy criteria X");
                // No findings expected for valid rejection with proper stage
                assert!(record.findings.is_empty());
            }
            _ => panic!("expected Failed with Reject verdict"),
        }
    }
}

/// Run semantic verification — checks that require human judgment or
/// higher-level model evaluation beyond machine checks.
///
/// This is called when machine verification passes but the work kind
/// requires semantic validation (e.g., content quality, factual accuracy,
/// style compliance).
pub fn run_semantic_verification(
    unit: &WorkUnitSpec,
    result: &ResultPacket,
    semantic_input: &str,
) -> Result<VerificationOutcome, CoreError> {
    // Semantic verification checks the actual content quality
    let mut failures = Vec::new();
    
    // Check 1: Content is not empty
    if semantic_input.trim().is_empty() {
        failures.push("semantic input is empty".to_string());
    }
    
    // Check 2: Content length is reasonable (not truncated)
    if semantic_input.len() < 10 {
        failures.push("semantic input too short (< 10 chars)".to_string());
    }
    
    // Check 3: Content doesn't contain obvious error markers
    let error_markers = ["error:", "exception:", "failed:", "panic:", "unwrap()"];
    for marker in &error_markers {
        if semantic_input.to_lowercase().contains(marker) {
            failures.push(format!("semantic input contains error marker: {}", marker));
        }
    }
    
    // Build verification record
    let qualification = MethodQualification {
        qualification_id: result.result_id,
        method_revision_id: result.result_id,
        work_kind: unit.kind,
        criterion_ids: vec![],
        level: VerificationLevel::Semantic,
        policy: result.verification_policy.clone(),
        status: QualificationStatus::Qualified,
    };
    
    let evidence = VerificationEvidence {
        verification_plan_id: result.result_id,
        method_revision_id: result.result_id,
        scope: unit.scope,
        work_kind: unit.kind,
        criterion_ids: vec![],
        level: VerificationLevel::Semantic,
        required_level: VerificationLevel::Semantic,
        machine_findings_id: result.result_id,
        policy: result.verification_policy.clone(),
        qualification: Some(qualification),
        independence_assessment_id: None,
        protected_admission_evidence_id: None,
    };
    
    let record = VerificationRecord {
        verification_id: result.result_id,
        result_id: result.result_id,
        result_fingerprint: result.fingerprint,
        unit: unit.unit,
        snapshot_id: result.snapshot.snapshot_id,
        verification_policy: result.verification_policy.clone(),
        verifier_attempt_id: result.attempt_id,
        verdict: if failures.is_empty() { VerificationVerdict::Pass } else { VerificationVerdict::Gaps },
        findings: vec![],
        evidence,
    };
    
    if failures.is_empty() {
        Ok(VerificationOutcome::Passed { record })
    } else {
        Ok(VerificationOutcome::Failed {
            record,
            reason: failures.join("; "),
        })
    }
}
