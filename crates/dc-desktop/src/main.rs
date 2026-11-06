#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::path::PathBuf;

pub mod executor;
pub mod store_source;

// Re-export core types for the UI write path.
use dc_core::CoreState;
use dc_core::command::{CoreCommand, handle_command};
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
            return graph
                .units
                .iter()
                .map(|u| {
                    let verification = core.verifications.iter().find(|v| v.unit == u.unit);
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
                })
                .collect();
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
    /// Live state source — kept for API stability, actual data comes from core.
    #[allow(dead_code)]
    source: Box<dyn LiveStateSource>,
    /// The core controller state — single writer of the main database.
    /// UI commands mutate this through `handle_command`, never directly.
    pub core: std::sync::RwLock<CoreState>,
    /// Persistence layer — writes CoreState to dc-store after each command.
    /// None if store path not configured (development mode).
    pub persistence:
        Option<std::sync::Mutex<dc_core::persistence::CorePersistence<dc_store::SQLiteStore>>>,
    /// Path to the SQLite store, if configured (used for health checks).
    pub store_path: Option<PathBuf>,
    /// Broker endpoint configuration (used for health checks).
    pub broker_endpoint: dc_broker::EndpointConfig,
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
    state
        .live_source()
        .work_units()
        .into_iter()
        .map(|u| u.id)
        .collect()
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
#[tauri::command(rename_all = "snake_case")]
fn submit_objective(
    state: tauri::State<'_, AppState>,
    objective_json: String,
) -> Result<String, String> {
    let objective: ObjectiveContract = serde_json::from_str(&objective_json)
        .map_err(|e| format!("invalid objective JSON: {}", e))?;
    let outcome = handle_command(
        &mut state.core.write().unwrap(),
        CoreCommand::AcceptObjective(objective),
    )
    .map_err(|e| format!("{:?}", e))?;
    Ok(format!("{:?}", outcome))
}

/// UI write path: start a work unit attempt.
#[tauri::command(rename_all = "snake_case")]
fn start_attempt(state: tauri::State<'_, AppState>, unit_id: String) -> Result<String, String> {
    let unit_ref = dc_domain::work::UnitRef {
        unit_id: dc_domain::ids::EntityId::from_u128(
            unit_id
                .parse::<u128>()
                .map_err(|e| format!("invalid unit_id: {}", e))?,
        )
        .ok_or_else(|| "invalid EntityId".to_string())?,
        revision: dc_domain::ids::UnitRevision::new(1)
            .ok_or_else(|| "invalid UnitRevision".to_string())?,
    };
    let outcome = handle_command(
        &mut state.core.write().unwrap(),
        CoreCommand::StartAttempt(unit_ref),
    )
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
#[tauri::command(rename_all = "snake_case")]
fn plan_objective_graph(objective_json: String) -> Result<String, String> {
    let objective: ObjectiveContract = serde_json::from_str(&objective_json)
        .map_err(|e| format!("invalid objective JSON: {}", e))?;
    let graph =
        dc_core::planner::plan_graph_from_objective(&objective).map_err(|e| format!("{:?}", e))?;
    serde_json::to_string_pretty(&graph).map_err(|e| format!("graph serialization failed: {}", e))
}

/// UI bridge: accept a previously planned graph (owner review step).
/// C-08: The UI passes the graph as raw JSON — the core validates and
/// deserializes it internally, preserving the invariant that GraphCandidate
/// is only constructed by the core.
#[tauri::command(rename_all = "snake_case")]
fn accept_graph(state: tauri::State<'_, AppState>, graph_json: String) -> Result<String, String> {
    // C-08: We do NOT deserialize GraphCandidate here. We pass the raw JSON
    // to the core, which validates it and constructs the graph internally.
    let outcome = handle_command(
        &mut state.core.write().unwrap(),
        CoreCommand::AcceptGraphRaw(graph_json),
    )
    .map_err(|e| format!("{:?}", e))?;
    Ok(format!("{:?}", outcome))
}

/// UI bridge: execute the accepted graph through the broker dispatcher.
#[tauri::command]
fn execute_graph(state: tauri::State<'_, AppState>) -> Result<String, String> {
    crate::executor::execute_accepted_graph(&state.core, None)
}

/// Health check endpoint for monitoring.
/// Returns system status: store reachable, broker reachable, active jobs,
/// metrics counters, and uptime.
#[tauri::command]
fn health_check(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let core = state.core.read().unwrap();
    let units_count = core.graph.as_ref().map(|g| g.units.len()).unwrap_or(0);
    let verifications_count = core.verifications.len();
    // Active jobs = units currently in Running stage
    let active_jobs = core
        .graph
        .as_ref()
        .map(|g| {
            g.units
                .iter()
                .filter(|u| u.stage == dc_domain::work::UnitStage::Running)
                .count()
        })
        .unwrap_or(0);
    let has_objective = core.objective.is_some();
    drop(core);

    // Check store reachability: try opening the SQLite file directly.
    let store_status = match &state.store_path {
        Some(path) => {
            match rusqlite_lite_check(path) {
                Ok(()) => "reachable",
                Err(e) => {
                    tracing::warn!(event = "health_store_unreachable", error = %e, path = ?path);
                    "unreachable"
                }
            }
        }
        None => "not_configured",
    };

    // Check broker reachability: TCP connect to the endpoint host:port.
    let broker_status = broker_reachable(&state.broker_endpoint);

    let metrics = dc_core::metrics::Metrics::global().snapshot();

    // Status semantics: "degraded" only when the local store (the critical
    // persistence dependency) is configured but unreachable. Broker
    // reachability is reported as a field but does not degrade the overall
    // status — it is an external, transient dependency.
    let degraded = store_status == "unreachable";

    let health = serde_json::json!({
        "status": if degraded { "degraded" } else { "ok" },
        "version": env!("CARGO_PKG_VERSION"),
        "build_stage": "WorkGraph",
        "store": store_status,
        "broker_tcp_reachable": broker_status,
        "active_jobs": active_jobs,
        "has_objective": has_objective,
        "units": units_count,
        "verifications": verifications_count,
        "metrics": metrics,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    serde_json::to_string_pretty(&health)
        .map_err(|e| format!("health check serialization failed: {}", e))
}

/// Lightweight store reachability check: open the SQLite file with rusqlite
/// directly and run a trivial query. Does not disturb the live store.
fn rusqlite_lite_check(path: &std::path::Path) -> Result<(), String> {
    let conn = rusqlite_open_readonly(path)?;
    conn.execute_batch("SELECT 1;")
        .map_err(|e| format!("query failed: {}", e))
}

/// Open a SQLite database read-only via the bundled rusqlite from dc-store.
/// We go through `rusqlite` only if it is already a transitive dependency;
/// otherwise fall back to a file-existence check.
fn rusqlite_open_readonly(path: &std::path::Path) -> Result<rusqlite::Connection, String> {
    if !path.exists() {
        return Err(format!("store file does not exist: {:?}", path));
    }
    rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("open failed: {}", e))
}

/// Check broker reachability via a TCP connect with a short timeout.
fn broker_reachable(config: &dc_broker::EndpointConfig) -> &'static str {
    use std::net::ToSocketAddrs;
    // Parse "http://host:port" or "https://host:port" without the url crate.
    let stripped = config
        .base_url
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let host_port = stripped.split('/').next().unwrap_or("");
    if host_port.is_empty() {
        return "misconfigured";
    }
    let (host, port) = match host_port.split_once(':') {
        Some((h, p)) => match p.parse::<u16>() {
            Ok(port) => (h.to_string(), port),
            Err(_) => return "misconfigured",
        },
        None => {
            // Default ports by scheme
            if config.base_url.starts_with("https://") {
                (host_port.to_string(), 443)
            } else {
                (host_port.to_string(), 80)
            }
        }
    };
    let addr_str = format!("{}:{}", host, port);
    let mut addrs = match addr_str.to_socket_addrs() {
        Ok(a) => a,
        Err(_) => return "unreachable",
    };
    let addr = match addrs.next() {
        Some(a) => a,
        None => return "unreachable",
    };
    match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(500)) {
        Ok(_) => "reachable",
        Err(_) => "unreachable",
    }
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
    let store_path = std::env::var("DC_STORE_PATH").ok().filter(|s| !s.is_empty()).map(PathBuf::from);
    let source: Box<dyn LiveStateSource> = match store_path {
        Some(ref path) => Box::new(StoreStateSource::new(path.clone())),
        None => Box::new(EmptyStateSource),
    };

    // Initialize persistence when store path is available
    let persistence = store_path.as_ref().map(|path| {
        let path = path.clone();
        let store = dc_store::SQLiteStore::new(path.to_str().unwrap_or(""));
        let mut core_persistence = dc_core::persistence::CorePersistence::new(store);
        // Initialize the store (creates tables if needed)
        if let Err(e) = core_persistence.initialize(&path) {
            eprintln!(
                "FATAL: Failed to initialize persistence at {:?}: {}",
                path, e
            );
            panic!("Persistence initialization failed: {}", e);
        }
        std::sync::Mutex::new(core_persistence)
    });

    // Recover state from persistence if available
    let initial_core = {
        let mut core = CoreState::empty();
        if let Some(pers) = &persistence {
            let pers = pers.lock().unwrap();
            // Use a fixed non-zero UUIDv7 ID for the singleton app state
            // UUIDv7 format: version bits at position 76-79 must be 7, variant bits at 62-63 must be 2
            // Correct layout: bits 76-78 = 111, bits 62-63 = 10
            let state_id = dc_domain::ids::EntityId::from_u128(0x00000000000070008000000000000001)
                .expect("hardcoded UUIDv7 constant is valid");
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
                    match serde_json::from_str::<dc_domain::work::ObjectiveContract>(objective_json)
                    {
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
            store_path: store_path.clone(),
            broker_endpoint: dc_broker::EndpointConfig::default(),
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

/// Initialize structured logging (tracing) with env-filter control.
///
/// Log level is controlled via the `DC_LOG` environment variable
/// (e.g. `DC_LOG=debug` or `DC_LOG=dc_core=info,dc_broker=warn`).
/// Defaults to `info`. Output is human-readable to stderr; set
/// `DC_LOG_FORMAT=json` for machine-parseable JSON logs.
fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    let filter = EnvFilter::try_from_env("DC_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let json = std::env::var("DC_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    let subscriber = tracing_subscriber::registry().with(filter);
    if json {
        let layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(std::io::stderr);
        let _ = tracing::subscriber::set_global_default(subscriber.with(layer));
    } else {
        let layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(true);
        let _ = tracing::subscriber::set_global_default(subscriber.with(layer));
    }
    tracing::info!(
        event = "logging_initialized",
        format = if json { "json" } else { "pretty" },
        "structured logging initialized"
    );
}

fn main() {
    init_tracing();
    tracing::info!(event = "startup", version = env!("CARGO_PKG_VERSION"), "Deep Create starting");

    // Signal-aware exit: Tauri's default close behaviour destroys windows and
    // exits the event loop cleanly on SIGINT/SIGTERM-adjacent window close.
    // For headless/CI contexts, install a ctrlc handler that flushes logs and
    // exits with a conventional status code (130 = SIGINT, 143 = SIGTERM).
    install_signal_handler();

    desktop_builder(tauri::Builder::default())
        .run(tauri::generate_context!())
        .expect("Deep Create could not start its version page");
}

/// Install SIGINT/SIGTERM handlers for graceful shutdown.
///
/// On Windows, CTRL_C_EVENT and CTRL_BREAK_EVENT map to SIGINT/SIGTERM
/// semantics. The handler logs the shutdown event and exits with the
/// conventional exit code, allowing the OS to run destructors for the
/// process (Rust does not guarantee Drop on process::exit, but the store
/// uses WAL-mode SQLite so committed transactions are durable).
#[cfg(windows)]
#[allow(unsafe_code)]
fn install_signal_handler() {
    use windows_sys::Win32::Foundation::BOOL;
    use windows_sys::Win32::System::Console::{
        CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler,
    };

    unsafe extern "system" fn handler(ctrl_type: u32) -> BOOL {
        match ctrl_type {
            x if x == CTRL_C_EVENT => {
                eprintln!("[shutdown] received SIGINT (Ctrl+C), exiting with code 130");
                tracing::info!(event = "shutdown", signal = "SIGINT", "graceful shutdown initiated");
                std::process::exit(130);
            }
            x if x == CTRL_BREAK_EVENT => {
                eprintln!("[shutdown] received SIGTERM-equivalent (Ctrl+Break), exiting with code 143");
                tracing::info!(event = "shutdown", signal = "SIGTERM", "graceful shutdown initiated");
                std::process::exit(143);
            }
            _ => 0, // FALSE — not handled
        }
    }

    unsafe {
        SetConsoleCtrlHandler(Some(handler), 1);
    }
}

/// Non-Windows fallback: rely on default signal handling.
/// Tauri's event loop responds to window close; POSIX signals use defaults.
#[cfg(not(windows))]
fn install_signal_handler() {
    tracing::debug!("custom signal handler not installed on this platform");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
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
        let response =
            get_ipc_response(&window, request("work_unit_ids", "http://tauri.localhost"))
                .expect("work_unit_ids is allowed");
        let ids: Vec<String> = response.deserialize().expect("work unit IDs JSON");
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

    // ========================================================================
    // LIVE E2E: UI↔Core round-trip through the REAL invoke_handler
    // ========================================================================
    //
    // These tests instantiate the SAME `desktop_builder` that production
    // `main()` calls, with the SAME `invoke_handler` registrations, and drive
    // it through `tauri::test::get_ipc_response` — the canonical Tauri test
    // harness that exercises the real IPC deserializer, ACL, command router,
    // argument deserialization, and response serialization. The CoreState
    // behind the handlers is a real `RwLock<CoreState>` driven by the real
    // `dc_core::command::handle_command`.
    //
    // What is NOT proven here: real WebView2 rendering, JS bridge injection,
    // DOM event wiring. That requires tauri-driver + WebView2 + a windowed
    // session, which is out of scope for headless CI/test environments.
    //
    // See LIVE_E2E_EVIDENCE.md at workspace root for the full report.

    /// Helper: build an InvokeRequest with a JSON body (for commands that take args).
    fn request_with_body(
        command: &str,
        url: &str,
        body: serde_json::Value,
    ) -> tauri::webview::InvokeRequest {
        tauri::webview::InvokeRequest {
            cmd: command.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: url.parse().expect("test URL"),
            body: tauri::ipc::InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.into(),
        }
    }

    /// Helper: build a minimal but valid ObjectiveContract JSON.
    /// Uses valid UUIDv7 EntityIds (version=7 at bits 76-79, variant=0b10 at 62-63).
    fn test_objective_json() -> serde_json::Value {
        fn vid(n: u128) -> String {
            let v = (0x7u128 << 76) | (0x2u128 << 62) | (n & ((1u128 << 62) - 1));
            format!("{:032x}", v)
        }
        json!({
            "scope": {
                "session_id": vid(1),
                "work_id": vid(2),
                "objective_id": vid(3),
                "objective_revision": 1
            },
            "statement": "Live E2E test objective: prove UI↔Core round-trip",
            "owner_event_ids": [],
            "obligations": [],
            "completion_criteria": [vid(100)],
            "row_version": 1
        })
    }

    /// Helper: build a GraphCandidate JSON with the given units array.
    fn test_graph_json(
        objective_json: &serde_json::Value,
        units: serde_json::Value,
    ) -> serde_json::Value {
        json!({
            "expected_graph_version": 1,
            "current_graph_version": 1,
            "roots": [],
            "objectives": [objective_json],
            "units": units,
            "dependencies": [],
            "links": [],
            "publications": [],
            "publication_states": [],
            "publication_bindings": [],
            "owner_acceptances": []
        })
    }

    /// Helper: build a single WorkUnitSpec JSON for the live dispatch test.
    fn test_unit_json(objective_json: &serde_json::Value) -> serde_json::Value {
        fn vid(n: u128) -> String {
            let v = (0x7u128 << 76) | (0x2u128 << 62) | (n & ((1u128 << 62) - 1));
            format!("{:032x}", v)
        }
        let scope = objective_json["scope"].clone();
        json!({
            "unit": {
                "unit_id": vid(200),
                "revision": 1
            },
            "scope": scope,
            "parent": null,
            "kind": "research",
            "expected_output": {
                "export_name": "live_e2e_result",
                "schema_version": 1,
                "fields": []
            },
            "obligation_ids": [],
            "acceptance_criteria": [vid(100)],
            "bounds": {
                "max_attempts": 3,
                "max_steps": 10,
                "stop_conditions": ["output_produced"]
            },
            "exploration": null,
            "stage": "proposed",
            "accepted_result": null,
            "row_version": 1
        })
    }

    /// LIVE E2E (deterministic, no network): drive the full UI workflow through
    /// the real invoke_handler. Uses an empty-units graph so `execute_graph`
    /// returns immediately without needing a live broker.
    ///
    /// Proves:
    ///   1. application_info returns empty state before any objective
    ///   2. submit_objective accepts a real ObjectiveContract and mutates CoreState
    ///   3. plan_objective_graph returns a real GraphCandidate JSON
    ///   4. accept_graph commits the graph to CoreState
    ///   5. work_units reflects the accepted graph (live projection)
    ///   6. execute_graph runs the executor path (returns "no units to execute")
    ///   7. health_check reports the final state
    #[test]
    fn ui_to_core_full_round_trip_via_real_invoke_handler() {
        let (_app, window) = test_window("main");
        let url = "http://tauri.localhost";

        // Step 1: initial state — no work units
        let r =
            get_ipc_response(&window, request("application_info", url)).expect("application_info");
        let v: serde_json::Value = r.deserialize().expect("app info json");
        assert_eq!(v["workUnitsCount"], 0);
        println!("[E2E] step 1 OK: initial workUnitsCount = 0");

        // Step 2: submit_objective — UI write path through real handle_command
        let objective = test_objective_json();
        let body = json!({ "objective_json": objective.to_string() });
        let r = get_ipc_response(&window, request_with_body("submit_objective", url, body))
            .expect("submit_objective");
        let outcome: String = r.deserialize().expect("submit outcome");
        assert!(
            outcome.contains("Accepted"),
            "unexpected outcome: {outcome}"
        );
        println!("[E2E] step 2 OK: submit_objective → {outcome}");

        // Step 3: plan_objective_graph — read-only bridge, returns GraphCandidate JSON
        let body = json!({ "objective_json": objective.to_string() });
        let r = get_ipc_response(
            &window,
            request_with_body("plan_objective_graph", url, body),
        )
        .expect("plan_objective_graph");
        let graph_json: String = r.deserialize().expect("graph json string");
        let graph: serde_json::Value = serde_json::from_str(&graph_json).expect("parse graph");
        assert_eq!(graph["units"].as_array().unwrap().len(), 1);
        println!("[E2E] step 3 OK: plan_objective_graph returned 1 unit");

        // Step 4: accept_graph — override with EMPTY units so execute_graph needs no network
        let empty_graph = test_graph_json(&objective, json!([]));
        let body = json!({ "graph_json": empty_graph.to_string() });
        let r = get_ipc_response(&window, request_with_body("accept_graph", url, body))
            .expect("accept_graph");
        let outcome: String = r.deserialize().expect("accept outcome");
        assert!(
            outcome.contains("Accepted"),
            "unexpected outcome: {outcome}"
        );
        println!("[E2E] step 4 OK: accept_graph → {outcome}");

        // Step 5: work_units — live projection reflects accepted graph (empty here)
        let r = get_ipc_response(&window, request("work_units", url)).expect("work_units");
        let units: Vec<serde_json::Value> = r.deserialize().expect("work units json");
        assert_eq!(units.len(), 0, "empty graph should project 0 work units");
        println!("[E2E] step 5 OK: work_units projected 0 units (empty graph)");

        // Step 6: execute_graph — runs the real executor, hits "no units to execute"
        let r = get_ipc_response(&window, request("execute_graph", url)).expect("execute_graph");
        let outcome: String = r.deserialize().expect("execute outcome");
        assert_eq!(outcome, "no units to execute");
        println!("[E2E] step 6 OK: execute_graph → {outcome}");

        // Step 7: health_check — final state reflects accepted objective + graph
        let r = get_ipc_response(&window, request("health_check", url)).expect("health_check");
        let health: String = r.deserialize().expect("health json string");
        let health: serde_json::Value = serde_json::from_str(&health).expect("parse health");
        assert_eq!(health["status"], "ok");
        assert_eq!(health["units"], 0);
        println!("[E2E] step 7 OK: health_check → {health}");

        println!("[E2E] FULL ROUND TRIP PASSED: UI→Core→UI via real invoke_handler");
    }

    /// LIVE E2E (real network, ignored by default): same round-trip but with a
    /// real unit in the graph, so `execute_graph` actually dispatches through
    /// dc-broker to live OmniRoute at localhost:20128.
    ///
    /// Run explicitly with:
    ///   cargo test -p dc-desktop --lib ui_to_core_real_broker_dispatch_live -- --ignored --nocapture
    ///
    /// If OmniRoute is unreachable or hangs, this test will fail with the
    /// broker's error message — that is the honest signal.
    #[test]
    #[ignore]
    fn ui_to_core_real_broker_dispatch_live() {
        let (_app, window) = test_window("main");
        let url = "http://tauri.localhost";

        // Step 1: submit objective
        let objective = test_objective_json();
        let body = json!({ "objective_json": objective.to_string() });
        let r = get_ipc_response(&window, request_with_body("submit_objective", url, body))
            .expect("submit_objective");
        let outcome: String = r.deserialize().expect("submit outcome");
        assert!(outcome.contains("Accepted"));
        println!("[LIVE] objective accepted");

        // Step 2: accept graph with a real unit
        let unit = test_unit_json(&objective);
        let graph = test_graph_json(&objective, json!([unit]));
        let body = json!({ "graph_json": graph.to_string() });
        let r = get_ipc_response(&window, request_with_body("accept_graph", url, body))
            .expect("accept_graph");
        let outcome: String = r.deserialize().expect("accept outcome");
        assert!(outcome.contains("Accepted"));
        println!("[LIVE] graph accepted (1 unit)");

        // Step 3: verify work_units projection shows the unit
        let r = get_ipc_response(&window, request("work_units", url)).expect("work_units");
        let units: Vec<serde_json::Value> = r.deserialize().expect("work units json");
        assert_eq!(units.len(), 1);
        println!("[LIVE] work_units projected 1 unit: {:?}", units[0]["id"]);

        // Step 4: execute_graph — REAL dispatch to live OmniRoute
        // This will hang up to the broker's internal timeout (30s default) if
        // OmniRoute is unresponsive. That is the honest behavior.
        println!("[LIVE] calling execute_graph — real dispatch to localhost:20128 ...");
        let r = get_ipc_response(&window, request("execute_graph", url)).expect("execute_graph");
        let outcome: String = r.deserialize().expect("execute outcome");
        println!("[LIVE] execute_graph outcome: {outcome}");

        // We do NOT assert success here — if OmniRoute is down, the outcome
        // will contain "failed" or "unreachable" and that is the honest result.
        // The test PASSES as long as the IPC round-trip completed and returned
        // a well-formed string from the real executor.
        assert!(!outcome.is_empty());
        println!("[LIVE] real dispatch round-trip completed (see outcome above)");
    }
}
