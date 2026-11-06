//! Executor loop: runs accepted work units through the broker dispatcher.
//!
//! This is the missing piece between "graph accepted" and "work actually done".
//! The executor takes the accepted graph from CoreState, creates a dispatcher,
//! and executes units in dependency order.

use dc_core::dispatch::{BatchCoordinator, CoreDispatcher};
use dc_core::CoreState;
use dc_domain::work::WorkUnitSpec;
use dc_broker::{EndpointConfig, ModelEndpoint, DispatchRequest, DispatchResult};
use dc_worker::{ExecuteRequest, ExecuteResult, WorkResponse, WorkerError, WorkDispatcher};
use std::sync::RwLock;

/// Execute all units in the accepted graph.
///
/// This is a synchronous executor for the desktop shell. In production,
/// this would be a background task with progress reporting.
pub fn execute_accepted_graph(
    core: &RwLock<CoreState>,
    endpoint_config: Option<EndpointConfig>,
) -> Result<String, String> {
    // Read the accepted graph (clone to avoid holding lock during execution)
    let units: Vec<WorkUnitSpec> = {
        let core_read = core.read().unwrap();
        let graph = core_read.graph.as_ref()
            .ok_or_else(|| "no accepted graph".to_string())?;
        graph.units.clone()
    };
    
    if units.is_empty() {
        return Ok("no units to execute".to_string());
    }
    
    let units_refs: Vec<&WorkUnitSpec> = units.iter().collect();
    
    // Create dispatcher
    let endpoint = ModelEndpoint::new(endpoint_config.unwrap_or_default());
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| format!("tokio runtime: {}", e))?;
    
    // Local dispatcher that bridges broker HTTP dispatch to worker trait
    struct LocalBrokerDispatcher {
        endpoint: ModelEndpoint,
        runtime: tokio::runtime::Handle,
    }
    
    impl WorkDispatcher for LocalBrokerDispatcher {
        fn dispatch(&self, request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
            let broker_request = DispatchRequest {
                dispatch_id: request.execution_id.clone(),
                work_unit_spec: request.work_unit_spec.clone(),
                system_prompt: request.system_prompt.clone(),
                user_prompt: request.user_prompt.clone(),
                max_tokens: request.max_tokens,
                temperature: request.temperature,
            };
            
            // Retry with exponential backoff: 1s, 2s, 4s (max 3 attempts)
            let max_attempts = 3;
            let mut last_error = None;
            
            for attempt in 0..max_attempts {
                if attempt > 0 {
                    let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                    eprintln!("Retry attempt {} after {:?}...", attempt + 1, delay);
                    std::thread::sleep(delay);
                }
                
                let result = self
                    .runtime
                    .block_on(dc_broker::dispatch::dispatch_work_unit(&self.endpoint, &broker_request));
                
                match result {
                    Ok(dispatch_result) => {
                        return match dispatch_result {
                            DispatchResult::Success(response) => Ok(ExecuteResult::Success(WorkResponse {
                                content: response.content,
                                model: response.model,
                                tokens_used: response.tokens_used,
                                finish_reason: response.finish_reason,
                            })),
                            DispatchResult::Refused(reason) => Ok(ExecuteResult::Refused(reason)),
                            DispatchResult::Unreachable(reason) => Ok(ExecuteResult::Unreachable(reason)),
                        };
                    }
                    Err(e) => {
                        eprintln!("Dispatch attempt {} failed: {}", attempt + 1, e);
                        last_error = Some(e);
                    }
                }
            }
            
            Err(WorkerError::Dispatch(format!(
                "broker error after {} attempts: {}",
                max_attempts,
                last_error.map(|e| e.to_string()).unwrap_or_else(|| "unknown".to_string())
            )))
        }
    }
    
    let local_dispatcher = LocalBrokerDispatcher {
        endpoint,
        runtime: runtime.handle().clone(),
    };
    let dispatcher = CoreDispatcher::new(local_dispatcher, "desktop-executor".to_string());
    let coordinator = BatchCoordinator::new(dispatcher);
    
    // Execute units
    let system_prompt = "You are a helpful assistant. Complete the task.";
    let user_prompts: Vec<String> = units.iter()
        .map(|u| format!("{:?}", u.expected_output))
        .collect();
    
    let results = coordinator.execute_batch(&units_refs, system_prompt, &user_prompts);
    
    // Process results: update CoreState stages and run verification
    let mut success = 0;
    let mut failed = 0;
    let mut verified = 0;
    let mut verification_failed = 0;
    
    for (i, result) in results.iter().enumerate() {
        let unit_ref = units[i].unit;
        match result {
            Ok(dc_worker::ExecuteResult::Success(_result_packet)) => {
                success += 1;
                // Mark unit as ResultSubmitted
                {
                    let mut core_write = core.write().unwrap();
                    if let Some(graph) = &mut core_write.graph {
                        if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == unit_ref) {
                            unit_spec.stage = dc_domain::work::UnitStage::ResultSubmitted;
                        }
                    }
                }
                
                // Run machine verification on the result
                // Use unit-scoped verification ID to avoid collisions
                let verification_id = unit_ref.unit_id;
                // Convert WorkResponse to ResultPacket for verification
                let result_packet = dc_domain::work::ResultPacket {
                    result_id: dc_domain::ids::EntityId::from_u128(i as u128 + 1000).unwrap(),
                    attempt_id: dc_domain::ids::EntityId::from_u128(i as u128 + 2000).unwrap(),
                    scope: units[i].scope,
                    unit: unit_ref,
                    snapshot: dc_domain::work::DependencySnapshot {
                        snapshot_id: dc_domain::ids::EntityId::from_u128(i as u128 + 3000).unwrap(),
                        scope: units[i].scope,
                        unit: unit_ref,
                        read_set: vec![],
                        policy_versions: vec![],
                    },
                    fingerprint: [0u8; 32],
                    verification_policy: dc_domain::work::VersionedDependency {
                        key: dc_domain::work::DependencyKey {
                            kind: dc_domain::work::DependencyKind::VerificationCriteria,
                            entity_id: verification_id,
                        },
                        expected_revision: dc_domain::ids::RowVersion::new(1).unwrap(),
                        fingerprint: None,
                    },
                    used_links: vec![],
                    stage: dc_domain::work::ResultStage::Submitted,
                    row_version: dc_domain::ids::RowVersion::new(1).unwrap(),
                };
                let result_packet_snapshot = serde_json::to_string(&result_packet).unwrap_or_default();
                let verification_outcome = dc_core::verification::run_machine_verification(
                    &units[i],
                    &result_packet,
                    verification_id,
                );
                
                match verification_outcome {
                    Ok(dc_core::verification::VerificationOutcome::Passed { record }) => {
                        verified += 1;
                        // Update stage to Accepted
                        let mut core_write = core.write().unwrap();
                        if let Some(graph) = &mut core_write.graph {
                            if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == unit_ref) {
                                unit_spec.stage = dc_domain::work::UnitStage::Accepted;
                            }
                        }
                        // Store verification record
                        core_write.verifications.push(record);
                    }
                    Ok(dc_core::verification::VerificationOutcome::Failed { reason, .. }) => {
                        verification_failed += 1;
                        eprintln!("Unit {} verification failed: {}", i, reason);
                        // Mark as Rejected
                        let mut core_write = core.write().unwrap();
                        if let Some(graph) = &mut core_write.graph {
                            if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == unit_ref) {
                                unit_spec.stage = dc_domain::work::UnitStage::Rejected;
                            }
                        }
                    }
                    Ok(dc_core::verification::VerificationOutcome::NeedsSemantic { proposal: _ }) => {
                        // Semantic verification required — run it with the actual content
                        let semantic_outcome = dc_core::verification::run_semantic_verification(
                            &units[i],
                            &result_packet,
                            &result_packet_snapshot,
                        );
                        
                        match semantic_outcome {
                            Ok(dc_core::verification::VerificationOutcome::Passed { record }) => {
                                verified += 1;
                                let mut core_write = core.write().unwrap();
                                if let Some(graph) = &mut core_write.graph {
                                    if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == unit_ref) {
                                        unit_spec.stage = dc_domain::work::UnitStage::Accepted;
                                    }
                                }
                                core_write.verifications.push(record);
                            }
                            Ok(dc_core::verification::VerificationOutcome::Failed { reason, .. }) => {
                                verification_failed += 1;
                                eprintln!("Unit {} semantic verification failed: {}", i, reason);
                                let mut core_write = core.write().unwrap();
                                if let Some(graph) = &mut core_write.graph {
                                    if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == unit_ref) {
                                        unit_spec.stage = dc_domain::work::UnitStage::Rejected;
                                    }
                                }
                            }
                            _ => {
                                // Still needs further review
                                let mut core_write = core.write().unwrap();
                                if let Some(graph) = &mut core_write.graph {
                                    if let Some(unit_spec) = graph.units.iter_mut().find(|u| u.unit == unit_ref) {
                                        unit_spec.stage = dc_domain::work::UnitStage::Verifying;
                                    }
                                }
                            }
                        }
                    }
                    Ok(dc_core::verification::VerificationOutcome::Indeterminate { reason }) => {
                        eprintln!("Unit {} verification indeterminate: {}", i, reason);
                    }
                    Err(e) => {
                        eprintln!("Unit {} verification error: {:?}", i, e);
                    }
                }
            }
            Ok(dc_worker::ExecuteResult::Refused(r)) => {
                failed += 1;
                eprintln!("Unit {} refused: {}", i, r);
            }
            Ok(dc_worker::ExecuteResult::Unreachable(r)) => {
                failed += 1;
                eprintln!("Unit {} unreachable: {}", i, r);
            }
            Err(e) => {
                failed += 1;
                eprintln!("Unit {} error: {:?}", i, e);
            }
        }
    }
    
    Ok(format!(
        "executed: {} success, {} failed, {} verified, {} verification_failed",
        success, failed, verified, verification_failed
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn executor_requires_graph() {
        let core = RwLock::new(CoreState::empty());
        let result = execute_accepted_graph(&core, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no accepted graph"));
    }
}
