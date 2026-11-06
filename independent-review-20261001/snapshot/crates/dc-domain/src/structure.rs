//! Versioned, finite engineering ceilings, separate from owner resource limits.
//! V1 is fixed, not a permission or a model/user-adjustable execution setting.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralLimitsV1 {
    pub policy_version: u32,
    pub max_text_bytes: usize,
    pub max_collection_items: usize,
    pub max_roots: usize,
    pub max_export_name_bytes: usize,
    pub max_output_fields: usize,
    pub max_link_targets: usize,
    pub max_condition_atoms: usize,
    pub max_policy_versions: usize,
    pub max_snapshot_reads: usize,
    pub max_current_dependency_rows: usize,
    pub max_wait_edges: usize,
}

/// A size change requires a new reviewed policy version and boundary tests.
/// GraphLimits may impose tighter limits; it cannot lift these ceilings.
pub const STRUCTURAL_LIMITS_V1: StructuralLimitsV1 = StructuralLimitsV1 {
    policy_version: 1,
    max_text_bytes: 4096,
    max_collection_items: 4096,
    max_roots: 64,
    max_export_name_bytes: 64,
    max_output_fields: 64,
    max_link_targets: 64,
    max_condition_atoms: 32,
    max_policy_versions: 64,
    max_snapshot_reads: 4096,
    max_current_dependency_rows: 32768,
    max_wait_edges: 16384,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_v1_limits_do_not_drift_silently() {
        let p = STRUCTURAL_LIMITS_V1;
        assert_eq!(
            (
                p.policy_version,
                p.max_text_bytes,
                p.max_collection_items,
                p.max_roots
            ),
            (1, 4096, 4096, 64)
        );
        assert_eq!(
            (
                p.max_export_name_bytes,
                p.max_output_fields,
                p.max_link_targets,
                p.max_condition_atoms,
                p.max_policy_versions
            ),
            (64, 64, 64, 32, 64)
        );
        assert_eq!(
            (
                p.max_snapshot_reads,
                p.max_current_dependency_rows,
                p.max_wait_edges
            ),
            (4096, 32768, 16384)
        );
    }
}
