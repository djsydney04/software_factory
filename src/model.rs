use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Deny,
    Ask,
    Allow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Access {
    #[serde(rename = "read-only")]
    ReadOnly,
    #[serde(rename = "worktree")]
    Worktree,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    pub commands: Permission,
    pub network: Permission,
    pub integrations: Permission,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            commands: Permission::Ask,
            network: Permission::Ask,
            integrations: Permission::Ask,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub model: String,
    pub effort: String,
    pub access: Access,
    pub permissions: Permissions,
    pub max_turn_seconds: u64,
    pub max_output_tokens: u64,
    pub max_attempts: u32,
    pub review_queue_cap: u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            model: String::new(),
            effort: "medium".into(),
            access: Access::Worktree,
            permissions: Permissions::default(),
            max_turn_seconds: 180,
            max_output_tokens: 8000,
            max_attempts: 1,
            review_queue_cap: 3,
        }
    }
}

impl ProjectSettings {
    /// Model choice is requested; permissions and resource limits can only narrow the ceiling.
    pub fn within(&self, ceiling: &Self) -> Self {
        let mut value = self.clone();
        value.access = if self.access == Access::Worktree && ceiling.access == Access::Worktree {
            Access::Worktree
        } else {
            Access::ReadOnly
        };
        value.permissions.commands = self.permissions.commands.min(ceiling.permissions.commands);
        value.permissions.network = self.permissions.network.min(ceiling.permissions.network);
        value.permissions.integrations = self
            .permissions
            .integrations
            .min(ceiling.permissions.integrations);
        value.max_turn_seconds = self.max_turn_seconds.min(ceiling.max_turn_seconds);
        value.max_output_tokens = self.max_output_tokens.min(ceiling.max_output_tokens);
        value.max_attempts = self.max_attempts.min(ceiling.max_attempts);
        value.review_queue_cap = self.review_queue_cap.min(ceiling.review_queue_cap);
        value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProject {
    pub name: String,
    pub repo_path: String,
    pub objective: String,
    #[serde(default)]
    pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repo_path: String,
    pub objective: String,
    pub paused: bool,
    pub context_revision: u64,
    pub settings: ProjectSettings,
    pub lead_thread_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Steering {
    pub id: String,
    pub project_id: String,
    pub revision: u64,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSnapshot {
    pub revision: u64,
    pub objective: String,
    pub repo_path: String,
    pub steering: Vec<Steering>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadWakeKind {
    Steering,
    CompletedRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadWakeStatus {
    Waiting,
    Prepared,
    Running,
    Completed,
    Failed,
    Covered,
    RecoveryRequired,
}

impl LeadWakeStatus {
    pub fn fences_execution(self) -> bool {
        matches!(
            self,
            Self::Prepared | Self::Running | Self::RecoveryRequired
        )
    }
}

/// A justified lead turn, separate from submitted intent and worker attempts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadWake {
    pub id: String,
    pub key: String,
    pub project_id: String,
    pub kind: LeadWakeKind,
    pub source_id: String,
    pub source_revision: Option<u64>,
    pub context_revision: u64,
    pub context: ContextSnapshot,
    pub requested_settings: ProjectSettings,
    pub effective_settings: Option<ProjectSettings>,
    pub status: LeadWakeStatus,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RequestPolicy {
    #[default]
    LeadDecides,
    NextAvailable,
    Scheduled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Pending,
    Planning,
    Waiting,
    Planned,
    Cancelled,
    RecoveryRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRequest {
    pub project_id: String,
    pub text: String,
    #[serde(default)]
    pub settings: Option<ProjectSettings>,
    #[serde(default)]
    pub policy: RequestPolicy,
    #[serde(default)]
    pub scheduled_at: Option<String>,
    #[serde(default)]
    pub time_zone: Option<String>,
    #[serde(default = "conversation_source")]
    pub source: String,
}
fn conversation_source() -> String {
    "conversation".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub id: String,
    pub project_id: String,
    pub text: String,
    pub policy: RequestPolicy,
    pub scheduled_at: Option<String>,
    pub time_zone: Option<String>,
    pub source: String,
    pub requested_settings: ProjectSettings,
    pub settings: ProjectSettings,
    pub context_revision: u64,
    pub context: ContextSnapshot,
    pub status: RequestStatus,
    pub reason: Option<String>,
    pub planning_thread_id: Option<String>,
    pub planning_turn_id: Option<String>,
    pub planning_summary: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckSpec {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTask {
    pub project_id: String,
    pub request_id: String,
    pub title: String,
    pub description: String,
    pub acceptance: Vec<String>,
    pub checks: Vec<CheckSpec>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Ready,
    Running,
    Review,
    Blocked,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub request_id: String,
    pub title: String,
    pub description: String,
    pub acceptance: Vec<String>,
    pub checks: Vec<CheckSpec>,
    pub depends_on: Vec<String>,
    pub status: TaskStatus,
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Claimed,
    Running,
    NeedsApproval,
    Checking,
    Interrupting,
    RecoveryRequired,
    Succeeded,
    Failed,
    Interrupted,
    Cancelled,
}
impl RunStatus {
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Claimed
                | Self::Running
                | Self::NeedsApproval
                | Self::Checking
                | Self::Interrupting
                | Self::RecoveryRequired
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunAttempt {
    pub id: String,
    pub project_id: String,
    pub task_id: String,
    pub request_id: String,
    pub attempt: u32,
    pub status: RunStatus,
    pub requested_settings: ProjectSettings,
    pub effective_settings: ProjectSettings,
    pub authorization: Option<ExecutionAuthorization>,
    pub context_revision: u64,
    pub context: ContextSnapshot,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub worktree_path: Option<String>,
    pub base_sha: Option<String>,
    pub head_sha: Option<String>,
    pub pid: Option<u32>,
    pub process_started_at: Option<u64>,
    pub executor_id: Option<String>,
    pub checks: Vec<CheckEvidence>,
    pub reason: Option<String>,
    pub summary: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionAuthorization {
    pub settings: ProjectSettings,
    pub approval_ids: Vec<String>,
    pub limitations: Vec<String>,
    pub authorized_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RunUpdate {
    pub run_id: String,
    pub status: Option<RunStatus>,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub worktree_path: Option<String>,
    pub base_sha: Option<String>,
    pub head_sha: Option<String>,
    pub pid: Option<u32>,
    pub process_started_at: Option<u64>,
    pub executor_id: Option<String>,
    pub checks: Option<Vec<CheckEvidence>>,
    pub reason: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub sequence: u64,
    pub project_id: String,
    pub run_id: Option<String>,
    pub kind: String,
    pub payload: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckEvidence {
    pub name: String,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub revision: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub output: String,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub title: String,
    pub kind: String,
    pub path: String,
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReview {
    pub run_id: String,
    pub revision: String,
    pub diff: String,
    pub checks: Vec<CheckEvidence>,
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    #[serde(default)]
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReview {
    pub id: String,
    pub project_id: String,
    pub run_id: String,
    pub task_id: String,
    pub attempt: u32,
    pub revision: String,
    pub diff: String,
    pub checks: Vec<CheckEvidence>,
    pub artifacts: Vec<Artifact>,
    pub limitations: Vec<String>,
    pub decision: Option<String>,
    pub feedback: Option<String>,
    pub decided_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    pub id: String,
    pub project_id: String,
    pub run_id: String,
    pub provider_request_id: String,
    pub scope: String,
    pub detail: String,
    pub decision: Option<bool>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub last_event_sequence: u64,
    pub projects: Vec<Project>,
    pub steering: Vec<Steering>,
    pub requests: Vec<Request>,
    pub tasks: Vec<Task>,
    pub runs: Vec<RunAttempt>,
    pub reviews: Vec<LocalReview>,
    pub approvals: Vec<Approval>,
    pub events: Vec<Event>,
    #[serde(default)]
    pub lead_wakes: Vec<LeadWake>,
    #[serde(default)]
    pub review_sources: Vec<ReviewSourceState>,
    #[serde(default)]
    pub repository_connections: Vec<RepositoryConnection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryProvider {
    Github,
    Gitlab,
}

impl RepositoryProvider {
    pub fn host(self) -> &'static str {
        match self {
            Self::Github => "github.com",
            Self::Gitlab => "gitlab.com",
        }
    }
    pub fn cli(self) -> &'static str {
        match self {
            Self::Github => "gh",
            Self::Gitlab => "glab",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryTarget {
    pub provider: RepositoryProvider,
    pub host: String,
    pub path: String,
    pub web_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryConnection {
    pub project_id: String,
    #[serde(default = "origin_remote")]
    pub remote_name: String,
    pub provider: RepositoryProvider,
    pub host: String,
    pub path: String,
    pub web_url: String,
    pub connected_at: String,
}

fn origin_remote() -> String {
    "origin".into()
}

impl RepositoryConnection {
    pub fn target(&self) -> RepositoryTarget {
        RepositoryTarget {
            provider: self.provider,
            host: self.host.clone(),
            path: self.path.clone(),
            web_url: self.web_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSourceState {
    pub review_id: String,
    pub source_current: bool,
    pub observed_head: Option<String>,
    pub reason: Option<String>,
}
