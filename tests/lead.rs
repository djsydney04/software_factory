use software_factory::{lead::wake_if_needed, model::*, service::ServiceState, store::Store};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command, time::Duration};
use tempfile::TempDir;
use tokio::sync::watch;

struct Fixture {
    _temp: TempDir,
    database: PathBuf,
    executable: PathBuf,
    project: Project,
    state: ServiceState,
}

impl Fixture {
    fn new(scenario: &str, configured: bool) -> Self {
        let temp = TempDir::new().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir(&repo).unwrap();
        assert!(Command::new("/usr/bin/git")
            .args(["init", "--quiet"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        let provider = temp.path().join("provider");
        fs::create_dir(&provider).unwrap();
        let executable = provider.join("codex");
        fs::write(&executable, include_bytes!("fixtures/provider_fake.py")).unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(provider.join("scenario"), scenario).unwrap();
        let database = temp.path().join("factory.sqlite3");
        let store = Store::open(&database).unwrap();
        let project = store
            .create_project(
                "register",
                NewProject {
                    name: "Lead fixture".into(),
                    repo_path: repo.to_string_lossy().into(),
                    objective: "Maintain this fixture".into(),
                    settings: ProjectSettings {
                        model: if configured {
                            "gpt-6.1-sol".into()
                        } else {
                            String::new()
                        },
                        max_turn_seconds: 5,
                        max_output_tokens: 1000,
                        ..Default::default()
                    },
                },
            )
            .unwrap();
        let state = ServiceState::new(store, "a".repeat(64), "127.0.0.1:38421".parse().unwrap());
        Self {
            _temp: temp,
            database,
            executable,
            project,
            state,
        }
    }

    fn steering(&self, command: &str) {
        self.state
            .store
            .add_steering(
                command,
                &self.project.id,
                "Keep changes focused and explain the result.",
            )
            .unwrap();
    }

    async fn wake(&self) -> bool {
        let (_stop, signal) = watch::channel(false);
        wake_if_needed(self.state.clone(), self.executable.clone(), signal)
            .await
            .unwrap()
    }

    fn turn_count(&self) -> u64 {
        fs::read_dir(self.executable.parent().unwrap())
            .unwrap()
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("turn-count-")
                    .then(|| fs::read_to_string(path).unwrap().parse::<u64>().unwrap())
            })
            .sum()
    }

    fn durable_row_counts(&self) -> (i64, i64) {
        let connection = rusqlite::Connection::open(&self.database).unwrap();
        (
            connection
                .query_row("SELECT count(*) FROM commands", [], |row| row.get(0))
                .unwrap(),
            connection
                .query_row("SELECT count(*) FROM events", [], |row| row.get(0))
                .unwrap(),
        )
    }

    fn ready_task(&self) -> (Request, Task) {
        let request = self
            .state
            .store
            .submit_request(
                "intent",
                NewRequest {
                    project_id: self.project.id.clone(),
                    text: "Check the fixture".into(),
                    settings: None,
                    policy: RequestPolicy::LeadDecides,
                    scheduled_at: None,
                    time_zone: None,
                    source: "conversation".into(),
                },
            )
            .unwrap();
        let task = self
            .state
            .store
            .plan_task(
                "plan",
                NewTask {
                    project_id: self.project.id.clone(),
                    request_id: request.id.clone(),
                    title: "Fixture check".into(),
                    description: "Run the fixture check".into(),
                    acceptance: vec!["The check passes".into()],
                    checks: vec![CheckSpec {
                        program: "/usr/bin/true".into(),
                        args: vec![],
                        cwd: ".".into(),
                        timeout_seconds: 1,
                    }],
                    depends_on: vec![],
                },
            )
            .unwrap();
        (request, task)
    }
}

#[tokio::test]
async fn steering_has_one_read_only_lead_turn_and_stays_deduplicated_after_reopening() {
    let f = Fixture::new("factory", true);
    f.steering("steering-1");
    assert!(f.wake().await);
    let snapshot = Store::open(&f.database).unwrap().snapshot().unwrap();
    assert_eq!(snapshot.lead_wakes.len(), 1);
    let wake = &snapshot.lead_wakes[0];
    assert_eq!(wake.status, LeadWakeStatus::Completed);
    assert_eq!(wake.context_revision, 2);
    assert_eq!(wake.source_revision, Some(2));
    assert_eq!(wake.thread_id, snapshot.projects[0].lead_thread_id);
    assert!(wake.turn_id.is_some() && wake.summary.is_some());
    let settings = wake.effective_settings.as_ref().unwrap();
    assert_eq!(settings.access, Access::ReadOnly);
    assert_eq!(
        settings.permissions,
        Permissions {
            commands: Permission::Deny,
            network: Permission::Deny,
            integrations: Permission::Deny
        }
    );
    assert!(snapshot.requests.is_empty() && snapshot.tasks.is_empty() && snapshot.runs.is_empty());
    assert!(!f.wake().await);
    assert_eq!(f.turn_count(), 1);
    f.steering("steering-2");
    assert!(f.wake().await);
    let reopened = Store::open(&f.database).unwrap().snapshot().unwrap();
    assert_eq!(reopened.lead_wakes.len(), 2);
    assert_eq!(
        reopened.lead_wakes[0].thread_id,
        reopened.lead_wakes[1].thread_id
    );
    assert_eq!(f.turn_count(), 2);
}

#[tokio::test]
async fn unchanged_idle_polls_never_append_commands_or_events() {
    let f = Fixture::new("factory", true);
    let empty_counts = f.durable_row_counts();
    for _ in 0..8 {
        assert!(!f.wake().await);
    }
    assert_eq!(f.durable_row_counts(), empty_counts);
    f.steering("steering");
    assert!(f.wake().await);
    let completed_counts = f.durable_row_counts();
    let reopened = Store::open(&f.database).unwrap();
    let state = ServiceState::new(reopened, "a".repeat(64), "127.0.0.1:38421".parse().unwrap());
    let (_stop, signal) = watch::channel(false);
    for _ in 0..8 {
        assert!(
            !wake_if_needed(state.clone(), f.executable.clone(), signal.clone())
                .await
                .unwrap()
        );
    }
    assert_eq!(f.durable_row_counts(), completed_counts);
    assert_eq!(f.turn_count(), 1);
}

#[tokio::test]
async fn one_completed_wake_applies_all_steering_captured_in_its_context() {
    let f = Fixture::new("factory", true);
    f.steering("first-steering");
    f.steering("second-steering");
    assert!(f.wake().await);
    let snapshot = f.state.store.snapshot().unwrap();
    assert_eq!(snapshot.lead_wakes.len(), 1);
    assert_eq!(snapshot.lead_wakes[0].context_revision, 3);
    assert_eq!(snapshot.lead_wakes[0].context.steering.len(), 2);
    assert_eq!(
        f.state
            .store
            .applied_lead_context_revision(&f.project.id)
            .unwrap(),
        3
    );
    let counts = f.durable_row_counts();
    assert!(!f.wake().await);
    assert_eq!(f.durable_row_counts(), counts);
    assert_eq!(f.turn_count(), 1);
}

#[tokio::test]
async fn request_context_application_covers_steering_even_beyond_the_snapshot_event_window() {
    let f = Fixture::new("factory", true);
    f.steering("steering");
    f.state
        .store
        .append_event(
            "covered",
            &f.project.id,
            None,
            "lead_context_applied",
            serde_json::json!({"contextRevision":2,"requestId":"observed-request"}),
        )
        .unwrap();
    for i in 0..501 {
        f.state
            .store
            .append_event(
                &format!("later-{i}"),
                &f.project.id,
                None,
                "fixture",
                serde_json::json!({}),
            )
            .unwrap();
    }
    assert!(!f.wake().await);
    assert_eq!(f.turn_count(), 0);
    assert!(f.state.store.snapshot().unwrap().lead_wakes.is_empty());
}

#[tokio::test]
async fn completed_attempt_gets_one_summary_without_creating_another_task_or_attempt() {
    let f = Fixture::new("factory", true);
    let (_, task) = f.ready_task();
    let run = f.state.store.claim_task("claim", &task.id).unwrap();
    f.state
        .store
        .update_run(
            "observed-failure",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Failed),
                reason: Some("Fixture failed before a provider process was launched".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.wake().await);
    let snapshot = f.state.store.snapshot().unwrap();
    assert_eq!(snapshot.requests.len(), 1);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.runs.len(), 1);
    assert_eq!(snapshot.runs[0].status, RunStatus::Failed);
    assert_eq!(snapshot.lead_wakes[0].kind, LeadWakeKind::CompletedRun);
    assert_eq!(snapshot.lead_wakes[0].source_id, run.id);
    assert!(snapshot
        .events
        .iter()
        .any(|event| event.kind == "lead_wake_completed"
            && event.run_id.as_deref() == Some(run.id.as_str())));
    assert!(!f.wake().await);
    assert_eq!(f.turn_count(), 1);
}

#[tokio::test]
async fn unconfigured_model_waits_once_without_a_process_and_configuration_releases_the_same_wake()
{
    let f = Fixture::new("factory", false);
    f.steering("steering");
    assert!(f.wake().await);
    let waiting = f.state.store.snapshot().unwrap();
    assert_eq!(waiting.lead_wakes[0].status, LeadWakeStatus::Waiting);
    assert!(waiting.lead_wakes[0]
        .reason
        .as_deref()
        .unwrap()
        .contains("Select a supported model"));
    assert!(!f.wake().await);
    let waiting_counts = f.durable_row_counts();
    assert!(!f.wake().await);
    assert_eq!(f.durable_row_counts(), waiting_counts);
    assert_eq!(
        f.state.store.snapshot().unwrap().last_event_sequence,
        waiting.last_event_sequence
    );
    assert!(fs::read_dir(f.executable.parent().unwrap())
        .unwrap()
        .all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("methods-")));
    let mut settings = f.project.settings.clone();
    settings.model = "gpt-6.1-sol".into();
    f.state
        .store
        .save_settings("configured", &f.project.id, settings)
        .unwrap();
    assert!(f.wake().await);
    let done = Store::open(&f.database).unwrap().snapshot().unwrap();
    assert_eq!(done.lead_wakes.len(), 1);
    assert_eq!(done.lead_wakes[0].id, waiting.lead_wakes[0].id);
    assert_eq!(done.lead_wakes[0].requested_settings.model, "");
    assert_eq!(
        done.lead_wakes[0]
            .effective_settings
            .as_ref()
            .unwrap()
            .model,
        "gpt-6.1-sol"
    );
    assert_eq!(done.lead_wakes[0].status, LeadWakeStatus::Completed);
}

#[tokio::test]
async fn steering_wait_is_retired_when_a_request_has_already_applied_it() {
    let f = Fixture::new("factory", false);
    f.steering("steering");
    assert!(f.wake().await);
    f.state
        .store
        .append_event(
            "request-applied",
            &f.project.id,
            None,
            "lead_context_applied",
            serde_json::json!({"contextRevision":2,"requestId":"observed-request"}),
        )
        .unwrap();
    let mut settings = f.project.settings.clone();
    settings.model = "gpt-6.1-sol".into();
    f.state
        .store
        .save_settings("configured", &f.project.id, settings)
        .unwrap();
    assert!(!f.wake().await);
    let snapshot = f.state.store.snapshot().unwrap();
    assert_eq!(snapshot.lead_wakes[0].status, LeadWakeStatus::Covered);
    assert!(snapshot.lead_wakes[0].turn_id.is_none());
    assert!(snapshot
        .events
        .iter()
        .any(|event| event.kind == "lead_wake_covered"));
    assert_eq!(f.turn_count(), 0);
}

#[tokio::test]
async fn observed_terminal_failure_is_quiet_and_does_not_fence_new_intent() {
    let f = Fixture::new("failure", true);
    f.steering("steering");
    assert!(f.wake().await);
    assert_eq!(
        f.state.store.snapshot().unwrap().lead_wakes[0].status,
        LeadWakeStatus::Failed
    );
    assert!(!f.wake().await);
    let (_, task) = f.ready_task();
    assert!(f
        .state
        .store
        .claim_task("claim-after-known-failure", &task.id)
        .is_ok());
    let methods = fs::read_to_string(PathBuf::from(&f.project.repo_path).join("methods")).unwrap();
    assert_eq!(
        methods
            .lines()
            .filter(|method| *method == "turn/start")
            .count(),
        1
    );
}

#[tokio::test]
async fn uncertain_turn_fences_planning_and_claims_and_is_never_replayed() {
    let f = Fixture::new("lost_start_eof", true);
    let (_, task) = f.ready_task();
    f.steering("steering");
    assert!(f.wake().await);
    let snapshot = f.state.store.snapshot().unwrap();
    let wake = &snapshot.lead_wakes[0];
    assert_eq!(wake.status, LeadWakeStatus::RecoveryRequired);
    assert!(wake.thread_id.is_some() && wake.turn_id.is_some());
    assert!(f
        .state
        .store
        .claim_task("forbidden-claim", &task.id)
        .is_err());
    let pending = f
        .state
        .store
        .submit_request(
            "next-request",
            NewRequest {
                project_id: f.project.id.clone(),
                text: "New intent".into(),
                settings: None,
                policy: RequestPolicy::LeadDecides,
                scheduled_at: None,
                time_zone: None,
                source: "conversation".into(),
            },
        )
        .unwrap();
    assert!(f
        .state
        .store
        .mark_request_planning(
            "forbidden-plan",
            &pending.id,
            wake.thread_id.as_deref().unwrap(),
            None
        )
        .is_err());
    assert!(!f.wake().await);
    let methods = fs::read_to_string(PathBuf::from(&f.project.repo_path).join("methods")).unwrap();
    assert_eq!(
        methods
            .lines()
            .filter(|method| *method == "turn/start")
            .count(),
        1
    );
    assert!(Store::open(&f.database)
        .unwrap()
        .snapshot()
        .unwrap()
        .runs
        .is_empty());
}

#[tokio::test]
async fn shutdown_interrupts_an_owned_turn_and_observes_a_terminal_outcome() {
    let f = Fixture::new("interrupt", true);
    f.steering("steering");
    let (stop, signal) = watch::channel(false);
    let operation = tokio::spawn(wake_if_needed(
        f.state.clone(),
        f.executable.clone(),
        signal,
    ));
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if f.state
                .store
                .snapshot()
                .unwrap()
                .lead_wakes
                .iter()
                .any(|wake| wake.turn_id.is_some())
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    stop.send(true).unwrap();
    assert!(tokio::time::timeout(Duration::from_secs(3), operation)
        .await
        .unwrap()
        .unwrap()
        .unwrap());
    assert_eq!(
        f.state.store.snapshot().unwrap().lead_wakes[0].status,
        LeadWakeStatus::Failed
    );
    assert!(!f.wake().await);
    let methods = fs::read_to_string(PathBuf::from(&f.project.repo_path).join("methods")).unwrap();
    assert!(methods.lines().any(|method| method == "turn/interrupt"));
}

#[tokio::test]
async fn interrupt_ack_without_terminal_requires_recovery_after_forced_group_stop() {
    let f = Fixture::new("interrupt_ack_only", true);
    f.steering("steering");
    let (stop, signal) = watch::channel(false);
    let operation = tokio::spawn(wake_if_needed(
        f.state.clone(),
        f.executable.clone(),
        signal,
    ));
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if f.state
                .store
                .snapshot()
                .unwrap()
                .lead_wakes
                .iter()
                .any(|wake| wake.turn_id.is_some())
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    stop.send(true).unwrap();
    assert!(tokio::time::timeout(Duration::from_secs(5), operation)
        .await
        .unwrap()
        .unwrap()
        .unwrap());
    assert_eq!(
        f.state.store.snapshot().unwrap().lead_wakes[0].status,
        LeadWakeStatus::RecoveryRequired
    );
    assert!(!f.wake().await);
}

#[tokio::test]
async fn unfinished_wake_after_restart_is_fenced_even_before_a_turn_identity_was_saved() {
    let f = Fixture::new("factory", true);
    f.steering("steering");
    let reserved = f
        .state
        .store
        .prepare_lead_wake("reserve-before-crash", &f.project.id)
        .unwrap()
        .unwrap();
    assert_eq!(reserved.status, LeadWakeStatus::Prepared);
    let reopened = Store::open(&f.database).unwrap();
    let recovered = reopened.recover_lead_wakes("restart").unwrap();
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].id, reserved.id);
    assert_eq!(recovered[0].status, LeadWakeStatus::RecoveryRequired);
    assert!(reopened
        .recover_lead_wakes("restart-again")
        .unwrap()
        .is_empty());
    assert!(!f.wake().await);
    assert_eq!(f.turn_count(), 0);
}

#[test]
fn schema_one_migration_preserves_existing_records_and_wake_reservation_rolls_back_with_its_event()
{
    let f = Fixture::new("factory", true);
    f.steering("steering");
    let connection = rusqlite::Connection::open(&f.database).unwrap();
    connection
        .execute_batch(
            "DROP TABLE repository_connections; DROP TABLE lead_wakes; PRAGMA user_version=1;",
        )
        .unwrap();
    let reopened = Store::open(&f.database).unwrap();
    let migrated = reopened.snapshot().unwrap();
    assert_eq!(migrated.projects[0].objective, "Maintain this fixture");
    assert_eq!(migrated.steering.len(), 1);
    connection.execute_batch("CREATE TRIGGER reject_wake_event BEFORE INSERT ON events WHEN NEW.kind='lead_wake_prepared' BEGIN SELECT RAISE(ABORT,'injected wake event failure'); END;").unwrap();
    assert!(reopened
        .prepare_lead_wake("transactional-reserve", &f.project.id)
        .is_err());
    let failed = Store::open(&f.database).unwrap().snapshot().unwrap();
    assert!(failed.lead_wakes.is_empty());
    assert_eq!(failed.last_event_sequence, migrated.last_event_sequence);
    connection
        .execute_batch("DROP TRIGGER reject_wake_event;")
        .unwrap();
    let wake = reopened
        .prepare_lead_wake("transactional-reserve", &f.project.id)
        .unwrap()
        .unwrap();
    assert_eq!(wake.status, LeadWakeStatus::Prepared);
    assert!(reopened
        .prepare_lead_wake("competing-reserve", &f.project.id)
        .unwrap()
        .is_none());
}

#[test]
fn concurrent_wake_claims_reserve_one_owned_turn() {
    let f = Fixture::new("factory", true);
    f.steering("steering");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|i| {
            let barrier = barrier.clone();
            let store = f.state.store.clone();
            let project_id = f.project.id.clone();
            std::thread::spawn(move || {
                barrier.wait();
                store
                    .prepare_lead_wake(&format!("claim-{i}"), &project_id)
                    .unwrap()
            })
        })
        .collect();
    assert_eq!(
        handles
            .into_iter()
            .filter_map(|handle| handle.join().unwrap())
            .count(),
        1
    );
    let snapshot = Store::open(&f.database).unwrap().snapshot().unwrap();
    assert_eq!(snapshot.lead_wakes.len(), 1);
    assert_eq!(
        snapshot
            .events
            .iter()
            .filter(|event| event.kind == "lead_wake_prepared")
            .count(),
        1
    );
}
