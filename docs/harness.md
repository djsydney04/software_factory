# Persistent factory harness

Status: broader harness design with a working local-alpha subset. The Rust/SQLite supervisor, authenticated Axum service, Codex app-server execution and Tauri client are implemented. The [runtime checkpoint](runtime/IMPLEMENTATION.md) and [bounded real integration trial](runtime/INTEGRATION-TRIAL.md) record observed results; the experiments and broader policies below remain design requirements unless that evidence says otherwise.

Current scope is one project, one Codex lead and one worker. Eligible Requests get a bounded plan or waiting reason; steering and completed runs get bounded lead acknowledgments/summaries. The objective alone does not spawn unlimited new work. TUI, additional providers, daily spend reservation, external writes, recurring triage and always-on hosting remain later work. Browser preview activity is simulated and is not daemon evidence.

## The operating model

A factory owns a repository, a persistent goal, accepted human steering, useful tasks, runs, evidence and memory. A Rust supervisor owns that state. The terminal and desktop clients read and control the same supervisor.

The supervisor remains available while workers run bounded tasks. It waits when there is no useful work. Persisting a goal does not require a model to produce text continuously.

A local supervisor only works while its host is awake and connected. Running through a day away from the computer requires an always-on host. The same service boundary should accommodate that later without changing task semantics.

## Durable records

| Record | Minimum information |
| --- | --- |
| Factory | Repository, goal, constraints, agency settings, integrations, budget and concurrency |
| Task | Intended outcome, acceptance criteria, scope, dependencies, priority, verification and required artifacts |
| Run | Task, provider/model/effort, attempt, worktree, base SHA, provider thread/session ID, lease, deadline and terminal outcome |
| Steering | Message, target scope, revision, creation time, queued/applied acknowledgment |
| Evidence | Exact head SHA, commands and exit codes, checks, preview/demo, review result |
| Memory | Accepted fact or decision, provenance, scope, source revision and timestamp |
| Event | Ordered sequence, record IDs, event kind and small payload |

Use SQLite as the source of truth. Store large logs and artifacts as files referenced by database records. SQLite full-text search is sufficient for an initial knowledge base; introduce semantic retrieval only when real usage demonstrates a gap.

## Scheduling useful work

Wake the controller on human input, a ready task, completion of a run, a meaningful CI/deployment change, or a due automation.

At a wakeup:

1. Read the goal, accepted steering, relevant memory, current repository evidence, budget and work in progress.
2. Select or propose at most three useful tasks. Require an outcome and a verification surface for each.
3. Prefer human commitments, then compare expected impact, confidence, effort and risk. A priority score is a decision aid, not proof of product improvement.
4. Dispatch one implementation task per factory by default. Increase concurrency only for independent scopes.
5. Verify the resulting commit and prepare its review evidence.
6. Continue to another ready task or wait with an explicit reason.

Persist only these task states: `Backlog`, `Ready`, `Running`, `Review`, `Done` and `Blocked`. Verification is visible activity and an evidence gate while a task is running, rather than another board column. A failed or cancelled run retains its task for reassessment; record a clear blocked reason when input is missing. A factory can be paused independently of task state. Completing an agent turn alone does not complete a task.

Terminal run states are `Succeeded`, `Failed`, `Interrupted` and `Cancelled`. `NeedsApproval` is nonterminal waiting, with a durable scoped approval record. The current alpha also retains `Interrupting` and `RecoveryRequired`: an acknowledgment, timeout, process loss or forced process-group stop alone cannot prove every executor stopped. Uncertain control fences successors until reconciliation. The alpha commits `Succeeded` and its Review packet atomically after independent checks at the exact revision; provider completion alone cannot establish that state.

## Avoiding noise

- Do not re-plan when goal revision, steering revision, repository state, task queue and external observations are unchanged.
- Deduplicate candidate tasks by their outcome and affected scope. Append evidence to an existing task instead of creating another copy.
- Require an observable improvement: demonstrated feature behavior, repaired regression, measured performance, resolved deployment problem or human-approved visual change.
- After two attempts without new evidence, reassess scope or request missing input. Do not repeat the same prompt indefinitely.
- Retry transient provider failures with bounded exponential backoff; credential failures wait for user action.
- Keep unchanged monitoring observations quiet. Notify for meaningful changes, completed review packets, failures or required user action.
- Measure accepted outcomes and review effort, not task count, generated lines or token consumption.

The controller should explain its current decision in a short shared summary. Clients can show plans, tool activity, evidence and provided model summaries. They must not promise access to a model's private reasoning.

## Steering, leases and recovery

Persist human steering before acknowledging receipt. Show whether it is queued or applied. Apply persistent constraints to every subsequent task; task-specific input stays with that task. Acceptance by a provider is not proof the resulting code honors the input.

Each run attempt receives a lease and fencing token. Start with a 10-second heartbeat and 30-second expiry, then calibrate under real workloads. A provider can be silent during a long tool call; silence alone is not a failed lease. The supervisor renews a lease while it owns the process and retains a separate run deadline.

After restart, reconcile local processes, provider sessions, worktrees and persisted outcomes before dispatching replacements. Reject state updates carrying an old fencing token. Where an old process might still mutate a worktree, terminate or isolate it before retrying.

Cancellation is a durable request. Interrupt the provider, then terminate its process group after a short grace period. Do not create another task while a factory is paused. A remote tool or deployment may continue after the local process stops; reconcile that external state before declaring cancellation complete.

Create external objects through an outbox with stable action keys. If creating a PR or deployment has an uncertain result, discover the existing remote object before retrying. Database fencing cannot provide exactly-once external writes by itself.

## Budgets and agency

Enforce run deadlines, concurrency, attempt limits and daily factory allowance. Enforce token/cost limits when the provider reports enough information; label absent or estimated accounting honestly. Reserve budget before dispatch to avoid concurrent overspend. Never reset accounting by silently restarting a session.

The implemented alpha enforces turn deadlines, one-worker concurrency, attempt and review-queue limits. Its output-token threshold is observed when the provider reports usage, not a hard prepaid cap; it has no daily spend reservation. Worktree writes and command networking use verified provider controls. Attempt-scoped command approvals are durable; agent command networking and integrations are narrowed to Deny. The legacy sandbox still permits host reads and does not isolate credentials.

Use isolated worktrees for worker changes. Routine local implementation can proceed within configured access. Publishing, merging and production changes follow the factory's explicit agency settings. Do not infer permission merely from a task's priority.

## Verification and review

Verification uses commands whose exit codes the supervisor observes. An agent's success claim is a claim to check. Choose checks appropriate to the changed runtime path; repeated broad checks without new evidence waste time.

A review packet contains the intended behavior, exact head SHA, diff, observed checks, preview/demo where applicable, and a concise review result. Bind every artifact to the commit it demonstrates. A later head change makes earlier checks and screenshots stale until renewed.

Review workers inspect an immutable diff and evidence in a fresh context. The human can request changes or merge from the same interface once the integration is implemented and authorized.

## Experiments

These are the original broader acceptance experiments. Deterministic runtime tests and native fake-provider checks now cover the local subset, including uncertain recovery, interruption and client independence; one real Codex trial verifies the coding/check/review cycle. Use the checkpoint and trial evidence for exact coverage rather than treating every row below as passed.

| Experiment | Procedure | Pass condition |
| --- | --- | --- |
| Crash recovery | Kill supervisor during a fixture task, then restart | Preserved history and worktree; one recovered task outcome; no duplicate PR |
| Durable steering | Send a scope correction during a run, restart | Correction remains visible and applies before completion |
| Cancellation | Cancel while a tool subprocess runs | Prompt termination, explicit terminal status, no successor task |
| Quiet monitoring | Feed 1,000 identical external observations | One meaningful persisted observation; no repeated model calls |
| Budget enforcement | Exhaust run and daily allowances | No new dispatch; partial work retained; truthful status |
| Verification gate | Return agent success while fixture check fails | Task stays outside review/done |
| Review integrity | Change head SHA after successful checks | Earlier evidence becomes visibly stale |
| Provider parsing | Replay partial lines, unknown fields and missing terminal event | Accurate outcome, no parser crash, no false success |
| Routing | Repeat the same fixture tasks at modest and high effort | Compare accepted completion, elapsed time and reported usage before changing defaults |

## Verified local probes

On October 31, 2026, the installed `codex-cli 0.159.3` completed a real app-server `initialize` handshake. `model/list` returned eight entries. A `thread/goal/get` request for a deliberately nonexistent UUID returned `thread not found`, establishing that the local method is recognized.

Installed Cursor CLI version was `2026.10.22-27dcf5f`. Claude and Gemini executables were absent. The probes used help/version and protocol metadata requests only; they performed no inference and no task implementation. See [provider integration notes](providers.md) for protocol sources and limitations.
