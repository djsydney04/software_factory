use software_factory::{
    model::*,
    runtime::Supervisor,
    service::{execute_command, CommandEnvelope, ServiceState},
    store::Store,
};
use std::{
    fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command, sync::Arc, time::Duration,
};
use tempfile::TempDir;
use tokio::sync::watch;

struct Fixture {
    _temp: TempDir,
    state: ServiceState,
    data: PathBuf,
    repo: PathBuf,
    executable: PathBuf,
    project: Project,
    request: Request,
}
impl Fixture {
    fn new(scenario: &str, commands: Permission) -> Self {
        let temp = TempDir::new().unwrap();
        let repo = temp.path().join("repo");
        let data = temp.path().join("data");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&data).unwrap();
        git(&repo, &["init"]);
        fs::write(repo.join("value.txt"), "pending\n").unwrap();
        fs::write(repo.join("check.py"),"from pathlib import Path\nassert Path('value.txt').read_text() == 'ready\\n', 'value must be ready'\n").unwrap();
        git(&repo, &["add", "--", "value.txt", "check.py"]);
        git(
            &repo,
            &[
                "-c",
                "user.name=Factory Fixture",
                "-c",
                "user.email=factory@fixture.invalid",
                "commit",
                "-m",
                "Fixture base",
            ],
        );
        let fake = temp.path().join("provider");
        fs::create_dir(&fake).unwrap();
        let executable = fake.join("codex");
        fs::write(&executable, include_bytes!("fixtures/provider_fake.py")).unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(fake.join("scenario"), scenario).unwrap();
        let store = Store::open(data.join("factory.sqlite3")).unwrap();
        let settings = ProjectSettings {
            model: "gpt-6.1-sol".into(),
            effort: "medium".into(),
            permissions: Permissions {
                commands,
                network: Permission::Deny,
                integrations: Permission::Deny,
            },
            max_turn_seconds: 10,
            max_output_tokens: 1000,
            ..ProjectSettings::default()
        };
        let project = store
            .create_project(
                "register",
                NewProject {
                    name: "Fixture".into(),
                    repo_path: repo.to_string_lossy().into_owned(),
                    objective: "Keep the fixture useful".into(),
                    settings,
                },
            )
            .unwrap();
        store
            .add_steering("steer", &project.id, "Keep the change small")
            .unwrap();
        let request = store
            .submit_request(
                "request",
                NewRequest {
                    project_id: project.id.clone(),
                    text: "Set the fixture to ready".into(),
                    settings: None,
                    policy: RequestPolicy::LeadDecides,
                    scheduled_at: None,
                    time_zone: Some("America/Los_Angeles".into()),
                    source: "conversation".into(),
                },
            )
            .unwrap();
        let state = ServiceState::new(store, "a".repeat(64), "127.0.0.1:38421".parse().unwrap())
            .with_data_dir(data.clone());
        Self {
            _temp: temp,
            state,
            data,
            repo,
            executable,
            project,
            request,
        }
    }
    fn supervisor(&self) -> Supervisor {
        Supervisor::new(
            self.state.clone(),
            self.data.clone(),
            self.executable.clone(),
        )
    }
}
fn git(repo: &std::path::Path, args: &[&str]) {
    let out = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
async fn until(mut condition: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !condition() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("observable state deadline");
}

#[tokio::test]
async fn request_to_real_worktree_checks_review_and_reopen_uses_one_attempt() {
    let f = Fixture::new("factory", Permission::Allow);
    let supervisor = f.supervisor();
    let (_stop, signal) = watch::channel(false);
    assert!(f.state.store.snapshot().unwrap().runs.is_empty());
    supervisor.tick(signal.clone()).await.unwrap();
    let planned = f.state.store.snapshot().unwrap();
    assert_eq!(planned.tasks.len(), 1);
    assert_eq!(planned.requests[0].status, RequestStatus::Planned);
    assert!(planned.runs.is_empty());
    supervisor.tick(signal.clone()).await.unwrap();
    let complete = f.state.store.snapshot().unwrap();
    assert_eq!(complete.runs.len(), 1);
    let run = &complete.runs[0];
    assert_eq!(run.status, RunStatus::Succeeded, "{:?}", run.reason);
    assert!(run.thread_id.is_some() && run.turn_id.is_some());
    assert_ne!(run.base_sha, run.head_sha);
    assert_eq!(run.checks[0].exit_code, Some(0));
    assert_eq!(
        run.checks[0].revision,
        run.head_sha.as_ref().unwrap().as_str()
    );
    assert_eq!(complete.reviews.len(), 1);
    assert!(complete.reviews[0].diff.contains("+ready"));
    assert_eq!(
        fs::read_to_string(f.repo.join("value.txt")).unwrap(),
        "pending\n"
    );
    assert_eq!(
        fs::read_to_string(PathBuf::from(run.worktree_path.as_ref().unwrap()).join("value.txt"))
            .unwrap(),
        "ready\n"
    );
    let accepted:CommandEnvelope=serde_json::from_value(serde_json::json!({"commandId":"accept-local","command":{"type":"record_local_review_decision","reviewId":complete.reviews[0].id,"decision":"accepted_local","feedback":"Looks good locally"}})).unwrap();
    execute_command(&f.state, accepted).unwrap();
    assert!(
        software_factory::service::observed_snapshot(&f.state)
            .unwrap()
            .review_sources[0]
            .source_current
    );
    fs::write(
        PathBuf::from(run.worktree_path.as_ref().unwrap()).join("value.txt"),
        "changed after review\n",
    )
    .unwrap();
    let replay=serde_json::from_value(serde_json::json!({"commandId":"accept-local","command":{"type":"record_local_review_decision","reviewId":complete.reviews[0].id,"decision":"accepted_local","feedback":"Looks good locally"}})).unwrap();
    assert_eq!(
        execute_command(&f.state, replay).unwrap()["decision"],
        "accepted_local"
    );
    assert!(
        !software_factory::service::observed_snapshot(&f.state)
            .unwrap()
            .review_sources[0]
            .source_current
    );
    assert_eq!(
        f.state.store.snapshot().unwrap().tasks[0].status,
        TaskStatus::Review
    );
    let restarted = f.supervisor();
    restarted.recover().unwrap();
    restarted.tick(signal).await.unwrap();
    let reopened = Store::open(f.data.join("factory.sqlite3"))
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(reopened.runs.len(), 1);
    assert_eq!(reopened.runs[0].id, run.id);
    assert_eq!(
        reopened.reviews[0].decision.as_deref(),
        Some("accepted_local")
    );
    assert_eq!(
        reopened.runs[0].context.objective,
        "Keep the fixture useful"
    );
    assert!(reopened
        .events
        .iter()
        .any(|e| e.kind == "lead_context_applied"));
}

#[tokio::test]
async fn failed_independent_check_retains_actual_exit_and_never_creates_success_review() {
    let f = Fixture::new("factory_failed_check", Permission::Allow);
    let supervisor = f.supervisor();
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    supervisor.tick(signal).await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.runs.len(), 1);
    assert_eq!(
        result.runs[0].status,
        RunStatus::Failed,
        "{:?}",
        result.runs[0].reason
    );
    assert_eq!(result.runs[0].checks[0].exit_code, Some(1));
    assert!(result.runs[0].checks[0]
        .output
        .contains("value must be ready"));
    assert!(result.reviews.is_empty());
}

#[tokio::test]
async fn lead_wait_and_project_pause_do_not_fabricate_execution() {
    let f = Fixture::new("factory_wait", Permission::Allow);
    let supervisor = f.supervisor();
    let (_stop, signal) = watch::channel(false);
    f.state
        .store
        .set_paused("pause", &f.project.id, true)
        .unwrap();
    supervisor.tick(signal.clone()).await.unwrap();
    assert_eq!(
        f.state.store.snapshot().unwrap().requests[0].status,
        RequestStatus::Pending
    );
    f.state
        .store
        .set_paused("resume", &f.project.id, false)
        .unwrap();
    supervisor.tick(signal.clone()).await.unwrap();
    supervisor.tick(signal).await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.requests[0].status, RequestStatus::Waiting);
    assert!(result.requests[0]
        .reason
        .as_deref()
        .unwrap()
        .contains("clearer request"));
    assert!(result.tasks.is_empty() && result.runs.is_empty());
}

#[tokio::test]
async fn ask_waits_for_durable_run_scoped_approval_before_worker_dispatch() {
    let f = Fixture::new("factory", Permission::Ask);
    let supervisor = Arc::new(f.supervisor());
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    let worker = {
        let supervisor = supervisor.clone();
        tokio::spawn(async move { supervisor.tick(signal).await.unwrap() })
    };
    until(|| !f.state.store.snapshot().unwrap().approvals.is_empty()).await;
    let waiting = f.state.store.snapshot().unwrap();
    assert_eq!(waiting.runs[0].status, RunStatus::NeedsApproval);
    assert!(waiting.runs[0].thread_id.is_none());
    assert!(waiting.runs[0].authorization.is_none());
    f.state
        .store
        .resolve_approval("allow-once", &waiting.approvals[0].id, true)
        .unwrap();
    worker.await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(
        result.runs[0].status,
        RunStatus::Succeeded,
        "{:?}",
        result.runs[0].reason
    );
    let authorization = result.runs[0].authorization.as_ref().unwrap();
    assert_eq!(
        authorization.approval_ids,
        vec![waiting.approvals[0].id.clone()]
    );
    assert_eq!(
        authorization.settings.permissions.commands,
        Permission::Allow
    );
    assert_eq!(
        result.runs[0].effective_settings.permissions.commands,
        Permission::Ask
    );
}

#[tokio::test]
async fn interrupt_acknowledgment_is_followed_by_observed_provider_stop() {
    let f = Fixture::new("factory_slow", Permission::Allow);
    let supervisor = Arc::new(f.supervisor());
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    let worker = {
        let supervisor = supervisor.clone();
        tokio::spawn(async move { supervisor.tick(signal).await.unwrap() })
    };
    until(|| {
        f.state
            .store
            .snapshot()
            .unwrap()
            .runs
            .iter()
            .any(|r| r.turn_id.is_some())
    })
    .await;
    let run = f.state.store.snapshot().unwrap().runs[0].clone();
    let command=serde_json::from_value(serde_json::json!({"commandId":"stop-once","command":{"type":"interrupt_run","runId":run.id}})).unwrap();
    execute_command(&f.state, command).unwrap();
    worker.await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(
        result.runs[0].status,
        RunStatus::Interrupted,
        "{:?}",
        result.runs[0].reason
    );
    assert!(result.reviews.is_empty());
    assert_eq!(unsafe { libc::kill(run.pid.unwrap() as i32, 0) }, -1);
}

#[tokio::test]
async fn restart_after_uncertain_claim_fences_successors_instead_of_relaunching() {
    let f = Fixture::new("factory", Permission::Allow);
    let supervisor = f.supervisor();
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    let task = f.state.store.snapshot().unwrap().tasks[0].clone();
    let claimed = f
        .state
        .store
        .claim_task("crash-after-claim", &task.id)
        .unwrap();
    let restarted = f.supervisor();
    restarted.recover().unwrap();
    restarted.tick(signal).await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.runs.len(), 1);
    assert_eq!(result.runs[0].id, claimed.id);
    assert_eq!(result.runs[0].status, RunStatus::RecoveryRequired);
    assert!(result.runs[0].thread_id.is_none());
    assert!(result.reviews.is_empty());
}

#[tokio::test]
async fn reduced_project_ceiling_invalidates_pending_approval_without_launch() {
    let f = Fixture::new("factory", Permission::Ask);
    let supervisor = Arc::new(f.supervisor());
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    let worker = {
        let supervisor = supervisor.clone();
        tokio::spawn(async move { supervisor.tick(signal).await.unwrap() })
    };
    until(|| !f.state.store.snapshot().unwrap().approvals.is_empty()).await;
    let waiting = f.state.store.snapshot().unwrap();
    let mut settings = f.project.settings.clone();
    settings.permissions.commands = Permission::Deny;
    f.state
        .store
        .save_settings("deny-now", &f.project.id, settings)
        .unwrap();
    assert!(f
        .state
        .store
        .resolve_approval("forbidden-allow", &waiting.approvals[0].id, true)
        .is_err());
    worker.await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.runs[0].status, RunStatus::Failed);
    assert!(result.runs[0].thread_id.is_none());
}

#[tokio::test]
async fn unresolved_lead_after_restart_fences_new_requests_and_worker_claims() {
    let f = Fixture::new("factory", Permission::Allow);
    f.state
        .store
        .set_lead_thread("lead-id", &f.project.id, "uncertain-lead")
        .unwrap();
    f.state
        .store
        .mark_request_planning(
            "planning",
            &f.request.id,
            "uncertain-lead",
            Some("uncertain-turn"),
        )
        .unwrap();
    f.state
        .store
        .submit_request(
            "fresh-intent",
            NewRequest {
                project_id: f.project.id.clone(),
                text: "A second request".into(),
                settings: None,
                policy: RequestPolicy::LeadDecides,
                scheduled_at: None,
                time_zone: None,
                source: "conversation".into(),
            },
        )
        .unwrap();
    let supervisor = f.supervisor();
    supervisor.recover().unwrap();
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal).await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(
        result
            .requests
            .iter()
            .find(|r| r.id == f.request.id)
            .unwrap()
            .status,
        RequestStatus::RecoveryRequired
    );
    assert_eq!(
        result
            .requests
            .iter()
            .filter(|r| r.status == RequestStatus::Pending)
            .count(),
        1
    );
    assert!(result.runs.is_empty() && result.tasks.is_empty());
    assert!(!fs::read_dir(f.executable.parent().unwrap())
        .unwrap()
        .any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("turn-count-")));
}

#[tokio::test]
async fn cancelled_ready_task_does_not_starve_later_eligible_intent() {
    let f = Fixture::new("factory", Permission::Allow);
    let supervisor = f.supervisor();
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    f.state
        .store
        .cancel_request("cancel-before-claim", &f.request.id)
        .unwrap();
    let second = f
        .state
        .store
        .submit_request(
            "second",
            NewRequest {
                project_id: f.project.id.clone(),
                text: "Set the fixture ready now".into(),
                settings: None,
                policy: RequestPolicy::NextAvailable,
                scheduled_at: None,
                time_zone: None,
                source: "conversation".into(),
            },
        )
        .unwrap();
    supervisor.tick(signal.clone()).await.unwrap();
    supervisor.tick(signal).await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.runs.len(), 1);
    assert_eq!(result.runs[0].request_id, second.id);
    assert_eq!(result.runs[0].status, RunStatus::Succeeded);
}

#[tokio::test]
async fn review_packet_write_failure_cannot_leave_a_succeeded_attempt() {
    let f = Fixture::new("factory", Permission::Allow);
    let supervisor = f.supervisor();
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    rusqlite::Connection::open(f.data.join("factory.sqlite3")).unwrap().execute_batch("CREATE TRIGGER reject_review BEFORE INSERT ON reviews BEGIN SELECT RAISE(ABORT,'injected review write failure'); END;").unwrap();
    supervisor.tick(signal).await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.runs[0].status, RunStatus::Failed);
    assert_eq!(result.runs[0].checks[0].exit_code, Some(0));
    assert!(result.reviews.is_empty());
    assert!(!result.events.iter().any(|e| e.kind == "review_ready"
        || e.kind == "run_updated" && e.payload["status"] == "succeeded"));
}

#[tokio::test]
async fn permission_revocation_during_a_check_stops_control_and_fences_unresolved_activity() {
    let f = Fixture::new("factory_long_check", Permission::Allow);
    fs::write(f.repo.join(".gitignore"), "check-started\n").unwrap();
    fs::write(f.repo.join("check.py"),"from pathlib import Path\nimport time\nPath('check-started').write_text('started')\ntime.sleep(30)\nassert Path('value.txt').read_text() == 'ready\\n'\n").unwrap();
    git(&f.repo, &["add", "--", ".gitignore", "check.py"]);
    git(
        &f.repo,
        &[
            "-c",
            "user.name=Factory Fixture",
            "-c",
            "user.email=factory@fixture.invalid",
            "commit",
            "-m",
            "Long check fixture",
        ],
    );
    let supervisor = Arc::new(f.supervisor());
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    let worker = {
        let supervisor = supervisor.clone();
        tokio::spawn(async move { supervisor.tick(signal).await.unwrap() })
    };
    until(|| {
        f.state.store.snapshot().unwrap().runs.iter().any(|r| {
            r.status == RunStatus::Checking
                && r.worktree_path
                    .as_ref()
                    .is_some_and(|p| PathBuf::from(p).join("check-started").exists())
        })
    })
    .await;
    let mut settings = f.project.settings.clone();
    settings.permissions.commands = Permission::Deny;
    f.state
        .store
        .save_settings("revoke-check", &f.project.id, settings)
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3), worker)
        .await
        .expect("revocation must stop promptly")
        .unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.runs[0].status, RunStatus::RecoveryRequired);
    assert!(result.reviews.is_empty());
    assert_eq!(
        unsafe { libc::kill(result.runs[0].pid.unwrap() as i32, 0) },
        -1
    );
}

#[tokio::test]
async fn forced_interrupt_without_terminal_keeps_an_unresolved_attempt_owned() {
    let f = Fixture::new("factory_ack_only", Permission::Allow);
    let supervisor = Arc::new(f.supervisor());
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    let worker = {
        let supervisor = supervisor.clone();
        let signal = signal.clone();
        tokio::spawn(async move { supervisor.tick(signal).await.unwrap() })
    };
    until(|| {
        f.state
            .store
            .snapshot()
            .unwrap()
            .runs
            .iter()
            .any(|r| r.turn_id.is_some())
    })
    .await;
    let run = f.state.store.snapshot().unwrap().runs[0].clone();
    f.state.store.interrupt_run("stop", &run.id).unwrap();
    worker.await.unwrap();
    supervisor.tick(signal).await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.runs.len(), 1);
    assert_eq!(result.runs[0].status, RunStatus::RecoveryRequired);
    assert!(result.reviews.is_empty());
}

#[tokio::test]
async fn full_review_queue_still_allows_one_bounded_lead_result_summary() {
    let f = Fixture::new("factory", Permission::Allow);
    let mut settings = f.project.settings.clone();
    settings.review_queue_cap = 1;
    f.state
        .store
        .save_settings("one-review", &f.project.id, settings)
        .unwrap();
    let supervisor = f.supervisor();
    let (_stop, signal) = watch::channel(false);
    supervisor.tick(signal.clone()).await.unwrap();
    supervisor.tick(signal.clone()).await.unwrap();
    assert_eq!(f.state.store.snapshot().unwrap().reviews.len(), 1);
    supervisor.tick(signal.clone()).await.unwrap();
    supervisor.tick(signal).await.unwrap();
    let result = f.state.store.snapshot().unwrap();
    assert_eq!(result.runs.len(), 1);
    assert_eq!(result.tasks.len(), 1);
    assert_eq!(
        result
            .lead_wakes
            .iter()
            .filter(
                |w| w.kind == LeadWakeKind::CompletedRun && w.status == LeadWakeStatus::Completed
            )
            .count(),
        1
    );
}
