//! Human-requested repository reads. These capabilities are never given to agents.
use crate::model::{RepositoryConnection, RepositoryProvider, RepositoryTarget};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::Semaphore,
};

const OUTPUT_LIMIT: usize = 2 * 1024 * 1024;
const PAGE_SIZE: usize = 20;

#[derive(Debug, Clone, thiserror::Error)]
pub enum RepositoryError {
    #[error("{0}")]
    Invalid(String),
    #[error("Install {0} and sign in using its supported authentication flow, then retry.")]
    MissingCli(&'static str),
    #[error("Repository authentication is required. Sign in with the provider CLI, then retry.")]
    Authentication,
    #[error("The signed-in account cannot read this repository or CI data. Check its repository permissions.")]
    Forbidden,
    #[error("The repository or change is unavailable to the signed-in account.")]
    NotFound,
    #[error("The provider rate limit was reached. Wait before refreshing again.")]
    RateLimited,
    #[error("A repository refresh is already in progress. Wait for it to finish.")]
    Busy,
    #[error("The repository request timed out. Saved local work was not changed.")]
    TimedOut,
    #[error("The provider response exceeded the local size limit.")]
    TooLarge,
    #[error("The provider returned an unsupported response. Saved local work was not changed.")]
    Malformed,
    #[error("The repository CLI could not complete this read. Check its installation and connection, then retry.")]
    Unavailable,
}
impl RepositoryError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Invalid(_) => "invalid_repository",
            Self::MissingCli(_) => "missing_cli",
            Self::Authentication => "provider_authentication",
            Self::Forbidden => "provider_forbidden",
            Self::NotFound => "provider_not_found",
            Self::RateLimited => "provider_rate_limited",
            Self::Busy => "repository_busy",
            Self::TimedOut => "repository_timeout",
            Self::TooLarge => "repository_response_limit",
            Self::Malformed => "repository_response_invalid",
            Self::Unavailable => "repository_unavailable",
        }
    }
}
type Result<T> = std::result::Result<T, RepositoryError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDetection {
    pub repository: Option<RepositoryTarget>,
    pub available: bool,
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteChange {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub author: String,
    pub state: String,
    pub draft: bool,
    pub source_branch: String,
    pub target_branch: String,
    pub head_sha: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryChanges {
    pub connection: RepositoryConnection,
    pub items: Vec<RemoteChange>,
    pub page: u32,
    pub has_more: bool,
    pub fetched_at: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiState {
    Success,
    Failure,
    Running,
    Pending,
    Cancelled,
    Skipped,
    Unknown,
    None,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CiItem {
    pub name: String,
    pub state: CiState,
    pub raw_status: String,
    pub url: Option<String>,
    pub revision: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCi {
    pub state: CiState,
    pub revision: String,
    pub items: Vec<CiItem>,
    pub complete: bool,
    pub limitations: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryChangeDetail {
    pub connection: RepositoryConnection,
    pub change: RemoteChange,
    pub ci: RemoteCi,
    pub fetched_at: String,
}

/// Only explicit public-provider targets are supported. No URLs become API endpoints.
pub fn parse_remote(remote: &str) -> Result<RepositoryTarget> {
    if remote.len() > 2048
        || remote.chars().any(char::is_control)
        || remote.trim() != remote
        || remote.contains('%')
        || remote.contains("/../")
        || remote.contains("/./")
        || remote.ends_with("/..")
        || remote.ends_with("/.")
    {
        return Err(RepositoryError::Invalid(
            "The origin remote must be a bounded GitHub.com or GitLab.com URL.".into(),
        ));
    }
    let (host, path) = if let Some(rest) = remote.strip_prefix("git@") {
        let (host, path) = rest
            .split_once(':')
            .ok_or_else(|| RepositoryError::Invalid("Unsupported origin remote format.".into()))?;
        (host.to_owned(), path.to_owned())
    } else {
        let url = reqwest::Url::parse(remote).map_err(|_| {
            RepositoryError::Invalid(
                "Use an HTTPS or SSH origin on GitHub.com or GitLab.com.".into(),
            )
        })?;
        let supported = match url.scheme() {
            "https" => {
                url.username().is_empty() && url.password().is_none() && url.port().is_none()
            }
            "ssh" => {
                url.username() == "git"
                    && url.password().is_none()
                    && url.port().is_none_or(|port| port == 22)
            }
            _ => false,
        };
        if !supported || url.query().is_some() || url.fragment().is_some() {
            return Err(RepositoryError::Invalid("Origin URLs must not contain credentials, query parameters, fragments or unsupported ports.".into()));
        }
        (
            url.host_str().unwrap_or_default().to_owned(),
            url.path().trim_start_matches('/').to_owned(),
        )
    };
    let provider = match host.as_str() {
        "github.com" => RepositoryProvider::Github,
        "gitlab.com" => RepositoryProvider::Gitlab,
        _ => return Err(RepositoryError::Invalid(
            "This connection supports GitHub.com and GitLab.com. Other hosts are not connected."
                .into(),
        )),
    };
    let path = path.strip_suffix(".git").unwrap_or(&path);
    let segments: Vec<_> = path.split('/').collect();
    if segments.len() < 2
        || (provider == RepositoryProvider::Github && segments.len() != 2)
        || !segments.iter().all(|segment| valid_segment(segment))
    {
        return Err(RepositoryError::Invalid("Origin must identify an owner or group and a repository, without encoded or parent paths.".into()));
    }
    Ok(RepositoryTarget {
        provider,
        host: host.clone(),
        path: path.into(),
        web_url: format!("https://{host}/{path}"),
    })
}
fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value.starts_with('-')
        && value.len() <= 255
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}
pub fn validate_target(target: &RepositoryTarget) -> Result<()> {
    if parse_remote(&target.web_url)? != *target {
        return Err(RepositoryError::Malformed);
    }
    Ok(())
}
fn safe_link(value: Option<&str>, host: &str) -> Option<String> {
    let value = value?;
    let url = reqwest::Url::parse(value).ok()?;
    if url.scheme() != "https"
        || url.host_str() != Some(host)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !url
            .path()
            .trim_start_matches('/')
            .split('/')
            .all(|s| s == "-" || valid_segment(s))
    {
        return None;
    }
    Some(url.to_string())
}
fn now() -> String {
    Utc::now().to_rfc3339()
}
fn sha(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|c| c.is_ascii_hexdigit())
}
fn string(value: &Value, key: &str, max: usize) -> Result<String> {
    let s = value
        .get(key)
        .and_then(Value::as_str)
        .ok_or(RepositoryError::Malformed)?;
    if s.len() > max || s.contains('\0') {
        return Err(RepositoryError::Malformed);
    }
    Ok(s.into())
}

#[derive(Clone)]
pub struct RepositoryClient {
    github: Option<PathBuf>,
    gitlab: Option<PathBuf>,
    cwd: PathBuf,
    gate: Arc<Semaphore>,
    deadline: Duration,
}
impl RepositoryClient {
    pub fn new(github: Option<PathBuf>, gitlab: Option<PathBuf>, cwd: PathBuf) -> Result<Self> {
        if !cwd.is_absolute()
            || github
                .iter()
                .chain(gitlab.iter())
                .any(|path| !path.is_absolute())
        {
            return Err(RepositoryError::Invalid(
                "Repository executables and service working directory must use absolute paths."
                    .into(),
            ));
        }
        Ok(Self {
            github,
            gitlab,
            cwd,
            gate: Arc::new(Semaphore::new(1)),
            deadline: Duration::from_secs(10),
        })
    }
    pub fn discover(cwd: PathBuf) -> Self {
        // The daemon also accepts relative --data-dir paths; CLI cwd never stays relative.
        let cwd = cwd.canonicalize().unwrap_or_else(|_| std::env::temp_dir());
        Self::new(discover_cli("gh"), discover_cli("glab"), cwd)
            .expect("absolute service working directory")
    }
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline.min(Duration::from_secs(10));
        self
    }
    pub fn available(&self, provider: RepositoryProvider) -> bool {
        self.executable(provider).is_some()
    }
    fn executable(&self, provider: RepositoryProvider) -> Option<&Path> {
        match provider {
            RepositoryProvider::Github => self.github.as_deref(),
            RepositoryProvider::Gitlab => self.gitlab.as_deref(),
        }
    }
    pub async fn detect(&self, repo: &Path) -> RepositoryDetection {
        match self.origin(repo).await {
            Ok(repository) => {
                let available = self.available(repository.provider);
                let reason = (!available)
                    .then(|| RepositoryError::MissingCli(repository.provider.cli()).to_string());
                RepositoryDetection {
                    repository: Some(repository),
                    available,
                    reason,
                }
            }
            Err(error) => RepositoryDetection {
                repository: None,
                available: false,
                reason: Some(error.to_string()),
            },
        }
    }
    pub async fn origin(&self, repo: &Path) -> Result<RepositoryTarget> {
        let mut command = Command::new("/usr/bin/git");
        command.args(["-C"]).arg(repo).args([
            "config",
            "--local",
            "--no-includes",
            "--get-all",
            "remote.origin.url",
        ]);
        command
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_CONFIG")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .env_remove("GIT_CONFIG_COUNT");
        let (success, output) = run(command, Duration::from_secs(3)).await?;
        let text = std::str::from_utf8(&output).map_err(|_| RepositoryError::Malformed)?;
        let lines: Vec<_> = text.lines().collect();
        if !success || lines.len() != 1 {
            return Err(RepositoryError::Invalid(
                "Set exactly one origin URL on this local Git repository before connecting.".into(),
            ));
        }
        parse_remote(lines[0])
    }
    pub async fn verify(&self, target: &RepositoryTarget) -> Result<RepositoryTarget> {
        validate_target(target)?;
        let _permit = self
            .gate
            .clone()
            .try_acquire_owned()
            .map_err(|_| RepositoryError::Busy)?;
        let value = self.api(target, &repository_endpoint(target)).await?;
        let key = match target.provider {
            RepositoryProvider::Github => "full_name",
            RepositoryProvider::Gitlab => "path_with_namespace",
        };
        let canonical = string(&value, key, 2048)?;
        if !canonical.eq_ignore_ascii_case(&target.path) {
            return Err(RepositoryError::Malformed);
        }
        parse_remote(&format!("https://{}/{canonical}", target.host))
    }
    pub async fn changes(
        &self,
        connection: RepositoryConnection,
        page: u32,
    ) -> Result<RepositoryChanges> {
        if !(1..=1000).contains(&page) {
            return Err(RepositoryError::Invalid(
                "Repository page must be between 1 and 1000.".into(),
            ));
        }
        let target = connection.target();
        validate_target(&target)?;
        let _permit = self
            .gate
            .clone()
            .try_acquire_owned()
            .map_err(|_| RepositoryError::Busy)?;
        let endpoint = match target.provider {
            RepositoryProvider::Github => format!("{}/pulls?state=open&sort=updated&direction=desc&per_page={PAGE_SIZE}&page={page}", repository_endpoint(&target)),
            RepositoryProvider::Gitlab => format!("{}/merge_requests?state=opened&order_by=updated_at&sort=desc&per_page={PAGE_SIZE}&page={page}", repository_endpoint(&target)),
        };
        let value = self.api(&target, &endpoint).await?;
        let values = value.as_array().ok_or(RepositoryError::Malformed)?;
        if values.len() > PAGE_SIZE {
            return Err(RepositoryError::Malformed);
        }
        let items = values
            .iter()
            .map(|item| parse_change(&target, item))
            .collect::<Result<Vec<_>>>()?;
        Ok(RepositoryChanges {
            connection,
            has_more: items.len() == PAGE_SIZE,
            items,
            page,
            fetched_at: now(),
        })
    }
    pub async fn change(
        &self,
        connection: RepositoryConnection,
        number: u64,
    ) -> Result<RepositoryChangeDetail> {
        if number == 0 || number > i64::MAX as u64 {
            return Err(RepositoryError::Invalid(
                "Change number must be a positive integer.".into(),
            ));
        }
        let target = connection.target();
        validate_target(&target)?;
        let _permit = self
            .gate
            .clone()
            .try_acquire_owned()
            .map_err(|_| RepositoryError::Busy)?;
        let endpoint = format!(
            "{}/{}/{number}",
            repository_endpoint(&target),
            match target.provider {
                RepositoryProvider::Github => "pulls",
                RepositoryProvider::Gitlab => "merge_requests",
            }
        );
        let value = self.api(&target, &endpoint).await?;
        let change = parse_change(&target, &value)?;
        if change.number != number {
            return Err(RepositoryError::Malformed);
        }
        let ci = match target.provider {
            RepositoryProvider::Github => self.github_ci(&target, &change.head_sha).await,
            RepositoryProvider::Gitlab => gitlab_ci(&target, &value, &change.head_sha),
        };
        Ok(RepositoryChangeDetail {
            connection,
            change,
            ci,
            fetched_at: now(),
        })
    }
    async fn github_ci(&self, target: &RepositoryTarget, revision: &str) -> RemoteCi {
        let mut ci = RemoteCi { state: CiState::Unknown, revision: revision.into(), items: vec![], complete: true, limitations: vec!["Reported head-commit CI is informational. It does not certify a local review, required checks, merged-result CI or merge readiness.".into()] };
        let root = format!("{}/commits/{revision}", repository_endpoint(target));
        match self
            .api(
                target,
                &format!("{root}/check-runs?filter=latest&per_page=100"),
            )
            .await
        {
            Ok(value) => {
                let parsed = (|| -> Result<()> {
                    let values = value["check_runs"]
                        .as_array()
                        .ok_or(RepositoryError::Malformed)?;
                    let count = value["total_count"]
                        .as_u64()
                        .ok_or(RepositoryError::Malformed)?;
                    if values.len() > 100 || count > values.len() as u64 {
                        ci.complete = false;
                        ci.limitations.push(
                            "Check-run results are incomplete or exceed the 100-item limit.".into(),
                        );
                    }
                    for value in values.iter().take(100) {
                        let observed = string(value, "head_sha", 64)?;
                        if observed != revision {
                            ci.complete = false;
                            ci.limitations
                                .push("A check reports a different revision.".into());
                            continue;
                        }
                        let status = string(value, "status", 40)?;
                        let conclusion = value["conclusion"].as_str().unwrap_or("");
                        let raw = if status == "completed" {
                            conclusion
                        } else {
                            &status
                        };
                        ci.items.push(CiItem {
                            name: string(value, "name", 500)?,
                            state: github_state(raw),
                            raw_status: raw.chars().take(100).collect(),
                            url: safe_link(value["html_url"].as_str(), &target.host)
                                .or_else(|| safe_link(value["details_url"].as_str(), &target.host)),
                            revision: Some(observed),
                        });
                    }
                    Ok(())
                })();
                if parsed.is_err() {
                    ci.complete = false;
                    ci.limitations.push(RepositoryError::Malformed.to_string());
                }
            }
            Err(error) => {
                ci.complete = false;
                ci.limitations
                    .push(format!("Check runs unavailable. {error}"));
            }
        }
        match self
            .api(target, &format!("{root}/status?per_page=100"))
            .await
        {
            Ok(value) => {
                let parsed = (|| -> Result<()> {
                    if value["sha"].as_str() != Some(revision) {
                        return Err(RepositoryError::Malformed);
                    }
                    let values = value["statuses"]
                        .as_array()
                        .ok_or(RepositoryError::Malformed)?;
                    let count = value["total_count"]
                        .as_u64()
                        .ok_or(RepositoryError::Malformed)?;
                    if values.len() > 100 || count > values.len() as u64 {
                        ci.complete = false;
                        ci.limitations
                            .push("Commit-status results exceed the 100-item limit.".into());
                    }
                    for value in values.iter().take(100) {
                        let raw = string(value, "state", 100)?;
                        ci.items.push(CiItem {
                            name: string(value, "context", 500)?,
                            state: github_state(&raw),
                            raw_status: raw,
                            url: safe_link(value["target_url"].as_str(), &target.host),
                            revision: Some(revision.into()),
                        });
                    }
                    Ok(())
                })();
                if parsed.is_err() {
                    ci.complete = false;
                    ci.limitations.push(
                        "Commit-status evidence is unavailable or reports another revision.".into(),
                    );
                }
            }
            Err(error) => {
                ci.complete = false;
                ci.limitations
                    .push(format!("Commit statuses unavailable. {error}"));
            }
        }
        ci.state = aggregate(&ci.items, ci.complete);
        ci
    }
    async fn api(&self, target: &RepositoryTarget, endpoint: &str) -> Result<Value> {
        // Endpoints are assembled only by the fixed read methods above, never from page data.
        let path = self
            .executable(target.provider)
            .ok_or(RepositoryError::MissingCli(target.provider.cli()))?;
        // A host flag alone can be redirected by per-host CLI api_host configuration.
        let endpoint = match target.provider {
            RepositoryProvider::Github => format!("https://api.github.com/{endpoint}"),
            RepositoryProvider::Gitlab => format!("https://gitlab.com/api/v4/{endpoint}"),
        };
        let mut command = Command::new(path);
        command.current_dir(&self.cwd).args([
            "api",
            "--method",
            "GET",
            "--hostname",
            &target.host,
            "--include",
            &endpoint,
        ]);
        if target.provider == RepositoryProvider::Github {
            command.args([
                "--header",
                "Accept: application/vnd.github+json",
                "--header",
                "X-GitHub-Api-Version: 2022-11-28",
            ]);
        }
        for key in [
            "GH_DEBUG",
            "GLAB_DEBUG_HTTP",
            "GLAB_DEBUG",
            "DEBUG",
            "GH_REPO",
            "GH_HOST",
            "GITLAB_REPO",
            "GITLAB_HOST",
            "GITLAB_URI",
            "GL_HOST",
            "GLAB_API_HOST",
            "GITLAB_API_HOST",
            "GLAB_API_PROTOCOL",
            "API_PROTOCOL",
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_CONFIG",
            "GIT_CONFIG_COUNT",
        ] {
            command.env_remove(key);
        }
        if target.provider == RepositoryProvider::Gitlab {
            // Generic tokens can belong to another GitLab host. Use only CLI host-scoped auth.
            for key in [
                "GITLAB_TOKEN",
                "GITLAB_ACCESS_TOKEN",
                "OAUTH_TOKEN",
                "JOB_TOKEN",
                "CI_JOB_TOKEN",
                "GLAB_IS_OAUTH2",
                "OAUTH2_REFRESH_TOKEN",
                "OAUTH2_EXPIRY_DATE",
                "REFRESH_TOKEN",
            ] {
                command.env_remove(key);
            }
            command.env("GLAB_ENABLE_CI_AUTOLOGIN", "false");
        }
        command
            .env("GH_PROMPT_DISABLED", "1")
            .env("GH_NO_UPDATE_NOTIFIER", "1")
            .env("GITLAB_HOST", "gitlab.com")
            .env("GITLAB_API_HOST", "gitlab.com")
            .env("GLAB_API_PROTOCOL", "https")
            .env("GLAB_SKIP_TLS_VERIFY", "false")
            .env("GLAB_NO_PROMPT", "1")
            .env("GLAB_CHECK_UPDATE", "false")
            .env("GLAB_SEND_TELEMETRY", "false")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("NO_COLOR", "1")
            .env("GH_PAGER", "cat")
            .env("PAGER", "cat");
        let (success, output) = run(command, self.deadline)
            .await
            .map_err(|error| match error {
                RepositoryError::Unavailable if !path.exists() => {
                    RepositoryError::MissingCli(target.provider.cli())
                }
                other => other,
            })?;
        let (status, body) = http_body(&output)?;
        match status {
            200..=299 if success => {
                serde_json::from_slice(body).map_err(|_| RepositoryError::Malformed)
            }
            401 => Err(RepositoryError::Authentication),
            403 => {
                if output
                    .windows(b"x-ratelimit-remaining: 0".len())
                    .any(|s| s.eq_ignore_ascii_case(b"x-ratelimit-remaining: 0"))
                {
                    Err(RepositoryError::RateLimited)
                } else {
                    Err(RepositoryError::Forbidden)
                }
            }
            404 => Err(RepositoryError::NotFound),
            429 => Err(RepositoryError::RateLimited),
            _ => Err(RepositoryError::Unavailable),
        }
    }
}
fn discover_cli(name: &str) -> Option<PathBuf> {
    let override_key = if name == "gh" {
        "FACTORY_GH_BIN"
    } else {
        "FACTORY_GLAB_BIN"
    };
    if let Some(path) = std::env::var_os(override_key).map(PathBuf::from) {
        return (path.is_absolute() && path.is_file()).then_some(path);
    }
    let mut paths: Vec<_> = std::env::var_os("PATH")
        .map(|v| {
            std::env::split_paths(&v)
                .filter(|p| p.is_absolute())
                .map(|p| p.join(name))
                .collect()
        })
        .unwrap_or_default();
    if let Some(home) = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
    {
        paths.push(home.join(".local/bin").join(name));
    }
    paths.into_iter().find(|path| {
        let Ok(meta) = std::fs::metadata(path) else {
            return false;
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            meta.is_file() && meta.permissions().mode() & 0o111 != 0
        }
        #[cfg(not(unix))]
        {
            meta.is_file()
        }
    })
}
async fn bounded(reader: impl AsyncRead + Unpin, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| RepositoryError::Unavailable)?;
    if bytes.len() > limit {
        return Err(RepositoryError::TooLarge);
    }
    Ok(bytes)
}
async fn run(mut command: Command, deadline: Duration) -> Result<(bool, Vec<u8>)> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|_| RepositoryError::Unavailable)?;
    let stdout = child.stdout.take().ok_or(RepositoryError::Unavailable)?;
    let stderr = child.stderr.take().ok_or(RepositoryError::Unavailable)?;
    let result = tokio::time::timeout(deadline, async {
        let (stdout, _, status) = tokio::try_join!(
            bounded(stdout, OUTPUT_LIMIT),
            bounded(stderr, 64 * 1024),
            async { child.wait().await.map_err(|_| RepositoryError::Unavailable) }
        )?;
        Ok((status.success(), stdout))
    })
    .await;
    match result {
        Ok(Ok(value)) => Ok(value),
        failure => {
            let _ = child.kill().await;
            match failure {
                Ok(Err(error)) => Err(error),
                _ => Err(RepositoryError::TimedOut),
            }
        }
    }
}
fn http_body(bytes: &[u8]) -> Result<(u16, &[u8])> {
    // Both supported CLIs emit response headers with --include. Never expose them to JS.
    let delimiter = bytes
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .map(|n| (n, 4))
        .or_else(|| {
            bytes
                .windows(2)
                .position(|part| part == b"\n\n")
                .map(|n| (n, 2))
        })
        .ok_or(RepositoryError::Unavailable)?;
    let headers =
        std::str::from_utf8(&bytes[..delimiter.0]).map_err(|_| RepositoryError::Malformed)?;
    let first = headers.lines().next().unwrap_or_default();
    if !first.starts_with("HTTP/") {
        return Err(RepositoryError::Unavailable);
    }
    let status = first
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .ok_or(RepositoryError::Malformed)?;
    Ok((status, &bytes[delimiter.0 + delimiter.1..]))
}
fn repository_endpoint(target: &RepositoryTarget) -> String {
    match target.provider {
        RepositoryProvider::Github => format!("repos/{}", target.path),
        RepositoryProvider::Gitlab => format!("projects/{}", target.path.replace('/', "%2F")),
    }
}
fn parse_change(target: &RepositoryTarget, value: &Value) -> Result<RemoteChange> {
    let github = target.provider == RepositoryProvider::Github;
    let number = value[if github { "number" } else { "iid" }]
        .as_u64()
        .filter(|v| *v > 0)
        .ok_or(RepositoryError::Malformed)?;
    let head_sha = if github {
        value["head"]["sha"].as_str()
    } else {
        value["sha"].as_str()
    }
    .ok_or(RepositoryError::Malformed)?
    .to_owned();
    if !sha(&head_sha) {
        return Err(RepositoryError::Malformed);
    }
    let expected_url = format!(
        "{}/{}/{number}",
        target.web_url,
        if github { "pull" } else { "-/merge_requests" }
    );
    let _reported_url = safe_link(
        value[if github { "html_url" } else { "web_url" }].as_str(),
        &target.host,
    )
    .filter(|url| url == &expected_url || (github && url.eq_ignore_ascii_case(&expected_url)))
    .ok_or(RepositoryError::Malformed)?;
    let author = if github {
        string(&value["user"], "login", 200)?
    } else {
        string(&value["author"], "username", 200)?
    };
    let source_branch = if github {
        string(&value["head"], "ref", 1000)?
    } else {
        string(value, "source_branch", 1000)?
    };
    let target_branch = if github {
        string(&value["base"], "ref", 1000)?
    } else {
        string(value, "target_branch", 1000)?
    };
    Ok(RemoteChange {
        number,
        title: string(value, "title", 4000)?,
        url: expected_url,
        author,
        state: string(value, "state", 40)?,
        draft: value["draft"].as_bool().unwrap_or(false),
        source_branch,
        target_branch,
        head_sha,
        updated_at: string(value, "updated_at", 100)?,
    })
}
fn github_state(raw: &str) -> CiState {
    match raw {
        "success" => CiState::Success,
        "failure" | "error" | "timed_out" | "action_required" | "startup_failure" => {
            CiState::Failure
        }
        "in_progress" => CiState::Running,
        "queued" | "pending" | "requested" | "waiting" => CiState::Pending,
        "cancelled" => CiState::Cancelled,
        "neutral" | "skipped" => CiState::Skipped,
        _ => CiState::Unknown,
    }
}
fn aggregate(items: &[CiItem], complete: bool) -> CiState {
    if items.iter().any(|item| item.state == CiState::Failure) {
        return CiState::Failure;
    }
    if !complete || items.iter().any(|item| item.state == CiState::Unknown) {
        return CiState::Unknown;
    }
    if items.is_empty() {
        return CiState::None;
    }
    for state in [CiState::Running, CiState::Pending, CiState::Cancelled] {
        if items.iter().any(|item| item.state == state) {
            return state;
        }
    }
    if items.iter().any(|item| item.state == CiState::Success) {
        CiState::Success
    } else {
        CiState::Skipped
    }
}
fn gitlab_ci(target: &RepositoryTarget, value: &Value, revision: &str) -> RemoteCi {
    let mut ci = RemoteCi { state: CiState::None, revision: revision.into(), items: vec![], complete: true, limitations: vec!["Reported pipeline status is informational. It does not certify local review checks or merge readiness.".into()] };
    let pipeline = &value["head_pipeline"];
    if pipeline.is_null() {
        ci.state = CiState::Unknown;
        ci.complete = false;
        ci.limitations.push(
            "No head pipeline was returned. CI may be absent or unavailable to this account."
                .into(),
        );
        return ci;
    }
    let Some(raw) = pipeline["status"].as_str().filter(|s| s.len() <= 100) else {
        ci.state = CiState::Unknown;
        ci.complete = false;
        return ci;
    };
    let observed = pipeline["sha"].as_str().filter(|s| sha(s));
    let state = match raw {
        "success" => CiState::Success,
        "failed" => CiState::Failure,
        "running" => CiState::Running,
        "created" | "waiting_for_resource" | "preparing" | "pending" | "manual" | "scheduled" => {
            CiState::Pending
        }
        "canceled" => CiState::Cancelled,
        "skipped" => CiState::Skipped,
        _ => CiState::Unknown,
    };
    ci.items.push(CiItem {
        name: "Head pipeline".into(),
        state,
        raw_status: raw.into(),
        url: safe_link(pipeline["web_url"].as_str(), &target.host),
        revision: observed.map(str::to_owned),
    });
    if observed != Some(revision) {
        ci.complete = false;
        ci.limitations.push("The pipeline revision differs from the merge-request head or was not reported. It may be a merged-results pipeline; it cannot certify this head commit.".into());
    }
    ci.state = aggregate(&ci.items, ci.complete);
    ci
}
