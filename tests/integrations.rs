use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use rusqlite::Connection;
use serde_json::{json, Value};
use software_factory::{
    integrations::{parse_remote, CiState, RepositoryClient, RepositoryError},
    model::{NewProject, Project, RepositoryConnection, RepositoryProvider},
    service::{router, ServiceState},
    store::Store,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use tempfile::TempDir;
use tower::ServiceExt;

struct Fixture {
    directory: TempDir,
    cli: PathBuf,
    repo: PathBuf,
    database: PathBuf,
    project: Project,
    state: ServiceState,
}
impl Fixture {
    fn new(provider: RepositoryProvider) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let repo = directory.path().join("repo");
        std::fs::create_dir(&repo).unwrap();
        assert!(Command::new("/usr/bin/git")
            .arg("init")
            .arg(&repo)
            .output()
            .unwrap()
            .status
            .success());
        let remote = match provider {
            RepositoryProvider::Github => "git@github.com:owner/repo.git",
            RepositoryProvider::Gitlab => "git@gitlab.com:group/subgroup/repo.git",
        };
        assert!(Command::new("/usr/bin/git")
            .arg("-C")
            .arg(&repo)
            .args(["remote", "add", "origin", remote])
            .status()
            .unwrap()
            .success());
        let cli = directory.path().join("repository-cli");
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/repository_fake.py"),
            &cli,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&cli, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let database = directory.path().join("factory.sqlite");
        let store = Store::open(&database).unwrap();
        let project = store
            .create_project(
                "project",
                NewProject {
                    name: "Repository fixture".into(),
                    repo_path: repo.to_string_lossy().into(),
                    objective: "Read statuses without changing external work".into(),
                    settings: Default::default(),
                },
            )
            .unwrap();
        let state = ServiceState::new(store, "a".repeat(64), "127.0.0.1:38421".parse().unwrap())
            .with_repository_client(
                RepositoryClient::new(
                    Some(cli.clone()),
                    Some(cli.clone()),
                    directory.path().into(),
                )
                .unwrap(),
            );
        Self {
            directory,
            cli,
            repo,
            database,
            project,
            state,
        }
    }
    fn mode(&self, mode: &str) {
        std::fs::write(
            self.directory.path().join("repository-config.json"),
            json!({"mode":mode}).to_string(),
        )
        .unwrap();
    }
    fn client(&self) -> RepositoryClient {
        RepositoryClient::new(
            Some(self.cli.clone()),
            Some(self.cli.clone()),
            self.directory.path().into(),
        )
        .unwrap()
    }
    fn connection(&self, provider: RepositoryProvider) -> RepositoryConnection {
        let target = parse_remote(match provider {
            RepositoryProvider::Github => "https://github.com/owner/repo",
            RepositoryProvider::Gitlab => "https://gitlab.com/group/subgroup/repo",
        })
        .unwrap();
        self.state
            .store
            .connect_repository("fixture-connect", &self.project.id, target)
            .unwrap()
    }
    fn log(&self) -> Vec<Value> {
        std::fs::read_to_string(self.directory.path().join("repository-requests.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}
async fn api(
    state: &ServiceState,
    path: &str,
    command: Option<Value>,
    authenticated: bool,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .uri(path)
        .header("host", "127.0.0.1:38421");
    if authenticated {
        request = request.header("authorization", format!("Bearer {}", "a".repeat(64)));
    }
    let body = match command {
        Some(command) => {
            request = request
                .method("POST")
                .header("content-type", "application/json");
            Body::from(command.to_string())
        }
        None => Body::empty(),
    };
    let response = router(state.clone())
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
fn connect(fixture: &Fixture, provider: RepositoryProvider) -> Value {
    json!({"commandId":"connect-once","command":{"type":"connect_repository","projectId":fixture.project.id,"provider":provider}})
}

#[test]
fn remote_parser_accepts_canonical_public_hosts_and_nested_gitlab_groups() {
    for remote in [
        "git@github.com:owner/repo.git",
        "https://github.com/owner/repo",
        "ssh://git@github.com/owner/repo.git",
    ] {
        let value = parse_remote(remote).unwrap();
        assert_eq!(value.host, "github.com");
        assert_eq!(value.path, "owner/repo");
        assert_eq!(value.web_url, "https://github.com/owner/repo");
    }
    assert_eq!(
        parse_remote("git@gitlab.com:group/subgroup/repo.git")
            .unwrap()
            .path,
        "group/subgroup/repo"
    );
    for remote in [
        "https://token@github.com/owner/repo",
        "https://user:token@github.com/owner/repo",
        "https://github.com/owner/repo?token=private",
        "https://github.com/owner/repo#private",
        "https://github.com.evil.test/owner/repo",
        "http://github.com/owner/repo",
        "https://gitlab.example.com/group/repo",
        "/local/repo",
        "ext::evil",
        "git@github.com:owner/../repo",
        "https://github.com/owner/extra/../repo",
        "https://gitlab.com/group%2Fsub/repo",
        "git@github.com:owner/repo\nsecret",
        "https://github.com/owner/repo/",
        "ssh://user@github.com/owner/repo",
    ] {
        assert!(parse_remote(remote).is_err(), "accepted {remote}");
    }
}

#[tokio::test]
async fn detection_is_local_and_connection_is_verified_then_replayed_without_cli_or_repository() {
    let fixture = Fixture::new(RepositoryProvider::Github);
    let (status, value) = api(
        &fixture.state,
        &format!("/v1/repository/detect?projectId={}", fixture.project.id),
        None,
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["repository"]["path"], "owner/repo");
    assert!(
        fixture.log().is_empty(),
        "detection must not contact an account"
    );
    let payload = connect(&fixture, RepositoryProvider::Github);
    let (status, first) = api(&fixture.state, "/v1/commands", Some(payload.clone()), true).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["remoteName"], "origin");
    assert_eq!(fixture.log().len(), 1);
    std::fs::remove_file(&fixture.cli).unwrap();
    std::fs::rename(
        &fixture.repo,
        fixture.directory.path().join("moved-repository"),
    )
    .unwrap();
    let (status, second) = api(&fixture.state, "/v1/commands", Some(payload), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first, second);
    assert_eq!(fixture.log().len(), 1);
    let snapshot = Store::open(&fixture.database).unwrap().snapshot().unwrap();
    assert_eq!(snapshot.repository_connections.len(), 1);
    assert_eq!(
        snapshot.projects[0].settings.permissions,
        fixture.project.settings.permissions
    );
    assert!(snapshot.runs.is_empty());
    assert!(snapshot.lead_wakes.is_empty());
    assert_eq!(
        snapshot
            .events
            .iter()
            .filter(|e| e.kind == "repository_connected")
            .count(),
        1
    );
    let (status, value) = api(&fixture.state, "/v1/commands", Some(json!({"commandId":"disconnect","command":{"type":"disconnect_repository","projectId":fixture.project.id}})), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["connected"], false);
    assert!(fixture
        .state
        .store
        .snapshot()
        .unwrap()
        .repository_connections
        .is_empty());
    assert_eq!(fixture.log().len(), 1);
}

#[tokio::test]
async fn github_and_nested_gitlab_use_only_fixed_get_requests_from_neutral_directory() {
    for provider in [RepositoryProvider::Github, RepositoryProvider::Gitlab] {
        let fixture = Fixture::new(provider);
        std::fs::write(
            fixture.directory.path().join("repository-config.json"),
            json!({"githubPath":"Owner/Repo","gitlabPath":"Group/Subgroup/Repo"}).to_string(),
        )
        .unwrap();
        let (status, body) = api(
            &fixture.state,
            "/v1/commands",
            Some(connect(&fixture, provider)),
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(
            body["path"],
            if provider == RepositoryProvider::Github {
                "Owner/Repo"
            } else {
                "Group/Subgroup/Repo"
            }
        );
        let (status, list) = api(
            &fixture.state,
            &format!(
                "/v1/repository/changes?projectId={}&page=1",
                fixture.project.id
            ),
            None,
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{list}");
        assert_eq!(list["items"].as_array().unwrap().len(), 1);
        assert_eq!(
            fixture.log().len(),
            2,
            "the list must not fetch CI for every item"
        );
        let (status, detail) = api(
            &fixture.state,
            &format!(
                "/v1/repository/change?projectId={}&number=7",
                fixture.project.id
            ),
            None,
            true,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{detail}");
        assert_eq!(detail["ci"]["state"], "success");
        assert_eq!(detail["ci"]["revision"], "a".repeat(40));
        for entry in fixture.log() {
            let args = entry["args"].as_array().unwrap();
            assert_eq!(&args[..3], &[json!("api"), json!("--method"), json!("GET")]);
            assert_ne!(entry["cwd"], fixture.repo.to_string_lossy().as_ref());
            if provider == RepositoryProvider::Gitlab {
                assert_eq!(
                    entry["gitlabAuthEnvPresent"], false,
                    "generic tokens must not be rebound to the public GitLab host"
                );
                assert!(args.iter().any(|arg| arg.as_str().is_some_and(|s| s
                    .to_ascii_lowercase()
                    .starts_with("https://gitlab.com/api/v4/projects/group%2fsubgroup%2frepo"))));
            }
        }
    }
}

#[tokio::test]
async fn combined_ci_never_turns_missing_partial_unknown_or_wrong_revision_evidence_green() {
    let fixture = Fixture::new(RepositoryProvider::Github);
    let connection = fixture.connection(RepositoryProvider::Github);
    for (mode, expected) in [
        ("success", CiState::Success),
        ("failure", CiState::Failure),
        ("mixed_ci", CiState::Failure),
        ("missing_ci", CiState::Unknown),
        ("unknown_ci", CiState::Unknown),
        ("stale_ci", CiState::Unknown),
        ("partial_ci", CiState::Unknown),
        ("no_checks", CiState::None),
    ] {
        fixture.mode(mode);
        let detail = fixture
            .client()
            .change(connection.clone(), 7)
            .await
            .unwrap();
        assert_eq!(detail.ci.state, expected, "mode {mode}");
        let rendered = serde_json::to_string(&detail).unwrap();
        assert!(!rendered.contains("PRIVATE"));
        assert!(!rendered.contains("third-party-ci.example"));
        if matches!(mode, "missing_ci" | "partial_ci" | "stale_ci" | "mixed_ci") {
            assert!(!detail.ci.complete);
        }
    }
    let fixture = Fixture::new(RepositoryProvider::Gitlab);
    let connection = fixture.connection(RepositoryProvider::Gitlab);
    for mode in ["pipeline_revision", "missing_ci"] {
        fixture.mode(mode);
        let detail = fixture
            .client()
            .change(connection.clone(), 7)
            .await
            .unwrap();
        assert_eq!(detail.ci.state, CiState::Unknown);
        assert!(!detail.ci.complete);
    }
}

#[tokio::test]
async fn response_limits_errors_pagination_and_link_validation_are_visible() {
    let fixture = Fixture::new(RepositoryProvider::Github);
    let connection = fixture.connection(RepositoryProvider::Github);
    fixture.mode("full_page");
    let page = fixture
        .client()
        .changes(connection.clone(), 2)
        .await
        .unwrap();
    assert_eq!(page.items.len(), 20);
    assert!(page.has_more);
    assert_eq!(page.page, 2);
    assert!(fixture
        .client()
        .changes(connection.clone(), 0)
        .await
        .is_err());
    assert!(fixture
        .client()
        .change(connection.clone(), 0)
        .await
        .is_err());
    for mode in [
        "oversized",
        "malformed",
        "bad_url",
        "authentication",
        "rate_limit",
    ] {
        fixture.mode(mode);
        let error = fixture
            .client()
            .change(connection.clone(), 7)
            .await
            .unwrap_err();
        assert!(!error.to_string().contains("PRIVATE"));
        assert!(match mode {
            "oversized" => matches!(error, RepositoryError::TooLarge),
            "authentication" => matches!(error, RepositoryError::Authentication),
            "rate_limit" => matches!(error, RepositoryError::RateLimited),
            _ => matches!(error, RepositoryError::Malformed),
        });
    }
    fixture.mode("timeout");
    let client = fixture.client().with_deadline(Duration::from_millis(50));
    assert!(matches!(
        client.changes(connection.clone(), 1).await,
        Err(RepositoryError::TimedOut)
    ));
    let client = fixture.client().with_deadline(Duration::from_millis(150));
    let first = client.changes(connection.clone(), 1);
    let second = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        client.changes(connection, 1).await
    };
    let (first, second) = tokio::join!(first, second);
    assert!(matches!(first, Err(RepositoryError::TimedOut)));
    assert!(matches!(second, Err(RepositoryError::Busy)));
}

#[tokio::test]
async fn missing_cli_and_provider_mismatch_do_not_create_connections_or_leak_secrets() {
    let fixture = Fixture::new(RepositoryProvider::Github);
    let state = fixture.state.clone().with_repository_client(
        RepositoryClient::new(None, None, fixture.directory.path().into()).unwrap(),
    );
    let (status, body) = api(
        &state,
        "/v1/commands",
        Some(connect(&fixture, RepositoryProvider::Github)),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "missing_cli");
    assert!(body["error"].as_str().unwrap().contains("Install gh"));
    let (status, _) = api(
        &fixture.state,
        "/v1/commands",
        Some(connect(&fixture, RepositoryProvider::Gitlab)),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(fixture.log().is_empty());
    assert!(fixture
        .state
        .store
        .snapshot()
        .unwrap()
        .repository_connections
        .is_empty());
    assert!(Command::new("/usr/bin/git")
        .arg("-C")
        .arg(&fixture.repo)
        .args([
            "remote",
            "set-url",
            "origin",
            "https://private:secret@github.com/owner/repo"
        ])
        .status()
        .unwrap()
        .success());
    let (status, body) = api(
        &fixture.state,
        "/v1/commands",
        Some(connect(&fixture, RepositoryProvider::Github)),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(!body.to_string().contains("private:secret"));
    assert!(fixture.log().is_empty());
    let database = Connection::open(&fixture.database).unwrap();
    let count: i64 = database
        .query_row(
            "SELECT count(*) FROM commands WHERE kind='connect_repository'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn repository_endpoints_share_daemon_auth_and_connection_requires_read_access() {
    let fixture = Fixture::new(RepositoryProvider::Github);
    for route in ["detect", "changes", "change"] {
        let (status, _) = api(
            &fixture.state,
            &format!("/v1/repository/{route}?projectId={}", fixture.project.id),
            None,
            false,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    fixture.mode("authentication");
    let (status, body) = api(
        &fixture.state,
        "/v1/commands",
        Some(connect(&fixture, RepositoryProvider::Github)),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body["code"], "provider_authentication");
    assert!(!body.to_string().contains("PRIVATE"));
    assert!(fixture
        .state
        .store
        .snapshot()
        .unwrap()
        .repository_connections
        .is_empty());
}

#[test]
fn schema_two_migrates_additively_and_connection_events_are_atomic() {
    let fixture = Fixture::new(RepositoryProvider::Github);
    let db = Connection::open(&fixture.database).unwrap();
    db.execute_batch("DROP TABLE repository_connections; PRAGMA user_version=2;")
        .unwrap();
    let store = Store::open(&fixture.database).unwrap();
    let schema: u32 = db
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(schema, 3);
    assert_eq!(store.snapshot().unwrap().projects[0].id, fixture.project.id);
    db.execute_batch("CREATE TRIGGER reject_repository_event BEFORE INSERT ON events WHEN NEW.kind='repository_connected' BEGIN SELECT RAISE(ABORT,'fixture event failure'); END;").unwrap();
    let target = parse_remote("https://github.com/owner/repo").unwrap();
    assert!(store
        .connect_repository("atomic-connect", &fixture.project.id, target.clone())
        .is_err());
    assert!(store.snapshot().unwrap().repository_connections.is_empty());
    db.execute_batch("DROP TRIGGER reject_repository_event")
        .unwrap();
    store
        .connect_repository("atomic-connect", &fixture.project.id, target)
        .unwrap();
    assert_eq!(store.snapshot().unwrap().repository_connections.len(), 1);
    let mut captured =
        serde_json::to_value(store.repository_connection(&fixture.project.id).unwrap()).unwrap();
    captured.as_object_mut().unwrap().remove("remoteName");
    let previous: RepositoryConnection = serde_json::from_value(captured).unwrap();
    assert_eq!(
        previous.remote_name, "origin",
        "existing schema3 connections retain origin provenance"
    );
}
