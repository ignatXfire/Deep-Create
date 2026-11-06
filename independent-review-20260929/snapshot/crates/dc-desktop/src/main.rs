#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::path::PathBuf;

pub mod store_source;
pub mod executor;

// Re-export core types for the UI write path.
use dc_core::command::{CoreCommand, handle_command};
use dc_core::CoreState;
use dc_domain::work::ObjectiveContract;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
enum BuildStage {
    EmptyShell,
    WorkGraph,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplicationInfo {
    version: &'static str,
    build_stage: BuildStage,
    work_units_count: usize,
}

/// Live-state projection: a single work unit as the UI sees it.
///
/// This is a READ MODEL — it is derived from authoritative state (dc-core /
/// dc-store) and never mutated from the UI. The UI can only observe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkUnitView {
    pub id: String,
    pub kind: String,
    pub stage: String,
    pub summary: String,
    pub verdict: Option<String>,
    pub artifacts: Vec<String>,
}

/// Source of live state for the desktop projection.
///
/// Production: backed by dc-store via dc-core queries.
/// Development: backed by `FixtureStateSource` until the store bridge lands.
///
/// The trait is object-safe so the Tauri command layer can hold
/// `Arc<dyn LiveStateSource>` without knowing the concrete backend.
pub trait LiveStateSource: Send + Sync {
    fn work_units(&self) -> Vec<WorkUnitView>;
    fn work_unit_count(&self) -> usize {
        self.work_units().len()
    }
}

/// Core-backed live state source: reads from `CoreState.graph` when available,
/// falls back to fixture only when no graph is accepted.
/// 
/// This replaces the pure fixture approach with real state projection.
pub struct CoreStateSource<'a> {
    core: &'a std::sync::RwLock<dc_core::CoreState>,
}

impl<'a> CoreStateSource<'a> {
    pub fn new(core: &'a std::sync::RwLock<dc_core::CoreState>) -> Self {
        Self { core }
    }
}

impl<'a> LiveStateSource for CoreStateSource<'a> {
    fn work_units(&self) -> Vec<WorkUnitView> {
        let core = self.core.read().unwrap();
        if let Some(graph) = &core.graph {
            return graph.units.iter().map(|u| {
                let verification = core.verifications.iter()
                    .find(|v| v.unit == u.unit);
                let verdict = verification.map(|v| format!("{:?}", v.verdict));
                let artifacts = verification
                    .map(|v| vec![format!("snapshot:{}", v.snapshot_id)])
                    .unwrap_or_default();
                WorkUnitView {
                    id: format!("{:?}", u.unit.unit_id),
                    kind: format!("{:?}", u.kind),
                    stage: format!("{:?}", u.stage),
                    summary: format!("{:?}", u.expected_output),
                    verdict,
                    artifacts,
                }
            }).collect();
        }
        // No graph yet — return empty (not fake data)
        Vec::new()
    }
}

/// Application state held by Tauri. Currently the fixture source; the type
/// is already generic over `LiveStateSource` so swapping backends is a
/// one-line change in `main`.
///
/// The UI write path needs mutable access to `CoreState` — the single
/// writer of the main database. The UI is a client, not the controller,
/// but it must send commands to the controller, not just read projections.
/// `RwLock` is required because `tauri::State` requires `Send + Sync`.
pub struct AppState {
    source: Box<dyn LiveStateSource>,
    /// The core controller state — single writer of the main database.
    /// UI commands mutate this through `handle_command`, never directly.
    pub core: std::sync::RwLock<CoreState>,
    /// Persistence layer — writes CoreState to dc-store after each command.
    /// None if store path not configured (development mode).
    pub persistence: Option<std::sync::Mutex<dc_core::persistence::CorePersistence<dc_store::SQLiteStore>>>,
}

impl AppState {
    /// Create a live state source backed by the current core state.
    /// This replaces the fixture with real data when available.
    pub fn live_source(&self) -> CoreStateSource<'_> {
        CoreStateSource::new(&self.core)
    }
}

/// C-14 / document 20 stage 1: the local version page is the only caller.
/// No inputs, state changes, files, clock, network, secrets or owner decisions.
/// This constant read does not create an event. ACL and response tests guard it.
#[tauri::command]
fn application_info(state: tauri::State<AppState>) -> ApplicationInfo {
    // Use live source (CoreState-backed) instead of fixture
    let live_source = state.live_source();
    ApplicationInfo {
        version: env!("CARGO_PKG_VERSION"),
        build_stage: BuildStage::WorkGraph,
        work_units_count: live_source.work_unit_count(),
    }
}

/// Read-only: list current work units as a live-state projection.
///
/// The data comes from the live `CoreStateSource`. When no graph is accepted,
/// returns empty — no fake fixture data.
#[tauri::command]
fn work_unit_ids(state: tauri::State<AppState>) -> Vec<String> {
    state.live_source().work_units().into_iter().map(|u| u.id).collect()
}

/// Read-only: full live-state projection for the work graph view.
///
/// Reads from CoreState.graph when a graph is accepted (real state),
/// otherwise returns empty — no fixture fallback.
#[tauri::command]
fn work_units(state: tauri::State<AppState>) -> Vec<WorkUnitView> {
    state.live_source().work_units()
}

/// UI write path: submit a new objective to dc-core.
///
/// This is the first WRITE command in the UI. It goes through the same
/// `handle_command` path as any other owner command — no shortcuts.
/// The UI is a client, not the controller.
#[tauri::command]
fn submit_objective(
    state: tauri::State<'_, AppState>,
    objective_json: String,
) -> Result<String, String> {
    let objective: ObjectiveContract = serde_json::from_str(&objective_json)
        .map_err(|e| format!("invalid objective JSON: {}", e))?;
    let outcome = handle_command(&mut state.core.write().unwrap(), CoreCommand::AcceptObjective(objective))
        .map_err(|e| format!("{:?}", e))?;
    Ok(format!("{:?}", outcome))
}

/// UI write path: start a work unit attempt.
#[tauri::command]
fn start_attempt(
    state: tauri::State<'_, AppState>,
    unit_id: String,
) -> Result<String, String> {
    let unit_ref = dc_domain::work::UnitRef {
        unit_id: dc_domain::ids::EntityId::from_u128(
            unit_id.parse::<u128>().map_err(|e| format!("invalid unit_id: {}", e))?
        ).ok_or_else(|| "invalid EntityId".to_string())?,
        revision: dc_domain::ids::UnitRevision::new(1).ok_or_else(|| "invalid UnitRevision".to_string())?,
    };
    let outcome = handle_command(&mut state.core.write().unwrap(), CoreCommand::StartAttempt(unit_ref))
        .map_err(|e| format!("{:?}", e))?;
    Ok(format!("{:?}", outcome))
}

/// UI bridge: derive a minimal `GraphCandidate` from an objective without
/// touching core state. The result still must be reviewed and accepted via
/// `submit_objective` + a future owner-acceptance step — this command is
/// read-only with respect to the store.
///
/// The returned JSON can be edited by the owner in the UI and then fed back
/// through `accept_graph` (future command) once the owner approves.
#[tauri::command]
fn plan_objective_graph(objective_json: String) -> Result<String, String> {
    let objective: ObjectiveContract = serde_json::from_str(&objective_json)
        .map_err(|e| format!("invalid objective JSON: {}", e))?;
    let graph = dc_core::planner::plan_graph_from_objective(&objective)
        .map_err(|e| format!("{:?}", e))?;
    serde_json::to_string_pretty(&graph)
        .map_err(|e| format!("graph serialization failed: {}", e))
}

/// UI bridge: accept a previously planned graph (owner review step).
/// C-08: The UI passes the graph as raw JSON — the core validates and
/// deserializes it internally, preserving the invariant that GraphCandidate
/// is only constructed by the core.
#[tauri::command]
fn accept_graph(state: tauri::State<'_, AppState>, graph_json: String) -> Result<String, String> {
    // C-08: We do NOT deserialize GraphCandidate here. We pass the raw JSON
    // to the core, which validates it and constructs the graph internally.
    let outcome = handle_command(&mut state.core.write().unwrap(), CoreCommand::AcceptGraphRaw(graph_json))
        .map_err(|e| format!("{:?}", e))?;
    Ok(format!("{:?}", outcome))
}

/// UI bridge: execute the accepted graph through the broker dispatcher.
#[tauri::command]
fn execute_graph(state: tauri::State<'_, AppState>) -> Result<String, String> {
    crate::executor::execute_accepted_graph(&state.core, None)
}

/// Health check endpoint for monitoring.
/// Returns system status: version, uptime, unit counts, endpoint status.
#[tauri::command]
fn health_check(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let core = state.core.read().unwrap();
    let units_count = core.graph.as_ref().map(|g| g.units.len()).unwrap_or(0);
    let verifications_count = core.verifications.len();
    
    // Check if persistence is configured
    let persistence_status = if state.persistence.is_some() { "enabled" } else { "disabled" };
    
    // Check broker endpoint reachability
    let endpoint_status = if let Some(ref _persistence) = state.persistence {
        // In production, would check actual endpoint
        "unknown"
    } else {
        "not_configured"
    };
    
    let health = serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "build_stage": "WorkGraph",
        "units": units_count,
        "verifications": verifications_count,
        "persistence": persistence_status,
        "broker_endpoint": endpoint_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    serde_json::to_string_pretty(&health)
        .map_err(|e| format!("health check serialization failed: {}", e))
}

fn is_bundled_page(url: &tauri::Url) -> bool {
    let local_origin = (url.scheme() == "tauri" && url.host_str() == Some("localhost"))
        || (url.scheme() == "http" && url.host_str() == Some("tauri.localhost"));
    local_origin
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && matches!(url.path(), "/" | "/index.html")
        && url.query().is_none()
}

use crate::store_source::StoreStateSource;

/// Empty live state source for when no graph is accepted.
/// Returns empty work units — no fake fixture data.
pub struct EmptyStateSource;

impl LiveStateSource for EmptyStateSource {
    fn work_units(&self) -> Vec<WorkUnitView> {
        Vec::new()
    }
}

fn desktop_builder<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    // Production: use the real store when a store path is provided via env.
    // Development: use empty core state (no fixture).
    let store_path = std::env::var("DC_STORE_PATH").ok().map(PathBuf::from);
    let source: Box<dyn LiveStateSource> = match store_path {
        Some(ref path) => Box::new(StoreStateSource::new(path.clone())),
        None => Box::new(EmptyStateSource),
    };
    
    // Initialize persistence when store path is available
    let persistence = store_path.map(|path| {
        let store = dc_store::SQLiteStore::new(path.to_str().unwrap_or(""));
        let mut core_persistence = dc_core::persistence::CorePersistence::new(store);
        // Initialize the store (creates tables if needed)
        let _ = core_persistence.initialize(&path);
        std::sync::Mutex::new(core_persistence)
    });

    // Recover state from persistence if available
    let initial_core = {
        let mut core = CoreState::empty();
        if let Some(pers) = &persistence {
            let pers = pers.lock().unwrap();
            // Try to load persisted state metadata
            // Use a fixed non-zero ID for the singleton app state
            let state_id = dc_domain::ids::EntityId::from_u128(1).unwrap();
            if let Ok(Some(stored)) = pers.load_state(&state_id) {
                eprintln!(
                    "Recovered persisted state: objective={:?}, graph_version={}, active_units={}, has_objective_json={}, has_graph_json={}",
                    stored.data.current_objective.is_some(),
                    stored.data.current_graph_version,
                    stored.data.active_units.len(),
                    stored.data.objective_json.is_some(),
                    stored.data.graph_json.is_some()
                );
                
                // Restore objective first (required for AcceptGraphRaw)
                if let Some(objective_json) = &stored.data.objective_json {
                    match serde_json::from_str::<dc_domain::work::ObjectiveContract>(objective_json) {
                        Ok(objective) => {
                            if let Err(e) = dc_core::command::handle_command(
                                &mut core,
                                dc_core::command::CoreCommand::AcceptObjective(objective),
                            ) {
                                eprintln!("Failed to restore objective: {}", e);
                            } else {
                                eprintln!("Objective restored successfully");
                            }
                        }
                        Err(e) => eprintln!("Failed to deserialize objective: {}", e),
                    }
                }
                
                // Restore full graph from serialized JSON (C-08: core deserializes via AcceptGraphRaw)
                if let Some(graph_json) = &stored.data.graph_json {
                    if let Err(e) = dc_core::command::handle_command(
                        &mut core,
                        dc_core::command::CoreCommand::AcceptGraphRaw(graph_json.clone()),
                    ) {
                        eprintln!("Failed to restore graph from persistence: {}", e);
                    } else {
                        eprintln!("Graph restored successfully from persistence");
                    }
                }
            }
        }
        core
    };

    builder
        .manage(AppState {
            source,
            core: std::sync::RwLock::new(initial_core),
            persistence,
        })
        .invoke_handler(tauri::generate_handler![
            application_info,
            work_unit_ids,
            work_units,
            submit_objective,
            start_attempt,
            plan_objective_graph,
            accept_graph,
            execute_graph,
            health_check,
        ])
        .setup(|app| {
            let config = app
                .config()
                .app
                .windows
                .first()
                .ok_or("Missing main window configuration")?;
            tauri::WebviewWindowBuilder::from_config(app, config)?
                .on_navigation(is_bundled_page)
                .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                .on_download(|_, _| false)
                .build()?;
            Ok(())
        })
}

fn main() {
    desktop_builder(tauri::Builder::default())
        .run(tauri::generate_context!())
        .expect("Deep Create could not start its version page");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};

    fn request(command: &str, url: &str) -> tauri::webview::InvokeRequest {
        tauri::webview::InvokeRequest {
            cmd: command.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: url.parse().expect("test URL"),
            body: tauri::ipc::InvokeBody::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.into(),
        }
    }

    fn test_window(label: &str) -> (tauri::App<MockRuntime>, tauri::WebviewWindow<MockRuntime>) {
        let app = desktop_builder(mock_builder())
            .build(tauri::generate_context!())
            .expect("test application");
        let window = tauri::WebviewWindowBuilder::new(&app, label, Default::default())
            .build()
            .expect("test webview");
        (app, window)
    }

    fn assert_acl_denied(window: &tauri::WebviewWindow<MockRuntime>, command: &str, url: &str) {
        let error = match get_ipc_response(window, request(command, url)) {
            Err(error) => error,
            Ok(_) => panic!("command must be denied"),
        };
        assert!(
            error
                .as_str()
                .is_some_and(|message| message.contains("not allowed")),
            "expected an ACL refusal for {command}, not a handler or argument error: {error}"
        );
    }

    #[test]
    fn metadata_has_only_closed_build_fields() {
        let expected = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "buildStage": "work_graph",
            "workUnitsCount": 0, // No fixture — empty until real graph is accepted
        });
        let (_app, window) = test_window("main");
        let response = get_ipc_response(
            &window,
            request("application_info", "http://tauri.localhost"),
        )
        .expect("application_info response");
        assert_eq!(
            response
                .deserialize::<serde_json::Value>()
                .expect("metadata JSON"),
            expected
        );
    }

    #[test]
    fn main_local_window_can_read_version() {
        let (_app, window) = test_window("main");
        let response = get_ipc_response(
            &window,
            request("application_info", "http://tauri.localhost"),
        )
        .expect("only application_info is allowed");
        let value = response
            .deserialize::<serde_json::Value>()
            .expect("metadata JSON");
        assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["buildStage"], "work_graph");
        assert_eq!(value["workUnitsCount"], 0); // No fixture — empty until real graph
    }

    #[test]
    fn main_local_window_can_read_work_units() {
        let (_app, window) = test_window("main");
        let response = get_ipc_response(
            &window,
            request("work_unit_ids", "http://tauri.localhost"),
        )
        .expect("work_unit_ids is allowed");
        let ids: Vec<String> = response
            .deserialize()
            .expect("work unit IDs JSON");
        // No fixture — empty until real graph is accepted
        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn window_without_capability_is_denied_even_for_registered_app_command() {
        let (_app, window) = test_window("unprivileged-test-window");
        assert_acl_denied(&window, "application_info", "http://tauri.localhost");
    }

    #[test]
    fn remote_origin_and_ungranted_commands_are_denied() {
        let (_app, window) = test_window("main");
        assert_acl_denied(&window, "application_info", "https://example.invalid");
        for command in [
            "plugin:app|version",
            "plugin:path|resolve_directory",
            "plugin:fs|read_file",
            "plugin:shell|execute",
            "plugin:http|fetch",
            "plugin:sql|load",
            "emit_owner_decision",
            "unknown_command",
        ] {
            assert_acl_denied(&window, command, "http://tauri.localhost");
        }
    }

    #[test]
    fn navigation_is_only_to_the_embedded_version_page() {
        for url in [
            "tauri://localhost/index.html",
            "http://tauri.localhost/",
            "http://tauri.localhost/#version",
        ] {
            assert!(is_bundled_page(&url.parse().expect("test URL")), "{url}");
        }
        for url in [
            "https://example.invalid",
            "http://127.0.0.1:1420/",
            "file:///C:/private.txt",
            "http://tauri.localhost.evil/index.html",
            "http://tauri.localhost:81/",
            "http://author@example.invalid/",
            "http://tauri.localhost/other.html",
            "http://tauri.localhost/?external=true",
            "data:text/html,hello",
        ] {
            assert!(!is_bundled_page(&url.parse().expect("test URL")), "{url}");
        }
    }
}
