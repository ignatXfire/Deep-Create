//! C-07: Context compilation — build prompts for model execution.

use dc_domain::work::{WorkUnitSpec, WorkKind};
use dc_domain::permission::{CapabilityLease, DataClass, LeaseStatus};

use crate::error::CoreError;

/// Plan for context needs of a work unit
#[derive(Debug, Clone)]
pub struct ContextPlan {
    /// Memory scope allowed
    pub memory_scope: Vec<String>,
    /// Data classes that may be accessed
    pub allowed_data_classes: Vec<DataClass>,
    /// Maximum context size
    pub max_context_tokens: usize,
}

/// Compiled prompt bundle ready for model execution
#[derive(Debug, Clone)]
pub struct PromptBundle {
    /// System instructions
    pub system_prompt: String,
    /// User/owner instructions
    pub user_prompt: String,
    /// Context fragments (evidence summaries)
    pub context_fragments: Vec<String>,
    /// Tool definitions available
    pub tools: Vec<ToolDefinition>,
    /// Output schema expected
    pub output_schema: serde_json::Value,
    /// Maximum response tokens
    pub max_response_tokens: usize,
    /// Data classification of this prompt
    pub data_class: DataClass,
}

/// Definition of a tool available to the model
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
}

/// Compile context needs for a work unit
pub fn plan_context_needs(
    unit: &WorkUnitSpec,
    memory_scope: &[String],
) -> Result<ContextPlan, CoreError> {
    // Determine allowed data classes based on work kind
    let allowed_data_classes = match unit.kind {
        WorkKind::Research | WorkKind::CreativeExploration => {
            vec![DataClass::Public, DataClass::Local]
        }
        WorkKind::Planning | WorkKind::Synthesis => {
            vec![DataClass::Public, DataClass::Local, DataClass::Personal]
        }
        _ => vec![DataClass::Public],
    };
    
    // Secret data class is NEVER allowed in model context (constitution rule 5)
    if allowed_data_classes.contains(&DataClass::Secret) {
        return Err(CoreError::MissingPermission(
            "secret data class cannot be in model context".into()
        ));
    }
    
    Ok(ContextPlan {
        memory_scope: memory_scope.to_vec(),
        allowed_data_classes,
        max_context_tokens: 8192,
    })
}

/// Compile a complete prompt bundle for a work unit
pub fn compile_prompt_bundle(
    unit: &WorkUnitSpec,
    plan: &ContextPlan,
    lease: &CapabilityLease,
) -> Result<PromptBundle, CoreError> {
    // Validate lease is active
    if lease.status != LeaseStatus::Active {
        return Err(CoreError::MissingPermission("lease expired".into()));
    }
    
    // Build system prompt from unit
    let system_prompt = format!(
        "You are executing work unit: {:?}\n\n\
        Kind: {:?}\n\
        You must produce output matching the required schema.\n\
        Do not request access to data outside your scope.\n\
        Do not attempt to modify system state directly.",
        unit.unit,
        unit.kind
    );
    
    // Build user prompt
    let user_prompt = format!(
        "Execute the work unit and return the result.\n\
        Expected output: {}",
        unit.expected_output.export_name
    );
    
    // Context fragments would be populated from evidence store
    let context_fragments = Vec::new();
    
    // Tools from contract
    let tools = Vec::new();
    
    // Determine data class (highest allowed)
    let data_class = plan.allowed_data_classes.iter()
        .max()
        .copied()
        .unwrap_or(DataClass::Public);
    
    Ok(PromptBundle {
        system_prompt,
        user_prompt,
        context_fragments,
        tools,
        output_schema: serde_json::json!({"type": "object"}),
        max_response_tokens: 4096,
        data_class,
    })
}

/// Parse a result packet from model response
pub fn parse_result_packet(
    response: &str,
    _expected_schema: &serde_json::Value,
) -> Result<dc_domain::work::ResultPacket, CoreError> {
    // Parse JSON response
    let value: serde_json::Value = serde_json::from_str(response)
        .map_err(|e| CoreError::InvalidCommand(format!("invalid JSON response: {}", e)))?;
    
    // Validate against schema (simplified)
    if !value.is_object() {
        return Err(CoreError::InvalidCommand("response must be object".into()));
    }
    
    // In real implementation, construct proper ResultPacket
    // For now, return error indicating this is not fully implemented
    Err(CoreError::InvalidCommand(
        "parse_result_packet not fully implemented — requires ResultPacket construction".into()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{test_id_n, active_lease};
    use dc_domain::ids::{EntityId, RowVersion, UnitRevision};
    use dc_domain::work::{WorkScope, WorkBounds, StopCondition, OutputContract, WorkUnitSpec, UnitRef, WorkKind};
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
            stage: dc_domain::work::UnitStage::Proposed,
            accepted_result: None,
            row_version: RowVersion::new(1).unwrap(),
        }
    }
    
    #[test]
    fn plan_context_needs_research() {
        let unit = test_unit();
        let scope = vec!["test".to_string()];
        
        let plan = plan_context_needs(&unit, &scope).unwrap();
        
        assert!(plan.allowed_data_classes.contains(&DataClass::Public));
        assert!(plan.allowed_data_classes.contains(&DataClass::Local));
    }
    
    #[test]
    fn compile_prompt_bundle_success() {
        let unit = test_unit();
        let scope = vec!["test".to_string()];
        let plan = plan_context_needs(&unit, &scope).unwrap();
        let lease = active_lease();
        
        let bundle = compile_prompt_bundle(&unit, &plan, &lease).unwrap();
        
        assert!(bundle.system_prompt.contains("Research"));
    }
    
    #[test]
    fn parse_result_packet_not_implemented() {
        let response = r#"{"status": "ok"}"#;
        let schema = serde_json::json!({"type": "object"});
        
        let result = parse_result_packet(response, &schema);
        
        // Should fail with "not fully implemented"
        assert!(result.is_err());
    }
}
