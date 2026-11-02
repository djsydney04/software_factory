fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "factory_snapshot",
            "factory_events",
            "factory_command",
            "factory_capabilities",
            "factory_repository_detect",
            "factory_repository_changes",
            "factory_repository_change",
            "factory_open_repository_url",
        ]),
    ))
    .expect("failed to prepare the Software Factory native shell");
}
