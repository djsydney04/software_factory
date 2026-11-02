use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use software_factory::{
    client::{read_connection, DaemonClient},
    service::{router, DataDirectory, ServiceState},
    store::Store,
};
use std::{net::SocketAddr, process::Command};
use tempfile::TempDir;
use tower::ServiceExt;

fn fixture() -> (TempDir, ServiceState) {
    let temp = TempDir::new().unwrap();
    let state = ServiceState::new(
        Store::open(temp.path().join("factory.sqlite3")).unwrap(),
        "a".repeat(64),
        "127.0.0.1:38421".parse::<SocketAddr>().unwrap(),
    );
    (temp, state)
}
fn request(path: &str, token: bool) -> axum::http::request::Builder {
    let req = Request::builder()
        .uri(path)
        .header("host", "127.0.0.1:38421");
    if token {
        req.header("authorization", format!("Bearer {}", "a".repeat(64)))
    } else {
        req
    }
}

#[tokio::test]
async fn project_reads_and_writes_require_authentication_and_reject_browser_origins() {
    let (_temp, state) = fixture();
    for path in ["/v1/snapshot", "/v1/events", "/v1/commands"] {
        let mut req = request(path, false);
        if path.ends_with("commands") {
            req = req
                .method("POST")
                .header("content-type", "application/json");
        }
        let res = router(state.clone())
            .oneshot(req.body(Body::from("{}")).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
    for (header, value) in [
        ("origin", "http://127.0.0.1:5173"),
        ("origin", "null"),
        ("sec-fetch-site", "same-origin"),
    ] {
        let res = router(state.clone())
            .oneshot(
                request("/v1/snapshot", true)
                    .header(header, value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }
    let res = router(state.clone())
        .oneshot(
            request("/v1/snapshot", true)
                .header("host", "evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert!(state.store.snapshot().unwrap().projects.is_empty());
    assert!(state.store.events(0, 100).unwrap().is_empty());
    let res = router(state.clone())
        .oneshot(request("/health", false).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn duplicate_command_http_delivery_creates_one_persistent_project_and_event() {
    let (temp, state) = fixture();
    let repo = temp.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    assert!(Command::new("/usr/bin/git")
        .arg("init")
        .arg(&repo)
        .output()
        .unwrap()
        .status
        .success());
    let payload = json!({"commandId":"register-once","command":{"type":"register_project","project":{"name":"Fixture","repoPath":repo,"objective":"Improve a small local fixture"}}});
    let mut ids = vec![];
    for _ in 0..2 {
        let req = request("/v1/commands", true)
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = router(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        ids.push(serde_json::from_slice::<Value>(&bytes).unwrap()["id"].clone());
    }
    assert_eq!(ids[0], ids[1]);
    let reopened = Store::open(temp.path().join("factory.sqlite3"))
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(reopened.projects.len(), 1);
    assert_eq!(
        reopened
            .events
            .iter()
            .filter(|e| e.kind == "project_created")
            .count(),
        1
    );
}

#[tokio::test]
async fn native_client_uses_private_connection_and_daemon_ownership_is_exclusive() {
    let temp = TempDir::new().unwrap();
    let data = DataDirectory::acquire(temp.path().join("state")).unwrap();
    assert!(DataDirectory::acquire(data.path.clone()).is_err());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let connection = data.publish_connection(address).unwrap();
    let state = ServiceState::new(
        Store::open(data.path.join("factory.sqlite3")).unwrap(),
        connection.token,
        address,
    );
    let server = tokio::spawn(async move { axum::serve(listener, router(state)).await.unwrap() });
    let client = DaemonClient::connect(data.path.clone()).unwrap();
    assert!(client.snapshot().await.unwrap().projects.is_empty());
    assert!(client.events(0, 10).await.unwrap().is_empty());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            data.path.join("connection.json"),
            std::fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        assert!(read_connection(&data.path).is_err());
    }
    server.abort();
    let _ = server.await;
    drop(data);
    assert!(DataDirectory::acquire(temp.path().join("state")).is_ok());
}

#[test]
fn native_connection_rejects_public_destinations_redirect_paths_and_symlinks() {
    let temp = TempDir::new().unwrap();
    let data = DataDirectory::acquire(temp.path().join("state")).unwrap();
    data.publish_connection("127.0.0.1:38421".parse().unwrap())
        .unwrap();
    let path = data.path.join("connection.json");
    let mut connection: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for url in [
        "http://localhost:38421/",
        "https://example.com/",
        "http://127.0.0.1:38421/private",
        "http://user@127.0.0.1:38421/",
    ] {
        connection["baseUrl"] = json!(url);
        std::fs::write(&path, serde_json::to_vec(&connection).unwrap()).unwrap();
        assert!(read_connection(&data.path).is_err());
    }
    #[cfg(unix)]
    {
        let other = temp.path().join("other.json");
        std::fs::rename(&path, &other).unwrap();
        std::os::unix::fs::symlink(&other, &path).unwrap();
        assert!(read_connection(&data.path).is_err());
    }
}
