use software_factory::{
    model::{Access, Permission, Permissions, ProjectSettings},
    provider::{
        discover_capabilities, CodexOptions, CodexSession, ProviderControl, ProviderError,
        ProviderEvent, ProviderStatus,
    },
};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};
use tempfile::TempDir;
use tokio::sync::{mpsc, oneshot};

fn fixture(scenario: &str) -> (TempDir, CodexOptions) {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("scenario"), scenario).unwrap();
    let executable = directory.path().join("fake-codex");
    fs::write(&executable, include_bytes!("fixtures/provider_fake.py")).unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let settings = ProjectSettings {
        model: "gpt-6.1-sol".into(),
        effort: "medium".into(),
        access: Access::Worktree,
        permissions: Permissions {
            commands: Permission::Allow,
            network: Permission::Deny,
            integrations: Permission::Deny,
        },
        max_turn_seconds: 5,
        max_output_tokens: 1000,
        ..ProjectSettings::default()
    };
    let options = CodexOptions {
        executable,
        cwd: directory.path().into(),
        settings,
        thread_id: None,
        planner: false,
    };
    (directory, options)
}

#[tokio::test]
async fn metadata_is_redacted_and_never_starts_a_thread_or_turn() {
    let (directory, options) = fixture("success");
    let caps = discover_capabilities(options.executable, options.cwd)
        .await
        .unwrap();
    assert!(caps.available && caps.authenticated);
    assert_eq!(caps.version.as_deref(), Some("0.159.3"));
    assert_eq!(caps.models[0]["displayName"], "Fixture Model");
    let methods = fs::read_to_string(directory.path().join("methods")).unwrap();
    assert!(!methods.contains("thread/start") && !methods.contains("turn/start"));
    let encoded = serde_json::to_string(&caps).unwrap();
    assert!(!encoded.contains("codexHome") && !encoded.contains("account"));
}

#[tokio::test]
async fn partial_frames_unknown_events_and_interleaved_notifications_preserve_terminal_identity() {
    let (directory, options) = fixture("partial");
    let mut session = CodexSession::connect(options).await.unwrap();
    let (_sender, controls) = mpsc::unbounded_channel();
    let mut events = Vec::new();
    let result = session
        .run("fixture", None, controls, |event| {
            events.push(event.clone());
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(result.status, ProviderStatus::Completed);
    assert!(result.terminal_observed);
    assert_eq!(result.text, "fixture result");
    assert_eq!(result.output_tokens, None);
    assert_eq!(result.total_tokens, None);
    assert_eq!(result.turn_id, "fixture-turn");
    assert!(events.iter().any(
        |event| matches!(event,ProviderEvent::Unknown{method} if method=="future/notification")
    ));
    assert!(events.iter().any(
        |event| matches!(event,ProviderEvent::TurnStarted{turn_id,..} if turn_id=="fixture-turn")
    ));
    assert_eq!(
        fs::read_to_string(directory.path().join("turn-start-count")).unwrap(),
        "1"
    );
    session.shutdown().await.unwrap();
}

#[tokio::test]
async fn failure_is_terminal_failure_and_eof_is_never_success() {
    for scenario in [
        "failure",
        "eof",
        "lost_start_eof",
        "truncated",
        "malformed",
        "oversized",
        "wrong_terminal",
    ] {
        let (_directory, options) = fixture(scenario);
        let mut session = CodexSession::connect(options).await.unwrap();
        let (_sender, controls) = mpsc::unbounded_channel();
        let mut observed = false;
        let result = session
            .run("fixture", None, controls, |event| {
                observed |= matches!(event, ProviderEvent::TurnStarted { .. });
                Ok(())
            })
            .await;
        if scenario == "failure" {
            assert_eq!(result.unwrap().status, ProviderStatus::Failed);
        } else {
            assert!(result.is_err(), "{scenario} unexpectedly succeeded");
        }
        assert!(observed, "{scenario} lost its provider identity");
    }
}

#[tokio::test]
async fn stderr_flood_is_drained_without_retaining_or_deadlocking() {
    let (_directory, options) = fixture("stderr_flood");
    let mut session = CodexSession::connect(options).await.unwrap();
    let (_sender, controls) = mpsc::unbounded_channel();
    assert_eq!(
        session
            .run("fixture", None, controls, |_| Ok(()))
            .await
            .unwrap()
            .status,
        ProviderStatus::Completed
    );
    assert!(session.stderr_bytes_discarded() >= 1024 * 1024);
    session.shutdown().await.unwrap();
}

#[tokio::test]
async fn approval_wait_remains_owned_until_a_scoped_response() {
    let (directory, options) = fixture("approval");
    let mut session = CodexSession::connect(options).await.unwrap();
    let (sender, controls) = mpsc::unbounded_channel();
    let (ready, received) = oneshot::channel();
    let mut ready = Some(ready);
    let run = tokio::spawn(async move {
        session
            .run("fixture", None, controls, |event| {
                if let ProviderEvent::Approval { request_id, .. } = event {
                    if let Some(ready) = ready.take() {
                        ready.send(request_id.clone()).unwrap();
                    }
                }
                Ok(())
            })
            .await
    });
    let request = tokio::time::timeout(Duration::from_secs(2), received)
        .await
        .unwrap()
        .unwrap();
    assert!(
        !run.is_finished(),
        "approval was incorrectly treated as terminal"
    );
    sender
        .send(ProviderControl::Approval {
            request_id: request,
            allow: false,
        })
        .unwrap();
    assert_eq!(run.await.unwrap().unwrap().status, ProviderStatus::Failed);
    assert_eq!(
        fs::read_to_string(directory.path().join("turn-start-count")).unwrap(),
        "1"
    );
}

#[tokio::test]
async fn interrupt_acknowledgment_alone_is_not_a_terminal_turn() {
    for scenario in ["interrupt", "interrupt_ack_only"] {
        let (_directory, options) = fixture(scenario);
        let mut session = CodexSession::connect(options).await.unwrap();
        let (sender, controls) = mpsc::unbounded_channel();
        let control = sender.clone();
        let result = session
            .run("fixture", None, controls, |event| {
                if matches!(event, ProviderEvent::TurnStarted { .. }) {
                    control.send(ProviderControl::Interrupt).unwrap();
                }
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(result.status, ProviderStatus::Interrupted);
        assert_eq!(result.terminal_observed, scenario == "interrupt");
        assert_eq!(result.process_stopped, scenario == "interrupt_ack_only");
        assert_eq!(result.control_unresolved, scenario == "interrupt_ack_only");
    }
}

#[tokio::test]
async fn steering_acknowledges_only_the_expected_live_turn() {
    let (_directory, options) = fixture("steer");
    let mut session = CodexSession::connect(options).await.unwrap();
    let (sender, controls) = mpsc::unbounded_channel();
    let mut acknowledged = false;
    let outcome = session
        .run("fixture", None, controls, |event| {
            match event {
                ProviderEvent::TurnStarted { .. } => sender
                    .send(ProviderControl::Steer {
                        id: "steer-42".into(),
                        text: "focus the check".into(),
                    })
                    .unwrap(),
                ProviderEvent::SteeringAccepted { id, turn_id } => {
                    assert_eq!(id, "steer-42");
                    assert_eq!(turn_id, "fixture-turn");
                    acknowledged = true;
                }
                _ => {}
            }
            Ok(())
        })
        .await
        .unwrap();
    assert!(acknowledged);
    assert_eq!(outcome.status, ProviderStatus::Completed);
}

#[tokio::test]
async fn policy_and_unsupported_models_fail_before_inference() {
    for mode in [
        "ignore_policy",
        "unsupported_model",
        "unresolved_ask",
        "extra_root",
        "network_expanded",
        "tmp_expanded",
    ] {
        let (directory, mut options) = fixture(mode);
        if mode == "unsupported_model" {
            options.settings.model = "not-in-catalog".into();
        }
        if mode == "unresolved_ask" {
            options.settings.permissions.network = Permission::Ask;
        }
        assert!(CodexSession::connect(options).await.is_err());
        assert!(!directory.path().join("turn-start-count").exists());
    }
}

#[tokio::test]
async fn observed_usage_and_deadline_stop_the_owned_execution() {
    for scenario in ["usage", "deadline"] {
        let (_directory, mut options) = fixture(scenario);
        options.settings.max_turn_seconds = 1;
        options.settings.max_output_tokens = 5;
        let mut session = CodexSession::connect(options).await.unwrap();
        let (_sender, controls) = mpsc::unbounded_channel();
        let result = session.run("fixture", None, controls, |_| Ok(())).await;
        assert!(matches!(
            (scenario, result),
            ("usage", Err(ProviderError::UsageLimit))
                | ("deadline", Err(ProviderError::ControlUnresolved { .. }))
        ));
    }
}

#[tokio::test]
async fn command_checks_use_explicit_sandbox_arguments_and_reject_outside_cwd() {
    let (_directory, options) = fixture("success");
    let cwd = options.cwd.clone();
    let mut session = CodexSession::connect(options).await.unwrap();
    let result = session
        .check(
            &["fixture-check".into(), "literal $(not-a-shell)".into()],
            &cwd,
            Duration::from_secs(2),
        )
        .await
        .unwrap();
    assert_eq!(result.exit_code, 7);
    assert_eq!(result.stdout, "observed check output");
    assert!(session
        .check(
            &["fixture-check".into()],
            &PathBuf::from("/"),
            Duration::from_secs(1)
        )
        .await
        .is_err());
    assert_eq!(session.turns().await.unwrap()[0]["id"], "fixture-turn");
    session.shutdown().await.unwrap();
}

#[tokio::test]
async fn interrupt_cleanup_stops_a_descendant_that_ignores_term() {
    let (directory, options) = fixture("descendant");
    let mut session = CodexSession::connect(options).await.unwrap();
    let (sender, controls) = mpsc::unbounded_channel();
    let (started, observed) = oneshot::channel();
    let mut started = Some(started);
    let run = tokio::spawn(async move {
        session
            .run("fixture", None, controls, |event| {
                if matches!(event, ProviderEvent::TurnStarted { .. }) {
                    if let Some(started) = started.take() {
                        started.send(()).unwrap();
                    }
                }
                Ok(())
            })
            .await
    });
    observed.await.unwrap();
    let child_pid = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Ok(pid) = fs::read_to_string(directory.path().join("descendant-pid")) {
                if let Ok(pid) = pid.trim().parse::<i32>() {
                    break pid;
                }
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    sender.send(ProviderControl::Interrupt).unwrap();
    assert!(run.await.unwrap().unwrap().process_stopped);
    // A killed orphan may briefly be a zombie; it must no longer be running.
    let stopped = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let output = tokio::process::Command::new("ps")
                .args(["-o", "stat=", "-p", &child_pid.to_string()])
                .output()
                .await
                .unwrap();
            let status = String::from_utf8_lossy(&output.stdout);
            if status.trim().is_empty() || status.trim_start().starts_with('Z') {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(stopped.is_ok(), "owned descendant survived interruption");
}

#[tokio::test]
async fn detached_descendant_forces_an_explicit_unresolved_control_result() {
    let (directory, options) = fixture("descendant_detached");
    let mut session = CodexSession::connect(options).await.unwrap();
    let (sender, controls) = mpsc::unbounded_channel();
    let (started, observed) = oneshot::channel();
    let mut started = Some(started);
    let run = tokio::spawn(async move {
        session
            .run("fixture", None, controls, |event| {
                if matches!(event, ProviderEvent::TurnStarted { .. }) {
                    if let Some(started) = started.take() {
                        let _ = started.send(());
                    }
                }
                Ok(())
            })
            .await
    });
    observed.await.unwrap();
    let pid = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Ok(value) = fs::read_to_string(directory.path().join("descendant-pid")) {
                if let Ok(pid) = value.trim().parse::<i32>() {
                    break pid;
                }
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    struct FixtureChild(i32);
    impl Drop for FixtureChild {
        fn drop(&mut self) {
            unsafe {
                libc::kill(self.0, libc::SIGKILL);
            }
        }
    }
    let _cleanup = FixtureChild(pid);
    sender.send(ProviderControl::Interrupt).unwrap();
    let outcome = run.await.unwrap().unwrap();
    assert!(outcome.control_unresolved);
    assert!(!outcome.terminal_observed);
    assert!(outcome.process_stopped);
    // This independently detached child is deliberately alive: the API must
    // not translate app-server group cleanup into a claimed complete stop.
    assert_eq!(unsafe { libc::kill(pid, 0) }, 0);
}

#[tokio::test]
async fn factory_fixture_plans_then_commits_and_supervisor_observes_real_check_exit() {
    let (directory, options) = fixture("factory");
    fs::write(directory.path().join("value.txt"), "pending\n").unwrap();
    fs::write(directory.path().join("check.py"),"from pathlib import Path\nassert Path('value.txt').read_text() == 'ready\\n'\nprint('fixture check passed')\n").unwrap();
    for argv in [
        vec!["init"],
        vec!["add", "--", "value.txt", "check.py"],
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@invalid.test",
            "commit",
            "-m",
            "Base fixture",
        ],
    ] {
        assert!(std::process::Command::new("git")
            .args(argv)
            .current_dir(directory.path())
            .output()
            .unwrap()
            .status
            .success());
    }
    let base = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(directory.path())
        .output()
        .unwrap()
        .stdout;
    let mut planner_options = options.clone();
    planner_options.planner = true;
    let mut planner = CodexSession::connect(planner_options).await.unwrap();
    let (_sender, controls) = mpsc::unbounded_channel();
    let plan = planner
        .run("Set the fixture value to ready", None, controls, |_| Ok(()))
        .await
        .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&plan.text).unwrap();
    assert_eq!(plan["action"], "plan");
    assert_eq!(plan["checks"][0]["args"][0], "check.py");
    planner.shutdown().await.unwrap();
    let mut worker = CodexSession::connect(options).await.unwrap();
    let (_sender, controls) = mpsc::unbounded_channel();
    assert_eq!(
        worker
            .run("Execute the accepted plan", None, controls, |_| Ok(()))
            .await
            .unwrap()
            .status,
        ProviderStatus::Completed
    );
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(directory.path())
        .output()
        .unwrap()
        .stdout;
    assert_ne!(base, head);
    assert_eq!(
        fs::read_to_string(directory.path().join("value.txt")).unwrap(),
        "ready\n"
    );
    let check = worker
        .check(
            &["/usr/bin/python3".into(), "check.py".into()],
            directory.path(),
            Duration::from_secs(5),
        )
        .await
        .unwrap();
    assert_eq!(check.exit_code, 0);
    assert_eq!(check.stdout, "fixture check passed\n");
    assert_eq!(
        worker.turns().await.unwrap().last().unwrap()["status"],
        "completed"
    );
    worker.shutdown().await.unwrap();
}

/// Explicit opt-in: exercises installed app-server command/exec and OS policy,
/// never turn/start or model inference. The fake-provider suite remains default.
#[tokio::test]
#[ignore = "requires the installed Codex executable and explicit local smoke-test authorization"]
async fn installed_codex_enforces_worktree_writes_and_denies_outside_and_network() {
    let directory = tempfile::tempdir().unwrap();
    let worktree = directory.path().join("worktree");
    fs::create_dir(&worktree).unwrap();
    assert!(std::process::Command::new("/usr/bin/git")
        .args(["init", "--quiet"])
        .current_dir(&worktree)
        .status()
        .unwrap()
        .success());
    let options = CodexOptions {
        executable: PathBuf::from("/Users/dylanmitic/.local/bin/codex"),
        cwd: worktree.clone(),
        settings: ProjectSettings {
            model: String::new(),
            effort: "low".into(),
            access: Access::Worktree,
            permissions: Permissions {
                commands: Permission::Allow,
                network: Permission::Deny,
                integrations: Permission::Deny,
            },
            max_turn_seconds: 10,
            max_output_tokens: 1000,
            ..ProjectSettings::default()
        },
        thread_id: None,
        planner: false,
    };
    let mut session = CodexSession::connect(options).await.unwrap();
    let inside=session.check(&["/usr/bin/python3".into(),"-c".into(),
        "from pathlib import Path;Path('inside-allowed').write_text('inside');print('inside allowed')".into()],
        &worktree,Duration::from_secs(5)).await.unwrap();
    assert_eq!(inside.exit_code, 0);
    assert_eq!(
        fs::read_to_string(worktree.join("inside-allowed")).unwrap(),
        "inside"
    );
    let outside = directory.path().join("outside-denied");
    let script = format!(
        "from pathlib import Path;Path({}).write_text('escaped')",
        serde_json::to_string(outside.to_str().unwrap()).unwrap()
    );
    let outside_result = session
        .check(
            &["/usr/bin/python3".into(), "-c".into(), script],
            &worktree,
            Duration::from_secs(5),
        )
        .await
        .unwrap();
    assert_ne!(outside_result.exit_code, 0);
    assert!(!outside.exists(), "OS sandbox allowed an outside write");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let script=format!("import socket;socket.create_connection(('127.0.0.1',{port}),timeout=1);print('network escaped')");
    let network = session
        .check(
            &["/usr/bin/python3".into(), "-c".into(), script],
            &worktree,
            Duration::from_secs(5),
        )
        .await
        .unwrap();
    assert_ne!(network.exit_code, 0);
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    let report = serde_json::json!({"noInference":true,"model":session.model(),"effort":session.effort(),
        "threadId":session.thread_id(),"allowedInsideWriteExit":inside.exit_code,"outsideWriteExit":outside_result.exit_code,
        "outsideFileAbsent":!outside.exists(),"networkExit":network.exit_code,"listenerReceivedConnection":false,
        "worktreeWriteEnforced":true,"hostReadIsolationClaimed":false});
    session.shutdown().await.unwrap();
    if let Ok(path) = std::env::var("FACTORY_SANDBOX_REPORT") {
        fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
