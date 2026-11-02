fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "factory_snapshot",
            "factory_events",
            "factory_command",
            "factory_capabilities",
        ]),
    ))
    .expect("failed to prepare the Software Factory native shell");
}
