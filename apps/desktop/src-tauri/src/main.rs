#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::Value;
use software_factory::{
    client::{DaemonClient, default_data_dir},
    model::{Event, Snapshot},
};
use tauri::{Url, WebviewWindow, WebviewWindowBuilder};

fn trusted_url(url: &Url) -> bool {
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    let bundled =
        url.scheme() == "tauri" && url.host_str() == Some("localhost") && url.port().is_none();
    let development = cfg!(debug_assertions)
        && url.scheme() == "http"
        && url.host_str() == Some("127.0.0.1")
        && url.port() == Some(5173);
    bundled || development
}

fn authorize(window: &WebviewWindow) -> Result<(), String> {
    let url = window
        .url()
        .map_err(|_| "Cannot verify the invoking window.")?;
    if window.label() != "main" || !trusted_url(&url) {
        return Err("Factory commands are available only in the trusted main window.".into());
    }
    Ok(())
}

fn client() -> Result<DaemonClient, String> {
    // The authenticated client reads native storage; credentials never enter JavaScript.
    DaemonClient::connect(default_data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
async fn factory_snapshot(window: WebviewWindow) -> Result<Snapshot, String> {
    authorize(&window)?;
    client()?
        .snapshot()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn factory_events(
    window: WebviewWindow,
    after: u64,
    limit: u32,
) -> Result<Vec<Event>, String> {
    authorize(&window)?;
    if limit == 0 || limit > 500 {
        return Err("Event page size must be between 1 and 500.".into());
    }
    client()?
        .events(after, limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn factory_command(window: WebviewWindow, command: Value) -> Result<Value, String> {
    authorize(&window)?;
    client()?
        .command(command)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn factory_capabilities(window: WebviewWindow) -> Result<Value, String> {
    authorize(&window)?;
    client()?
        .capabilities()
        .await
        .map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            factory_snapshot,
            factory_events,
            factory_command,
            factory_capabilities
        ])
        .setup(|app| {
            let config = app
                .config()
                .app
                .windows
                .first()
                .ok_or("Main window configuration is missing.")?;
            WebviewWindowBuilder::from_config(app, config)?
                .on_navigation(trusted_url)
                .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run the Software Factory native shell");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_bridge_rejects_remote_and_confusable_origins() {
        for url in [
            "https://example.com/",
            "http://127.0.0.1.evil.test:5173/",
            "http://localhost:5173/",
            "http://127.0.0.1:5174/",
            "http://user@127.0.0.1:5173/",
            "tauri://example.com/",
            "tauri://localhost:5173/",
            "file:///tmp/project/index.html",
        ] {
            assert!(!trusted_url(&Url::parse(url).unwrap()), "accepted {url}");
        }
        assert!(trusted_url(
            &Url::parse("tauri://localhost/index.html#/conversation").unwrap()
        ));
        assert_eq!(
            trusted_url(&Url::parse("http://127.0.0.1:5173/#/conversation").unwrap()),
            cfg!(debug_assertions)
        );
    }
}
