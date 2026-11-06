//! Self-development gate: Deep Create analyzes pre-loaded codebase facts and
//! classifies improvement proposals for owner-authorized work units.
//!
//! This module is C-01 pure: it performs no I/O, reads no files, and has no
//! clock, network, or environment access. All facts must be supplied by the
//! caller (e.g. a broker that has already scanned the workspace).
//!
//! # Authority model
//!
//! **Confidence never creates modification authority.** A proposal with
//! `confidence = 1.0` still requires explicit owner preauthorization before
//! any real effect. The gate's only job is to:
//! 1. Identify candidate improvements from evidence.
//! 2. Classify them by impact and required authority.
//! 3. Emit structured proposals that a coordinator can turn into work units.
//!
//! No proposal is ever "auto-applied" by this module.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ids::EntityId;
use crate::work::{WorkKind, WorkScope};

/// A self-improvement proposal generated from actual codebase evidence.
///
/// Unlike the prior hardcoded placeholder, every field must be traceable
/// to a concrete observation supplied by the caller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelfImprovementProposal {
    /// Unique identifier for this proposal (caller-assigned).
    pub proposal_id: EntityId,
    /// Work scope this proposal belongs to — prevents cross-work contamination.
    pub work_scope: WorkScope,
    /// Area of the codebase to improve.
    pub target_area: TargetArea,
    /// What problem was identified, with evidence reference.
    pub problem: String,
    /// Proposed solution.
    pub solution: String,
    /// Confidence in the proposal (0.0 - 1.0). Never used for authority.
    pub confidence: f64,
    /// Expected impact.
    pub impact: ImpactLevel,
    /// Required authority level to apply. Never `Automatic` for real changes.
    pub required_authority: AuthorityLevel,
    /// Evidence references that justify this proposal.
    pub evidence_ids: Vec<EntityId>,
    /// The kind of work this proposal would generate.
    pub work_kind: WorkKind,
    /// Obligations that must be satisfied before this proposal can execute.
    pub prerequisite_obligations: Vec<EntityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetArea {
    DomainModel,
    Protocol,
    Storage,
    DesktopUi,
    ControlPlane,
    Documentation,
    TestCoverage,
    Performance,
    Security,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactLevel {
    Minor,
    Moderate,
    Major,
    Architectural,
}

/// Authority required before a proposal can be executed.
///
/// `Automatic` is **not** a valid level for real codebase changes.
/// It exists only for proposals that produce no side effects
/// (e.g. generating a report). Any proposal that would modify
/// code, configuration, or running state must be at least
/// `OwnerApproval`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityLevel {
    /// Read-only analysis or report generation. No modification.
    Automatic,
    /// Requires explicit owner approval before execution.
    OwnerApproval,
    /// Requires independent review by a second worker before execution.
    ExternalReview,
}

/// Facts about the codebase supplied by the caller.
/// The gate does not read files; it reasons over these facts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodebaseFacts {
    /// Test coverage observations.
    pub test_coverage: Vec<TestCoverageFact>,
    /// Documentation observations.
    pub documentation: Vec<DocumentationFact>,
    /// Protocol completeness observations.
    pub protocol: Vec<ProtocolFact>,
    /// Known security observations.
    pub security: Vec<SecurityFact>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestCoverageFact {
    pub area: String,
    pub has_integration_tests: bool,
    pub has_property_tests: bool,
    pub evidence_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentationFact {
    pub topic: String,
    pub is_linked_from_index: bool,
    pub evidence_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolFact {
    pub feature: String,
    pub has_versioning: bool,
    pub has_capability_negotiation: bool,
    pub evidence_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityFact {
    pub concern: String,
    pub is_mitigated: bool,
    pub evidence_id: EntityId,
}

/// Classification result: proposals grouped by the authority they require.
/// No proposal is ever executed by this module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalClassification {
    /// Proposals that are read-only and can proceed without owner approval.
    pub automatic: Vec<SelfImprovementProposal>,
    /// Proposals that require explicit owner approval.
    pub owner_approval: Vec<SelfImprovementProposal>,
    /// Proposals that require independent external review.
    pub external_review: Vec<SelfImprovementProposal>,
}

/// Self-development gate controller.
///
/// Analyzes pre-loaded codebase facts and generates improvement proposals.
/// The gate is C-01 pure: no I/O, no side effects, no authority to modify.
pub struct SelfDevelopmentGate {
    /// Minimum confidence to include a proposal in output.
    /// This is a filtering threshold, not an authority grant.
    pub min_confidence: f64,
    /// Maximum proposals per run.
    pub max_proposals: usize,
}

impl Default for SelfDevelopmentGate {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_proposals: 20,
        }
    }
}

impl SelfDevelopmentGate {
    /// Analyze pre-loaded codebase facts and generate improvement proposals.
    ///
    /// # Arguments
    /// * `facts` - Pre-loaded observations about the codebase.
    /// * `scope` - The work scope for traceability.
    ///
    /// Analyze codebase facts and generate improvement proposals.
    ///
    /// `id_allocator` generates fresh, unique proposal IDs. The caller (controller)
    /// owns ID generation to prevent collision with evidence IDs in the shared
    /// entity ID space. Each proposal gets its own ID, distinct from `evidence_id`.
    ///
    /// # Returns
    /// Proposals sorted by impact (highest first), then confidence.
    /// No proposal is executed; all are returned for classification.
    /// Scope binding proof: every proposal carries the exact work scope it
    /// was generated for. This prevents cross-work contamination where
    /// evidence from one objective leaks into another's proposals.
    ///
    /// The scope is not just metadata — it is the boundary that ensures
    /// proposal generation is deterministic and isolated per work unit.
    pub fn analyze(
        &self,
        facts: &CodebaseFacts,
        scope: &WorkScope,
        id_allocator: &mut dyn FnMut() -> EntityId,
    ) -> Vec<SelfImprovementProposal> {
        let mut proposals = Vec::new();

        // Each sub-analyzer receives the scope and MUST embed it in every
        // proposal it creates. This is the binding that makes proposals
        // scope-exclusive and prevents evidence drift across works.
        proposals.extend(self.analyze_test_coverage(&facts.test_coverage, scope, id_allocator));
        proposals.extend(self.analyze_documentation(&facts.documentation, scope, id_allocator));
        proposals.extend(self.analyze_protocol(&facts.protocol, scope, id_allocator));
        proposals.extend(self.analyze_security(&facts.security, scope, id_allocator));

        // Filter by minimum confidence (quality gate, not authority).
        proposals.retain(|p| p.confidence >= self.min_confidence);

        // Sort by impact (highest first), then confidence (highest first).
        proposals.sort_by(|a, b| {
            b.impact
                .cmp(&a.impact)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap())
        });

        proposals.truncate(self.max_proposals);
        proposals
    }

    fn analyze_test_coverage(
        &self,
        facts: &[TestCoverageFact],
        scope: &WorkScope,
        id_allocator: &mut dyn FnMut() -> EntityId,
    ) -> Vec<SelfImprovementProposal> {
        facts
            .iter()
            .filter(|f| !f.has_integration_tests || !f.has_property_tests)
            .map(|f| {
                let (problem, solution, impact) = if !f.has_integration_tests {
                    (
                        format!("No integration tests for {}", f.area),
                        format!("Add integration tests that exercise {} end-to-end", f.area),
                        ImpactLevel::Moderate,
                    )
                } else {
                    (
                        format!("No property-based tests for {}", f.area),
                        format!("Add property-based tests for {} invariants", f.area),
                        ImpactLevel::Minor,
                    )
                };
                SelfImprovementProposal {
                    proposal_id: id_allocator(),
                work_scope: *scope,
                    target_area: TargetArea::TestCoverage,
                    problem,
                    solution,
                    confidence: 0.85,
                    impact,
                    required_authority: AuthorityLevel::OwnerApproval,
                    evidence_ids: vec![f.evidence_id],
                    work_kind: WorkKind::LocalChangeProposal,
                    prerequisite_obligations: Vec::new(),
                }
            })
            .collect()
    }

    fn analyze_documentation(
        &self,
        facts: &[DocumentationFact],
        scope: &WorkScope,
        id_allocator: &mut dyn FnMut() -> EntityId,
    ) -> Vec<SelfImprovementProposal> {
        facts
            .iter()
            .filter(|f| !f.is_linked_from_index)
            .map(|f| SelfImprovementProposal {
                proposal_id: id_allocator(),
                work_scope: *scope,
                target_area: TargetArea::Documentation,
                problem: format!("Documentation topic '{}' not linked from index", f.topic),
                solution: format!("Add link to '{}' in the appropriate index", f.topic),
                confidence: 0.90,
                impact: ImpactLevel::Minor,
                required_authority: AuthorityLevel::OwnerApproval,
                evidence_ids: vec![f.evidence_id],
                work_kind: WorkKind::LocalChangeProposal,
                prerequisite_obligations: Vec::new(),
            })
            .collect()
    }

    fn analyze_protocol(
        &self,
        facts: &[ProtocolFact],
        scope: &WorkScope,
        id_allocator: &mut dyn FnMut() -> EntityId,
    ) -> Vec<SelfImprovementProposal> {
        facts
            .iter()
            .filter(|f| !f.has_versioning || !f.has_capability_negotiation)
            .map(|f| {
                let (problem, solution, impact, authority) = if !f.has_versioning {
                    (
                        format!("No message versioning for {}", f.feature),
                        format!("Add version field to {} messages", f.feature),
                        ImpactLevel::Major,
                        AuthorityLevel::ExternalReview,
                    )
                } else {
                    (
                        format!("No capability negotiation for {}", f.feature),
                        format!("Add capability exchange to {}", f.feature),
                        ImpactLevel::Major,
                        AuthorityLevel::ExternalReview,
                    )
                };
                SelfImprovementProposal {
                    proposal_id: id_allocator(),
                work_scope: *scope,
                    target_area: TargetArea::Protocol,
                    problem,
                    solution,
                    confidence: 0.75,
                    impact,
                    required_authority: authority,
                    evidence_ids: vec![f.evidence_id],
                    work_kind: WorkKind::LocalChangeProposal,
                    prerequisite_obligations: Vec::new(),
                }
            })
            .collect()
    }

    fn analyze_security(
        &self,
        facts: &[SecurityFact],
        scope: &WorkScope,
        id_allocator: &mut dyn FnMut() -> EntityId,
    ) -> Vec<SelfImprovementProposal> {
        facts
            .iter()
            .filter(|f| !f.is_mitigated)
            .map(|f| SelfImprovementProposal {
                proposal_id: id_allocator(),
                work_scope: *scope,
                target_area: TargetArea::Security,
                problem: format!("Unmitigated security concern: {}", f.concern),
                solution: format!("Implement mitigation for: {}", f.concern),
                confidence: 0.80,
                impact: ImpactLevel::Architectural,
                required_authority: AuthorityLevel::ExternalReview,
                evidence_ids: vec![f.evidence_id],
                work_kind: WorkKind::LocalChangeProposal,
                prerequisite_obligations: Vec::new(),
            })
            .collect()
    }

    /// Classify proposals by the authority they require.
    ///
    /// **No proposal is ever executed by this method.**
    /// The returned classification is a pure data structure
    /// that a coordinator uses to route work to the appropriate
    /// approval channel.
    ///
    /// # Invariants
    /// - A proposal with `AuthorityLevel::Automatic` must have
    ///   `work_kind` that produces no side effects.
    /// - No proposal with `confidence >= threshold` is promoted
    ///   to a higher authority class.
    pub fn classify_proposals(
        &self,
        proposals: Vec<SelfImprovementProposal>,
    ) -> ProposalClassification {
        let mut automatic = Vec::new();
        let mut owner_approval = Vec::new();
        let mut external_review = Vec::new();

        for proposal in proposals {
            match proposal.required_authority {
                AuthorityLevel::Automatic => automatic.push(proposal),
                AuthorityLevel::OwnerApproval => owner_approval.push(proposal),
                AuthorityLevel::ExternalReview => external_review.push(proposal),
            }
        }

        ProposalClassification {
            automatic,
            owner_approval,
            external_review,
        }
    }

    /// Validate that a set of proposals does not violate authority rules.
    ///
    /// Returns the set of proposal IDs that violate the rule
    /// "confidence never creates authority". A proposal violates
    /// this rule if it claims `AuthorityLevel::Automatic` but
    /// its `work_kind` would produce side effects.
    pub fn validate_authority(&self, proposals: &[SelfImprovementProposal]) -> BTreeSet<EntityId> {
        let mut violations = BTreeSet::new();
        for proposal in proposals {
            if proposal.required_authority == AuthorityLevel::Automatic {
                // Only read-only work kinds may be automatic.
                match proposal.work_kind {
                    WorkKind::Summary | WorkKind::DeterministicCheck => {}
                    _ => {
                        violations.insert(proposal.proposal_id);
                    }
                }
            }
        }
        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::EntityId;

    fn test_id(seed: u128) -> EntityId {
        let base = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f_u128;
        EntityId::from_u128(base + seed).expect("valid UUIDv7 test fixture")
    }

    fn test_scope() -> WorkScope {
        WorkScope {
            session_id: test_id(1),
            work_id: test_id(2),
            objective_id: test_id(3),
            objective_revision: crate::ids::UnitRevision::MIN,
        }
    }

    fn test_evidence_id() -> EntityId {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1000);
        test_id(COUNTER.fetch_add(1, Ordering::SeqCst) as u128)
    }

    fn fresh_proposal_id() -> EntityId {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(500_000);
        test_id(COUNTER.fetch_add(1, Ordering::SeqCst) as u128)
    }

    #[test]
    fn gate_generates_proposals_from_facts_not_hardcoded() {
        let gate = SelfDevelopmentGate::default();
        let scope = test_scope();

        let facts = CodebaseFacts {
            test_coverage: vec![
                TestCoverageFact {
                    area: "dc-desktop commands".to_string(),
                    has_integration_tests: false,
                    has_property_tests: false,
                    evidence_id: test_evidence_id(),
                },
                TestCoverageFact {
                    area: "protocol serialization".to_string(),
                    has_integration_tests: true,
                    has_property_tests: false,
                    evidence_id: test_evidence_id(),
                },
            ],
            documentation: vec![DocumentationFact {
                topic: "AGGG4ks integration patterns".to_string(),
                is_linked_from_index: false,
                evidence_id: test_evidence_id(),
            }],
            protocol: vec![
                ProtocolFact {
                    feature: "DeepCreateCommand".to_string(),
                    has_versioning: false,
                    has_capability_negotiation: false,
                    evidence_id: test_evidence_id(),
                },
                ProtocolFact {
                    feature: "handshake".to_string(),
                    has_versioning: true,
                    has_capability_negotiation: false,
                    evidence_id: test_evidence_id(),
                },
            ],
            security: vec![SecurityFact {
                concern: "unauthenticated owner events".to_string(),
                is_mitigated: false,
                evidence_id: test_evidence_id(),
            }],
        };

        let proposals = gate.analyze(&facts, &scope, &mut fresh_proposal_id);

        // All facts with gaps should produce proposals.
        assert!(!proposals.is_empty());
        assert!(proposals.len() <= gate.max_proposals);

        // Verify no proposal is auto-apply for code changes.
        for proposal in &proposals {
            if proposal.work_kind == WorkKind::LocalChangeProposal {
                assert_ne!(
                    proposal.required_authority,
                    AuthorityLevel::Automatic,
                    "Code changes must never be automatic"
                );
            }
        }

        // Verify evidence traceability.
        for proposal in &proposals {
            assert!(
                !proposal.evidence_ids.is_empty(),
                "Every proposal must cite evidence"
            );
        }
    }

    #[test]
    fn empty_facts_produce_no_proposals() {
        let gate = SelfDevelopmentGate::default();
        let scope = test_scope();

        let facts = CodebaseFacts {
            test_coverage: vec![],
            documentation: vec![],
            protocol: vec![],
            security: vec![],
        };

        let proposals = gate.analyze(&facts, &scope, &mut fresh_proposal_id);
        assert!(proposals.is_empty());
    }

    #[test]
    fn all_satisfied_facts_produce_no_proposals() {
        let gate = SelfDevelopmentGate::default();
        let scope = test_scope();

        let facts = CodebaseFacts {
            test_coverage: vec![TestCoverageFact {
                area: "all covered".to_string(),
                has_integration_tests: true,
                has_property_tests: true,
                evidence_id: test_evidence_id(),
            }],
            documentation: vec![DocumentationFact {
                topic: "linked topic".to_string(),
                is_linked_from_index: true,
                evidence_id: test_evidence_id(),
            }],
            protocol: vec![ProtocolFact {
                feature: "versioned feature".to_string(),
                has_versioning: true,
                has_capability_negotiation: true,
                evidence_id: test_evidence_id(),
            }],
            security: vec![SecurityFact {
                concern: "mitigated concern".to_string(),
                is_mitigated: true,
                evidence_id: test_evidence_id(),
            }],
        };

        let proposals = gate.analyze(&facts, &scope, &mut fresh_proposal_id);
        assert!(proposals.is_empty());
    }

    #[test]
    fn classification_groups_by_authority() {
        let gate = SelfDevelopmentGate::default();
        let _scope = test_scope();

        let proposals = vec![
            SelfImprovementProposal {
                proposal_id: test_evidence_id(),
                work_scope: test_scope(),
                target_area: TargetArea::Documentation,
                problem: "Missing link".to_string(),
                solution: "Add link".to_string(),
                confidence: 0.96,
                impact: ImpactLevel::Minor,
                required_authority: AuthorityLevel::OwnerApproval,
                evidence_ids: vec![test_evidence_id()],
                work_kind: WorkKind::LocalChangeProposal,
                prerequisite_obligations: Vec::new(),
            },
            SelfImprovementProposal {
                proposal_id: test_evidence_id(),
                work_scope: test_scope(),
                target_area: TargetArea::Protocol,
                problem: "Major change".to_string(),
                solution: "Complex fix".to_string(),
                confidence: 0.80,
                impact: ImpactLevel::Major,
                required_authority: AuthorityLevel::ExternalReview,
                evidence_ids: vec![test_evidence_id()],
                work_kind: WorkKind::LocalChangeProposal,
                prerequisite_obligations: Vec::new(),
            },
        ];

        let classification = gate.classify_proposals(proposals);
        assert_eq!(classification.automatic.len(), 0);
        assert_eq!(classification.owner_approval.len(), 1);
        assert_eq!(classification.external_review.len(), 1);
    }

    #[test]
    fn validate_authority_rejects_automatic_code_changes() {
        let gate = SelfDevelopmentGate::default();

        let proposals = vec![
            // This is invalid: automatic + code change.
            SelfImprovementProposal {
                proposal_id: test_evidence_id(),
                work_scope: test_scope(),
                target_area: TargetArea::TestCoverage,
                problem: "Missing tests".to_string(),
                solution: "Add tests".to_string(),
                confidence: 0.99,
                impact: ImpactLevel::Minor,
                required_authority: AuthorityLevel::Automatic,
                evidence_ids: vec![test_evidence_id()],
                work_kind: WorkKind::LocalChangeProposal,
                prerequisite_obligations: Vec::new(),
            },
            // This is valid: automatic + read-only.
            SelfImprovementProposal {
                proposal_id: test_evidence_id(),
                work_scope: test_scope(),
                target_area: TargetArea::Documentation,
                problem: "Missing summary".to_string(),
                solution: "Generate summary".to_string(),
                confidence: 0.95,
                impact: ImpactLevel::Minor,
                required_authority: AuthorityLevel::Automatic,
                evidence_ids: vec![test_evidence_id()],
                work_kind: WorkKind::Summary,
                prerequisite_obligations: Vec::new(),
            },
        ];

        let violations = gate.validate_authority(&proposals);
        assert_eq!(violations.len(), 1);
        assert!(violations.contains(&proposals[0].proposal_id));
        assert!(!violations.contains(&proposals[1].proposal_id));
    }

    #[test]
    fn high_confidence_does_not_grant_authority() {
        let gate = SelfDevelopmentGate::default();
        let scope = test_scope();

        let facts = CodebaseFacts {
            test_coverage: vec![],
            documentation: vec![],
            protocol: vec![ProtocolFact {
                feature: "critical feature".to_string(),
                has_versioning: false,
                has_capability_negotiation: false,
                evidence_id: test_evidence_id(),
            }],
            security: vec![],
        };

        let proposals = gate.analyze(&facts, &scope, &mut fresh_proposal_id);
        let classification = gate.classify_proposals(proposals);

        // Even with high confidence, protocol changes go to external review.
        assert_eq!(classification.automatic.len(), 0);
        assert!(classification.external_review.len() > 0);
    }

    #[test]
    fn min_confidence_filters_low_quality_proposals() {
        let gate = SelfDevelopmentGate {
            min_confidence: 0.90,
            max_proposals: 10,
        };
        let scope = test_scope();

        let facts = CodebaseFacts {
            test_coverage: vec![TestCoverageFact {
                area: "low confidence area".to_string(),
                has_integration_tests: false,
                has_property_tests: false,
                evidence_id: test_evidence_id(),
            }],
            documentation: vec![],
            protocol: vec![],
            security: vec![],
        };

        let proposals = gate.analyze(&facts, &scope, &mut fresh_proposal_id);
        // Test coverage proposals have confidence 0.85, below the 0.90 threshold.
        assert!(proposals.is_empty());
    }
}
