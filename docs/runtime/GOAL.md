/goal Build Software Factory into the first usable local application
that I can use to manage real software work, preserving the approved
interface and proving the complete workflow with real execution.

WHAT I WANT

I want to connect a repository, give its lead agent a persistent goal,
steer it through one conversation, monitor delegated work, and review
results without constantly organizing every task myself.

Deliver a working local alpha: one project, one Codex-backed lead,
one bounded worker, and a real review packet. This goal is complete
only when that workflow works through the actual application.

START WITH THE EXISTING WORK

Read:
- START-HERE.md
- AGENTS.md
- apps/desktop/AGENTS.md
- docs/handoff/BUILD-PLAN.md
- docs/handoff/DESIGN-INDEX.md
- The current interaction contracts linked from those files.

Inspect the code, mocks, latest screenshots, tests, and working tree.
Preserve uncommitted work. Use the full supplied handoff: a fresh clone
of its recorded base commit will omit current source and design files.

The existing React/Vite app is a functional design preview. Agent runs,
permissions, scheduling, file operations, and review actions are not
connected to a real runtime yet. Keep simulated and real behavior
clearly distinguished throughout implementation.

HOW TO BUILD IT

Use Rust for the durable application core, SQLite for persistent state,
Axum for an independent authenticated local daemon, and a thin Tauri
desktop shell around the existing React interface.

Keep the implementation straightforward: one core crate with focused
modules, clear ownership, and small validated increments.

Use Codex app-server as the first provider. Verify its installed
protocol, authentication, model options, and enforcement capabilities
against current official documentation before implementing the adapter.

The daemon owns planning, scheduling, execution, approvals, events,
memory, and evidence. Closing the desktop client must not terminate
ongoing work on an awake host.

Use bounded planning turns. Wake the lead for new steering, eligible
requests, and completed work. Continue with justified work or wait
quietly with a clear reason. Enforce concurrency, time, usage, retry,
and review-queue limits.

Keep Request, Task, and Run distinct:
- Request: submitted intent.
- Task: planned work with acceptance criteria.
- Run: one execution attempt with a stable identity.

Capture immutable requested and effective settings, permissions,
context, provider IDs, worktree identity, and revision evidence.

PRESERVE MY PRODUCT DIRECTION

- One human-facing lead conversation; workers report through it.
- Compact, soft, warm-paper UI with clear hierarchy.
- Keep the approved layout and proportions rather than redesigning it.
- Composer controls belong at the bottom, with borderless timing.
- Quiet activity summaries, visible approvals, one receipt per send,
  and scrolling that preserves my reading position.
- Project-level permissions for edits, commands, network, and
  integrations; individual sends may restrict those permissions.
- Graph, Board, and our original Sundial Radar for monitoring.
  Keep changes, diffs, previews, and merge actions outside Radar.
- A compact Reviews inbox opening a large Preview/Changes/Evidence
  workspace, with feedback and checks tied to the exact revision.
- Scratchpad remains a blank autosaved document. Its settings remain
  in Settings, with the configurable morning review plan preserved.

IMPLEMENT AND VERIFY

1. Establish the existing frontend build/test baseline.

2. Implement durable project goals, settings, steering, requests,
   tasks, attempts, events, and evidence. Prove transactional rollback
   and persistence after reopening the database.

3. Implement the local daemon and deterministic fake-provider tests
   before real inference. Cover duplicate commands, concurrent claims,
   failed execution, approvals, interruption, and restart recovery.

4. Connect the existing application to real service records.
   A request must reach the lead, produce a justified task or an
   explicit waiting reason, and dispatch one authorized worker.

5. Run coding attempts in isolated Git worktrees. Enforce permissions
   through actual runtime/provider controls. A worktree alone is not
   a security sandbox. Unsupported restrictions must block execution
   visibly rather than silently granting access.

6. Have the supervisor independently run verification and capture
   actual exit codes and the exact tested revision. Populate Reviews
   with the real diff, checks, artifacts, and limitations.

7. After fake-provider checks pass, perform one bounded integration
   trial in a disposable local repository using existing supported
   authentication and explicit configured time/usage limits.
   Do not publish, push, merge, deploy, or install recurring jobs
   as part of this trial.

FINISH LINE

Demonstrate through the real native application that I can:
- Register a repository and persistent objective.
- Save project permissions and select supported execution settings.
- Send steering and a one-off request to the lead.
- Observe a real worker and its execution history.
- Inspect its actual diff and supervisor-observed checks in Reviews.
- Pause future dispatch and interrupt active work with an observed
  outcome or an explicit unresolved control state.
- Close/reopen the client and restart the daemon without losing
  records, rewriting historical snapshots, or duplicating execution.

Keep existing tests passing. Add meaningful runtime tests and check
the changed interface at desktop and phone sizes. Record the actual
commands, results, revision, and evidence for the integration trial.

WORKING STYLE AND BOUNDARIES

Begin implementation and continue toward this finish line. Use
subagents for independent implementation and review, with clear file
ownership. Resolve routine technical choices autonomously.

Maintain a durable checkpoint with completed work, verification,
remaining blockers, and the next exact action.

If credentials, access, or an external dependency blocks progress,
complete independent work and identify the specific input needed.
Never substitute fabricated success for missing evidence.

Keep credentials out of browser assets, source, prompts, and logs.
Preserve unrelated work and existing access boundaries.

Additional providers, TUI parity, GitHub review/merge, deployments,
real recurring Scratchpad triage, and hosted continuity remain later
milestones. Preserve that roadmap without making it a prerequisite
for this first usable local alpha or claiming the full vision is done.

Finish by launching the application and providing exact startup,
usage, and verification instructions, with connected capabilities
and remaining limitations stated clearly.