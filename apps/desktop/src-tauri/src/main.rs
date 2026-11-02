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

#[tauri::command]
async fn factory_repository_detect(
    window: WebviewWindow,
    project_id: String,
) -> Result<Value, String> {
    authorize(&window)?;
    client()?
        .repository_detect(&project_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn factory_repository_changes(
    window: WebviewWindow,
    project_id: String,
    page: u32,
) -> Result<Value, String> {
    authorize(&window)?;
    client()?
        .repository_changes(&project_id, page)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn factory_repository_change(
    window: WebviewWindow,
    project_id: String,
    number: u64,
) -> Result<Value, String> {
    authorize(&window)?;
    client()?
        .repository_change(&project_id, number)
        .await
        .map_err(|error| error.to_string())
}

fn repository_url(value: &str) -> Option<Url> {
    let url = Url::parse(value).ok()?;
    if url.scheme() != "https"
        || !matches!(url.host_str(), Some("github.com" | "gitlab.com"))
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !url
            .path()
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
    {
        return None;
    }
    let segments: Vec<_> = url.path().trim_matches('/').split('/').collect();
    if segments.len() < 2
        || segments
            .iter()
            .any(|s| s.is_empty() || matches!(*s, "." | ".."))
    {
        return None;
    }
    Some(url)
}

#[tauri::command]
async fn factory_open_repository_url(window: WebviewWindow, url: String) -> Result<(), String> {
    authorize(&window)?;
    let url = repository_url(&url)
        .ok_or("Only plain GitHub or GitLab HTTPS repository links can be opened.")?;
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(move || {
            // Open the system browser; project content never enters the trusted webview.
            let mut child = std::process::Command::new("/usr/bin/open")
                .arg(url.as_str())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|_| "Cannot open the system browser.")?;
            let start = std::time::Instant::now();
            loop {
                if let Some(status) = child
                    .try_wait()
                    .map_err(|_| "Cannot observe the browser opener.")?
                {
                    return if status.success() {
                        Ok(())
                    } else {
                        Err("The system browser could not open this link.")
                    };
                }
                if start.elapsed() > std::time::Duration::from_secs(5) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("Opening the system browser timed out.");
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        })
        .await
        .map_err(|_| "The browser opener failed.".to_owned())?
        .map_err(str::to_owned)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = url;
        Err("Opening provider links is currently supported only on macOS. Copy the displayed URL into your browser.".into())
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            factory_snapshot,
            factory_events,
            factory_command,
            factory_capabilities,
            factory_repository_detect,
            factory_repository_changes,
            factory_repository_change,
            factory_open_repository_url
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

    #[test]
    fn external_repository_links_cannot_target_local_or_credential_bearing_destinations() {
        for value in [
            "https://github.com/owner/repo/pull/24",
            "https://github.com/owner/repo/actions/runs/123/job/456",
            "https://gitlab.com/group/subgroup/repo/-/merge_requests/3",
        ] {
            assert!(repository_url(value).is_some());
        }
        for value in [
            "http://github.com/owner/repo",
            "https://github.com.evil.test/owner/repo",
            "https://user:secret@github.com/owner/repo",
            "https://github.com:444/owner/repo",
            "https://github.com/owner/repo?token=secret",
            "https://github.com/owner/repo#secret",
            "https://github.com/owner%2Frepo",
            "https://github.com/",
            "http://127.0.0.1/private",
            "file:///tmp/private",
            "javascript:alert(1)",
        ] {
            assert!(repository_url(value).is_none(), "accepted {value}");
        }
    }
}
