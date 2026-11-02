use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
use thiserror::Error;

const OUTPUT_LIMIT: u64 = 2_000_000;

#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("{0}")]
    Invalid(String),
    #[error("Git failed: {0}")]
    Git(String),
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("worktree identity error: {0}")]
    Json(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, WorktreeError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Worktree {
    pub path: PathBuf,
    pub base_sha: String,
    pub repo_path: PathBuf,
    pub git_dir: PathBuf,
    run_id: String,
    common_dir: PathBuf,
    managed_root: PathBuf,
}

/// The caller owns the supervisor lock. Partial failures preserve the directory for reconciliation.
pub fn prepare(data_dir: &Path, repo_path: &Path, run_id: &str) -> Result<Worktree> {
    valid_run_id(run_id)?;
    let (data_dir, repo_path, managed_root, identities) = locations(data_dir, repo_path)?;
    let path = managed_root.join(run_id);
    let marker = identities.join(format!("{run_id}.json"));
    if entry_exists(&path)? || entry_exists(&marker)? {
        return inspect(&data_dir, &repo_path, run_id);
    }
    let base_sha = git_text(&repo_path, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    valid_sha(&base_sha)?;
    reject_gitlinks(&repo_path, Some(&base_sha))?;
    let filters = disabled_filters(&repo_path)?;
    let mut args: Vec<String> = filters;
    args.extend([
        "worktree".into(),
        "add".into(),
        "--detach".into(),
        "--".into(),
        path_string(&path)?,
        base_sha.clone(),
    ]);
    run_git(&repo_path, args.iter().map(String::as_str))?;
    let common_dir = common_dir(&repo_path)?;
    let git_dir = absolute_git_dir(&path)?;
    let worktree = Worktree {
        path,
        base_sha,
        repo_path,
        git_dir,
        run_id: run_id.into(),
        common_dir,
        managed_root,
    };
    worktree.validate()?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(marker)?;
    file.write_all(&serde_json::to_vec_pretty(&worktree)?)?;
    file.sync_all()?;
    Ok(worktree)
}

pub fn inspect(data_dir: &Path, repo_path: &Path, run_id: &str) -> Result<Worktree> {
    valid_run_id(run_id)?;
    let (_, repo_path, managed_root, identities) = locations(data_dir, repo_path)?;
    let marker = identities.join(format!("{run_id}.json"));
    let metadata = fs::symlink_metadata(&marker)?;
    if !metadata.file_type().is_file() || metadata.len() > 32_000 {
        return Err(invalid(
            "managed worktree identity must be a bounded regular file",
        ));
    }
    let worktree: Worktree = serde_json::from_slice(&fs::read(marker)?)?;
    if worktree.run_id != run_id
        || worktree.repo_path != repo_path
        || worktree.managed_root != managed_root
        || worktree.path != managed_root.join(run_id)
    {
        return Err(invalid(
            "existing worktree identity does not match this run and repository",
        ));
    }
    worktree.validate()?;
    Ok(worktree)
}

impl Worktree {
    pub fn validate(&self) -> Result<()> {
        valid_run_id(&self.run_id)?;
        valid_sha(&self.base_sha)?;
        if self.path != self.managed_root.join(&self.run_id)
            || self.path.canonicalize()? != self.path
            || self.managed_root.canonicalize()? != self.managed_root
        {
            return Err(invalid(
                "worktree must remain at its exact canonical managed path",
            ));
        }
        if !fs::symlink_metadata(&self.path)?.is_dir()
            || !fs::symlink_metadata(self.path.join(".git"))?
                .file_type()
                .is_file()
        {
            return Err(invalid("managed path or Git identity was replaced"));
        }
        if repository_root(&self.repo_path)? != self.repo_path
            || repository_root(&self.path)? != self.path
            || common_dir(&self.repo_path)? != self.common_dir
            || common_dir(&self.path)? != self.common_dir
        {
            return Err(invalid(
                "managed worktree no longer belongs to its original repository",
            ));
        }
        if absolute_git_dir(&self.path)? != self.git_dir
            || self.git_dir.parent() != Some(self.common_dir.join("worktrees").as_path())
        {
            return Err(invalid("managed Git directory identity changed"));
        }
        let backpointer = fs::read_to_string(self.git_dir.join("gitdir"))?;
        if Path::new(backpointer.trim_end_matches(['\r', '\n'])).canonicalize()?
            != self.path.join(".git").canonicalize()?
        {
            return Err(invalid(
                "Git metadata no longer points back to the managed worktree",
            ));
        }
        let listing = run_git(&self.repo_path, ["worktree", "list", "--porcelain", "-z"])?;
        let expected = format!("worktree {}", path_string(&self.path)?);
        if !listing
            .stdout
            .split(|byte| *byte == 0)
            .any(|field| field == expected.as_bytes())
        {
            return Err(invalid(
                "managed worktree is absent from the repository registration",
            ));
        }
        run_git(
            &self.path,
            ["cat-file", "-e", &format!("{}^{{commit}}", self.base_sha)],
        )?;
        Ok(())
    }

    pub fn head(&self) -> Result<String> {
        self.validate()?;
        let head = git_text(&self.path, &["rev-parse", "--verify", "HEAD^{commit}"])?;
        valid_sha(&head)?;
        Ok(head)
    }

    pub fn diff(&self) -> Result<String> {
        self.diff_at(&self.head()?)
    }

    pub fn diff_at(&self, head: &str) -> Result<String> {
        self.validate()?;
        valid_sha(head)?;
        reject_gitlinks(&self.path, Some(head))?;
        run_git(
            &self.path,
            ["merge-base", "--is-ancestor", &self.base_sha, head],
        )?;
        let output = run_git(
            &self.path,
            [
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--binary",
                "--no-renames",
                &self.base_sha,
                head,
                "--",
            ],
        )?;
        String::from_utf8(output.stdout).map_err(|_| invalid("review diff is not UTF-8"))
    }

    pub fn is_clean(&self) -> Result<bool> {
        self.validate()?;
        reject_gitlinks(&self.path, None)?;
        let mut args = disabled_filters(&self.path)?;
        args.extend(
            [
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--ignore-submodules=none",
            ]
            .map(String::from),
        );
        Ok(run_git(&self.path, args)?.stdout.is_empty())
    }

    /// Only the supervisor checkpoints after the provider's bounded turn; provider Git metadata stays read-only.
    pub fn checkpoint(&self, message: &str) -> Result<String> {
        if message.trim().is_empty() || message.len() > 2000 || message.contains('\0') {
            return Err(invalid(
                "checkpoint message must be nonempty and fit within 2000 bytes without NUL",
            ));
        }
        self.validate()?;
        detached_head(&self.path)?;
        let before = self.head()?;
        reject_gitlinks(&self.path, Some(&before))?;
        run_git(
            &self.path,
            ["merge-base", "--is-ancestor", &self.base_sha, &before],
        )?;
        if self.is_clean()? {
            if before == self.base_sha || self.diff_at(&before)?.trim().is_empty() {
                return Err(invalid(
                    "worker produced no committed or uncommitted change relative to its base",
                ));
            }
            return Ok(before);
        }
        let mut policy = disabled_filters(&self.path)?;
        policy.extend(
            [
                "-c",
                "user.name=Software Factory",
                "-c",
                "user.email=factory@localhost",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "gc.auto=0",
                "-c",
                "maintenance.auto=false",
            ]
            .map(String::from),
        );
        let mut add = policy.clone();
        add.extend(["add", "--all", "--", "."].map(String::from));
        run_git(&self.path, add)?;
        reject_gitlinks(&self.path, None)?;
        self.validate()?;
        detached_head(&self.path)?;
        if self.head()? != before {
            return Err(invalid("worktree head changed during checkpoint preparation; inspect the preserved index before retrying"));
        }
        let mut commit = policy;
        commit.extend(
            [
                "commit",
                "--quiet",
                "--no-gpg-sign",
                "--no-verify",
                "--author=Software Factory <factory@localhost>",
                "-m",
                message.trim(),
            ]
            .map(String::from),
        );
        run_git(&self.path, commit)?;
        let head = self.head()?;
        let parent = git_text(&self.path, &["rev-parse", "--verify", "HEAD^1"])?;
        if head == before
            || parent != before
            || self.diff_at(&head)?.trim().is_empty()
            || !self.is_clean()?
        {
            return Err(invalid("checkpoint did not leave one new source-bound commit and a clean worktree; preserved for inspection"));
        }
        Ok(head)
    }

    pub fn relative_cwd(&self, relative: &str) -> Result<PathBuf> {
        self.validate()?;
        if relative.contains('\0')
            || Path::new(relative)
                .components()
                .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
        {
            return Err(invalid(
                "check directory must be relative without parent traversal",
            ));
        }
        let path = self.path.join(relative).canonicalize()?;
        if !path.is_dir() || !path.starts_with(&self.path) {
            return Err(invalid("check directory escapes the managed worktree"));
        }
        Ok(path)
    }
}

fn locations(data_dir: &Path, repo_path: &Path) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf)> {
    if !data_dir.is_absolute() || !repo_path.is_absolute() {
        return Err(invalid("data and repository paths must be absolute"));
    }
    fs::create_dir_all(data_dir)?;
    let data_dir = data_dir.canonicalize()?;
    let repo_path = repository_root(repo_path)?;
    if data_dir.starts_with(&repo_path) {
        return Err(invalid(
            "managed execution data must live outside the source repository",
        ));
    }
    let managed_root = child_directory(&data_dir, "worktrees")?;
    let identities = child_directory(&data_dir, "worktree-identities")?;
    Ok((data_dir, repo_path, managed_root, identities))
}
fn child_directory(parent: &Path, name: &str) -> Result<PathBuf> {
    let path = parent.join(name);
    if entry_exists(&path)? {
        if !fs::symlink_metadata(&path)?.is_dir() {
            return Err(invalid(
                "managed directory was replaced by a file or symlink",
            ));
        }
    } else {
        fs::create_dir(&path)?;
    }
    if path.canonicalize()? != path {
        return Err(invalid("managed directory must be canonical"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(path)
}
fn repository_root(path: &Path) -> Result<PathBuf> {
    let path = path.canonicalize()?;
    if !path.is_dir() || path_string(&path)?.chars().any(char::is_control) {
        return Err(invalid(
            "repository must be a directory without path control characters",
        ));
    }
    let observed = git_text(&path, &["rev-parse", "--show-toplevel"])?;
    if Path::new(&observed).canonicalize()? != path {
        return Err(invalid("select an exact Git working-tree root"));
    }
    Ok(path)
}
fn common_dir(repo: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from(git_text(
        repo,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?)
    .canonicalize()?)
}
fn absolute_git_dir(repo: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from(git_text(repo, &["rev-parse", "--absolute-git-dir"])?).canonicalize()?)
}
fn disabled_filters(repo: &Path) -> Result<Vec<String>> {
    let output = git_command(
        repo,
        [
            "config",
            "--null",
            "--name-only",
            "--get-regexp",
            r"^filter\..*\.(smudge|clean|process|required)$",
        ],
    )?;
    if !output.status.success() && output.status.code() != Some(1) {
        return Err(WorktreeError::Git(
            "could not inspect checkout filter policy".into(),
        ));
    }
    let keys = String::from_utf8(output.stdout)
        .map_err(|_| invalid("filter configuration must be UTF-8"))?;
    let mut drivers = HashSet::new();
    for key in keys.split('\0').filter(|key| !key.is_empty()) {
        if let Some((driver, _)) = key.rsplit_once('.') {
            drivers.insert(driver.to_owned());
        }
    }
    let mut drivers: Vec<_> = drivers.into_iter().collect();
    drivers.sort();
    let mut args = vec![];
    for driver in drivers {
        for field in ["smudge", "clean", "process"] {
            args.extend(["-c".into(), format!("{driver}.{field}=")]);
        }
        args.extend(["-c".into(), format!("{driver}.required=false")]);
    }
    Ok(args)
}
fn detached_head(repo: &Path) -> Result<()> {
    let output = git_command(repo, ["symbolic-ref", "--quiet", "HEAD"])?;
    if output.status.code() != Some(1) {
        return Err(invalid("supervisor checkpoints require the managed detached HEAD; refusing to update a named branch"));
    }
    Ok(())
}
fn reject_gitlinks(repo: &Path, revision: Option<&str>) -> Result<()> {
    let output = if let Some(revision) = revision {
        run_git(repo, ["ls-tree", "-r", "-z", revision])?
    } else {
        run_git(repo, ["ls-files", "--stage", "-z"])?
    };
    if output
        .stdout
        .split(|byte| *byte == 0)
        .any(|entry| entry.starts_with(b"160000 "))
    {
        return Err(invalid("This local alpha cannot safely inspect submodules/gitlinks. Select a repository without submodules; recursive permission and verification support is not connected."));
    }
    Ok(())
}
fn git_text(repo: &Path, args: &[&str]) -> Result<String> {
    String::from_utf8(run_git(repo, args.iter().copied())?.stdout)
        .map(|value| value.trim_end_matches(['\r', '\n']).into())
        .map_err(|_| invalid("Git metadata is not UTF-8"))
}
fn run_git<I, S>(repo: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_command(repo, args)?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr)
            .chars()
            .take(2000)
            .collect::<String>();
        return Err(WorktreeError::Git(format!(
            "exit {}: {}",
            output.status,
            detail.trim()
        )));
    }
    Ok(output)
}
fn git_command<I, S>(repo: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0");
    command
        .args([
            "--no-pager",
            "--no-optional-locks",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.attributesFile=/dev/null",
            "-c",
            "diff.external=",
            "-c",
            "submodule.recurse=false",
            "-c",
            "protocol.allow=never",
            "-C",
        ])
        .arg(repo)
        .args(args);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid("Git stdout pipe unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| invalid("Git stderr pipe unavailable"))?;
    let read = |pipe: Box<dyn Read + Send>| {
        thread::spawn(move || {
            let mut bytes = vec![];
            pipe.take(OUTPUT_LIMIT + 1)
                .read_to_end(&mut bytes)
                .map(|_| bytes)
        })
    };
    let stdout = read(Box::new(stdout));
    let stderr = read(Box::new(stderr));
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout = stdout
        .join()
        .map_err(|_| invalid("Git stdout reader failed"))??;
    let stderr = stderr
        .join()
        .map_err(|_| invalid("Git stderr reader failed"))??;
    if timed_out {
        return Err(WorktreeError::Git(
            "deadline exceeded; preserve the worktree and inspect its identity before retrying"
                .into(),
        ));
    }
    if stdout.len() as u64 > OUTPUT_LIMIT || stderr.len() as u64 > OUTPUT_LIMIT {
        return Err(WorktreeError::Git(
            "output exceeded 2MB; preserve the worktree for inspection".into(),
        ));
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
fn entry_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
fn path_string(path: &Path) -> Result<String> {
    path.to_str()
        .map(Into::into)
        .ok_or_else(|| invalid("paths must be UTF-8"))
}
fn valid_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || run_id.len() > 100
        || !run_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(invalid(
            "run identity must contain only letters, digits, hyphens or underscores",
        ));
    }
    Ok(())
}
fn valid_sha(sha: &str) -> Result<()> {
    if !matches!(sha.len(), 40 | 64) || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid("revision must be a complete Git object identity"));
    }
    Ok(())
}
fn invalid(message: &str) -> WorktreeError {
    WorktreeError::Invalid(message.into())
}
