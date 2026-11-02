use rusqlite::Connection;
use software_factory::{
    model::*,
    store::{Store, StoreError},
};
use std::{
    path::PathBuf,
    process::Command,
    sync::{Arc, Barrier},
    thread,
};
use tempfile::TempDir;

struct Fixture {
    _directory: TempDir,
    path: PathBuf,
    store: Store,
    project: Project,
}
fn fixture() -> Fixture {
    let directory = tempfile::tempdir().unwrap();
    let repo = directory.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    assert!(Command::new("git")
        .args(["init", "--quiet"])
        .arg(&repo)
        .status()
        .unwrap()
        .success());
    let path = directory.path().join("factory.sqlite");
    let store = Store::open(&path).unwrap();
    let settings = ProjectSettings {
        model: "observed-test-model".into(),
        ..ProjectSettings::default()
    };
    let project = store
        .create_project(
            "project",
            NewProject {
                name: "Local fixture".into(),
                repo_path: repo.to_string_lossy().into_owned(),
                objective: "Make the disposable fixture correct".into(),
                settings,
            },
        )
        .unwrap();
    Fixture {
        _directory: directory,
        path,
        store,
        project,
    }
}
fn request(fixture: &Fixture, command: &str) -> Request {
    fixture
        .store
        .submit_request(
            command,
            NewRequest {
                project_id: fixture.project.id.clone(),
                text: "Fix the fixture".into(),
                settings: None,
                policy: RequestPolicy::LeadDecides,
                scheduled_at: None,
                time_zone: None,
                source: "conversation".into(),
            },
        )
        .unwrap()
}
fn task(fixture: &Fixture, command: &str) -> Task {
    let request = request(fixture, &format!("{command}-request"));
    fixture
        .store
        .plan_task(
            command,
            NewTask {
                project_id: fixture.project.id.clone(),
                request_id: request.id,
                title: "Fix fixture".into(),
                description: "Produce one independently checked change".into(),
                acceptance: vec!["Executable fixture check passes".into()],
                checks: vec![CheckSpec {
                    program: "node".into(),
                    args: vec!["check.js".into()],
                    cwd: String::new(),
                    timeout_seconds: 10,
                }],
                depends_on: vec![],
            },
        )
        .unwrap()
}
fn sha(digit: char) -> String {
    std::iter::repeat_n(digit, 40).collect()
}
fn evidence(revision: &str, exit_code: Option<i32>) -> CheckEvidence {
    CheckEvidence {
        name: "fixture check".into(),
        program: "node".into(),
        args: vec!["check.js".into()],
        cwd: String::new(),
        revision: revision.into(),
        exit_code,
        timed_out: false,
        output: "supervisor output".into(),
        started_at: "2026-11-01T00:00:00Z".into(),
        finished_at: "2026-11-01T00:00:01Z".into(),
    }
}
fn checked_run(fixture: &Fixture, task: &Task) -> RunAttempt {
    let run = fixture.store.claim_task("claim", &task.id).unwrap();
    fixture
        .store
        .update_run(
            "start",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Running),
                thread_id: Some("worker-thread".into()),
                turn_id: Some("worker-turn".into()),
                ..Default::default()
            },
        )
        .unwrap();
    fixture
        .store
        .update_run(
            "checking",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Checking),
                head_sha: Some(sha('a')),
                ..Default::default()
            },
        )
        .unwrap();
    fixture
        .store
        .update_run(
            "succeeded",
            RunUpdate {
                run_id: run.id,
                status: Some(RunStatus::Succeeded),
                checks: Some(vec![evidence(&sha('a'), Some(0))]),
                ..Default::default()
            },
        )
        .unwrap()
}

#[test]
fn reopened_store_preserves_objective_steering_snapshots_and_ordered_events() {
    let fixture = fixture();
    fixture
        .store
        .set_lead_thread("lead", &fixture.project.id, "persistent-lead")
        .unwrap();
    fixture
        .store
        .add_steering("steer", &fixture.project.id, "Keep the change small")
        .unwrap();
    let request = request(&fixture, "request");
    let settings = ProjectSettings {
        permissions: Permissions {
            commands: Permission::Deny,
            network: Permission::Deny,
            integrations: Permission::Deny,
        },
        model: "new-observed-model".into(),
        ..ProjectSettings::default()
    };
    fixture
        .store
        .save_settings("settings", &fixture.project.id, settings)
        .unwrap();
    let snapshot = Store::open(&fixture.path).unwrap().snapshot().unwrap();
    assert_eq!(
        snapshot.projects[0].objective,
        "Make the disposable fixture correct"
    );
    assert_eq!(
        snapshot.projects[0].lead_thread_id.as_deref(),
        Some("persistent-lead")
    );
    assert_eq!(snapshot.projects[0].context_revision, 2);
    assert_eq!(snapshot.steering[0].text, "Keep the change small");
    assert_eq!(snapshot.requests[0].id, request.id);
    assert_eq!(snapshot.requests[0].settings.model, "observed-test-model");
    assert_eq!(
        snapshot.requests[0].settings.permissions.commands,
        Permission::Ask
    );
    assert_eq!(
        snapshot.projects[0].settings.permissions.commands,
        Permission::Deny
    );
    assert_eq!(snapshot.last_event_sequence, 5);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&fixture.path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert_eq!(
        snapshot
            .events
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );
}

#[test]
fn event_failure_rolls_back_state_revision_and_command_identity() {
    let fixture = fixture();
    let connection = Connection::open(&fixture.path).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_steering_event BEFORE INSERT ON events WHEN NEW.kind='steering_received' BEGIN SELECT RAISE(ABORT,'injected event failure'); END;").unwrap();
    assert!(fixture
        .store
        .add_steering(
            "retryable-command",
            &fixture.project.id,
            "Do not lose this guidance"
        )
        .is_err());
    let snapshot = fixture.store.snapshot().unwrap();
    assert_eq!(snapshot.projects[0].context_revision, 1);
    assert!(snapshot.steering.is_empty());
    assert_eq!(snapshot.last_event_sequence, 1);
    connection
        .execute_batch("DROP TRIGGER reject_steering_event")
        .unwrap();
    let steering = fixture
        .store
        .add_steering(
            "retryable-command",
            &fixture.project.id,
            "Do not lose this guidance",
        )
        .unwrap();
    assert_eq!(steering.revision, 2);
    assert_eq!(fixture.store.snapshot().unwrap().last_event_sequence, 2);
}

#[test]
fn duplicate_command_replays_once_and_conflicting_payload_is_rejected() {
    let fixture = fixture();
    let first = request(&fixture, "same-command");
    let second = request(&fixture, "same-command");
    assert_eq!(first.id, second.id);
    let conflict =
        fixture
            .store
            .add_steering("same-command", &fixture.project.id, "Different operation");
    assert!(matches!(conflict, Err(StoreError::Conflict(_))));
    let snapshot = fixture.store.snapshot().unwrap();
    assert_eq!(snapshot.requests.len(), 1);
    assert_eq!(snapshot.last_event_sequence, 2);
    assert!(
        snapshot.runs.is_empty(),
        "saving intent must not manufacture an execution attempt"
    );
}

#[test]
fn concurrent_sqlite_claims_create_one_active_attempt() {
    let fixture = fixture();
    let task = task(&fixture, "task");
    let barrier = Arc::new(Barrier::new(8));
    let handles: Vec<_> = (0..8)
        .map(|index| {
            let store = fixture.store.clone();
            let task_id = task.id.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                store
                    .claim_task(&format!("claim-{index}"), &task_id)
                    .is_ok()
            })
        })
        .collect();
    assert_eq!(
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|claimed| *claimed)
            .count(),
        1
    );
    let snapshot = fixture.store.snapshot().unwrap();
    assert_eq!(snapshot.runs.len(), 1);
    assert_eq!(snapshot.runs[0].attempt, 1);
    assert_eq!(snapshot.runs[0].status, RunStatus::Claimed);
}

#[test]
fn pause_current_permission_ceiling_and_attempt_limit_gate_claims() {
    let fixture = fixture();
    let task = task(&fixture, "task");
    fixture
        .store
        .set_paused("pause", &fixture.project.id, true)
        .unwrap();
    assert!(fixture.store.claim_task("while-paused", &task.id).is_err());
    assert!(fixture.store.snapshot().unwrap().runs.is_empty());
    fixture
        .store
        .set_paused("resume", &fixture.project.id, false)
        .unwrap();
    let mut settings = fixture.project.settings.clone();
    settings.access = Access::ReadOnly;
    settings.permissions.commands = Permission::Deny;
    fixture
        .store
        .save_settings("narrow", &fixture.project.id, settings)
        .unwrap();
    let run = fixture.store.claim_task("claim", &task.id).unwrap();
    assert_eq!(
        run.effective_settings.permissions.commands,
        Permission::Deny
    );
    assert_eq!(run.effective_settings.access, Access::ReadOnly);
    assert_eq!(run.requested_settings.permissions.commands, Permission::Ask);
    fixture
        .store
        .update_run(
            "fail",
            RunUpdate {
                run_id: run.id,
                status: Some(RunStatus::Failed),
                reason: Some("enforcement unsupported".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        fixture.store.claim_task("retry", &task.id).is_err(),
        "the captured one-attempt ceiling must survive later claims"
    );
}

#[test]
fn uncertain_restart_keeps_execution_owned_until_observed_terminal_outcome() {
    let fixture = fixture();
    let task = task(&fixture, "task");
    let run = fixture.store.claim_task("claim", &task.id).unwrap();
    fixture
        .store
        .update_run(
            "start",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Running),
                executor_id: Some("owner-one".into()),
                pid: Some(123),
                process_started_at: Some(456),
                ..Default::default()
            },
        )
        .unwrap();
    let reopened = Store::open(&fixture.path).unwrap();
    let recovered = reopened.mark_recovery_required("restart").unwrap();
    assert_eq!(recovered[0].status, RunStatus::RecoveryRequired);
    assert_eq!(recovered[0].pid, Some(123));
    assert!(reopened.claim_task("unsafe-retry", &task.id).is_err());
    assert!(reopened
        .update_run(
            "stale-writer",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Interrupted),
                executor_id: Some("different-owner".into()),
                ..Default::default()
            }
        )
        .is_err());
    reopened
        .update_run(
            "observed-stopped",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Interrupted),
                executor_id: Some("owner-one".into()),
                reason: Some(
                    "Process identity no longer exists; no provider completion was observed".into(),
                ),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        reopened.snapshot().unwrap().runs[0].status,
        RunStatus::Interrupted
    );
    assert!(reopened
        .update_run(
            "late-success",
            RunUpdate {
                run_id: run.id,
                status: Some(RunStatus::Succeeded),
                executor_id: Some("owner-one".into()),
                ..Default::default()
            }
        )
        .is_err());
}

#[test]
fn approval_waiting_is_nonterminal_and_cannot_broaden_reduced_project_ceiling() {
    let fixture = fixture();
    let task = task(&fixture, "task");
    let run = fixture.store.claim_task("claim", &task.id).unwrap();
    fixture
        .store
        .update_run(
            "start",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Running),
                ..Default::default()
            },
        )
        .unwrap();
    let approval = fixture
        .store
        .add_approval(
            "approval",
            &run.id,
            "provider-1",
            "commands",
            "Run node check.js in this worktree",
        )
        .unwrap();
    assert!(fixture
        .store
        .update_run(
            "premature-resume",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Running),
                ..Default::default()
            }
        )
        .is_err());
    assert_eq!(
        fixture.store.snapshot().unwrap().runs[0].status,
        RunStatus::NeedsApproval
    );
    assert!(fixture.store.snapshot().unwrap().runs[0]
        .finished_at
        .is_none());
    let mut settings = fixture.project.settings.clone();
    settings.permissions.commands = Permission::Deny;
    fixture
        .store
        .save_settings("deny", &fixture.project.id, settings)
        .unwrap();
    assert!(fixture
        .store
        .resolve_approval("unsafe-allow", &approval.id, true)
        .is_err());
    let denied = fixture
        .store
        .resolve_approval("deny-approval", &approval.id, false)
        .unwrap();
    assert_eq!(denied.decision, Some(false));
    assert_eq!(
        fixture.store.snapshot().unwrap().runs[0].status,
        RunStatus::NeedsApproval,
        "recording a decision does not fabricate a provider resume acknowledgment"
    );
}

#[test]
fn success_requires_observed_exact_revision_checks_and_review_acceptance_stays_local() {
    let fixture = fixture();
    let task = task(&fixture, "task");
    let run = fixture.store.claim_task("claim", &task.id).unwrap();
    fixture
        .store
        .update_run(
            "start",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Running),
                ..Default::default()
            },
        )
        .unwrap();
    fixture
        .store
        .update_run(
            "checking",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Checking),
                head_sha: Some(sha('a')),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(fixture
        .store
        .update_run(
            "unchecked",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Succeeded),
                ..Default::default()
            }
        )
        .is_err());
    assert!(fixture
        .store
        .update_run(
            "stale",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Succeeded),
                checks: Some(vec![evidence(&sha('b'), Some(0))]),
                ..Default::default()
            }
        )
        .is_err());
    assert!(fixture
        .store
        .update_run(
            "failed-check",
            RunUpdate {
                run_id: run.id.clone(),
                status: Some(RunStatus::Succeeded),
                checks: Some(vec![evidence(&sha('a'), Some(1))]),
                ..Default::default()
            }
        )
        .is_err());
    let run = fixture
        .store
        .update_run(
            "succeeded",
            RunUpdate {
                run_id: run.id,
                status: Some(RunStatus::Succeeded),
                checks: Some(vec![evidence(&sha('a'), Some(0))]),
                ..Default::default()
            },
        )
        .unwrap();
    let review = fixture
        .store
        .add_review(
            "review",
            NewReview {
                run_id: run.id,
                revision: sha('a'),
                diff: "diff --git a/check.js b/check.js".into(),
                checks: run.checks,
                artifacts: vec![],
                limitations: vec!["No screenshot required for this fixture".into()],
            },
        )
        .unwrap();
    assert!(fixture
        .store
        .decide_review(
            "stale-decision",
            &review.id,
            &sha('b'),
            "accepted_local",
            ""
        )
        .is_err());
    fixture
        .store
        .decide_review("accept", &review.id, &sha('a'), "accepted_local", "")
        .unwrap();
    let snapshot = Store::open(&fixture.path).unwrap().snapshot().unwrap();
    assert_eq!(
        snapshot.reviews[0].decision.as_deref(),
        Some("accepted_local")
    );
    assert_eq!(
        snapshot.tasks[0].status,
        TaskStatus::Review,
        "local acceptance is not delivered coding work"
    );
    assert!(fixture
        .store
        .decide_review("again", &review.id, &sha('a'), "accepted_local", "")
        .is_err());
}

#[test]
fn cancelled_or_future_intent_never_dispatches_and_checks_cannot_escape_worktree() {
    let fixture = fixture();
    let task = task(&fixture, "task");
    fixture
        .store
        .cancel_request("cancel", &task.request_id)
        .unwrap();
    assert!(fixture
        .store
        .claim_task("cancelled-claim", &task.id)
        .is_err());
    let request = request(&fixture, "another-request");
    assert!(fixture
        .store
        .plan_task(
            "escape",
            NewTask {
                project_id: fixture.project.id.clone(),
                request_id: request.id,
                title: "Escape".into(),
                description: "Unsafe check".into(),
                acceptance: vec!["Must fail".into()],
                checks: vec![CheckSpec {
                    program: "node".into(),
                    args: vec![],
                    cwd: "../outside".into(),
                    timeout_seconds: 10
                }],
                depends_on: vec![]
            }
        )
        .is_err());
    let scheduled = fixture
        .store
        .submit_request(
            "future",
            NewRequest {
                project_id: fixture.project.id.clone(),
                text: "Wait until the future".into(),
                settings: None,
                policy: RequestPolicy::Scheduled,
                scheduled_at: Some("2099-01-01T00:00:00Z".into()),
                time_zone: Some("America/Los_Angeles".into()),
                source: "conversation".into(),
            },
        )
        .unwrap();
    let scheduled_task = fixture
        .store
        .plan_task(
            "future-task",
            NewTask {
                project_id: fixture.project.id.clone(),
                request_id: scheduled.id,
                title: "Future".into(),
                description: "Earliest start is later".into(),
                acceptance: vec!["Do not start early".into()],
                checks: vec![CheckSpec {
                    program: "node".into(),
                    args: vec![],
                    cwd: String::new(),
                    timeout_seconds: 10,
                }],
                depends_on: vec![],
            },
        )
        .unwrap();
    assert!(fixture
        .store
        .claim_task("too-early", &scheduled_task.id)
        .is_err());
    assert!(fixture.store.snapshot().unwrap().runs.is_empty());
}

#[test]
fn source_bound_reviews_use_recorded_evidence_not_replaced_check_text() {
    let fixture = fixture();
    let task = task(&fixture, "task");
    let run = checked_run(&fixture, &task);
    let mut changed_evidence = run.checks.clone();
    changed_evidence[0].output = "provider self-reported substitute".into();
    assert!(fixture
        .store
        .add_review(
            "fabricated",
            NewReview {
                run_id: run.id,
                revision: sha('a'),
                diff: String::new(),
                checks: changed_evidence,
                artifacts: vec![],
                limitations: vec![]
            }
        )
        .is_err());
    assert!(fixture.store.snapshot().unwrap().reviews.is_empty());
}

#[test]
fn objective_and_later_steering_cannot_rewrite_submitted_or_started_context() {
    let fixture = fixture();
    fixture
        .store
        .add_steering("before", &fixture.project.id, "Keep the first criterion")
        .unwrap();
    let task = task(&fixture, "task");
    fixture
        .store
        .set_objective(
            "objective",
            &fixture.project.id,
            "A refined persistent objective",
        )
        .unwrap();
    let run = fixture.store.claim_task("claim", &task.id).unwrap();
    fixture
        .store
        .add_steering(
            "after",
            &fixture.project.id,
            "A later instruction for future work",
        )
        .unwrap();
    let snapshot = Store::open(&fixture.path).unwrap().snapshot().unwrap();
    assert_eq!(
        snapshot.requests[0].context.objective,
        "Make the disposable fixture correct"
    );
    assert_eq!(snapshot.requests[0].context.revision, 2);
    assert_eq!(
        snapshot.runs[0].context.objective,
        "A refined persistent objective"
    );
    assert_eq!(snapshot.runs[0].context.revision, 3);
    assert_eq!(snapshot.runs[0].context.steering.len(), 1);
    assert_eq!(
        snapshot.runs[0].context.steering[0].text,
        "Keep the first criterion"
    );
    assert_eq!(snapshot.projects[0].context_revision, 4);
    assert_eq!(snapshot.runs[0].id, run.id);
}

#[test]
fn forged_git_directory_is_rejected_and_project_creation_replays_after_repository_moves() {
    let directory = tempfile::tempdir().unwrap();
    let fake = directory.path().join("fake");
    std::fs::create_dir_all(fake.join(".git")).unwrap();
    let store = Store::open(directory.path().join("factory.sqlite")).unwrap();
    assert!(store
        .create_project(
            "bad-repo",
            NewProject {
                name: "Forged".into(),
                repo_path: fake.to_string_lossy().into_owned(),
                objective: "Do nothing unsafe".into(),
                settings: ProjectSettings::default()
            }
        )
        .is_err());
    assert!(store.snapshot().unwrap().projects.is_empty());
    assert_eq!(store.snapshot().unwrap().last_event_sequence, 0);
    let fixture = fixture();
    let repo = fixture._directory.path().join("repo");
    std::fs::rename(&repo, fixture._directory.path().join("moved")).unwrap();
    let replay = fixture
        .store
        .create_project(
            "project",
            NewProject {
                name: "Local fixture".into(),
                repo_path: repo.to_string_lossy().into_owned(),
                objective: "Make the disposable fixture correct".into(),
                settings: ProjectSettings {
                    model: "observed-test-model".into(),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    assert_eq!(replay.id, fixture.project.id);
    assert_eq!(fixture.store.snapshot().unwrap().projects.len(), 1);
}

#[test]
fn missing_historical_permissions_do_not_acquire_current_defaults() {
    let fixture = fixture();
    let submitted = request(&fixture, "request");
    let mut captured = serde_json::to_value(submitted).unwrap();
    captured["settings"]
        .as_object_mut()
        .unwrap()
        .remove("permissions");
    assert!(serde_json::from_value::<Request>(captured).is_err());
    let new_project: NewProject = serde_json::from_value(serde_json::json!({"name":"New project","repoPath":"/some/repository","objective":"Explicit new setup"})).unwrap();
    assert_eq!(new_project.settings.permissions.commands, Permission::Ask);
    assert_eq!(new_project.settings.permissions.network, Permission::Ask);
    assert_eq!(
        new_project.settings.permissions.integrations,
        Permission::Ask
    );
    assert!(
        new_project.settings.model.is_empty(),
        "no model is presumed configured before a capability observation"
    );
}
