//! Codex 0.159.3 app-server transport. Factory state owns continuation, permissions,
//! approvals and provider identity persistence; no native goal is enabled here.
use crate::model::{Access, Permission, ProjectSettings};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, ChildStdin, Command},
    sync::mpsc,
    task::JoinHandle,
    time::{timeout, Instant},
};

const FRAME_CAP: usize = 1024 * 1024;
const TEXT_CAP: usize = 128 * 1024;
const RPC_TIMEOUT: Duration = Duration::from_secs(20);
const INTERRUPT_GRACE: Duration = Duration::from_secs(2);
const DISABLED_FEATURES: &[&str] = &[
    "apps",
    "plugins",
    "remote_plugin",
    "hooks",
    "goals",
    "memories",
    "multi_agent",
    "skill_mcp_dependency_install",
    "in_app_local_automation",
    "shell_snapshot",
    "request_permissions_tool",
    "browser_use",
    "browser_use_external",
    "browser_use_full_cdp_access",
    "computer_use",
    "workspace_dependencies",
];

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Provider IO failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("Provider protocol violation: {0}")]
    Protocol(&'static str),
    #[error("Provider ended without a matching terminal turn")]
    UnexpectedEof,
    #[error("Provider RPC {method} failed with code {code}")]
    Rpc { method: String, code: i64 },
    #[error("Provider metadata deadline expired")]
    MetadataDeadline,
    #[error("Provider turn deadline expired")]
    Deadline,
    #[error("Provider output usage limit reached")]
    UsageLimit,
    #[error("Provider control is unresolved after owned group cleanup: {reason}")]
    ControlUnresolved { reason: &'static str },
    #[error("Unsupported execution policy: {0}")]
    Policy(String),
    #[error("Requested model or reasoning effort is not in the provider catalog")]
    UnsupportedModel,
    #[error("Durable provider event could not be saved: {0}")]
    EventSink(String),
}

#[derive(Debug, Clone)]
pub struct CodexOptions {
    pub executable: PathBuf,
    pub cwd: PathBuf,
    pub settings: ProjectSettings,
    pub thread_id: Option<String>,
    pub planner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapability {
    pub id: String,
    pub model: String,
    pub is_default: bool,
    pub efforts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    pub available: bool,
    pub version: Option<String>,
    pub authenticated: bool,
    pub models: Vec<Value>,
    pub limitations: Vec<String>,
}

/// Metadata only: never creates/resumes a thread or starts inference. Raw account
/// and configuration payloads stay inside the adapter and are never returned.
pub async fn discover_capabilities(
    executable: PathBuf,
    cwd: PathBuf,
) -> Result<ProviderCapabilities, ProviderError> {
    let options = CodexOptions {
        executable,
        cwd: cwd.canonicalize()?,
        settings: ProjectSettings::default(),
        thread_id: None,
        planner: true,
    };
    let mut process = RpcProcess::spawn(&options, &[], &[], false).await?;
    let initialized = process
        .rpc(
            "initialize",
            json!({"clientInfo":{"name":"software_factory","version":"0.1.0"}}),
        )
        .await?;
    process
        .send(&json!({"method":"initialized","params":{}}))
        .await?;
    let version = initialized["userAgent"]
        .as_str()
        .and_then(|agent| {
            agent.split_whitespace().find_map(|token| {
                token
                    .strip_prefix("codex_cli_rs/")
                    .or_else(|| token.strip_prefix("codex/"))
            })
        })
        .filter(|s| s.chars().all(|c| c.is_ascii_digit() || c == '.'))
        .map(str::to_owned);
    let account = process
        .rpc("account/read", json!({"refreshToken":false}))
        .await?;
    let authenticated = account.get("account").is_some_and(|v| !v.is_null());
    drop(account);
    let mut models = Vec::new();
    let mut cursor = Value::Null;
    loop {
        let page = process
            .rpc("model/list", json!({"limit":100,"cursor":cursor}))
            .await?;
        for model in page["data"]
            .as_array()
            .ok_or(ProviderError::Protocol("missing model catalog"))?
        {
            models.push(json!({"id":string_at(model,"model")?,"displayName":model["displayName"],
                "defaultReasoningEffort":model["defaultReasoningEffort"],"supportedReasoningEfforts":model["supportedReasoningEfforts"]}));
        }
        cursor = page["nextCursor"].clone();
        if cursor.is_null() {
            break;
        }
        if models.len() > 1000 {
            return Err(ProviderError::Protocol("model catalog exceeds limit"));
        }
    }
    process.shutdown().await?;
    Ok(ProviderCapabilities {
        available: true,
        version,
        authenticated,
        models,
        limitations: vec![
            "Catalog availability does not establish successful inference or usage entitlement."
                .into(),
            "Legacy sandbox limits writes and command networking but permits host reads.".into(),
            "Output-token allowance is enforced when usage is observed, not an exact prepaid cap."
                .into(),
            "External integrations and native goal continuation are disabled.".into(),
        ],
    })
}

#[derive(Debug, Clone)]
pub enum ProviderControl {
    Interrupt,
    Steer { id: String, text: String },
    Approval { request_id: String, allow: bool },
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ProviderEvent {
    TurnStarted {
        thread_id: String,
        turn_id: String,
    },
    Message {
        text: String,
    },
    Usage {
        output_tokens: u64,
        total_tokens: u64,
    },
    Approval {
        request_id: String,
        thread_id: String,
        turn_id: String,
        item_id: String,
        method: String,
        detail: String,
    },
    SteeringAccepted {
        id: String,
        turn_id: String,
    },
    ControlRejected {
        reason: String,
    },
    Unknown {
        method: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Completed,
    Failed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnOutcome {
    pub thread_id: String,
    pub turn_id: String,
    pub status: ProviderStatus,
    pub text: String,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub terminal_observed: bool,
    pub process_stopped: bool,
    /// A killed app-server group does not prove detached tool descendants stopped.
    pub control_unresolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

enum Wire {
    Message(Value),
    Invalid(&'static str),
    Eof,
}

struct RpcProcess {
    child: Child,
    stdin: ChildStdin,
    incoming: mpsc::Receiver<Wire>,
    stdout_task: JoinHandle<()>,
    stderr_task: JoinHandle<()>,
    stderr_bytes: Arc<AtomicU64>,
    metadata_notifications: usize,
    next_id: u64,
    pgid: u32,
    stopped: bool,
}

impl RpcProcess {
    async fn spawn(
        options: &CodexOptions,
        mcp: &[String],
        skills: &[String],
        shell: bool,
    ) -> Result<Self, ProviderError> {
        let mut command = Command::new(&options.executable);
        command
            .args(["app-server", "--listen", "stdio://", "--strict-config"])
            .current_dir(&options.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        // Do not forward daemon credentials or arbitrary user tokens into tools.
        command.env_clear();
        for name in [
            "HOME",
            "PATH",
            "TMPDIR",
            "LANG",
            "LC_ALL",
            "CODEX_HOME",
            "CODEX_CA_CERT",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
            "NODE_EXTRA_CA_CERTS",
        ] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        for feature in DISABLED_FEATURES {
            command.args(["-c", &format!("features.{feature}=false")]);
        }
        command.args([
            "-c",
            &format!("features.shell_tool={shell}"),
            "-c",
            "web_search=\"disabled\"",
            "-c",
            "skills.bundled.enabled=false",
            "-c",
            "skills.include_instructions=false",
            "-c",
            "shell_environment_policy.inherit=none",
        ]);
        for server in mcp {
            // Server identifiers come from effective provider metadata, not task text.
            if !server
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(ProviderError::Policy(
                    "Unrepresentable MCP server identifier".into(),
                ));
            }
            command.args(["-c", &format!("mcp_servers.{server}.enabled=false")]);
        }
        let skill_config = skills
            .iter()
            .map(|path| format!("{{path={},enabled=false}}", json!(path)))
            .collect::<Vec<_>>()
            .join(",");
        command.args(["-c", &format!("skills.config=[{skill_config}]")]);
        #[cfg(unix)]
        {
            command.process_group(0);
        }
        let mut child = command.spawn()?;
        let pgid = child
            .id()
            .ok_or(ProviderError::Protocol("child has no process identity"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or(ProviderError::Protocol("stdin unavailable"))?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or(ProviderError::Protocol("stdout unavailable"))?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or(ProviderError::Protocol("stderr unavailable"))?;
        let (sender, incoming) = mpsc::channel(8);
        let stdout_task = tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 8192];
            loop {
                match stdout.read(&mut chunk).await {
                    Ok(0) => {
                        let _ = sender
                            .send(if buffer.is_empty() {
                                Wire::Eof
                            } else {
                                Wire::Invalid("truncated JSONL frame")
                            })
                            .await;
                        break;
                    }
                    Ok(n) => {
                        for byte in &chunk[..n] {
                            if *byte == b'\n' {
                                if !buffer.is_empty() {
                                    let wire = match serde_json::from_slice(&buffer) {
                                        Ok(value) => Wire::Message(value),
                                        Err(_) => Wire::Invalid("malformed JSONL frame"),
                                    };
                                    let invalid = matches!(wire, Wire::Invalid(_));
                                    if sender.send(wire).await.is_err() || invalid {
                                        return;
                                    }
                                    buffer.clear();
                                }
                            } else {
                                buffer.push(*byte);
                                if buffer.len() > FRAME_CAP {
                                    let _ = sender
                                        .send(Wire::Invalid("JSONL frame exceeds limit"))
                                        .await;
                                    return;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        let _ = sender.send(Wire::Invalid("stdout read failed")).await;
                        break;
                    }
                }
            }
        });
        let stderr_bytes = Arc::new(AtomicU64::new(0));
        let count = stderr_bytes.clone();
        let stderr_task = tokio::spawn(async move {
            let mut chunk = [0u8; 8192];
            while let Ok(n) = stderr.read(&mut chunk).await {
                if n == 0 {
                    break;
                }
                count.fetch_add(n as u64, Ordering::Relaxed);
            }
        });
        Ok(Self {
            child,
            stdin,
            incoming,
            stdout_task,
            stderr_task,
            stderr_bytes,
            metadata_notifications: 0,
            next_id: 1,
            pgid,
            stopped: false,
        })
    }

    async fn send(&mut self, value: &Value) -> Result<(), ProviderError> {
        let mut bytes = serde_json::to_vec(value)
            .map_err(|_| ProviderError::Protocol("cannot encode request"))?;
        if bytes.len() > FRAME_CAP {
            return Err(ProviderError::Protocol("request exceeds limit"));
        }
        bytes.push(b'\n');
        timeout(RPC_TIMEOUT, async {
            self.stdin.write_all(&bytes).await?;
            self.stdin.flush().await
        })
        .await
        .map_err(|_| ProviderError::MetadataDeadline)??;
        Ok(())
    }
    async fn request(&mut self, method: &str, params: Value) -> Result<u64, ProviderError> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"id":id,"method":method,"params":params}))
            .await?;
        Ok(id)
    }
    async fn receive(&mut self) -> Result<Value, ProviderError> {
        match self.incoming.recv().await {
            Some(Wire::Message(value)) => Ok(value),
            Some(Wire::Invalid(reason)) => Err(ProviderError::Protocol(reason)),
            Some(Wire::Eof) | None => Err(ProviderError::UnexpectedEof),
        }
    }
    async fn rpc(&mut self, method: &str, params: Value) -> Result<Value, ProviderError> {
        self.rpc_for(method, params, RPC_TIMEOUT).await
    }
    async fn rpc_for(
        &mut self,
        method: &str,
        params: Value,
        duration: Duration,
    ) -> Result<Value, ProviderError> {
        let result = timeout(duration, async {
            let id = self.request(method, params).await?;
            loop {
                let value = self.receive().await?;
                if value.get("id").and_then(Value::as_u64) == Some(id) && value.get("method").is_none() {
                    return reply_result(&value, method);
                }
                if value.get("id").is_some() && value.get("method").is_some() {
                    self.send(&json!({"id":value["id"],"error":{"code":-32601,"message":"Unsupported during metadata"}})).await?;
                } else {
                    self.metadata_notifications += 1;
                    if self.metadata_notifications >= 128 { return Err(ProviderError::Protocol("metadata notification limit")); }
                }
            }
        }).await;
        result.map_err(|_| ProviderError::MetadataDeadline)?
    }
    async fn initialize(&mut self) -> Result<(), ProviderError> {
        self.rpc("initialize", json!({"clientInfo":{"name":"software_factory","title":"Software Factory","version":"0.1.0"}})).await?;
        self.send(&json!({"method":"initialized","params":{}}))
            .await
    }
    async fn shutdown(&mut self) -> Result<(), ProviderError> {
        if self.stopped {
            return Ok(());
        }
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.pgid as i32), libc::SIGTERM);
        }
        #[cfg(not(unix))]
        {
            let _ = self.child.start_kill();
        }
        if timeout(Duration::from_millis(500), self.child.wait())
            .await
            .is_err()
        {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(self.pgid as i32), libc::SIGKILL);
            }
            let _ = self.child.start_kill();
            self.child.wait().await?;
        }
        // The parent can exit before a descendant; signal the entire owned group.
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.pgid as i32), libc::SIGKILL);
        }
        self.stopped = true;
        self.stdout_task.abort();
        self.stderr_task.abort();
        Ok(())
    }
}
impl Drop for RpcProcess {
    fn drop(&mut self) {
        if !self.stopped {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(self.pgid as i32), libc::SIGKILL);
            }
            let _ = self.child.start_kill();
        }
        self.stdout_task.abort();
        self.stderr_task.abort();
    }
}

fn reply_result(value: &Value, method: &str) -> Result<Value, ProviderError> {
    if let Some(error) = value.get("error") {
        return Err(ProviderError::Rpc {
            method: method.into(),
            code: error["code"].as_i64().unwrap_or(-32603),
        });
    }
    value
        .get("result")
        .cloned()
        .ok_or(ProviderError::Protocol("missing RPC result"))
}
fn string_at<'a>(value: &'a Value, field: &str) -> Result<&'a str, ProviderError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or(ProviderError::Protocol("missing required string"))
}

fn verify_sandbox(actual: &Value, expected: &Value) -> Result<(), ProviderError> {
    let invalid = || ProviderError::Policy("Provider changed requested sandbox boundaries".into());
    if actual["type"] != expected["type"] {
        return Err(ProviderError::Policy(
            "Provider changed sandbox type".into(),
        ));
    }
    if actual["networkAccess"].as_bool().unwrap_or(false) != expected["networkAccess"] {
        return Err(ProviderError::Policy(
            "Provider changed sandbox networkAccess".into(),
        ));
    }
    if expected["type"] == "workspaceWrite" {
        if actual["excludeSlashTmp"] != true {
            return Err(ProviderError::Policy(
                "Provider did not exclude /tmp writes".into(),
            ));
        }
        if actual["excludeTmpdirEnvVar"] != true {
            return Err(ProviderError::Policy(
                "Provider did not exclude TMPDIR writes".into(),
            ));
        }
        let roots = actual["writableRoots"].as_array().ok_or_else(invalid)?;
        let desired = expected["writableRoots"].as_array().ok_or_else(invalid)?;
        let mut roots = roots
            .iter()
            .map(|v| v.as_str().ok_or_else(invalid))
            .collect::<Result<Vec<_>, _>>()?;
        let mut desired = desired
            .iter()
            .map(|v| v.as_str().ok_or_else(invalid))
            .collect::<Result<Vec<_>, _>>()?;
        // Legacy writableRoots are additional to the already-verified cwd;
        // Codex normalizes an explicit duplicate of cwd to an empty list.
        if let Some(cwd) = desired.first() {
            roots.push(cwd);
        }
        roots.sort_unstable();
        roots.dedup();
        desired.sort_unstable();
        desired.dedup();
        if roots != desired {
            return Err(ProviderError::Policy(format!(
                "Provider writable roots differ (observed {}, expected {})",
                roots.len(),
                desired.len()
            )));
        }
    }
    Ok(())
}
fn metadata_config(config: &Value) -> Result<Vec<String>, ProviderError> {
    config.get("mcp_servers").map_or(Ok(Vec::new()), |servers| {
        servers
            .as_object()
            .map(|v| v.keys().cloned().collect())
            .ok_or(ProviderError::Protocol("invalid MCP configuration"))
    })
}
fn skill_paths(value: &Value) -> Result<Vec<String>, ProviderError> {
    let groups = value["data"]
        .as_array()
        .ok_or(ProviderError::Protocol("missing skill metadata"))?;
    let mut paths = Vec::new();
    for group in groups {
        if group["errors"].as_array().is_some_and(|e| !e.is_empty()) {
            return Err(ProviderError::Policy(
                "Skill discovery errors prevent safe execution".into(),
            ));
        }
        for skill in group["skills"]
            .as_array()
            .ok_or(ProviderError::Protocol("missing skill list"))?
        {
            paths.push(string_at(skill, "path")?.to_owned());
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub struct CodexSession {
    process: RpcProcess,
    options: CodexOptions,
    thread_id: String,
    model: String,
    effort: String,
    catalog: Vec<ModelCapability>,
    sandbox: Value,
}

impl CodexSession {
    /// No model turn starts here. The caller must persist thread_id before run().
    pub async fn connect(mut options: CodexOptions) -> Result<Self, ProviderError> {
        options.cwd = options.cwd.canonicalize()?;
        if options.settings.max_turn_seconds == 0 || options.settings.max_output_tokens == 0 {
            return Err(ProviderError::Policy(
                "Time and usage limits must be positive".into(),
            ));
        }
        if !options.planner
            && (options.settings.permissions.commands == Permission::Ask
                || options.settings.permissions.network == Permission::Ask)
        {
            return Err(ProviderError::Policy(
                "Resolve scoped Commands/Network approvals before dispatch".into(),
            ));
        }
        if !options.planner && options.settings.permissions.integrations == Permission::Allow {
            return Err(ProviderError::Policy(
                "Integration execution is unavailable in this adapter".into(),
            ));
        }
        let mut discovery = RpcProcess::spawn(&options, &[], &[], false).await?;
        discovery.initialize().await?;
        let config = discovery
            .rpc(
                "config/read",
                json!({"includeLayers":false,"cwd":options.cwd}),
            )
            .await?;
        let mcp = metadata_config(&config["config"])?;
        let skills = skill_paths(
            &discovery
                .rpc("skills/list", json!({"cwds":[options.cwd]}))
                .await?,
        )?;
        discovery.shutdown().await?;
        let shell = !options.planner && options.settings.permissions.commands == Permission::Allow;
        let mut process = RpcProcess::spawn(&options, &mcp, &skills, shell).await?;
        process.initialize().await?;
        let config = process
            .rpc(
                "config/read",
                json!({"includeLayers":false,"cwd":options.cwd}),
            )
            .await?;
        let effective = &config["config"];
        for feature in DISABLED_FEATURES {
            if effective["features"][feature] != false {
                return Err(ProviderError::Policy(format!(
                    "Cannot disable capability {feature}"
                )));
            }
        }
        if effective["features"]["shell_tool"] != shell || effective["web_search"] != "disabled" {
            return Err(ProviderError::Policy(
                "Command/search policy was not applied".into(),
            ));
        }
        if effective
            .get("mcp_servers")
            .and_then(Value::as_object)
            .is_some_and(|servers| servers.values().any(|server| server["enabled"] != false))
        {
            return Err(ProviderError::Policy(
                "An MCP server remains enabled".into(),
            ));
        }
        let verified_skills = process
            .rpc("skills/list", json!({"cwds":[options.cwd]}))
            .await?;
        if skill_paths(&verified_skills)? != skills
            || verified_skills["data"]
                .as_array()
                .unwrap()
                .iter()
                .any(|group| {
                    group["skills"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|skill| skill["enabled"] != false)
                })
        {
            return Err(ProviderError::Policy(
                "Skills changed or remain enabled".into(),
            ));
        }
        let account = process
            .rpc("account/read", json!({"refreshToken":false}))
            .await?;
        if account.get("account").is_none_or(Value::is_null) {
            return Err(ProviderError::Policy(
                "Sign in with the supported Codex login flow".into(),
            ));
        }
        let mut catalog = Vec::new();
        let mut cursor = Value::Null;
        loop {
            let page = process
                .rpc("model/list", json!({"limit":100,"cursor":cursor}))
                .await?;
            for model in page["data"]
                .as_array()
                .ok_or(ProviderError::Protocol("missing model catalog"))?
            {
                let efforts = model["supportedReasoningEfforts"]
                    .as_array()
                    .ok_or(ProviderError::Protocol("missing efforts"))?
                    .iter()
                    .map(|v| string_at(v, "reasoningEffort").map(str::to_owned))
                    .collect::<Result<Vec<_>, _>>()?;
                catalog.push(ModelCapability {
                    id: string_at(model, "id")?.into(),
                    model: string_at(model, "model")?.into(),
                    is_default: model["isDefault"].as_bool().unwrap_or(false),
                    efforts,
                });
            }
            cursor = page["nextCursor"].clone();
            if cursor.is_null() {
                break;
            }
            if catalog.len() > 1000 {
                return Err(ProviderError::Protocol("model catalog exceeds limit"));
            }
        }
        let selected = catalog
            .iter()
            .find(|m| {
                if options.settings.model.is_empty() {
                    m.is_default
                } else {
                    m.id == options.settings.model || m.model == options.settings.model
                }
            })
            .ok_or(ProviderError::UnsupportedModel)?;
        if !selected.efforts.contains(&options.settings.effort) {
            return Err(ProviderError::UnsupportedModel);
        }
        let model = selected.model.clone();
        let effort = options.settings.effort.clone();
        let write = !options.planner && options.settings.access == Access::Worktree;
        let sandbox = if write {
            json!({"type":"workspaceWrite","writableRoots":[options.cwd],
            "networkAccess":options.settings.permissions.network == Permission::Allow,
            "excludeSlashTmp":true,"excludeTmpdirEnvVar":true})
        } else {
            json!({"type":"readOnly","networkAccess":!options.planner&&options.settings.permissions.network==Permission::Allow})
        };
        let mut params = json!({"cwd":options.cwd,"model":model,"sandbox":if write {"workspace-write"} else {"read-only"},
            "approvalPolicy":"never","approvalsReviewer":"user",
            "config":{"model_reasoning_effort":effort,"sandbox_workspace_write":{"network_access":options.settings.permissions.network == Permission::Allow,
                "writable_roots":[options.cwd],
                "exclude_slash_tmp":true,"exclude_tmpdir_env_var":true}}});
        let method = if let Some(thread) = &options.thread_id {
            params["threadId"] = json!(thread);
            params["excludeTurns"] = json!(true);
            "thread/resume"
        } else {
            "thread/start"
        };
        let response = process.rpc(method, params).await?;
        let thread_id = string_at(&response["thread"], "id")?.to_owned();
        if response["cwd"].as_str() != options.cwd.to_str()
            || response["model"] != model
            || response["approvalPolicy"] != "never"
            || response["reasoningEffort"] != effort
        {
            return Err(ProviderError::Policy(
                "Provider changed requested execution settings".into(),
            ));
        }
        verify_sandbox(&response["sandbox"], &sandbox)?;
        Ok(Self {
            process,
            options,
            thread_id,
            model,
            effort,
            catalog,
            sandbox,
        })
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }
    pub fn pid(&self) -> u32 {
        self.process.pgid
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    pub fn effort(&self) -> &str {
        &self.effort
    }
    pub fn catalog(&self) -> &[ModelCapability] {
        &self.catalog
    }
    pub fn stderr_bytes_discarded(&self) -> u64 {
        self.process.stderr_bytes.load(Ordering::Relaxed)
    }
    pub async fn shutdown(&mut self) -> Result<(), ProviderError> {
        self.process.shutdown().await
    }

    /// Read persisted provider history without starting/resuming another turn.
    pub async fn turns(&mut self) -> Result<Vec<Value>, ProviderError> {
        let mut turns = Vec::new();
        let mut cursor = Value::Null;
        loop {
            let page = self
                .process
                .rpc(
                    "thread/turns/list",
                    json!({"threadId":self.thread_id,
                "limit":100,"itemsView":"full","cursor":cursor}),
                )
                .await?;
            turns.extend(
                page["data"]
                    .as_array()
                    .ok_or(ProviderError::Protocol("missing turn history"))?
                    .iter()
                    .cloned(),
            );
            cursor = page["nextCursor"].clone();
            if cursor.is_null() {
                break;
            }
            if turns.len() > 1000 {
                return Err(ProviderError::Protocol("turn history exceeds limit"));
            }
        }
        Ok(turns)
    }

    /// Supervisor checks use the provider's OS sandbox, never thread/shellCommand.
    pub async fn check(
        &mut self,
        argv: &[String],
        cwd: &Path,
        deadline: Duration,
    ) -> Result<CommandOutcome, ProviderError> {
        if argv.is_empty()
            || deadline.is_zero()
            || self.options.planner
            || self.options.settings.permissions.commands != Permission::Allow
        {
            return Err(ProviderError::Policy(
                "Supervisor check is not authorized".into(),
            ));
        }
        let cwd = cwd.canonicalize()?;
        if !cwd.starts_with(&self.options.cwd) {
            return Err(ProviderError::Policy(
                "Check cwd leaves owned worktree".into(),
            ));
        }
        let result = self.process.rpc_for("command/exec", json!({
            "command":argv,"cwd":cwd,"sandboxPolicy":self.sandbox,"timeoutMs":deadline.as_millis().min(i64::MAX as u128) as i64,
            "outputBytesCap":65536,"env":{"HOME":null,"CODEX_HOME":null}}),deadline+Duration::from_secs(2)).await;
        let parsed = match result {
            Ok(value) => serde_json::from_value(value)
                .map_err(|_| ProviderError::Protocol("malformed command result")),
            Err(ProviderError::MetadataDeadline) => Err(ProviderError::Deadline),
            Err(error) => Err(error),
        };
        if parsed.is_err() {
            self.process.shutdown().await?;
        }
        parsed
    }

    pub async fn run<F>(
        &mut self,
        prompt: &str,
        output_schema: Option<Value>,
        mut controls: mpsc::UnboundedReceiver<ProviderControl>,
        mut on_event: F,
    ) -> Result<TurnOutcome, ProviderError>
    where
        F: FnMut(&ProviderEvent) -> Result<(), String>,
    {
        let result = self
            .run_inner(prompt, output_schema, &mut controls, &mut on_event)
            .await;
        if result.is_err() {
            self.process.shutdown().await?;
        }
        result
    }

    async fn run_inner<F>(
        &mut self,
        prompt: &str,
        output_schema: Option<Value>,
        controls: &mut mpsc::UnboundedReceiver<ProviderControl>,
        on_event: &mut F,
    ) -> Result<TurnOutcome, ProviderError>
    where
        F: FnMut(&ProviderEvent) -> Result<(), String>,
    {
        let mut params = json!({"threadId":self.thread_id,"input":[{"type":"text","text":prompt}],
            "model":self.model,"effort":self.effort,"approvalPolicy":"never","approvalsReviewer":"user",
            "sandboxPolicy":self.sandbox});
        if let Some(schema) = output_schema {
            params["outputSchema"] = schema;
        }
        let deadline = Instant::now() + Duration::from_secs(self.options.settings.max_turn_seconds);
        let start_id =
            tokio::time::timeout_at(deadline, self.process.request("turn/start", params))
                .await
                .map_err(|_| ProviderError::Deadline)??;
        let mut stop_at = None;
        let mut stopped_for = None;
        let mut turn_id = String::new();
        let mut text = String::new();
        let mut output_tokens = None;
        let mut total_tokens = None;
        let mut steering = HashMap::<u64, String>::new();
        let mut approvals = HashMap::<String, (Value, String)>::new();
        let mut controls_open = true;
        loop {
            let wait_until = stop_at.unwrap_or(deadline);
            let message = tokio::select! {
                _=tokio::time::sleep_until(wait_until) => {
                    if stopped_for.is_some() || turn_id.is_empty() {
                        self.process.shutdown().await?;
                        return match stopped_for {Some("interrupt")=>Ok(TurnOutcome{thread_id:self.thread_id.clone(),turn_id,
                            status:ProviderStatus::Interrupted,text,output_tokens,total_tokens,terminal_observed:false,process_stopped:true,control_unresolved:true}),
                            Some("usage")=>Err(ProviderError::ControlUnresolved{reason:"usage limit without matching terminal"}),
                            _=>Err(ProviderError::ControlUnresolved{reason:"deadline without matching terminal"})};
                    }
                    self.process.request("turn/interrupt",json!({"threadId":self.thread_id,"turnId":turn_id})).await?;
                    stopped_for=Some("deadline");stop_at=Some(Instant::now()+INTERRUPT_GRACE);continue;
                },
                control=controls.recv(),if controls_open=>{
                    match control {
                        None=>controls_open=false,
                        Some(ProviderControl::Interrupt)=>{
                            if turn_id.is_empty(){self.process.shutdown().await?;return Err(ProviderError::Protocol("interrupted before turn identity; reconcile before retry"));}
                            self.process.request("turn/interrupt",json!({"threadId":self.thread_id,"turnId":turn_id})).await?;
                            stopped_for=Some("interrupt");stop_at=Some(Instant::now()+INTERRUPT_GRACE);
                        },
                        Some(ProviderControl::Steer{id,text})=>{
                            if turn_id.is_empty(){on_event(&ProviderEvent::ControlRejected{reason:"Turn identity not yet known".into()}).map_err(ProviderError::EventSink)?;}
                            else {let rpc=self.process.request("turn/steer",json!({"threadId":self.thread_id,"expectedTurnId":turn_id,
                                "clientUserMessageId":id,"input":[{"type":"text","text":text}]})).await?;steering.insert(rpc,id);}
                        },
                        Some(ProviderControl::Approval{request_id,allow})=>{
                            if allow {on_event(&ProviderEvent::ControlRejected{reason:"Native sandbox expansion is unavailable; approve a bounded run before dispatch".into()}).map_err(ProviderError::EventSink)?;}
                            else if let Some((id,method))=approvals.remove(&request_id){
                                let result=if method=="item/permissions/requestApproval"{json!({"permissions":{},"scope":"turn"})}else{json!({"decision":"decline"})};
                                self.process.send(&json!({"id":id,"result":result})).await?;
                            }
                        }
                    }continue;
                },
                value=self.process.receive()=>value?,
            };
            if let Some(method) = message.get("method").and_then(Value::as_str) {
                let p = &message["params"];
                if let Some(id) = message.get("id") {
                    if [
                        "item/commandExecution/requestApproval",
                        "item/fileChange/requestApproval",
                        "item/permissions/requestApproval",
                    ]
                    .contains(&method)
                        && p["threadId"] == self.thread_id
                        && p["turnId"].as_str() == Some(&turn_id)
                    {
                        let key = id.to_string();
                        if approvals.len() >= 32 {
                            return Err(ProviderError::Protocol("pending approval limit"));
                        }
                        let detail = if method == "item/commandExecution/requestApproval" {
                            p["command"]
                                .as_str()
                                .unwrap_or("Native command requires approval")
                        } else {
                            "Native permission expansion requires approval"
                        };
                        on_event(&ProviderEvent::Approval {
                            request_id: key.clone(),
                            thread_id: self.thread_id.clone(),
                            turn_id: turn_id.clone(),
                            item_id: p["itemId"].as_str().unwrap_or("").into(),
                            method: method.into(),
                            detail: detail.chars().take(2048).collect(),
                        })
                        .map_err(ProviderError::EventSink)?;
                        approvals.insert(key, (id.clone(), method.into()));
                    } else {
                        self.process.send(&json!({"id":id,"error":{"code":-32601,"message":"Unsupported provider request"}})).await?;
                    }
                    continue;
                }
                match method {
                    "turn/started" if p["threadId"] == self.thread_id => {
                        let observed = string_at(&p["turn"], "id")?;
                        if !turn_id.is_empty() && turn_id != observed {
                            return Err(ProviderError::Protocol("conflicting turn identity"));
                        }
                        if turn_id.is_empty() {
                            turn_id = observed.into();
                            on_event(&ProviderEvent::TurnStarted {
                                thread_id: self.thread_id.clone(),
                                turn_id: turn_id.clone(),
                            })
                            .map_err(ProviderError::EventSink)?;
                        }
                    }
                    "turn/completed" if p["threadId"] == self.thread_id => {
                        if turn_id.is_empty() || p["turn"]["id"] != turn_id {
                            return Err(ProviderError::Protocol("terminal turn identity mismatch"));
                        }
                        let status = match p["turn"]["status"].as_str() {
                            Some("completed") => ProviderStatus::Completed,
                            Some("failed") => ProviderStatus::Failed,
                            Some("interrupted") => ProviderStatus::Interrupted,
                            _ => return Err(ProviderError::Protocol("invalid terminal status")),
                        };
                        if stopped_for == Some("deadline") {
                            return Err(ProviderError::Deadline);
                        }
                        if stopped_for == Some("usage") {
                            return Err(ProviderError::UsageLimit);
                        }
                        return Ok(TurnOutcome {
                            thread_id: self.thread_id.clone(),
                            turn_id,
                            status,
                            text,
                            output_tokens,
                            total_tokens,
                            terminal_observed: true,
                            process_stopped: false,
                            control_unresolved: false,
                        });
                    }
                    "item/agentMessage/delta"
                        if p["threadId"] == self.thread_id && p["turnId"] == turn_id =>
                    {
                        let delta = p["delta"]
                            .as_str()
                            .ok_or(ProviderError::Protocol("missing message delta"))?;
                        if text.len() + delta.len() > TEXT_CAP {
                            return Err(ProviderError::Protocol("assistant output exceeds limit"));
                        }
                        text.push_str(delta);
                        on_event(&ProviderEvent::Message { text: delta.into() })
                            .map_err(ProviderError::EventSink)?;
                    }
                    "item/completed"
                        if p["threadId"] == self.thread_id
                            && p["turnId"] == turn_id
                            && p["item"]["type"] == "agentMessage" =>
                    {
                        if let Some(final_text) = p["item"]["text"].as_str() {
                            if final_text.len() > TEXT_CAP {
                                return Err(ProviderError::Protocol(
                                    "assistant output exceeds limit",
                                ));
                            }
                            text = final_text.into();
                        }
                    }
                    "thread/tokenUsage/updated"
                        if p["threadId"] == self.thread_id && p["turnId"] == turn_id =>
                    {
                        let output_count = p["tokenUsage"]["last"]["outputTokens"]
                            .as_u64()
                            .ok_or(ProviderError::Protocol("missing usage output count"))?;
                        let total_count = p["tokenUsage"]["last"]["totalTokens"]
                            .as_u64()
                            .ok_or(ProviderError::Protocol("missing usage total count"))?;
                        output_tokens = Some(output_count);
                        total_tokens = Some(total_count);
                        on_event(&ProviderEvent::Usage {
                            output_tokens: output_count,
                            total_tokens: total_count,
                        })
                        .map_err(ProviderError::EventSink)?;
                        if output_count > self.options.settings.max_output_tokens
                            && stopped_for.is_none()
                        {
                            self.process
                                .request(
                                    "turn/interrupt",
                                    json!({"threadId":self.thread_id,"turnId":turn_id}),
                                )
                                .await?;
                            stopped_for = Some("usage");
                            stop_at = Some(Instant::now() + INTERRUPT_GRACE);
                        }
                    }
                    _ => on_event(&ProviderEvent::Unknown {
                        method: method.chars().take(128).collect(),
                    })
                    .map_err(ProviderError::EventSink)?,
                }
            } else if let Some(id) = message["id"].as_u64() {
                if id == start_id {
                    let result = reply_result(&message, "turn/start")?;
                    let observed = string_at(&result["turn"], "id")?;
                    if !turn_id.is_empty() && turn_id != observed {
                        return Err(ProviderError::Protocol("conflicting start response"));
                    }
                    if turn_id.is_empty() {
                        turn_id = observed.into();
                        on_event(&ProviderEvent::TurnStarted {
                            thread_id: self.thread_id.clone(),
                            turn_id: turn_id.clone(),
                        })
                        .map_err(ProviderError::EventSink)?;
                    }
                } else if let Some(steer_id) = steering.remove(&id) {
                    match reply_result(&message, "turn/steer") {
                        Ok(result) if result["turnId"] == turn_id => {
                            on_event(&ProviderEvent::SteeringAccepted {
                                id: steer_id,
                                turn_id: turn_id.clone(),
                            })
                            .map_err(ProviderError::EventSink)?
                        }
                        _ => on_event(&ProviderEvent::ControlRejected {
                            reason: "Provider did not accept steering".into(),
                        })
                        .map_err(ProviderError::EventSink)?,
                    }
                }
            } else {
                return Err(ProviderError::Protocol("unclassified protocol message"));
            }
        }
    }
}
