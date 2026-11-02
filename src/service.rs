use crate::{
    client::ConnectionInfo,
    integrations::{RepositoryClient, RepositoryError},
    model::*,
    store::{Store, StoreError},
};
use axum::{
    extract::{DefaultBodyLimit, Query, State},
    http::{header, HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use fs2::FileExt;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use tokio::sync::Notify;
use uuid::Uuid;

#[derive(Clone)]
pub struct ServiceState {
    pub store: Store,
    pub wake: Arc<Notify>,
    pub capabilities: Arc<RwLock<Value>>,
    repository_client: RepositoryClient,
    data_dir: Option<PathBuf>,
    token: String,
    host: String,
}
impl ServiceState {
    pub fn new(store: Store, token: String, address: SocketAddr) -> Self {
        Self {
            store,
            wake: Arc::new(Notify::new()),
            capabilities: Arc::new(RwLock::new(
                json!({"available":false,"authenticated":false,"models":[],"limitations":["Provider capabilities are being checked."]}),
            )),
            repository_client: RepositoryClient::discover(std::env::temp_dir()),
            data_dir: None,
            token,
            host: address.to_string(),
        }
    }
    pub fn with_data_dir(mut self, path: PathBuf) -> Self {
        self.repository_client = RepositoryClient::discover(path.clone());
        self.data_dir = Some(path);
        self
    }
    pub fn with_repository_client(mut self, client: RepositoryClient) -> Self {
        self.repository_client = client;
        self
    }
}

pub struct DataDirectory {
    pub path: PathBuf,
    _lock: File,
}
impl DataDirectory {
    pub fn acquire(path: PathBuf) -> std::io::Result<Self> {
        if path.exists() && std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(std::io::Error::other("data directory cannot be a symlink"));
        }
        std::fs::create_dir_all(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            if std::fs::metadata(&path)?.uid() != unsafe { libc::geteuid() } {
                return Err(std::io::Error::other("data directory has another owner"));
            }
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
        }
        let lock_path = path.join("supervisor.lock");
        if std::fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file()) {
            return Err(std::io::Error::other("invalid supervisor lock"));
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let lock = options.open(lock_path)?;
        lock.try_lock_exclusive()
            .map_err(|_| std::io::Error::other("a supervisor already owns this data directory"))?;
        Ok(Self { path, _lock: lock })
    }
    pub fn publish_connection(&self, address: SocketAddr) -> std::io::Result<ConnectionInfo> {
        if !address.ip().is_loopback() {
            return Err(std::io::Error::other("service must bind loopback"));
        }
        let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let info = ConnectionInfo {
            schema_version: 1,
            base_url: format!("http://{address}/"),
            token,
        };
        let temp = self
            .path
            .join(format!(".connection-{}.json", Uuid::new_v4()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        file.write_all(&serde_json::to_vec(&info)?)?;
        file.sync_all()?;
        std::fs::rename(temp, self.path.join("connection.json"))?;
        File::open(&self.path)?.sync_all()?;
        Ok(info)
    }
}

pub fn router(state: ServiceState) -> Router {
    Router::new()
        .route("/health", get(|| async { Json(json!({"ready":true})) }))
        .route("/v1/snapshot", get(snapshot))
        .route("/v1/events", get(events))
        .route("/v1/commands", post(command))
        .route("/v1/capabilities", get(capabilities))
        .route("/v1/repository/detect", get(repository_detect))
        .route("/v1/repository/changes", get(repository_changes))
        .route("/v1/repository/change", get(repository_change))
        .layer(DefaultBodyLimit::max(128 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), authorize))
        .with_state(state)
}

async fn authorize(
    State(state): State<ServiceState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if request.headers().get_all(header::HOST).iter().count() != 1
        || request
            .headers()
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            != Some(state.host.as_str())
        || request.headers().contains_key(header::ORIGIN)
        || request.headers().contains_key("sec-fetch-site")
    {
        return error(
            StatusCode::FORBIDDEN,
            "Untrusted local service origin or host",
        );
    }
    if request.uri().path() != "/health" && !valid_token(request.headers(), &state.token) {
        return error(
            StatusCode::UNAUTHORIZED,
            "Local service authentication required",
        );
    }
    next.run(request).await
}
fn valid_token(headers: &HeaderMap, expected: &str) -> bool {
    let Some(actual) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
    else {
        return false;
    };
    actual.len() == expected.len()
        && actual
            .bytes()
            .zip(expected.bytes())
            .fold(0u8, |n, (a, b)| n | (a ^ b))
            == 0
}
fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({"error":message}))).into_response()
}
enum ApiError {
    Store(StoreError),
    Repository(RepositoryError),
}
impl From<StoreError> for ApiError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}
impl From<RepositoryError> for ApiError {
    fn from(value: RepositoryError) -> Self {
        Self::Repository(value)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let store = match self {
            Self::Repository(error) => return repository_error(error),
            Self::Store(error) => error,
        };
        let status = match store {
            StoreError::Invalid(_) => StatusCode::BAD_REQUEST,
            StoreError::Conflict(_) => StatusCode::CONFLICT,
            StoreError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
            "Local storage operation failed".into()
        } else {
            store.to_string()
        };
        error(status, &message)
    }
}
async fn snapshot(State(state): State<ServiceState>) -> Result<Json<Snapshot>, ApiError> {
    let value = tokio::task::spawn_blocking(move || observed_snapshot(&state))
        .await
        .map_err(|_| {
            ApiError::Store(StoreError::Io(std::io::Error::other(
                "snapshot observer failed",
            )))
        })??;
    Ok(Json(value))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RepositoryQuery {
    project_id: String,
    #[serde(default = "repository_page")]
    page: u32,
    #[serde(default)]
    number: u64,
}
fn repository_page() -> u32 {
    1
}
fn repository_error(error: RepositoryError) -> Response {
    let status = match error {
        RepositoryError::Invalid(_) => StatusCode::BAD_REQUEST,
        RepositoryError::Busy => StatusCode::CONFLICT,
        RepositoryError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        RepositoryError::MissingCli(_) => StatusCode::SERVICE_UNAVAILABLE,
        RepositoryError::TimedOut => StatusCode::GATEWAY_TIMEOUT,
        _ => StatusCode::BAD_GATEWAY,
    };
    (
        status,
        Json(json!({"error":error.to_string(),"code":error.code()})),
    )
        .into_response()
}
async fn repository_detect(
    State(state): State<ServiceState>,
    Query(query): Query<RepositoryQuery>,
) -> Response {
    let project = match state.store.project(&query.project_id) {
        Ok(value) => value,
        Err(error) => return ApiError::Store(error).into_response(),
    };
    Json(
        state
            .repository_client
            .detect(Path::new(&project.repo_path))
            .await,
    )
    .into_response()
}
async fn repository_changes(
    State(state): State<ServiceState>,
    Query(query): Query<RepositoryQuery>,
) -> Response {
    let connection = match state.store.repository_connection(&query.project_id) {
        Ok(value) => value,
        Err(error) => return ApiError::Store(error).into_response(),
    };
    match state
        .repository_client
        .changes(connection, query.page)
        .await
    {
        Ok(value) => Json(value).into_response(),
        Err(error) => repository_error(error),
    }
}
async fn repository_change(
    State(state): State<ServiceState>,
    Query(query): Query<RepositoryQuery>,
) -> Response {
    let connection = match state.store.repository_connection(&query.project_id) {
        Ok(value) => value,
        Err(error) => return ApiError::Store(error).into_response(),
    };
    match state
        .repository_client
        .change(connection, query.number)
        .await
    {
        Ok(value) => Json(value).into_response(),
        Err(error) => repository_error(error),
    }
}
pub fn observed_snapshot(state: &ServiceState) -> Result<Snapshot, StoreError> {
    let mut snapshot = state.store.snapshot()?;
    for review in &snapshot.reviews {
        let source = (|| -> Result<String, String> {
            let run = snapshot
                .runs
                .iter()
                .find(|r| r.id == review.run_id)
                .ok_or("Source attempt unavailable")?;
            if run.status != RunStatus::Succeeded
                || run.head_sha.as_deref() != Some(&review.revision)
                || snapshot
                    .runs
                    .iter()
                    .any(|r| r.task_id == run.task_id && r.attempt > run.attempt)
            {
                return Err("Source attempt changed or was superseded".into());
            }
            let project = snapshot
                .projects
                .iter()
                .find(|p| p.id == run.project_id)
                .ok_or("Source project unavailable")?;
            let directory = state
                .data_dir
                .as_deref()
                .ok_or("Managed source observer unavailable")?;
            let worktree =
                crate::worktree::inspect(directory, Path::new(&project.repo_path), &run.id)
                    .map_err(|_| "Managed source identity unavailable")?;
            let head = worktree
                .head()
                .map_err(|_| "Cannot observe source revision")?;
            if head != review.revision
                || !worktree
                    .is_clean()
                    .map_err(|_| "Cannot inspect source changes")?
            {
                return Err("Source changed since verification. Recorded checks certify the saved revision only".into());
            }
            Ok(head)
        })();
        snapshot.review_sources.push(match source {
            Ok(head) => ReviewSourceState {
                review_id: review.id.clone(),
                source_current: true,
                observed_head: Some(head),
                reason: None,
            },
            Err(reason) => ReviewSourceState {
                review_id: review.id.clone(),
                source_current: false,
                observed_head: None,
                reason: Some(reason),
            },
        });
    }
    Ok(snapshot)
}
async fn capabilities(State(state): State<ServiceState>) -> Json<Value> {
    Json(
        state
            .capabilities
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone(),
    )
}
#[derive(Deserialize)]
struct EventQuery {
    #[serde(default)]
    after: u64,
    #[serde(default = "event_limit")]
    limit: u32,
}
fn event_limit() -> u32 {
    200
}
async fn events(
    State(state): State<ServiceState>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Vec<Event>>, ApiError> {
    Ok(Json(state.store.events(query.after, query.limit)?))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandEnvelope {
    pub command_id: String,
    pub command: UserCommand,
}
#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum UserCommand {
    ConnectRepository {
        project_id: String,
        provider: RepositoryProvider,
    },
    DisconnectRepository {
        project_id: String,
    },
    RegisterProject {
        project: NewProject,
    },
    SaveSettings {
        project_id: String,
        settings: ProjectSettings,
    },
    SetObjective {
        project_id: String,
        objective: String,
    },
    SendSteering {
        project_id: String,
        text: String,
    },
    SubmitRequest {
        request: NewRequest,
    },
    CancelRequest {
        request_id: String,
    },
    SetPaused {
        project_id: String,
        paused: bool,
    },
    InterruptRun {
        run_id: String,
    },
    ResolveApproval {
        approval_id: String,
        allow: bool,
    },
    RecordLocalReviewDecision {
        review_id: String,
        decision: String,
        feedback: String,
    },
}
async fn command(
    State(state): State<ServiceState>,
    Json(input): Json<CommandEnvelope>,
) -> Result<Json<Value>, ApiError> {
    if let UserCommand::ConnectRepository {
        project_id,
        provider,
    } = &input.command
    {
        if let Some(previous) =
            state
                .store
                .replay_repository_connection(&input.command_id, project_id, *provider)?
        {
            return Ok(Json(
                serde_json::to_value(previous).map_err(StoreError::from)?,
            ));
        }
        let project = state.store.project(project_id)?;
        let target = state
            .repository_client
            .origin(Path::new(&project.repo_path))
            .await?;
        if target.provider != *provider {
            return Err(StoreError::Invalid(
                "The selected provider does not match this project's origin.".into(),
            )
            .into());
        }
        let target = state.repository_client.verify(&target).await?;
        let saved = state
            .store
            .connect_repository(&input.command_id, project_id, target)?;
        return Ok(Json(serde_json::to_value(saved).map_err(StoreError::from)?));
    }
    let value = execute_command(&state, input)?;
    state.wake.notify_one();
    Ok(Json(value))
}
pub fn execute_command(state: &ServiceState, input: CommandEnvelope) -> Result<Value, StoreError> {
    let id = &input.command_id;
    let result = match input.command {
        UserCommand::ConnectRepository { .. } => {
            return Err(StoreError::Invalid(
                "Repository connection requires the authenticated asynchronous verification path"
                    .into(),
            ))
        }
        UserCommand::DisconnectRepository { project_id } => {
            state.store.disconnect_repository(id, &project_id)?
        }
        UserCommand::RegisterProject { project } => {
            serde_json::to_value(state.store.create_project(id, project)?)?
        }
        UserCommand::SaveSettings {
            project_id,
            settings,
        } => serde_json::to_value(state.store.save_settings(id, &project_id, settings)?)?,
        UserCommand::SetObjective {
            project_id,
            objective,
        } => serde_json::to_value(state.store.set_objective(id, &project_id, &objective)?)?,
        UserCommand::SendSteering { project_id, text } => {
            serde_json::to_value(state.store.add_steering(id, &project_id, &text)?)?
        }
        UserCommand::SubmitRequest { request } => {
            serde_json::to_value(state.store.submit_request(id, request)?)?
        }
        UserCommand::CancelRequest { request_id } => {
            serde_json::to_value(state.store.cancel_request(id, &request_id)?)?
        }
        UserCommand::SetPaused { project_id, paused } => {
            serde_json::to_value(state.store.set_paused(id, &project_id, paused)?)?
        }
        UserCommand::ResolveApproval { approval_id, allow } => {
            serde_json::to_value(state.store.resolve_approval(id, &approval_id, allow)?)?
        }
        UserCommand::InterruptRun { run_id } => {
            serde_json::to_value(state.store.interrupt_run(id, &run_id)?)?
        }
        UserCommand::RecordLocalReviewDecision {
            review_id,
            decision,
            feedback,
        } => {
            if let Some(outcome) = state
                .store
                .replay_review_decision(id, &review_id, &decision, &feedback)?
            {
                return Ok(serde_json::to_value(outcome)?);
            }
            let snapshot = state.store.snapshot()?;
            let review = snapshot
                .reviews
                .iter()
                .find(|r| r.id == review_id)
                .ok_or_else(|| StoreError::NotFound("review".into()))?;
            let run = snapshot
                .runs
                .iter()
                .find(|r| r.id == review.run_id)
                .ok_or_else(|| StoreError::NotFound("source run".into()))?;
            let path = run
                .worktree_path
                .as_deref()
                .ok_or_else(|| StoreError::Conflict("source worktree unavailable".into()))?;
            let project = state.store.project(&run.project_id)?;
            let directory = state.data_dir.as_deref().ok_or_else(|| {
                StoreError::Conflict("managed source validation unavailable".into())
            })?;
            let worktree =
                crate::worktree::inspect(directory, Path::new(&project.repo_path), &run.id)
                    .map_err(|_| {
                        StoreError::Conflict(
                            "managed review source identity changed or unavailable".into(),
                        )
                    })?;
            if worktree.path != Path::new(path)
                || !worktree
                    .is_clean()
                    .map_err(|_| StoreError::Conflict("cannot inspect current source".into()))?
            {
                return Err(StoreError::Conflict(
                    "review source has uncommitted changes; old checks cannot certify it".into(),
                ));
            }
            let head = worktree
                .head()
                .map_err(|_| StoreError::Conflict("cannot observe source revision".into()))?;
            serde_json::to_value(
                state
                    .store
                    .decide_review(id, &review_id, &head, &decision, &feedback)?,
            )?
        }
    };
    Ok(result)
}

pub fn observed_head(path: &Path) -> Result<String, StoreError> {
    let output = std::process::Command::new("/usr/bin/git")
        .args(["-C"])
        .arg(path)
        .args(["rev-parse", "--verify", "HEAD"])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()?;
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success()
        || !(sha.len() == 40 || sha.len() == 64)
        || !sha.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(StoreError::Conflict(
            "cannot observe source revision".into(),
        ));
    }
    Ok(sha)
}
