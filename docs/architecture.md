# Software Factory architecture

Status: proposed v1 architecture, 31 October 2026. This document describes the intended contract; it does not certify that every capability is implemented.

Software Factory keeps a project objective alive, turns justified work into small tasks, runs agents, and assembles evidence for human review. A Rust daemon owns execution and durable state. A terminal client and desktop client present the same factory, tasks, runs, steering, and review actions.

## Decisions

| Decision | Reason |
| --- | --- |
| One Rust crate with modules | A small application does not need an internal framework or distributed services. |
| SQLite through `rusqlite` | Transactions, durable queues, audit history, and recovery on one host without another service. |
| Axum daemon | Execution continues when a client closes; both clients share an API. |
| Ratatui terminal client | Keyboard-first monitoring and steering. |
| Tauri desktop shell | Rust desktop integration around a lightweight web interface. |
| Codex app-server first | Use native persistent threads, streamed events, steering, and interruption; use headless CLI adapters for other providers. |
| One Git worktree per coding task | Separate branches and files without provisioning a cloud runtime. |
| Human merge gate | Make the final code and production decision deliberate and easy to review. |

The daemon can run on an awake local machine or an always-on remote host. Closing the GUI should not stop it. A sleeping or powered-off laptop cannot continue local work; remote execution is a separate deployment requirement.

## Client experience

Both clients expose a workspace overview across projects. Each project owns its own Conversation, Progress and Reviews destinations; navigation and actions make the selected project explicit. Conversation is the primary surface, with one human-facing main agent that owns the goal, accepts steering, and explains current decisions. Worker agents report through it; they do not create competing chat inboxes.

Progress is a separate surface for inspecting work through graph, board, or radar views. These are alternate projections of the same tasks and runs, not independent workflows. Start with a quiet overview and reveal worker activity, dependencies, logs, provider details, and evidence when a person selects an item. The current Sundial radar shows only planning, building, checking and waiting work; changes, PR previews, diffs and merge actions belong exclusively to Reviews. Reviews gives the selected change most of its workspace. Keep operational metadata out of the main conversation until it helps a decision. Final layout and visual treatment remain subject to the user's refined visual selection.

Radar geometry must derive stage and stage-entry time from actual run/check/approval events, rather than generic task update times. Preserve stage angle and age radius during label/collision layout. Waiting marks may retain a static delegation arm without implying a running process or a live lease; heartbeat styling applies only to observed active runs. Missing and disconnected state must be explicit. See the [current design](design/soft-charcoal/README.md).

Model selection has explicit scope: a workspace default initializes new projects; a project default applies to new worker runs without a task override; a lead selector applies from the next conversation turn; and a task override applies to its next run. Existing runs retain their recorded provider, model and effort. Populate choices from observed capabilities rather than treating mockup examples as available models.

Runs are inspectable within the owning project’s Progress view. Task/run links and selected radar work resolve to the same run identifier. A detail view exposes the effective provider/model/effort, attempt history, tool activity, changes, checks and artifacts. Historical attempts retain their own evidence. Steer and interrupt target the selected active run; retry requires a terminal retryable outcome and no other active run for that task. Next-run model overrides never rewrite active-run metadata. Review and merge actions remain in Reviews.

## Modules and ownership

`model` holds serializable types shared by the clients and daemon. `store` owns SQLite migrations and transactional updates. `scheduler` claims tasks and recovers interrupted runs. `provider` starts, resumes, observes, and interrupts agents. `server` exposes snapshots, events, and commands. `tui` renders the shared state. The desktop frontend consumes that same contract through Tauri.

One source of truth is enough: ordinary SQL state plus an append-only events table. Update state and its event in the same transaction. Avoid implementing a second, independent event-sourcing system.

## Proposed shared types

Use strings for opaque IDs and RFC 3339 UTC timestamps at the API boundary. Persist paths as paths and expose them as strings. Serialize enum variants in `snake_case`.

| Type | Fields |
| --- | --- |
| `Factory` | `id`, `name`, `repo_path`, `objective`, `paused`, `context_revision`, `agency`, `created_at` |
| `Task` | `id`, `factory_id`, `parent_id?`, `depends_on`, `kind`, `title`, `description`, `acceptance`, `checks`, `status`, `priority`, `provider`, `model?`, `reasoning`, `created_at`, `updated_at` |
| `Run` | `id`, `task_id`, `provider`, `model?`, `reasoning`, `status`, `reason?`, `provider_session_id?`, `worktree_path?`, `pid?`, `lease_owner?`, `lease_expires_at?`, `context_revision`, `started_at?`, `finished_at?`, `exit_code?`, `summary?` |
| `Steering` | `id`, `factory_id`, `task_id?`, `revision`, `text`, `mode`, `created_at`, `acknowledged_run_id?` |
| `Event` | `sequence`, `factory_id`, `run_id?`, `kind`, `payload`, `created_at` |
| `MemoryEntry` | `id`, `factory_id`, `kind`, `title`, `body`, `source?`, `verified_at?`, `created_at` |
| `Artifact` | `id`, `run_id`, `kind`, `title`, `path_or_url`, `head_sha?`, `created_at` |
| `Automation` | `id`, `factory_id`, `name`, `enabled`, `interval_seconds`, `next_due_at`, `task_template` |
| `IntegrationStatus` | `kind`, `availability`, `detail`, `checked_at?` |
| `Snapshot` | `last_event_sequence`, `factories`, `tasks`, `runs`, `steering`, `memory`, `artifacts`, `automations`, `integrations` |

Start with these enums:

- `TaskStatus`: `backlog`, `ready`, `running`, `review`, `done`, `blocked`.
- `RunStatus`: `queued`, `running`, then terminal `succeeded`, `failed`, `interrupted`, `cancelled`, or `needs_approval`. Rust variant names are `Queued`, `Running`, `Succeeded`, `Failed`, `Interrupted`, `Cancelled`, and `NeedsApproval`.
- `Provider`: `codex`, `claude`, `cursor`, `gemini`.
- `Reasoning`: `low`, `medium`, `high`; an adapter must report when a requested setting is unsupported.
- `SteeringMode`: `queue`, `now`.
- `MemoryKind`: `decision`, `lesson`, `reference`.
- `ArtifactKind`: `test_report`, `screenshot`, `demo`, `preview`, `pull_request`.

`IntegrationStatus.availability` distinguishes unavailable executable, missing authentication, configured, and observed working. A configured command or model catalog is not proof of access.

`parent_id` groups a larger outcome into smaller tasks; `depends_on` records execution prerequisites. They are different relationships. Reject cross-factory links, self-links and cycles. A task is eligible only when its prerequisites have reached Done. Graph views label delegation and prerequisite edges separately; they do not infer dependencies from screen placement. `kind` identifies planning, implementation, review or monitoring so the radar can derive stage counts from real records.

Each check is an executable plus an argument array, relative working directory and timeout. Keep actual exit codes and tested commit identity in evidence. Provider-generated verification text cannot replace a supervisor-observed check. Validate paths remain within the task worktree and apply the factory's execution policy before running a check.

Factory agency is an explicit ceiling: observation only, workspace changes, or publishing branches and PRs. A task may request less agency, never more. Merging and production deployment require separately authorized actions with the reviewed commit or deployment identifier. The main agent uses the configured high-reasoning planner profile; individual worker tasks can choose lower effort, subject to provider capability. Store effective provider/model/effort and agency on each run so a later configuration change cannot rewrite its history.

## State transitions and review

Normal task flow is `backlog → ready → running → review → done`. A run can fail or be interrupted without losing the task or its worktree. Retrying creates a new run linked to the same task. Reconcile an abandoned run, record its execution outcome, and retain its task as Blocked with a visible reason before deciding whether to resume.

Verification is run activity and an evidence gate while the task remains Running; it is not another board column. Completing an agent process is insufficient for Review. The runtime checks its terminal outcome, then records relevant verification evidence. A review bundle contains the proposed change, test results, screenshots or demo when appropriate, preview URL when available, and unresolved limitations. Bind evidence to the tested commit SHA. A changed PR head makes old verification visibly stale.

Run outcomes describe execution, separately from task progress. A timeout is `failed` with reason `timeout`. An explicit durable user cancellation is `cancelled`, even when the provider reports an interrupted turn. A permission request that cannot be resolved within the run becomes `needs_approval`; retain the task and surface its blocking reason. A `succeeded` run still needs verification before its task reaches Review.

Done means an accepted result: merged PR for coding work, or an explicitly accepted research or operational result. Preserve this distinction so the board does not confuse generated code with delivered software.

## Durable scheduling

The supervisor retains the macro objective but runs bounded planning turns. Wake it for new steering, completed work, meaningful integration changes, or a due automation. Sleep when there is no justified next action. Present its current focus, decision summary, and next action, alongside tool activity.

Begin with one planner and a small configurable worker cap. Prioritize human steering, production failures, and work that unblocks existing tasks. Apply a review queue cap so unattended coding cannot create unlimited review work. Do not select work by PR count or lines generated.

Claim the next task in an immediate SQLite transaction, checking the factory is active and the concurrency budget is available. Enforce one active run per task with a database constraint. Store the run lease before launching a process; heartbeat while it runs. A lease grants ownership of bookkeeping, not permission to duplicate an uncertain external side effect.

On restart, reconcile leases, child processes, provider sessions, and worktree state. Mark unknown outcomes interrupted. Re-read GitHub or deployment state before retrying a push, PR creation, or merge. Use an idempotency key for each automation occurrence and each external action that supports one. Bound automatic retries and record the reason for stopping.

## Steering and context

Commit steering before acknowledging it to the client. Increment the context revision. Include the objective, applicable steering, acceptance criteria, relevant decisions and evidence, prior run summary, and repository instructions in the next run's context packet. Keep the packet bounded and link to deeper material instead of copying entire histories.

Queue is the default during an active run. Immediate steering uses the provider's supported steering operation; otherwise interrupt and resume at a safe boundary. Display received, queued, and acknowledged states separately. Accepted input does not prove the agent has acted on it.

Memory belongs to the factory and survives individual provider sessions. Keep observations and verified lessons distinct. Every claimed lesson should cite a run, test, incident, or human decision. Do not automatically turn model speculation into permanent policy. Improvements to the harness should be versioned, evaluated against fixtures, and reviewed like other changes.

## Proposed store and runtime functions

These names are a coordination contract, not committed implementation signatures:

- `Store::open(path)` applies schema migrations and enables WAL.
- `snapshot(factory_id?)` returns one coherent projection and event cursor.
- `create_factory(CreateFactory)` validates the repository path and objective.
- `create_task(CreateTask)` validates factory, hierarchy, prerequisite graph, acceptance criteria and verification commands.
- `steer(SteerRequest)` persists revisioned guidance and its event.
- `set_factory_paused(factory_id, paused)` controls future dispatch.
- `claim_next_run(owner, now)` claims one eligible task transactionally.
- `heartbeat(run_id, owner, now)` renews an owned run lease.
- `append_event(NewEvent)` records normalized activity without changing unrelated state.
- `finish_run(run_id, owner, outcome)` records a terminal result exactly once.
- `recover_expired_runs(now)` exposes interrupted work for reconciliation.
- `due_automations(now)` creates each due occurrence once.
- `ProviderAdapter::start`, `resume`, `interrupt`, and `capabilities` retain provider-specific behavior behind normalized run events.

Avoid holding a `rusqlite::Connection` across an async wait. Use a short-lived connection per database operation, or one blocking store worker. Launching the child, waiting for output, and calling external services belongs outside database transactions.

## Proposed API

| Endpoint | Purpose |
| --- | --- |
| `GET /api/snapshot?factory_id=…` | Current view with the latest event sequence. |
| `GET /api/events?after=…` | Ordered events after a cursor; bounded polling is sufficient initially. |
| `POST /api/commands` | Tagged command shared by GUI and terminal client. |
| `GET /health` | Daemon readiness, without secret or project contents. |

`Command` initially covers creating a factory or task, steering, pausing/resuming a factory, running an eligible task, interrupting a run, saving memory, and enabling/disabling an automation. Add GitHub refresh and merge commands when the corresponding adapter is implemented. An explicit merge command must bind to the expected PR head SHA and recheck required checks.

Return a structured error with `code` and `message`; do not convert failures into successful snapshots. Validate state transitions in the store/runtime, so both clients get the same behavior. Persist events before broadcasting them. Clients reconnect using the cursor; they can fetch a fresh snapshot if their cursor is no longer retained.

Bind the local daemon to loopback and require a per-installation token for project data and mutations. Keep provider credentials in their supported local credential stores or secret references, outside SQLite event payloads. Do not expose the daemon remotely merely by changing its bind address.

## Provider and integration boundaries

Build Codex integration on app-server first: persist its thread/turn identifiers and normalize its protocol events. A read-only local capability probe completed initialization and protocol metadata requests; it did not perform inference. Claude, Cursor, and Gemini use headless CLI adapters with their own success parsing, resume, interruption, and authentication checks. A generic API chat completion is not an agent harness; API-backed execution needs a supported agent SDK/runtime or an explicit tool loop. See [providers.md](providers.md) for the current transport contract and probe limits.

CLI and MCP access should be scoped per factory. Reuse provider-native MCP configuration initially. A worktree separates edits; it does not isolate dependencies, ports, databases, credentials, or network access. Move execution into containers or remote sandboxes when that boundary is needed.

GitHub supplies PR state and review actions. Cloudflare and Vercel observation should show observed deployment identifiers, URLs, status, and time checked. Failed observation is unknown, never healthy. Deployment writes and rollback require an explicit factory policy and adapter support.

## Delivery boundary

The first complete slice is one factory, a persistent objective, queued tasks, the Codex app-server integration, revisioned steering, daemon recovery, shared terminal/desktop Conversation and Progress surfaces, and a review bundle. The UI should identify seeded examples and unavailable connections clearly. Graph, board, and radar reuse the shared task/run state; worker details remain progressively disclosed.

Later increments: additional tested providers, GitHub review/merge, preview capture, deployment observation, recurring automations, always-on remote workers, container isolation, and evaluated harness improvements. These are subsequent capabilities unless the implementation and its verification actually land.

Verify scheduling and recovery with a fake provider before spending model tokens: concurrent claims, daemon restart after claim, truncated provider output, steering during a run, retry exhaustion, automation deduplication, and review queue saturation. Then exercise a real bounded provider task. Record actual outcomes separately from planned experiments.
