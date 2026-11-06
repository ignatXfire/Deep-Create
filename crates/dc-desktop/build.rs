fn main() {
    // Tauri custom app commands are otherwise callable from every window.
    // Register the complete command surface so capability checks apply to it.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "application_info",
            "work_unit_ids",
            "work_units",
            "submit_objective",
            "start_attempt",
            "plan_objective_graph",
            "accept_graph",
            "execute_graph",
            "health_check",
        ]),
    ))
    .expect("Deep Create desktop configuration failed validation");
}
