use software_factory::worktree::{inspect, prepare};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

struct Fixture {
    _directory: TempDir,
    repo: PathBuf,
    data: PathBuf,
}
fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git fixture command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn fixture() -> Fixture {
    let directory = tempfile::tempdir().unwrap();
    let repo = directory.path().join("source");
    let data = directory.path().join("runtime");
    fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "--quiet"]);
    fs::write(repo.join("value.txt"), "original\n").unwrap();
    git(&repo, &["add", "value.txt"]);
    git(
        &repo,
        &[
            "-c",
            "user.name=Local Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            "Initial fixture",
        ],
    );
    Fixture {
        _directory: directory,
        repo,
        data,
    }
}
fn commit(repo: &Path, message: &str) {
    git(repo, &["add", "--all"]);
    git(
        repo,
        &[
            "-c",
            "user.name=Local Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            message,
        ],
    );
}

#[test]
fn isolated_attempts_preserve_dirty_source_and_diff_binds_actual_committed_head() {
    let fixture = fixture();
    fs::write(fixture.repo.join("value.txt"), "user work must remain\n").unwrap();
    fs::write(fixture.repo.join("untracked.txt"), "keep this too\n").unwrap();
    let before = git(&fixture.repo, &["status", "--porcelain=v1"]);
    let first = prepare(&fixture.data, &fixture.repo, "run-one").unwrap();
    let second = prepare(&fixture.data, &fixture.repo, "run-two").unwrap();
    assert_ne!(first.path, second.path);
    assert_eq!(
        fs::read_to_string(first.path.join("value.txt")).unwrap(),
        "original\n"
    );
    assert!(first.is_clean().unwrap());
    assert_eq!(first.head().unwrap(), first.base_sha);
    fs::write(first.path.join("value.txt"), "worker change\n").unwrap();
    assert!(!first.is_clean().unwrap());
    commit(&first.path, "Bounded worker change");
    let head = first.head().unwrap();
    assert_ne!(head, first.base_sha);
    assert!(first.is_clean().unwrap());
    let diff = first.diff_at(&head).unwrap();
    assert!(diff.contains("-original"));
    assert!(diff.contains("+worker change"));
    assert_eq!(first.diff().unwrap(), diff);
    assert_eq!(second.head().unwrap(), second.base_sha);
    assert_eq!(git(&fixture.repo, &["status", "--porcelain=v1"]), before);
    assert_eq!(
        fs::read_to_string(fixture.repo.join("value.txt")).unwrap(),
        "user work must remain\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.repo.join("untracked.txt")).unwrap(),
        "keep this too\n"
    );
    assert_eq!(
        inspect(&fixture.data, &fixture.repo, "run-one")
            .unwrap()
            .head()
            .unwrap(),
        head
    );
    assert_eq!(
        prepare(&fixture.data, &fixture.repo, "run-one")
            .unwrap()
            .head()
            .unwrap(),
        head,
        "replaying a run must inspect its existing identity, never reset worker changes"
    );
}

#[test]
fn unrelated_paths_and_forged_identity_are_preserved_and_rejected() {
    let fixture = fixture();
    let first = prepare(&fixture.data, &fixture.repo, "run-one").unwrap();
    let unrelated = fixture.data.join("worktrees/run-unrelated");
    fs::create_dir(&unrelated).unwrap();
    fs::write(unrelated.join("important.txt"), "Do not remove").unwrap();
    assert!(prepare(&fixture.data, &fixture.repo, "run-unrelated").is_err());
    assert_eq!(
        fs::read_to_string(unrelated.join("important.txt")).unwrap(),
        "Do not remove"
    );
    let markers = fixture.data.join("worktree-identities");
    fs::copy(markers.join("run-one.json"), markers.join("run-other.json")).unwrap();
    assert!(inspect(&fixture.data, &fixture.repo, "run-other").is_err());
    let original = fs::read_to_string(first.path.join(".git")).unwrap();
    let second = prepare(&fixture.data, &fixture.repo, "run-two").unwrap();
    fs::write(
        first.path.join(".git"),
        fs::read(second.path.join(".git")).unwrap(),
    )
    .unwrap();
    assert!(first.validate().is_err());
    fs::write(first.path.join(".git"), original).unwrap();
    assert!(first.validate().is_ok());
    assert!(prepare(&fixture.data, &fixture.repo, "../escape").is_err());
}

#[test]
fn check_directory_rejects_traversal_and_symlink_escape() {
    let fixture = fixture();
    let worktree = prepare(&fixture.data, &fixture.repo, "run-one").unwrap();
    fs::create_dir(worktree.path.join("checks")).unwrap();
    assert_eq!(
        worktree.relative_cwd("checks").unwrap(),
        worktree.path.join("checks")
    );
    assert!(worktree.relative_cwd("../run-two").is_err());
    assert!(worktree
        .relative_cwd(fixture.repo.to_str().unwrap())
        .is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&fixture.repo, worktree.path.join("outside")).unwrap();
        assert!(worktree.relative_cwd("outside").is_err());
    }
}

#[test]
fn checkout_and_cleanliness_do_not_execute_repo_filters_or_hooks() {
    let fixture = fixture();
    fs::write(
        fixture.repo.join(".gitattributes"),
        "value.txt filter=unsafe\n",
    )
    .unwrap();
    commit(&fixture.repo, "Fixture filter attribute");
    let smudge = fixture._directory.path().join("smudge-ran");
    let clean = fixture._directory.path().join("clean-ran");
    let hook = fixture._directory.path().join("hook-ran");
    git(
        &fixture.repo,
        &[
            "config",
            "filter.unsafe.smudge",
            &format!("touch '{}'; cat", smudge.display()),
        ],
    );
    git(
        &fixture.repo,
        &[
            "config",
            "filter.unsafe.clean",
            &format!("touch '{}'; cat", clean.display()),
        ],
    );
    git(&fixture.repo, &["config", "filter.unsafe.required", "true"]);
    let hook_path = fixture.repo.join(".git/hooks/post-checkout");
    fs::write(
        &hook_path,
        format!("#!/bin/sh\ntouch '{}'\n", hook.display()),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(hook_path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let worktree = prepare(&fixture.data, &fixture.repo, "run-safe").unwrap();
    fs::write(
        worktree.path.join("value.txt"),
        "changed but not committed\n",
    )
    .unwrap();
    assert!(!worktree.is_clean().unwrap());
    assert!(!smudge.exists(), "checkout must not run smudge tools");
    assert!(!clean.exists(), "status must not run clean filters");
    assert!(
        !hook.exists(),
        "worktree checkout must not invoke repository hooks"
    );
}

#[test]
fn replaced_managed_parent_symlink_and_missing_marker_are_not_reused() {
    let fixture = fixture();
    let worktree = prepare(&fixture.data, &fixture.repo, "run-one").unwrap();
    let marker = fixture.data.join("worktree-identities/run-one.json");
    fs::remove_file(&marker).unwrap();
    assert!(prepare(&fixture.data, &fixture.repo, "run-one").is_err());
    assert!(
        worktree.path.join("value.txt").exists(),
        "failed reconciliation preserves the actual worktree"
    );
    #[cfg(unix)]
    {
        let parent = fixture.data.join("worktrees");
        let moved = fixture.data.join("original-worktrees");
        fs::rename(&parent, &moved).unwrap();
        std::os::unix::fs::symlink(&moved, &parent).unwrap();
        assert!(prepare(&fixture.data, &fixture.repo, "run-two").is_err());
        assert!(moved.join("run-one/value.txt").exists());
    }
}

#[test]
fn ambient_git_environment_is_ignored() {
    if let Some(repo) = std::env::var_os("FACTORY_WORKTREE_CHILD_REPO") {
        let data = PathBuf::from(std::env::var_os("FACTORY_WORKTREE_CHILD_DATA").unwrap());
        let worktree = prepare(&data, &PathBuf::from(repo), "run-isolated-env").unwrap();
        assert_eq!(
            fs::read_to_string(worktree.path.join("value.txt")).unwrap(),
            "original\n"
        );
        assert!(worktree.is_clean().unwrap());
        return;
    }
    let fixture = fixture();
    let unrelated = self::fixture();
    fs::write(unrelated.repo.join("value.txt"), "unrelated private work\n").unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "ambient_git_environment_is_ignored",
            "--nocapture",
        ])
        .env("FACTORY_WORKTREE_CHILD_REPO", &fixture.repo)
        .env("FACTORY_WORKTREE_CHILD_DATA", &fixture.data)
        .env("GIT_DIR", unrelated.repo.join(".git"))
        .env("GIT_WORK_TREE", &unrelated.repo)
        .env("GIT_INDEX_FILE", unrelated.repo.join(".git/index"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(unrelated.repo.join("value.txt")).unwrap(),
        "unrelated private work\n"
    );
}

#[test]
fn submodule_gitlinks_fail_closed_before_checkout_and_before_status_recursion() {
    let fixture = fixture();
    let worktree = prepare(&fixture.data, &fixture.repo, "run-before-submodule").unwrap();
    let initial = git(&fixture.repo, &["rev-parse", "HEAD"]);
    git(
        &fixture.repo,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{initial},dependency"),
        ],
    );
    git(
        &fixture.repo,
        &[
            "-c",
            "user.name=Local Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            "Fixture gitlink",
        ],
    );
    let error = prepare(&fixture.data, &fixture.repo, "run-submodule")
        .unwrap_err()
        .to_string();
    assert!(error.contains("submodules/gitlinks"));
    assert!(!fixture.data.join("worktrees/run-submodule").exists());
    git(
        &worktree.path,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{initial},dependency"),
        ],
    );
    assert!(worktree
        .is_clean()
        .unwrap_err()
        .to_string()
        .contains("submodules/gitlinks"));
}

#[test]
fn supervisor_checkpoint_creates_actual_clean_commit_without_touching_source_work() {
    let fixture = fixture();
    fs::write(fixture.repo.join("value.txt"), "user edits stay here\n").unwrap();
    fs::write(
        fixture.repo.join("user-note.txt"),
        "untracked source note\n",
    )
    .unwrap();
    let source_head = git(&fixture.repo, &["rev-parse", "HEAD"]);
    let source_status = git(&fixture.repo, &["status", "--porcelain=v1"]);
    let worktree = prepare(&fixture.data, &fixture.repo, "run-checkpoint").unwrap();
    fs::write(
        worktree.path.join("value.txt"),
        "supervisor records the worker change\n",
    )
    .unwrap();
    fs::write(worktree.path.join("added.txt"), "new intended file\n").unwrap();
    let head = worktree
        .checkpoint("Complete the bounded fixture task")
        .unwrap();
    assert_ne!(head, source_head);
    assert_eq!(git(&worktree.path, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(&worktree.path, &["rev-parse", "HEAD^1"]), source_head);
    assert_eq!(git(&worktree.path,&["show","-s","--format=%an <%ae>|%cn <%ce>|%s","HEAD"]),"Software Factory <factory@localhost>|Software Factory <factory@localhost>|Complete the bounded fixture task");
    assert!(worktree.is_clean().unwrap());
    assert!(worktree
        .diff_at(&head)
        .unwrap()
        .contains("+supervisor records the worker change"));
    assert_eq!(
        worktree
            .checkpoint("Must not create a second commit")
            .unwrap(),
        head
    );
    assert_eq!(git(&fixture.repo, &["rev-parse", "HEAD"]), source_head);
    assert_eq!(
        git(&fixture.repo, &["status", "--porcelain=v1"]),
        source_status
    );
    assert_eq!(
        fs::read_to_string(fixture.repo.join("value.txt")).unwrap(),
        "user edits stay here\n"
    );
}

#[test]
fn supervisor_checkpoint_disables_clean_filters_commit_hooks_and_signing() {
    let fixture = fixture();
    fs::write(
        fixture.repo.join(".gitattributes"),
        "value.txt filter=unsafe\n",
    )
    .unwrap();
    commit(&fixture.repo, "Fixture filter attribute");
    let filter = fixture._directory.path().join("checkpoint-filter-ran");
    let hook = fixture._directory.path().join("checkpoint-hook-ran");
    let signing = fixture._directory.path().join("checkpoint-signer-ran");
    git(
        &fixture.repo,
        &[
            "config",
            "filter.unsafe.clean",
            &format!("touch '{}'; cat", filter.display()),
        ],
    );
    git(&fixture.repo, &["config", "filter.unsafe.required", "true"]);
    git(&fixture.repo, &["config", "commit.gpgsign", "true"]);
    let hook_path = fixture.repo.join(".git/hooks/pre-commit");
    fs::write(
        &hook_path,
        format!("#!/bin/sh\ntouch '{}'\nexit 1\n", hook.display()),
    )
    .unwrap();
    let signer_path = fixture._directory.path().join("signer.sh");
    fs::write(
        &signer_path,
        format!("#!/bin/sh\ntouch '{}'\nexit 1\n", signing.display()),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(&signer_path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    git(
        &fixture.repo,
        &["config", "gpg.program", signer_path.to_str().unwrap()],
    );
    let worktree = prepare(&fixture.data, &fixture.repo, "run-checkpoint-safe").unwrap();
    fs::write(
        worktree.path.join("value.txt"),
        "checked change without external tools\n",
    )
    .unwrap();
    let head = worktree.checkpoint("Sanitized local checkpoint").unwrap();
    assert_eq!(worktree.head().unwrap(), head);
    assert!(worktree.is_clean().unwrap());
    assert!(!filter.exists());
    assert!(!hook.exists());
    assert!(!signing.exists());
}

#[test]
fn checkpoint_rejects_empty_work_or_attached_branches_and_preserves_changes() {
    let fixture = fixture();
    let worktree = prepare(&fixture.data, &fixture.repo, "run-checkpoint-guard").unwrap();
    assert!(worktree.checkpoint("No fabricated empty commit").is_err());
    assert_eq!(worktree.head().unwrap(), worktree.base_sha);
    git(
        &worktree.path,
        &["switch", "--quiet", "-c", "protected-fixture-branch"],
    );
    fs::write(
        worktree.path.join("value.txt"),
        "preserve this rather than committing to the branch\n",
    )
    .unwrap();
    assert!(worktree
        .checkpoint("Refuse named-branch writes")
        .unwrap_err()
        .to_string()
        .contains("detached HEAD"));
    assert_eq!(
        git(&worktree.path, &["rev-parse", "HEAD"]),
        worktree.base_sha
    );
    assert_eq!(
        fs::read_to_string(worktree.path.join("value.txt")).unwrap(),
        "preserve this rather than committing to the branch\n"
    );
    assert!(worktree.checkpoint("").is_err());
    assert!(worktree.checkpoint(&"x".repeat(2001)).is_err());
}
