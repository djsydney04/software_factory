use crate::model::*;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
    time::Duration,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0} not found")]
    NotFound(String),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("stored data error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, StoreError>;

/// Connections never survive a method call or cross an async wait.
#[derive(Clone, Debug)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path == Path::new(":memory:") {
            return Err(StoreError::Invalid("use a durable database path".into()));
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let store = Self {
            path: path.to_owned(),
        };
        let mut connection = store.connect()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        connection.pragma_update(None, "journal_mode", "WAL")?;
        let version: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > 2 {
            return Err(StoreError::Invalid(
                "database schema is newer than this application".into(),
            ));
        }
        if version == 0 {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tx.execute_batch(SCHEMA)?;
            tx.pragma_update(None, "user_version", 1)?;
            tx.commit()?;
        }
        if version < 2 {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tx.execute_batch("CREATE TABLE lead_wakes(id TEXT PRIMARY KEY,project_id TEXT NOT NULL REFERENCES projects(id),wake_key TEXT NOT NULL UNIQUE,json TEXT NOT NULL);")?;
            tx.pragma_update(None, "user_version", 2)?;
            tx.commit()?;
        }
        Ok(store)
    }

    fn connect(&self) -> Result<Connection> {
        let connection = Connection::open(&self.path)?;
        connection.busy_timeout(Duration::from_secs(10))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(connection)
    }

    fn command<I: Serialize, T: Serialize + DeserializeOwned>(
        &self,
        command_id: &str,
        kind: &str,
        input: &I,
        apply: impl FnOnce(&Transaction<'_>) -> Result<T>,
    ) -> Result<T> {
        text(command_id, "command identity", 200)?;
        let payload = serde_json::to_string(input)?;
        let mut connection = self.connect()?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let previous: Option<(String, String, String)> = tx
            .query_row(
                "SELECT kind,payload,result FROM commands WHERE id=?1",
                [command_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        if let Some((old_kind, old_payload, result)) = previous {
            if old_kind != kind || old_payload != payload {
                return Err(StoreError::Conflict(
                    "command identity was already used for different input".into(),
                ));
            }
            return Ok(serde_json::from_str(&result)?);
        }
        let result = apply(&tx)?;
        tx.execute(
            "INSERT INTO commands(id,kind,payload,result) VALUES(?1,?2,?3,?4)",
            params![command_id, kind, payload, serde_json::to_string(&result)?],
        )?;
        tx.commit()?;
        Ok(result)
    }

    pub fn create_project(&self, command_id: &str, input: NewProject) -> Result<Project> {
        text(&input.name, "project name", 200)?;
        text(&input.objective, "objective", 32_000)?;
        validate_settings(&input.settings)?;
        self.command(command_id, "create_project", &input, |tx| {
            if tx.query_row("SELECT count(*) FROM projects", [], |r| r.get::<_, i64>(0))? != 0 {
                return Err(StoreError::Conflict(
                    "this local alpha supports one project".into(),
                ));
            }
            let repo = validate_repo(&input.repo_path)?;
            let project = Project {
                id: id(),
                name: input.name.trim().into(),
                repo_path: repo.to_str().ok_or_else(|| StoreError::Invalid("repository path must be UTF-8".into()))?.into(),
                objective: input.objective.trim().into(),
                paused: false,
                context_revision: 1,
                settings: input.settings.clone(),
                lead_thread_id: None,
                created_at: now(),
            };
            insert(tx, "projects", &project.id, &project)?;
            event(
                tx,
                &project.id,
                None,
                "project_created",
                json!({"projectId":project.id,"objective":project.objective,"contextRevision":project.context_revision}),
            )?;
            Ok(project)
        })
    }

    pub fn project(&self, project_id: &str) -> Result<Project> {
        read(&self.connect()?, "projects", project_id)
    }

    pub fn set_objective(
        &self,
        command_id: &str,
        project_id: &str,
        objective: &str,
    ) -> Result<Project> {
        text(objective, "objective", 32_000)?;
        self.command(command_id,"set_objective",&(project_id,objective),|tx| {
            let mut project: Project = read(tx,"projects",project_id)?;
            project.objective = objective.trim().into();
            project.context_revision = next_revision(project.context_revision)?;
            replace(tx,"projects",project_id,&project)?;
            event(tx,project_id,None,"objective_saved",json!({"objective":project.objective,"contextRevision":project.context_revision}))?;
            Ok(project)
        })
    }

    pub fn set_lead_thread(
        &self,
        command_id: &str,
        project_id: &str,
        thread_id: &str,
    ) -> Result<Project> {
        text(thread_id, "lead thread identity", 300)?;
        self.command(
            command_id,
            "set_lead_thread",
            &(project_id, thread_id),
            |tx| {
                let mut project: Project = read(tx, "projects", project_id)?;
                immutable_field(
                    &mut project.lead_thread_id,
                    &Some(thread_id.into()),
                    "lead thread identity",
                )?;
                replace(tx, "projects", project_id, &project)?;
                event(
                    tx,
                    project_id,
                    None,
                    "lead_thread_recorded",
                    json!({"threadId":thread_id}),
                )?;
                Ok(project)
            },
        )
    }

    pub fn save_settings(
        &self,
        command_id: &str,
        project_id: &str,
        settings: ProjectSettings,
    ) -> Result<Project> {
        validate_settings(&settings)?;
        self.command(
            command_id,
            "save_settings",
            &(project_id, &settings),
            |tx| {
                let mut project: Project = read(tx, "projects", project_id)?;
                project.settings = settings.clone();
                replace(tx, "projects", project_id, &project)?;
                event(
                    tx,
                    project_id,
                    None,
                    "settings_saved",
                    json!({"settings":settings}),
                )?;
                Ok(project)
            },
        )
    }

    pub fn set_paused(&self, command_id: &str, project_id: &str, paused: bool) -> Result<Project> {
        self.command(command_id, "set_paused", &(project_id, paused), |tx| {
            let mut project: Project = read(tx, "projects", project_id)?;
            project.paused = paused;
            replace(tx, "projects", project_id, &project)?;
            event(
                tx,
                project_id,
                None,
                if paused {
                    "dispatch_paused"
                } else {
                    "dispatch_resumed"
                },
                json!({"paused":paused}),
            )?;
            Ok(project)
        })
    }

    pub fn add_steering(
        &self,
        command_id: &str,
        project_id: &str,
        guidance: &str,
    ) -> Result<Steering> {
        text(guidance, "steering", 32_000)?;
        self.command(command_id, "add_steering", &(project_id, guidance), |tx| {
            let mut project: Project = read(tx, "projects", project_id)?;
            project.context_revision = next_revision(project.context_revision)?;
            let steering = Steering {
                id: id(),
                project_id: project_id.into(),
                revision: project.context_revision,
                text: guidance.trim().into(),
                created_at: now(),
            };
            tx.execute(
                "INSERT INTO steering(id,project_id,revision,json) VALUES(?1,?2,?3,?4)",
                params![
                    steering.id,
                    project_id,
                    sql_integer(steering.revision, "context revision")?,
                    serde_json::to_string(&steering)?
                ],
            )?;
            replace(tx, "projects", project_id, &project)?;
            event(
                tx,
                project_id,
                None,
                "steering_received",
                json!({"steeringId":steering.id,"revision":steering.revision,"text":steering.text}),
            )?;
            Ok(steering)
        })
    }

    pub fn submit_request(&self, command_id: &str, input: NewRequest) -> Result<Request> {
        if input
            .time_zone
            .as_deref()
            .is_some_and(|zone| zone.parse::<chrono_tz::Tz>().is_err())
        {
            return Err(StoreError::Invalid(
                "request time zone must be a recognized IANA zone".into(),
            ));
        }
        text(&input.text, "request", 32_000)?;
        text(&input.source, "request source", 100)?;
        if let Some(settings) = &input.settings {
            validate_settings(settings)?;
        }
        if input.policy == RequestPolicy::Scheduled {
            let stamp = input.scheduled_at.as_deref().ok_or_else(|| {
                StoreError::Invalid(
                    "scheduled request needs an RFC3339 earliest-start timestamp".into(),
                )
            })?;
            DateTime::parse_from_rfc3339(stamp).map_err(|_| {
                StoreError::Invalid("scheduled time must include its UTC offset".into())
            })?;
        } else if input.scheduled_at.is_some() {
            return Err(StoreError::Invalid(
                "only scheduled requests may have a scheduled time".into(),
            ));
        }
        self.command(command_id, "submit_request", &input, |tx| {
            let project: Project = read(tx, "projects", &input.project_id)?;
            let requested_settings = input
                .settings
                .clone()
                .unwrap_or_else(|| project.settings.clone());
            let request = Request {
                id: id(),
                project_id: project.id.clone(),
                text: input.text.trim().into(),
                policy: input.policy,
                scheduled_at: input.scheduled_at.clone(),
                time_zone: input.time_zone.clone(),
                source: input.source.clone(),
                settings: requested_settings.within(&project.settings),
                requested_settings,
                context_revision: project.context_revision,
                context: context_snapshot(tx, &project)?,
                status: RequestStatus::Pending,
                reason: None,
                planning_thread_id: None,
                planning_turn_id: None,
                planning_summary: None,
                created_at: now(),
            };
            tx.execute(
                "INSERT INTO requests(id,project_id,json) VALUES(?1,?2,?3)",
                params![request.id, project.id, serde_json::to_string(&request)?],
            )?;
            event(
                tx,
                &project.id,
                None,
                "request_received",
                json!({"requestId":request.id,"text":request.text,"policy":request.policy}),
            )?;
            Ok(request)
        })
    }

    pub fn mark_request_planning(
        &self,
        command_id: &str,
        request_id: &str,
        thread_id: &str,
        turn_id: Option<&str>,
    ) -> Result<Request> {
        text(thread_id, "planning thread identity", 300)?;
        if let Some(turn_id) = turn_id {
            text(turn_id, "planning turn identity", 300)?;
        }
        self.command(
            command_id,
            "mark_request_planning",
            &(request_id, thread_id, turn_id),
            |tx| {
                let mut request: Request = read(tx, "requests", request_id)?;
                no_unresolved_lead_wake(tx, &request.project_id)?;
                if !matches!(
                    request.status,
                    RequestStatus::Pending | RequestStatus::Waiting | RequestStatus::Planning
                ) {
                    return Err(StoreError::Conflict(
                        "request already planned or cancelled".into(),
                    ));
                }
                let project: Project = read(tx, "projects", &request.project_id)?;
                if project.lead_thread_id.as_deref() != Some(thread_id) {
                    return Err(StoreError::Conflict(
                        "planning must use the project's recorded lead thread".into(),
                    ));
                }
                immutable_field(
                    &mut request.planning_thread_id,
                    &Some(thread_id.into()),
                    "planning thread",
                )?;
                immutable_field(
                    &mut request.planning_turn_id,
                    &turn_id.map(Into::into),
                    "planning turn",
                )?;
                request.status = RequestStatus::Planning;
                replace(tx, "requests", request_id, &request)?;
                event(
                    tx,
                    &request.project_id,
                    None,
                    "lead_planning",
                    json!({"requestId":request.id,"threadId":thread_id,"turnId":turn_id}),
                )?;
                Ok(request)
            },
        )
    }

    pub fn set_planning_summary(
        &self,
        command_id: &str,
        request_id: &str,
        summary: &str,
    ) -> Result<Request> {
        text(summary, "planning summary", 16_000)?;
        self.command(
            command_id,
            "set_planning_summary",
            &(request_id, summary),
            |tx| {
                let mut request: Request = read(tx, "requests", request_id)?;
                if request.status == RequestStatus::Cancelled {
                    return Err(StoreError::Conflict("request was cancelled".into()));
                }
                immutable_field(
                    &mut request.planning_summary,
                    &Some(summary.into()),
                    "planning summary",
                )?;
                replace(tx, "requests", request_id, &request)?;
                event(
                    tx,
                    &request.project_id,
                    None,
                    "lead_summary",
                    json!({"requestId":request_id,"summary":summary}),
                )?;
                Ok(request)
            },
        )
    }

    pub fn set_request_waiting(
        &self,
        command_id: &str,
        request_id: &str,
        reason: &str,
    ) -> Result<Request> {
        text(reason, "waiting reason", 4000)?;
        self.command(
            command_id,
            "set_request_waiting",
            &(request_id, reason),
            |tx| {
                let mut request: Request = read(tx, "requests", request_id)?;
                if !matches!(
                    request.status,
                    RequestStatus::Pending | RequestStatus::Planning | RequestStatus::Waiting
                ) {
                    return Err(StoreError::Conflict(
                        "request is no longer pending intent".into(),
                    ));
                }
                request.status = RequestStatus::Waiting;
                request.reason = Some(reason.into());
                replace(tx, "requests", request_id, &request)?;
                event(
                    tx,
                    &request.project_id,
                    None,
                    "lead_waiting",
                    json!({"requestId":request_id,"reason":reason}),
                )?;
                Ok(request)
            },
        )
    }

    pub fn set_request_recovery(
        &self,
        command_id: &str,
        request_id: &str,
        reason: &str,
    ) -> Result<Request> {
        text(reason, "lead recovery reason", 8000)?;
        self.command(command_id,"set_request_recovery",&(request_id,reason),|tx|{
            let mut request:Request=read(tx,"requests",request_id)?;
            if request.planning_thread_id.is_none() || !matches!(request.status,RequestStatus::Planning|RequestStatus::RecoveryRequired) {return Err(StoreError::Conflict("only an owned planning request can require lead reconciliation".into()));}
            request.status=RequestStatus::RecoveryRequired;request.reason=Some(reason.into());
            replace(tx,"requests",request_id,&request)?;
            event(tx,&request.project_id,None,"lead_recovery_required",json!({"requestId":request_id,"threadId":request.planning_thread_id,"turnId":request.planning_turn_id,"reason":reason}))?;
            Ok(request)
        })
    }

    pub fn cancel_request(&self, command_id: &str, request_id: &str) -> Result<Request> {
        self.command(command_id, "cancel_request", &request_id, |tx| {
            let mut request: Request = read(tx, "requests", request_id)?;
            if matches!(
                request.status,
                RequestStatus::Planning | RequestStatus::RecoveryRequired
            ) {
                return Err(StoreError::Conflict(
                    "planning must stop or be reconciled before this intent can be released".into(),
                ));
            }
            request.status = RequestStatus::Cancelled;
            request.reason = Some(
                "Pending intent cancelled; any existing attempt retains its own control state."
                    .into(),
            );
            replace(tx, "requests", request_id, &request)?;
            event(
                tx,
                &request.project_id,
                None,
                "request_cancelled",
                json!({"requestId":request_id}),
            )?;
            Ok(request)
        })
    }

    pub fn plan_task(&self, command_id: &str, input: NewTask) -> Result<Task> {
        text(&input.title, "task title", 200)?;
        text(&input.description, "task description", 32_000)?;
        if input.acceptance.is_empty() || input.acceptance.len() > 20 {
            return Err(StoreError::Invalid(
                "task needs one to twenty acceptance criteria".into(),
            ));
        }
        for criterion in &input.acceptance {
            text(criterion, "acceptance criterion", 4000)?;
        }
        if input.checks.is_empty() || input.checks.len() > 10 {
            return Err(StoreError::Invalid(
                "task needs one to ten supervisor-checkable checks".into(),
            ));
        }
        for check in &input.checks {
            validate_check(check)?;
        }
        if input.depends_on.len() > 20
            || input.depends_on.iter().collect::<HashSet<_>>().len() != input.depends_on.len()
        {
            return Err(StoreError::Invalid(
                "task prerequisites must be distinct and bounded".into(),
            ));
        }
        self.command(command_id, "plan_task", &input, |tx| {
            let request: Request = read(tx, "requests", &input.request_id)?;
            no_unresolved_lead_wake(tx, &request.project_id)?;
            if request.project_id != input.project_id {
                return Err(StoreError::Invalid(
                    "request belongs to a different project".into(),
                ));
            }
            if !matches!(
                request.status,
                RequestStatus::Pending | RequestStatus::Planning | RequestStatus::Waiting
            ) {
                return Err(StoreError::Conflict(
                    "request already planned or cancelled".into(),
                ));
            }
            for dependency in &input.depends_on {
                let task: Task = read(tx, "tasks", dependency)?;
                if task.project_id != input.project_id {
                    return Err(StoreError::Invalid(
                        "prerequisite belongs to a different project".into(),
                    ));
                }
            }
            let task = Task {
                id: id(),
                project_id: input.project_id.clone(),
                request_id: input.request_id.clone(),
                title: input.title.trim().into(),
                description: input.description.clone(),
                acceptance: input.acceptance.clone(),
                checks: input.checks.clone(),
                depends_on: input.depends_on.clone(),
                status: TaskStatus::Ready,
                reason: None,
                created_at: now(),
            };
            tx.execute(
                "INSERT INTO tasks(id,project_id,request_id,json) VALUES(?1,?2,?3,?4)",
                params![
                    task.id,
                    task.project_id,
                    task.request_id,
                    serde_json::to_string(&task)?
                ],
            )?;
            let mut request = request;
            request.status = RequestStatus::Planned;
            request.reason = None;
            replace(tx, "requests", &request.id, &request)?;
            event(
                tx,
                &task.project_id,
                None,
                "task_planned",
                json!({"taskId":task.id,"requestId":task.request_id,"title":task.title}),
            )?;
            Ok(task)
        })
    }

    pub fn claim_task(&self, command_id: &str, task_id: &str) -> Result<RunAttempt> {
        self.command(command_id, "claim_task", &task_id, |tx| {
            let mut task: Task = read(tx, "tasks", task_id)?;
            let project: Project = read(tx, "projects", &task.project_id)?;
            no_unresolved_lead_wake(tx, &project.id)?;
            let request: Request = read(tx, "requests", &task.request_id)?;
            if project.paused { return Err(StoreError::Conflict("project dispatch is paused".into())); }
            if request.status == RequestStatus::Cancelled { return Err(StoreError::Conflict("request was cancelled".into())); }
            if let Some(stamp) = &request.scheduled_at {
                let due = DateTime::parse_from_rfc3339(stamp).map_err(|_| StoreError::Invalid("stored scheduled time is invalid".into()))?;
                if due.with_timezone(&Utc) > Utc::now() { return Err(StoreError::Conflict("request earliest start has not arrived".into())); }
            }
            if !matches!(task.status, TaskStatus::Ready | TaskStatus::Blocked) { return Err(StoreError::Conflict("task is not eligible for a new attempt".into())); }
            for dependency in &task.depends_on {
                let prerequisite: Task = read(tx, "tasks", dependency)?;
                if prerequisite.status != TaskStatus::Done { return Err(StoreError::Conflict("prerequisite has not reached Done".into())); }
            }
            let attempts: u32 = tx.query_row("SELECT count(*) FROM runs WHERE task_id=?1", [task_id], |r| r.get(0))?;
            let effective_settings = request.settings.within(&project.settings);
            if effective_settings.model.trim().is_empty() { return Err(StoreError::Conflict("select an observed supported model before dispatch".into())); }
            if attempts >= effective_settings.max_attempts { return Err(StoreError::Conflict("attempt limit exhausted".into())); }
            let active: u32 = tx.query_row("SELECT count(*) FROM runs WHERE project_id=?1 AND active=1", [&project.id], |r| r.get(0))?;
            if active > 0 { return Err(StoreError::Conflict("one worker is already active or awaiting recovery".into())); }
            let pending_reviews: u32 = list::<LocalReview>(tx, "reviews")?.iter().filter(|review| review.project_id == project.id && review.decision.is_none()).count() as u32;
            if pending_reviews >= effective_settings.review_queue_cap { return Err(StoreError::Conflict("review queue limit reached".into())); }
            let run = RunAttempt { id: id(), project_id: project.id.clone(), task_id: task.id.clone(), request_id: request.id,
                attempt: attempts + 1, status: RunStatus::Claimed, requested_settings: request.requested_settings,
                effective_settings, authorization:None, context_revision: project.context_revision, context: context_snapshot(tx,&project)?, thread_id: None, turn_id: None, worktree_path: None,
                base_sha: None, head_sha: None, pid: None, process_started_at: None, executor_id: None, checks: vec![], reason: None, summary: None, started_at: now(), finished_at: None };
            tx.execute("INSERT INTO runs(id,project_id,task_id,attempt,active,json) VALUES(?1,?2,?3,?4,1,?5)", params![run.id,run.project_id,run.task_id,run.attempt,serde_json::to_string(&run)?])?;
            task.status = TaskStatus::Running;
            task.reason = None;
            replace(tx, "tasks", task_id, &task)?;
            event(tx, &project.id, Some(&run.id), "run_claimed", json!({"taskId":task.id,"attempt":run.attempt,"contextRevision":run.context_revision,"effectiveSettings":run.effective_settings}))?;
            Ok(run)
        })
    }

    pub fn authorize_run(
        &self,
        command_id: &str,
        run_id: &str,
        settings: ProjectSettings,
        limitations: Vec<String>,
    ) -> Result<RunAttempt> {
        validate_settings(&settings)?;
        self.command(command_id,"authorize_run",&(run_id,&settings,&limitations),|tx|{
            let mut run:RunAttempt=read(tx,"runs",run_id)?;
            if run.authorization.is_some() || run.thread_id.is_some() || !matches!(run.status,RunStatus::Running|RunStatus::NeedsApproval) {
                return Err(StoreError::Conflict("execution authorization must be captured once before provider dispatch".into()));
            }
            let project:Project=read(tx,"projects",&run.project_id)?;
            let ceiling=run.effective_settings.within(&project.settings);
            if settings.model!=ceiling.model || settings.effort!=ceiling.effort || settings.access!=ceiling.access
                || settings.max_turn_seconds>ceiling.max_turn_seconds || settings.max_output_tokens>ceiling.max_output_tokens
                || settings.max_attempts>ceiling.max_attempts || settings.review_queue_cap>ceiling.review_queue_cap {
                return Err(StoreError::Conflict("execution settings exceed the captured/current ceiling".into()));
            }
            let approvals:Vec<Approval>=list(tx,"approvals")?;
            let mut ids=Vec::new();
            for (scope,wanted,allowed) in [("commands",settings.permissions.commands,ceiling.permissions.commands),("network",settings.permissions.network,ceiling.permissions.network),("integrations",settings.permissions.integrations,ceiling.permissions.integrations)] {
                if wanted==Permission::Ask {return Err(StoreError::Invalid("resolve Ask to an authorized or disabled capability before dispatch".into()));}
                if wanted==Permission::Allow && allowed==Permission::Deny {return Err(StoreError::Conflict("execution cannot broaden a denied capability".into()));}
                if wanted==Permission::Allow && allowed==Permission::Ask {
                    let approval=approvals.iter().find(|a|a.run_id==run.id&&a.scope==scope&&a.decision==Some(true))
                        .ok_or_else(||StoreError::Conflict(format!("{scope} needs a durable approval for this attempt")))?;
                    ids.push(approval.id.clone());
                }
            }
            if approvals.iter().any(|a|a.run_id==run.id&&a.decision.is_none()) {return Err(StoreError::Conflict("pending approvals must be resolved before dispatch".into()));}
            run.authorization=Some(ExecutionAuthorization{settings:settings.clone(),approval_ids:ids,limitations:limitations.clone(),authorized_at:now()});
            run.status=RunStatus::Running;
            replace(tx,"runs",run_id,&run)?;
            event(tx,&run.project_id,Some(run_id),"execution_authorized",json!({"settings":settings,"limitations":limitations,"approvalIds":run.authorization.as_ref().unwrap().approval_ids}))?;
            Ok(run)
        })
    }

    pub fn update_run(&self, command_id: &str, input: RunUpdate) -> Result<RunAttempt> {
        if let Some(path) = &input.worktree_path {
            if !Path::new(path).is_absolute() || !Path::new(path).is_dir() {
                return Err(StoreError::Invalid(
                    "worktree must be an existing absolute directory".into(),
                ));
            }
        }
        for sha in [&input.base_sha, &input.head_sha].into_iter().flatten() {
            validate_revision(sha)?;
        }
        for value in [&input.thread_id, &input.turn_id].into_iter().flatten() {
            text(value, "provider identity", 300)?;
        }
        if let Some(summary) = &input.summary {
            text(summary, "run summary", 32_000)?;
        }
        if let Some(reason) = &input.reason {
            text(reason, "run reason", 8000)?;
        }
        self.command(command_id, "update_run", &input, |tx| {
            let mut run: RunAttempt = read(tx, "runs", &input.run_id)?;
            if !run.status.is_active() { return Err(StoreError::Conflict("terminal attempt evidence cannot be rewritten".into())); }
            if run.executor_id.is_some() && input.executor_id != run.executor_id { return Err(StoreError::Conflict("executor identity does not own this attempt".into())); }
            immutable_field(&mut run.executor_id, &input.executor_id, "executor identity")?;
            immutable_field(&mut run.thread_id, &input.thread_id, "thread identity")?;
            immutable_field(&mut run.turn_id, &input.turn_id, "turn identity")?;
            immutable_field(&mut run.worktree_path, &input.worktree_path, "worktree identity")?;
            immutable_field(&mut run.base_sha, &input.base_sha, "base revision")?;
            if let Some(value) = &input.head_sha { run.head_sha = Some(value.clone()); }
            if let Some(value) = input.pid {
                if run.pid.is_some_and(|prior| prior != value) { return Err(StoreError::Conflict("process identity is immutable".into())); }
                run.pid = Some(value);
            }
            if let Some(value) = input.process_started_at {
                if run.process_started_at.is_some_and(|prior| prior != value) { return Err(StoreError::Conflict("process start identity is immutable".into())); }
                run.process_started_at = Some(value);
            }
            if let Some(value) = &input.reason { run.reason = Some(value.clone()); }
            if let Some(value) = &input.summary { run.summary = Some(value.clone()); }
            if let Some(status) = input.status {
                if !allowed_transition(run.status, status) { return Err(StoreError::Conflict(format!("invalid attempt transition {:?} → {:?}", run.status, status))); }
                if run.status == RunStatus::NeedsApproval && status == RunStatus::Running {
                    let pending = list::<Approval>(tx, "approvals")?.iter().any(|approval| approval.run_id == run.id && approval.decision.is_none());
                    if pending { return Err(StoreError::Conflict("approval decision must be durable before recording a provider resume acknowledgment".into())); }
                }
                run.status = status;
            }
            if let Some(checks) = &input.checks {
                let revision = run.head_sha.as_deref().ok_or_else(|| StoreError::Invalid("record the tested head before check evidence".into()))?;
                if checks.iter().any(|check| check.revision != revision) { return Err(StoreError::Invalid("check evidence does not match the recorded head".into())); }
                run.checks = checks.clone();
            }
            if run.status == RunStatus::Succeeded {
                let task: Task = read(tx, "tasks", &run.task_id)?;
                let revision = run.head_sha.as_deref().ok_or_else(|| StoreError::Conflict("successful coding attempt needs its exact checked head".into()))?;
                if !passing_checks(&run.checks, &task.checks, revision) { return Err(StoreError::Conflict("successful attempt requires every configured supervisor check on its exact head".into())); }
            }
            if !run.status.is_active() { run.finished_at = Some(now()); }
            tx.execute("UPDATE runs SET active=?1,json=?2 WHERE id=?3", params![run.status.is_active(),serde_json::to_string(&run)?,run.id])?;
            if !run.status.is_active() && run.status != RunStatus::Succeeded {
                let mut task: Task = read(tx, "tasks", &run.task_id)?;
                task.status = TaskStatus::Blocked;
                task.reason = run.reason.clone().or_else(|| Some(format!("Attempt {:?}", run.status)));
                replace(tx, "tasks", &task.id, &task)?;
            }
            event(tx, &run.project_id, Some(&run.id), "run_updated", json!({"status":run.status,"threadId":run.thread_id,"turnId":run.turn_id,"headSha":run.head_sha,"reason":run.reason,"summary":run.summary}))?;
            Ok(run)
        })
    }

    /// Retains active ownership until the supervisor proves the old process/session stopped.
    pub fn mark_recovery_required(&self, command_id: &str) -> Result<Vec<RunAttempt>> {
        self.command(command_id, "mark_recovery_required", &(), |tx| {
            let mut runs: Vec<RunAttempt> = list(tx, "runs")?;
            runs.retain(|run| run.status.is_active());
            for run in &mut runs {
                run.status = RunStatus::RecoveryRequired;
                run.reason = Some("Daemon restarted; prior process/session outcome requires reconciliation before dispatch.".into());
                replace(tx, "runs", &run.id, run)?;
                event(tx, &run.project_id, Some(&run.id), "run_recovery_required", json!({"reason":run.reason}))?;
            }
            Ok(runs)
        })
    }

    pub fn append_event(
        &self,
        command_id: &str,
        project_id: &str,
        run_id: Option<&str>,
        kind: &str,
        payload: Value,
    ) -> Result<Event> {
        text(kind, "event kind", 100)?;
        if serde_json::to_vec(&payload)?.len() > 128_000 {
            return Err(StoreError::Invalid(
                "event payload exceeds 128KB; store large output as a referenced artifact".into(),
            ));
        }
        self.command(
            command_id,
            "append_event",
            &(project_id, run_id, kind, &payload),
            |tx| {
                let _: Project = read(tx, "projects", project_id)?;
                if let Some(run_id) = run_id {
                    let run: RunAttempt = read(tx, "runs", run_id)?;
                    if run.project_id != project_id {
                        return Err(StoreError::Invalid(
                            "event run belongs to another project".into(),
                        ));
                    }
                }
                event(tx, project_id, run_id, kind, payload.clone())
            },
        )
    }

    pub fn add_review(&self, command_id: &str, input: NewReview) -> Result<LocalReview> {
        validate_revision(&input.revision)?;
        if input.diff.len() > 2_000_000 {
            return Err(StoreError::Invalid("review diff exceeds 2MB".into()));
        }
        self.command(command_id, "add_review", &input, |tx| {
            let run: RunAttempt = read(tx, "runs", &input.run_id)?;
            let mut task: Task = read(tx, "tasks", &run.task_id)?;
            if run.status != RunStatus::Succeeded
                || run.head_sha.as_deref() != Some(&input.revision)
            {
                return Err(StoreError::Conflict(
                    "review requires a succeeded attempt and its exact recorded head revision"
                        .into(),
                ));
            }
            if !passing_checks(&input.checks, &task.checks, &input.revision)
                || serde_json::to_string(&input.checks)? != serde_json::to_string(&run.checks)?
            {
                return Err(StoreError::Conflict(
                    "review needs every configured supervisor check passing on this exact revision"
                        .into(),
                ));
            }
            if input
                .artifacts
                .iter()
                .any(|artifact| artifact.revision != input.revision)
            {
                return Err(StoreError::Invalid(
                    "review artifact belongs to another revision".into(),
                ));
            }
            let latest: u32 = tx.query_row(
                "SELECT max(attempt) FROM runs WHERE task_id=?1",
                [&run.task_id],
                |row| row.get(0),
            )?;
            if latest != run.attempt {
                return Err(StoreError::Conflict("attempt was superseded".into()));
            }
            let review = LocalReview {
                id: id(),
                project_id: run.project_id.clone(),
                run_id: run.id.clone(),
                task_id: task.id.clone(),
                attempt: run.attempt,
                revision: input.revision.clone(),
                diff: input.diff.clone(),
                checks: input.checks.clone(),
                artifacts: input.artifacts.clone(),
                limitations: input.limitations.clone(),
                decision: None,
                feedback: None,
                decided_at: None,
                created_at: now(),
            };
            tx.execute(
                "INSERT INTO reviews(id,project_id,run_id,json) VALUES(?1,?2,?3,?4)",
                params![
                    review.id,
                    review.project_id,
                    review.run_id,
                    serde_json::to_string(&review)?
                ],
            )?;
            task.status = TaskStatus::Review;
            replace(tx, "tasks", &task.id, &task)?;
            event(
                tx,
                &review.project_id,
                Some(&run.id),
                "review_ready",
                json!({"reviewId":review.id,"revision":review.revision}),
            )?;
            Ok(review)
        })
    }

    pub fn finish_verified(
        &self,
        command_id: &str,
        executor_id: &str,
        input: NewReview,
    ) -> Result<LocalReview> {
        validate_revision(&input.revision)?;
        if input.diff.is_empty()
            || input.diff.len() > 2_000_000
            || input.artifacts.iter().any(|a| a.revision != input.revision)
        {
            return Err(StoreError::Invalid(
                "verified review needs a bounded actual diff and matching artifacts".into(),
            ));
        }
        self.command(command_id,"finish_verified",&(executor_id,&input),|tx|{
            let mut run:RunAttempt=read(tx,"runs",&input.run_id)?;
            let mut task:Task=read(tx,"tasks",&run.task_id)?;
            if run.status!=RunStatus::Checking || run.executor_id.as_deref()!=Some(executor_id)
                || run.head_sha.as_deref()!=Some(&input.revision)
                || !passing_checks(&input.checks,&task.checks,&input.revision)
                || serde_json::to_string(&run.checks)?!=serde_json::to_string(&input.checks)? {
                return Err(StoreError::Conflict("completion requires the owned checking attempt and its recorded exact-revision evidence".into()));
            }
            let review=LocalReview{id:id(),project_id:run.project_id.clone(),run_id:run.id.clone(),task_id:run.task_id.clone(),attempt:run.attempt,
                revision:input.revision.clone(),diff:input.diff.clone(),checks:input.checks.clone(),artifacts:input.artifacts.clone(),limitations:input.limitations.clone(),decision:None,feedback:None,decided_at:None,created_at:now()};
            run.status=RunStatus::Succeeded;run.finished_at=Some(now());run.reason=Some("Supervisor checks passed; awaiting local review".into());
            tx.execute("UPDATE runs SET active=0,json=?1 WHERE id=?2",params![serde_json::to_string(&run)?,run.id])?;
            tx.execute("INSERT INTO reviews(id,project_id,run_id,json) VALUES(?1,?2,?3,?4)",params![review.id,review.project_id,review.run_id,serde_json::to_string(&review)?])?;
            task.status=TaskStatus::Review;replace(tx,"tasks",&task.id,&task)?;
            event(tx,&run.project_id,Some(&run.id),"run_updated",json!({"status":"succeeded","revision":review.revision}))?;
            event(tx,&run.project_id,Some(&run.id),"review_ready",json!({"reviewId":review.id,"revision":review.revision}))?;
            Ok(review)
        })
    }

    /// The caller must independently re-read Git HEAD immediately before this source-bound decision.
    pub fn decide_review(
        &self,
        command_id: &str,
        review_id: &str,
        observed_head: &str,
        decision: &str,
        feedback: &str,
    ) -> Result<LocalReview> {
        if !matches!(decision, "accepted_local" | "changes_requested") {
            return Err(StoreError::Invalid(
                "only local acceptance or request-changes is supported".into(),
            ));
        }
        if decision == "changes_requested" {
            text(feedback, "review feedback", 16_000)?;
        }
        validate_revision(observed_head)?;
        self.command(command_id, "decide_review", &(review_id,decision,feedback), |tx| {
            let mut review: LocalReview = read(tx, "reviews", review_id)?;
            let run: RunAttempt = read(tx, "runs", &review.run_id)?;
            let latest: u32 = tx.query_row("SELECT max(attempt) FROM runs WHERE task_id=?1", [&run.task_id], |row| row.get(0))?;
            if review.decision.is_some() { return Err(StoreError::Conflict("review already has a recorded local decision".into())); }
            if review.revision != observed_head || run.head_sha.as_deref() != Some(observed_head) || run.status != RunStatus::Succeeded || latest != run.attempt {
                return Err(StoreError::Conflict("review source changed or was superseded; refresh evidence".into()));
            }
            review.decision = Some(decision.into());
            review.feedback = if feedback.trim().is_empty() { None } else { Some(feedback.trim().into()) };
            review.decided_at = Some(now());
            replace(tx, "reviews", review_id, &review)?;
            // Coding remains in Review: local acceptance does not merge or deliver changes.
            event(tx, &review.project_id, Some(&run.id), "review_decided_local", json!({"reviewId":review.id,"revision":review.revision,"decision":decision,"feedback":review.feedback}))?;
            Ok(review)
        })
    }

    pub fn replay_review_decision(
        &self,
        command_id: &str,
        review_id: &str,
        decision: &str,
        feedback: &str,
    ) -> Result<Option<LocalReview>> {
        let row: Option<(String, String, String)> = self
            .connect()?
            .query_row(
                "SELECT kind,payload,result FROM commands WHERE id=?1",
                [command_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((kind, payload, result)) = row else {
            return Ok(None);
        };
        if kind != "decide_review"
            || payload != serde_json::to_string(&(review_id, decision, feedback))?
        {
            return Err(StoreError::Conflict(
                "command identity was already used for different input".into(),
            ));
        }
        Ok(Some(serde_json::from_str(&result)?))
    }

    pub fn interrupt_run(&self, command_id: &str, run_id: &str) -> Result<RunAttempt> {
        self.command(command_id,"interrupt_run",&run_id,|tx|{
            let mut run:RunAttempt=read(tx,"runs",run_id)?;
            if !run.status.is_active() {return Err(StoreError::Conflict("attempt already stopped".into()));}
            if run.status==RunStatus::RecoveryRequired {return Err(StoreError::Conflict("This attempt requires process/session reconciliation. Its stop remains unresolved; a successor is fenced".into()));}
            run.status=RunStatus::Interrupting;
            run.reason=Some("Stop requested; awaiting observed provider or process stop".into());
            replace(tx,"runs",run_id,&run)?;
            event(tx,&run.project_id,Some(run_id),"run_updated",json!({"status":"interrupting","reason":run.reason}))?;
            Ok(run)
        })
    }

    pub fn add_approval(
        &self,
        command_id: &str,
        run_id: &str,
        provider_request_id: &str,
        scope: &str,
        detail: &str,
    ) -> Result<Approval> {
        text(provider_request_id, "provider approval identity", 300)?;
        text(detail, "approval detail", 8000)?;
        self.command(
            command_id,
            "add_approval",
            &(run_id, provider_request_id, scope, detail),
            |tx| {
                let mut run: RunAttempt = read(tx, "runs", run_id)?;
                if !matches!(run.status, RunStatus::Running | RunStatus::NeedsApproval) {
                    return Err(StoreError::Conflict(
                        "attempt cannot wait for approval in its current state".into(),
                    ));
                }
                let project: Project = read(tx, "projects", &run.project_id)?;
                let permission =
                    scoped_permission(&run.effective_settings.within(&project.settings), scope)?;
                if permission == Permission::Deny {
                    return Err(StoreError::Conflict(
                        "project or attempt ceiling denies this approval scope".into(),
                    ));
                }
                let approval = Approval {
                    id: id(),
                    project_id: run.project_id.clone(),
                    run_id: run.id.clone(),
                    provider_request_id: provider_request_id.into(),
                    scope: scope.into(),
                    detail: detail.into(),
                    decision: None,
                    created_at: now(),
                    resolved_at: None,
                };
                tx.execute(
                    "INSERT INTO approvals(id,run_id,provider_request_id,json) VALUES(?1,?2,?3,?4)",
                    params![
                        approval.id,
                        run_id,
                        provider_request_id,
                        serde_json::to_string(&approval)?
                    ],
                )?;
                run.status = RunStatus::NeedsApproval;
                replace(tx, "runs", run_id, &run)?;
                event(
                    tx,
                    &run.project_id,
                    Some(run_id),
                    "approval_requested",
                    json!({"approvalId":approval.id,"scope":scope,"detail":detail}),
                )?;
                Ok(approval)
            },
        )
    }

    pub fn resolve_approval(
        &self,
        command_id: &str,
        approval_id: &str,
        allow: bool,
    ) -> Result<Approval> {
        self.command(
            command_id,
            "resolve_approval",
            &(approval_id, allow),
            |tx| {
                let mut approval: Approval = read(tx, "approvals", approval_id)?;
                if approval.decision.is_some() {
                    return Err(StoreError::Conflict("approval already resolved".into()));
                }
                let run: RunAttempt = read(tx, "runs", &approval.run_id)?;
                if run.status != RunStatus::NeedsApproval {
                    return Err(StoreError::Conflict(
                        "attempt is no longer waiting for this approval".into(),
                    ));
                }
                let project: Project = read(tx, "projects", &run.project_id)?;
                if allow
                    && scoped_permission(
                        &run.effective_settings.within(&project.settings),
                        &approval.scope,
                    )? == Permission::Deny
                {
                    return Err(StoreError::Conflict(
                        "approval cannot broaden a denied project or attempt ceiling".into(),
                    ));
                }
                approval.decision = Some(allow);
                approval.resolved_at = Some(now());
                replace(tx, "approvals", approval_id, &approval)?;
                event(
                    tx,
                    &approval.project_id,
                    Some(&run.id),
                    "approval_decided",
                    json!({"approvalId":approval.id,"allowed":allow}),
                )?;
                Ok(approval)
            },
        )
    }

    pub fn events(&self, after: u64, limit: u32) -> Result<Vec<Event>> {
        let connection = self.connect()?;
        let mut statement = connection.prepare("SELECT sequence,project_id,run_id,kind,payload,created_at FROM events WHERE sequence>?1 ORDER BY sequence LIMIT ?2")?;
        let rows = statement.query_map(
            params![sql_integer(after, "event cursor")?, limit.clamp(1, 500)],
            decode_event,
        )?;
        rows.map(|row| Ok(row?)).collect()
    }

    /// Uses durable history rather than the snapshot's bounded event window.
    pub fn applied_lead_context_revision(&self, project_id: &str) -> Result<u64> {
        applied_context_revision(&self.connect()?, project_id)
    }

    /// Reserve one justified wake and its context before starting any provider process.
    pub fn prepare_lead_wake(
        &self,
        command_id: &str,
        project_id: &str,
    ) -> Result<Option<LeadWake>> {
        self.command(command_id, "prepare_lead_wake", &project_id, |tx| {
            let project: Project = read(tx, "projects", project_id)?;
            let mut wakes = list::<LeadWake>(tx, "lead_wakes")?;
            if project.paused
                || wakes.iter().any(|w| w.project_id == project_id && w.status.fences_execution())
                || list::<RunAttempt>(tx, "runs")?.iter().any(|r| r.project_id == project_id && r.status.is_active())
                || list::<Request>(tx, "requests")?.iter().any(|r| r.project_id == project_id && matches!(r.status, RequestStatus::Planning | RequestStatus::RecoveryRequired))
            {
                return Ok(None);
            }
            let covered = applied_context_revision(tx, project_id)?;
            for wake in wakes.iter_mut().filter(|w| w.project_id == project_id && w.status == LeadWakeStatus::Waiting && w.source_revision.is_some_and(|revision| revision <= covered)) {
                wake.status = LeadWakeStatus::Covered;
                wake.reason = Some("A request planning turn already applied this steering revision; no additional inference turn was submitted.".into());
                wake.finished_at = Some(now());
                replace(tx, "lead_wakes", &wake.id, wake)?;
                wake_event(tx, wake, "lead_wake_covered")?;
            }
            // Waiting means no provider turn was attempted. Only configuration can make it eligible.
            if !project.settings.model.trim().is_empty() {
                if let Some(mut wake) = wakes.iter().find(|w| w.project_id == project_id && w.status == LeadWakeStatus::Waiting && !w.source_revision.is_some_and(|revision| revision <= covered)).cloned() {
                    wake.effective_settings = Some(lead_settings(&project.settings));
                    wake.status = LeadWakeStatus::Prepared;
                    wake.reason = None;
                    replace(tx, "lead_wakes", &wake.id, &wake)?;
                    wake_event(tx, &wake, "lead_wake_prepared")?;
                    return Ok(Some(wake));
                }
            }
            let mut sources = Vec::new();
            for steering in list::<Steering>(tx, "steering")?.into_iter().filter(|s| s.project_id == project_id && s.revision > covered) {
                sources.push((format!("steering:{project_id}:{}", steering.revision), LeadWakeKind::Steering, steering.id, Some(steering.revision)));
            }
            for run in list::<RunAttempt>(tx, "runs")?.into_iter().filter(|r| r.project_id == project_id && !r.status.is_active()) {
                sources.push((format!("completed-run:{}", run.id), LeadWakeKind::CompletedRun, run.id, None));
            }
            let Some((key, kind, source_id, source_revision)) = sources.into_iter().find(|(key, _, _, _)| !wakes.iter().any(|w| &w.key == key)) else {return Ok(None);};
            let configured = !project.settings.model.trim().is_empty();
            let wake = LeadWake {
                id: id(), key, project_id: project.id.clone(), kind, source_id, source_revision,
                context_revision: project.context_revision, context: context_snapshot(tx, &project)?,
                requested_settings: project.settings.clone(),
                effective_settings: configured.then(|| lead_settings(&project.settings)),
                status: if configured {LeadWakeStatus::Prepared} else {LeadWakeStatus::Waiting},
                thread_id: None, turn_id: None, summary: None,
                reason: (!configured).then(|| "Select a supported model in Settings before the lead can respond. No provider turn has started.".into()),
                output_tokens: None, total_tokens: None, created_at: now(), started_at: None, finished_at: None,
            };
            tx.execute("INSERT INTO lead_wakes(id,project_id,wake_key,json) VALUES(?1,?2,?3,?4)", params![wake.id,project_id,wake.key,serde_json::to_string(&wake)?])?;
            wake_event(tx, &wake, if configured {"lead_wake_prepared"} else {"lead_wake_waiting"})?;
            Ok(Some(wake))
        })
    }

    /// Save the owned thread before inference, then attach the observed turn identity.
    pub fn start_lead_wake(
        &self,
        command_id: &str,
        wake_id: &str,
        thread_id: &str,
        turn_id: Option<&str>,
    ) -> Result<LeadWake> {
        text(thread_id, "lead wake thread", 300)?;
        if let Some(turn) = turn_id {
            text(turn, "lead wake turn", 300)?;
        }
        self.command(
            command_id,
            "start_lead_wake",
            &(wake_id, thread_id, turn_id),
            |tx| {
                let mut wake: LeadWake = read(tx, "lead_wakes", wake_id)?;
                if !matches!(
                    wake.status,
                    LeadWakeStatus::Prepared | LeadWakeStatus::Running
                ) {
                    return Err(StoreError::Conflict(
                        "wake is no longer prepared or running".into(),
                    ));
                }
                let project: Project = read(tx, "projects", &wake.project_id)?;
                if project.lead_thread_id.as_deref() != Some(thread_id) {
                    return Err(StoreError::Conflict(
                        "wake must use the project's recorded lead thread".into(),
                    ));
                }
                immutable_field(
                    &mut wake.thread_id,
                    &Some(thread_id.into()),
                    "lead wake thread",
                )?;
                immutable_field(
                    &mut wake.turn_id,
                    &turn_id.map(Into::into),
                    "lead wake turn",
                )?;
                wake.status = LeadWakeStatus::Running;
                if wake.started_at.is_none() {
                    wake.started_at = Some(now());
                }
                replace(tx, "lead_wakes", wake_id, &wake)?;
                wake_event(tx, &wake, "lead_wake_started")?;
                Ok(wake)
            },
        )
    }

    pub fn finish_lead_wake(
        &self,
        command_id: &str,
        wake_id: &str,
        status: LeadWakeStatus,
        summary: Option<&str>,
        reason: &str,
        output_tokens: Option<u64>,
        total_tokens: Option<u64>,
    ) -> Result<LeadWake> {
        text(reason, "lead wake result", 8000)?;
        if let Some(summary) = summary {
            text(summary, "lead wake summary", 16000)?;
        }
        if !matches!(
            status,
            LeadWakeStatus::Completed | LeadWakeStatus::Failed | LeadWakeStatus::RecoveryRequired
        ) {
            return Err(StoreError::Invalid(
                "invalid lead wake terminal state".into(),
            ));
        }
        self.command(
            command_id,
            "finish_lead_wake",
            &(
                wake_id,
                status,
                summary,
                reason,
                output_tokens,
                total_tokens,
            ),
            |tx| {
                let mut wake: LeadWake = read(tx, "lead_wakes", wake_id)?;
                if !matches!(
                    wake.status,
                    LeadWakeStatus::Prepared | LeadWakeStatus::Running
                ) {
                    return Err(StoreError::Conflict("wake already has an outcome".into()));
                }
                if status == LeadWakeStatus::Completed
                    && (wake.turn_id.is_none() || summary.is_none())
                {
                    return Err(StoreError::Conflict(
                        "a completed lead wake requires an observed turn and summary".into(),
                    ));
                }
                wake.status = status;
                wake.summary = summary.map(Into::into);
                wake.reason = Some(reason.into());
                wake.output_tokens = output_tokens;
                wake.total_tokens = total_tokens;
                wake.finished_at = (status != LeadWakeStatus::RecoveryRequired).then(now);
                replace(tx, "lead_wakes", wake_id, &wake)?;
                wake_event(
                    tx,
                    &wake,
                    match status {
                        LeadWakeStatus::Completed => "lead_wake_completed",
                        LeadWakeStatus::Failed => "lead_wake_failed",
                        _ => "lead_wake_recovery_required",
                    },
                )?;
                if status == LeadWakeStatus::Completed {
                    event(
                        tx,
                        &wake.project_id,
                        None,
                        "lead_context_applied",
                        json!({
                            "contextRevision":wake.context_revision,"wakeId":wake.id,
                            "threadId":wake.thread_id,"turnId":wake.turn_id,
                            "outputTokens":wake.output_tokens,"totalTokens":wake.total_tokens
                        }),
                    )?;
                }
                Ok(wake)
            },
        )
    }

    pub fn recover_lead_wakes(&self, command_id: &str) -> Result<Vec<LeadWake>> {
        self.command(command_id, "recover_lead_wakes", &(), |tx| {
            let mut recovered = Vec::new();
            for mut wake in list::<LeadWake>(tx, "lead_wakes")?.into_iter().filter(|w| matches!(w.status, LeadWakeStatus::Prepared | LeadWakeStatus::Running)) {
                wake.status = LeadWakeStatus::RecoveryRequired;
                wake.reason = Some("The daemon stopped while a lead wake was owned. Reconcile its saved thread/turn before further planning or dispatch; it will not be resubmitted automatically.".into());
                replace(tx, "lead_wakes", &wake.id, &wake)?;
                wake_event(tx, &wake, "lead_wake_recovery_required")?;
                recovered.push(wake);
            }
            Ok(recovered)
        })
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        let mut connection = self.connect()?;
        let tx = connection.transaction()?;
        let mut snapshot = Snapshot {
            projects: list(&tx, "projects")?,
            steering: list(&tx, "steering")?,
            requests: list(&tx, "requests")?,
            tasks: list(&tx, "tasks")?,
            runs: list(&tx, "runs")?,
            reviews: list(&tx, "reviews")?,
            approvals: list(&tx, "approvals")?,
            lead_wakes: list(&tx, "lead_wakes")?,
            ..Default::default()
        };
        snapshot.last_event_sequence =
            tx.query_row("SELECT coalesce(max(sequence),0) FROM events", [], |row| {
                nonnegative_sql(row.get::<_, i64>(0)?, 0)
            })?;
        let mut statement = tx.prepare("SELECT sequence,project_id,run_id,kind,payload,created_at FROM (SELECT * FROM events ORDER BY sequence DESC LIMIT 500) ORDER BY sequence")?;
        snapshot.events = statement
            .query_map([], decode_event)?
            .map(|row| Ok(row?))
            .collect::<Result<_>>()?;
        drop(statement);
        tx.commit()?;
        Ok(snapshot)
    }
}

fn no_unresolved_lead_wake(tx: &Transaction<'_>, project_id: &str) -> Result<()> {
    if list::<LeadWake>(tx, "lead_wakes")?
        .iter()
        .any(|w| w.project_id == project_id && w.status.fences_execution())
    {
        return Err(StoreError::Conflict(
            "an owned lead wake must finish or be reconciled before planning or dispatch".into(),
        ));
    }
    Ok(())
}

fn applied_context_revision(connection: &Connection, project_id: &str) -> Result<u64> {
    let mut statement = connection.prepare(
        "SELECT payload FROM events WHERE project_id=?1 AND kind='lead_context_applied'",
    )?;
    let payloads = statement.query_map([project_id], |row| row.get::<_, String>(0))?;
    let mut covered = 0;
    for payload in payloads {
        covered = covered.max(
            serde_json::from_str::<Value>(&payload?)?["contextRevision"]
                .as_u64()
                .unwrap_or(0),
        );
    }
    Ok(covered)
}

fn lead_settings(settings: &ProjectSettings) -> ProjectSettings {
    let mut settings = settings.clone();
    settings.access = Access::ReadOnly;
    settings.permissions = Permissions {
        commands: Permission::Deny,
        network: Permission::Deny,
        integrations: Permission::Deny,
    };
    settings
}

fn wake_event(tx: &Transaction<'_>, wake: &LeadWake, kind: &str) -> Result<()> {
    event(
        tx,
        &wake.project_id,
        (wake.kind == LeadWakeKind::CompletedRun).then_some(wake.source_id.as_str()),
        kind,
        json!({"wakeId":wake.id,"kind":wake.kind,"sourceId":wake.source_id,"contextRevision":wake.context_revision,"threadId":wake.thread_id,"turnId":wake.turn_id,"summary":wake.summary,"reason":wake.reason,"outputTokens":wake.output_tokens,"totalTokens":wake.total_tokens}),
    )?;
    Ok(())
}

fn text(value: &str, name: &str, max: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > max || value.contains('\0') {
        return Err(StoreError::Invalid(format!(
            "{name} must be nonempty, contain no NUL, and fit within {max} bytes"
        )));
    }
    Ok(())
}
fn validate_settings(settings: &ProjectSettings) -> Result<()> {
    if settings.model.len() > 200 || settings.model.contains('\0') {
        return Err(StoreError::Invalid(
            "model identity must be bounded and contain no NUL".into(),
        ));
    }
    text(&settings.effort, "reasoning effort", 50)?;
    if !(1..=3600).contains(&settings.max_turn_seconds)
        || !(1..=100_000).contains(&settings.max_output_tokens)
        || !(1..=10).contains(&settings.max_attempts)
        || !(1..=20).contains(&settings.review_queue_cap)
    {
        return Err(StoreError::Invalid(
            "execution and review limits must be positive and bounded".into(),
        ));
    }
    Ok(())
}
fn validate_repo(value: &str) -> Result<PathBuf> {
    text(value, "repository path", 4096)?;
    let path = Path::new(value);
    if !path.is_absolute() || value.chars().any(char::is_control) {
        return Err(StoreError::Invalid(
            "repository must be an absolute path without control characters".into(),
        ));
    }
    let path = path.canonicalize()?;
    if !path.is_dir() || !path.join(".git").exists() {
        return Err(StoreError::Invalid(
            "repository path must identify a Git working-tree root".into(),
        ));
    }
    let mut command = std::process::Command::new("git");
    command
        .args(["--no-optional-locks", "-c", "core.fsmonitor=false", "-C"])
        .arg(&path)
        .args(["rev-parse", "--show-toplevel"]);
    for variable in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
    ] {
        command.env_remove(variable);
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(StoreError::Invalid(
            "repository is not a readable Git working-tree root".into(),
        ));
    }
    let root = String::from_utf8(output.stdout)
        .map_err(|_| StoreError::Invalid("repository path must be UTF-8".into()))?;
    if Path::new(root.trim_end_matches(['\r', '\n'])).canonicalize()? != path {
        return Err(StoreError::Invalid(
            "select the repository root rather than a nested directory".into(),
        ));
    }
    Ok(path)
}
fn validate_check(check: &CheckSpec) -> Result<()> {
    text(&check.program, "check executable", 1000)?;
    if check.args.len() > 100
        || check
            .args
            .iter()
            .any(|arg| arg.contains('\0') || arg.len() > 8000)
    {
        return Err(StoreError::Invalid(
            "check arguments must be bounded and contain no NUL".into(),
        ));
    }
    if check.cwd.contains('\0')
        || Path::new(&check.cwd)
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(StoreError::Invalid("check working directory must remain relative to the worktree, without parent traversal".into()));
    }
    if !(1..=1800).contains(&check.timeout_seconds) {
        return Err(StoreError::Invalid(
            "check timeout must be between 1 and 1800 seconds".into(),
        ));
    }
    Ok(())
}
fn validate_revision(revision: &str) -> Result<()> {
    if !matches!(revision.len(), 40 | 64) || !revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(StoreError::Invalid(
            "revision must be a full Git object identity".into(),
        ));
    }
    Ok(())
}
fn passing_checks(evidence: &[CheckEvidence], specs: &[CheckSpec], revision: &str) -> bool {
    !specs.is_empty()
        && evidence.len() == specs.len()
        && evidence.iter().zip(specs).all(|(evidence, check)| {
            evidence.revision == revision
                && evidence.program == check.program
                && evidence.args == check.args
                && evidence.cwd == check.cwd
                && !evidence.timed_out
                && evidence.exit_code == Some(0)
        })
}
fn immutable_field(
    existing: &mut Option<String>,
    incoming: &Option<String>,
    name: &str,
) -> Result<()> {
    if let Some(value) = incoming {
        if existing.as_ref().is_some_and(|prior| prior != value) {
            return Err(StoreError::Conflict(format!(
                "{name} is immutable once recorded"
            )));
        }
        *existing = Some(value.clone());
    }
    Ok(())
}
fn scoped_permission(settings: &ProjectSettings, scope: &str) -> Result<Permission> {
    match scope {
        "commands" => Ok(settings.permissions.commands),
        "network" => Ok(settings.permissions.network),
        "integrations" => Ok(settings.permissions.integrations),
        "edits" => Ok(if settings.access == Access::Worktree {
            Permission::Ask
        } else {
            Permission::Deny
        }),
        _ => Err(StoreError::Invalid("unsupported approval scope".into())),
    }
}
fn allowed_transition(from: RunStatus, to: RunStatus) -> bool {
    if from == to {
        return true;
    }
    if !from.is_active() {
        return false;
    }
    if from == RunStatus::RecoveryRequired {
        return matches!(
            to,
            RunStatus::Interrupted
                | RunStatus::Failed
                | RunStatus::Cancelled
                | RunStatus::Succeeded
        );
    }
    if from == RunStatus::Interrupting {
        return matches!(
            to,
            RunStatus::Interrupted
                | RunStatus::Cancelled
                | RunStatus::Failed
                | RunStatus::Succeeded
                | RunStatus::RecoveryRequired
        );
    }
    match to {
        RunStatus::Claimed => false,
        RunStatus::Running => matches!(from, RunStatus::Claimed | RunStatus::NeedsApproval),
        RunStatus::Checking => from == RunStatus::Running,
        RunStatus::NeedsApproval => matches!(from, RunStatus::Running | RunStatus::Checking),
        RunStatus::Succeeded => from == RunStatus::Checking,
        _ => true,
    }
}
fn now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
fn sql_integer(value: u64, name: &str) -> Result<i64> {
    i64::try_from(value)
        .map_err(|_| StoreError::Invalid(format!("{name} exceeds SQLite's signed integer range")))
}
fn next_revision(revision: u64) -> Result<u64> {
    let next = revision
        .checked_add(1)
        .ok_or_else(|| StoreError::Invalid("context revision exhausted".into()))?;
    sql_integer(next, "context revision")?;
    Ok(next)
}
fn context_snapshot(tx: &Transaction<'_>, project: &Project) -> Result<ContextSnapshot> {
    let steering = list::<Steering>(tx, "steering")?
        .into_iter()
        .filter(|guidance| {
            guidance.project_id == project.id && guidance.revision <= project.context_revision
        })
        .collect();
    Ok(ContextSnapshot {
        revision: project.context_revision,
        objective: project.objective.clone(),
        repo_path: project.repo_path.clone(),
        steering,
    })
}
fn id() -> String {
    Uuid::new_v4().to_string()
}
fn read<T: DeserializeOwned>(connection: &Connection, table: &str, id: &str) -> Result<T> {
    let json: Option<String> = connection
        .query_row(
            &format!("SELECT json FROM {table} WHERE id=?1"),
            [id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(serde_json::from_str(
        &json.ok_or_else(|| StoreError::NotFound(table.into()))?,
    )?)
}
fn list<T: DeserializeOwned>(connection: &Connection, table: &str) -> Result<Vec<T>> {
    let mut statement = connection.prepare(&format!("SELECT json FROM {table} ORDER BY rowid"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
}
fn insert<T: Serialize>(tx: &Transaction<'_>, table: &str, id: &str, value: &T) -> Result<()> {
    tx.execute(
        &format!("INSERT INTO {table}(id,json) VALUES(?1,?2)"),
        params![id, serde_json::to_string(value)?],
    )?;
    Ok(())
}
fn replace<T: Serialize>(tx: &Transaction<'_>, table: &str, id: &str, value: &T) -> Result<()> {
    tx.execute(
        &format!("UPDATE {table} SET json=?1 WHERE id=?2"),
        params![serde_json::to_string(value)?, id],
    )?;
    Ok(())
}
fn event(
    tx: &Transaction<'_>,
    project_id: &str,
    run_id: Option<&str>,
    kind: &str,
    payload: Value,
) -> Result<Event> {
    let created_at = now();
    tx.execute(
        "INSERT INTO events(project_id,run_id,kind,payload,created_at) VALUES(?1,?2,?3,?4,?5)",
        params![
            project_id,
            run_id,
            kind,
            serde_json::to_string(&payload)?,
            created_at
        ],
    )?;
    Ok(Event {
        sequence: u64::try_from(tx.last_insert_rowid())
            .map_err(|_| StoreError::Invalid("event sequence must be nonnegative".into()))?,
        project_id: project_id.into(),
        run_id: run_id.map(Into::into),
        kind: kind.into(),
        payload,
        created_at,
    })
}
fn decode_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<Event> {
    let json: String = row.get(4)?;
    let payload = serde_json::from_str(&json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Event {
        sequence: nonnegative_sql(row.get::<_, i64>(0)?, 0)?,
        project_id: row.get(1)?,
        run_id: row.get(2)?,
        kind: row.get(3)?,
        payload,
        created_at: row.get(5)?,
    })
}
fn nonnegative_sql(value: i64, column: usize) -> rusqlite::Result<u64> {
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}

const SCHEMA: &str = "
CREATE TABLE projects(id TEXT PRIMARY KEY,json TEXT NOT NULL);
CREATE UNIQUE INDEX one_local_project ON projects((1));
CREATE TABLE steering(id TEXT PRIMARY KEY,project_id TEXT NOT NULL REFERENCES projects(id),revision INTEGER NOT NULL,json TEXT NOT NULL,UNIQUE(project_id,revision));
CREATE TABLE requests(id TEXT PRIMARY KEY,project_id TEXT NOT NULL REFERENCES projects(id),json TEXT NOT NULL);
CREATE TABLE tasks(id TEXT PRIMARY KEY,project_id TEXT NOT NULL REFERENCES projects(id),request_id TEXT NOT NULL REFERENCES requests(id),json TEXT NOT NULL);
CREATE TABLE runs(id TEXT PRIMARY KEY,project_id TEXT NOT NULL REFERENCES projects(id),task_id TEXT NOT NULL REFERENCES tasks(id),attempt INTEGER NOT NULL,active INTEGER NOT NULL CHECK(active IN (0,1)),json TEXT NOT NULL,UNIQUE(task_id,attempt));
CREATE UNIQUE INDEX one_active_attempt_per_task ON runs(task_id) WHERE active=1;
CREATE UNIQUE INDEX one_active_worker_per_project ON runs(project_id) WHERE active=1;
CREATE TABLE reviews(id TEXT PRIMARY KEY,project_id TEXT NOT NULL REFERENCES projects(id),run_id TEXT NOT NULL UNIQUE REFERENCES runs(id),json TEXT NOT NULL);
CREATE TABLE approvals(id TEXT PRIMARY KEY,run_id TEXT NOT NULL REFERENCES runs(id),provider_request_id TEXT NOT NULL,json TEXT NOT NULL,UNIQUE(run_id,provider_request_id));
CREATE TABLE events(sequence INTEGER PRIMARY KEY AUTOINCREMENT,project_id TEXT NOT NULL REFERENCES projects(id),run_id TEXT REFERENCES runs(id),kind TEXT NOT NULL,payload TEXT NOT NULL,created_at TEXT NOT NULL);
CREATE TABLE commands(id TEXT PRIMARY KEY,kind TEXT NOT NULL,payload TEXT NOT NULL,result TEXT NOT NULL);
";
