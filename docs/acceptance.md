# Acceptance and architecture review

Status: independent design review, October 31, 2026. These are proposed checks, not completed results. This file defines a useful first release and the additional work needed to satisfy the full project goal.

## Smallest useful release

An awake-host alpha must complete one real cycle: a saved factory objective produces a justified task, an agent changes an isolated worktree, the supervisor verifies the change, and the human can inspect evidence from either client. Saved steering must change subsequent work. Closing either client must not stop the supervisor. A board populated with sample runs is a prototype, not this release.

The alpha can support one authenticated provider and one repository. It must show every other provider and integration as unavailable, unconfigured, untested, or supported according to observed evidence. A local alpha does not satisfy the full promise of working through a day while the user's laptop sleeps; that requires an always-on execution host.

| Acceptance | Concrete observation |
| --- | --- |
| Durable objective and context | Create a factory, accepted decision and steering; restart the daemon; both clients show the same records and context revision. The next task context includes them. |
| Real controller | A bounded planning turn chooses useful work with acceptance criteria, or records a reason to wait. Completing a task wakes it. It stops producing work at the review cap, budget limit or lack of justified work. |
| Real worker | One authenticated provider edits a disposable fixture repository in a separate worktree. Record the provider session, observed terminal outcome, actual changed commit, and supervisor-observed check results. CLI installation or a model catalog alone does not pass. |
| Honest task states | Agent completion moves a coding task to review only after required evidence exists. A failed check cannot appear as passed. Done requires accepted delivery, not a successful agent process. |
| Durable steering | Submit steering during a run; show received/queued/applied distinctly. Restart before the next turn; verify that the subsequent context includes the new revision. A provider without live steering must display queued behavior. |
| Human control | Pause prevents new dispatch. Interrupt targets the current run; verify that the provider process group terminates, or retain a visible unresolved interruption. Resume does not duplicate the attempt. |
| Independent service | Close the GUI and TUI during a bounded run; observe completion in daemon records and reconnect from either client. Killing a terminal that launched the GUI must not silently kill the daemon. |
| Restart recovery | Kill the daemon after a task claim and after a worker starts. Restart and reconcile the old process/session and worktree before another attempt. No two workers may write the same task worktree. |
| One main conversation | Opening a factory shows one conversation with its lead agent. Send a message there to steer the factory; do not require choosing a worker. Surface a short current focus and useful result summaries. Worker logs, execution details and setup remain secondary. |
| Progress without clutter | A separate Progress destination offers work graph, board and radar views. The graph shows ownership and actual dependencies. Sundial shows only Plan, Build, Check and Waiting; no change list, PR preview, diff or merge appears in monitoring. Its dimensions derive from observed stage events and age, with missing or stale evidence shown honestly. |
| Dedicated review | Reviews has its own destination with a compact queue and dominant selected preview/diff, exact commit, observed checks, limitations and actions. Conversation may link to a result there without embedding the review workspace. |
| Shared client behavior | In GUI and TUI, use the same Conversation, Progress and Reviews destinations to inspect the goal, work, decision summary and artifacts, then steer, pause or review. Changes from one client appear in the other. A disconnected client shows its stale state. Workers and integrations use progressive disclosure rather than permanent panels. |
| Usable clients | Keyboard focus, small-window layout, empty state, long log lines and error states work. TUI restores terminal settings on normal exit and panic. Logs cannot inject terminal control sequences. |
| Real desktop build | Produce and launch a macOS `.app` from the documented build command. It connects to the independent daemon and survives a daemon disconnect/reconnect. A browser preview alone does not pass. |
| Local control boundary | Without the per-installation credential, read and mutation requests fail. Browser-origin requests cannot start runs, modify steering, or read project logs. The daemon listens only on the intended loopback address. |

Use a temporary repository with one small executable check for the first real run. Retain the run transcript, exact commit, check exit code and a short result. Do not let this smoke test push to a user's application repository.

The revised product direction is a quiet main conversation plus a separate view of the factory's work. On a small screen, the lead conversation remains usable without shrinking a board into its margins. TUI Progress can express the same owners, dependencies and radar measurements through terminal-friendly lists or bars; parity means the same information and actions, not pixel-identical rendering. The lead can provide decision summaries and tool evidence, but the interface must not promise access to private model reasoning.

## Completion of the larger goal

These features may follow the alpha, but they remain part of the requested project. Do not call the whole objective complete while they are only documentation or interface placeholders.

- A GitHub review flow fetches live PR state, checks and immutable evidence, requests changes, and merges the reviewed head from the interface. Observe the eventual merged result before moving a coding task to done.
- A UI task produces a screenshot, demonstration or preview tied to its tested commit. Expired URLs, missing captures and stale commits are visible.
- Cloudflare and Vercel observation returns actual deployment identifiers, URLs, commit association, state and check time. An API error is unknown, not healthy. Deployment writes require an implemented policy and explicit integration capability.
- Codex, Claude, Cursor and Gemini each pass the documented adapter checks for authentication, terminal outcomes, cancellation and session continuation. API-backed agents include a working tool execution runtime; accepting an API key alone is insufficient.
- Scheduled work persists due times, deduplicates each occurrence, records missed-run policy and remains quiet for unchanged monitoring results.
- An always-on host continues real work during a day with both local clients closed and the laptop unavailable. Reconnect and inspect the accumulated evidence. Remote transport has its own authenticated access design; do not expose the local service by binding it to every interface.
- Proposed harness improvements are versioned, checked against reproducible fixtures and compared with a baseline. A model's claim that a new prompt is better is not an experiment result.

## Simplest shared client boundary

Keep the existing single-crate, SQLite, Axum design. The daemon owns project state and execution. Share serializable commands, snapshots, events and a small Rust HTTP client module between the TUI and Tauri backend. The desktop WebView calls typed Tauri commands; those commands call the same loopback API as the TUI. Keep the daemon token in native Rust. No second desktop execution engine or database is needed.

This is a project recommendation, not a requirement imposed by either framework. Ratatui leaves event handling to the application and documents centralized input with message passing. One UI loop can receive keyboard actions and network updates, then render a coherent snapshot; network requests must not block keyboard handling. [Ratatui event handling](https://ratatui.rs/concepts/event-handling/)

Tauri supports invoking Rust commands from its frontend. Its HTTP plugin also supports JavaScript and Rust clients, but a native bridge lets both clients reuse one Rust client without giving the WebView a daemon bearer credential or general HTTP access. Use ordinary Rust HTTP dependencies already required by the TUI; the HTTP plugin is unnecessary for this design. [Calling Rust from the frontend](https://v2.tauri.app/develop/calling-rust/), [Tauri HTTP client](https://v2.tauri.app/plugin/http-client/)

Serve the desktop UI through Tauri's bundled default protocol. The separate daemon's loopback API does not require Tauri's localhost asset plugin. Tauri itself cautions that the localhost plugin adds risk. [Tauri localhost plugin](https://v2.tauri.app/plugin/localhost/)

### Required checks at the daemon boundary

- Bind a literal loopback address. Verify the accepted `Host`; reject unexpected hosts and browser `Origin` values. Native clients need no CORS grant. Keep any development origin exception narrow and out of release configuration.
- Require a random per-installation bearer credential for snapshots, logs, events and commands. Read it from an owner-only runtime file; do not put it in URLs, frontend assets, task prompts, event payloads or provider child environments. `/health` may return readiness without project data.
- Mutations use authenticated JSON commands. GET must never mutate. Do not accept arbitrary shell command strings or filesystem paths as a generic client action; map typed commands to validated factory and task records.
- Authenticate and validate in the daemon even when a caller is Tauri. Capabilities and CSP constrain the WebView; they do not authorize an independent HTTP service.
- Hold one supervisor lock for the selected data directory. Fail clearly on a second daemon rather than relying only on the port conflict. Use SQLite constraints and owned run attempts to prevent duplicate claims.
- If transport fails after a mutation, reconcile its persisted command or action identifier before retrying an external side effect. Client timeout is not evidence that a PR, merge or deployment failed.

Explicitly restrict Tauri custom commands to the trusted main window. Tauri's documentation says registered application commands are accessible to application windows by default unless command permissions are configured. Capabilities do not protect against incorrect Rust scope checks. [Tauri capabilities](https://v2.tauri.app/security/capabilities/)

Set a restrictive CSP for bundled assets and IPC. Open agent previews in the system browser for the first release, keeping project web content away from native command privileges. Tauri's CSP must be configured to be enabled. [Tauri CSP](https://v2.tauri.app/security/csp/)

## Dependencies that change delivery claims

| Dependency | Consequence and acceptance |
| --- | --- |
| Host sleep and network | A detached process or launch agent survives client exit but cannot execute while the host sleeps or is powered off. Display host availability and distinguish paused, disconnected and active work. |
| Daemon installation | A Tauri sidecar launched as a GUI-owned child does not establish independent service lifetime. Provide a documented daemon start/stop/status path; add OS service installation when unattended startup is required. |
| Launch environment | GUI apps and OS services may not inherit interactive shell PATH or login state. Resolve configured executable paths; probe them from the daemon's environment. Verify credentials under the actual service user. |
| Account entitlement | A vendor subscription, API credential and installed CLI are separate capabilities. Use supported authentication paths; probe an actual bounded inference before showing working. Unsupported model/effort combinations fail clearly rather than silently substituting. |
| Integration scope | CLI availability is not GitHub merge or Cloudflare/Vercel deployment authorization. Store the selected account/repository/project scope and check the specific operation, without exposing credential values. |
| macOS distribution | A local `.app` smoke test can use ad-hoc signing. Smooth distribution to other Macs needs signing/notarization credentials and a clean-machine install check. Do not claim a public installer was validated from a developer build. |
| Preview provenance | A screenshot of an old branch or a deployment from a different SHA does not verify the reviewed change. Require source commit metadata or label the artifact unverified. |
| Worktree boundaries | Worktrees isolate edited files, not databases, ports, global dependencies or credentials. Fixture tasks need independent resources; untrusted execution needs a real sandbox before stronger isolation can be claimed. |

Tauri documents ad-hoc signing as available but still subject to macOS security prompts; notarization requires the appropriate Apple account and signing configuration. Treat these as distribution prerequisites, not blockers to a local development alpha. [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/)

## Exact-commit review and merge

A review packet should contain repository identity, PR number, head SHA, tested base SHA, verification command/outcome, artifact SHA and observation times. Re-read the PR immediately before merging; send the reviewed head SHA to GitHub and respect current repository rules. If the head changes, invalidate old approval/evidence. If the base changes and the relevant checks were only local, re-verify the proposed merged result or rely on required server-side checks under repository policy.

GitHub accepts an expected `sha` on merge requests. Its current REST documentation recommends the asynchronous merge endpoint, which also supports merge queues. A returned `202`, `pending`, or `enqueued` is not a completed merge; poll the operation and, for queued requests, the eventual PR state. A synchronous head mismatch returns `409`. [GitHub pull request merge API](https://docs.github.com/en/rest/pulls/pulls#merge-a-pull-request)

Keep an immutable action record before issuing the merge. After a network timeout, query the PR state before any retry. Never bypass repository rules merely to make the interface's merge button succeed.

## Correctness risks to resolve before implementation

1. **One provider contract.** Resolved in the design: Codex uses app-server first; other providers use their supported headless CLIs. Implementation must retain the documented distinction between live steering and queued input, and verify the installed protocol before depending on optional methods.
2. **One task state contract.** Resolved in the design: tasks use Backlog, Ready, Running, Review, Done and Blocked. Verification is activity and an evidence gate; terminal execution failures belong to runs. GUI, TUI and store must use this same contract.
3. **Claimed work versus observed work.** A provider's final text, process exit or exhausted stream must not become verified success. Normalize terminal provider state, then independently capture required check exit codes and commit identity.
4. **Recovery versus duplicate execution.** An expired lease cannot prove that a child or remote action stopped. Reconcile or terminate the old executor before retry; refuse stale run updates. Preserve the worktree for inspection.
5. **Persistent constraints versus chat history.** Steering acknowledgment means receipt until its revision is actually included in the next applicable turn. Verify application through resulting behavior and evidence; do not equate provider acceptance with compliance.
6. **Continuous work versus continuous noise.** An unchanged state must not produce another planning call or duplicate task. A day away should end with bounded reviewable outcomes, spent budget and clear blockers.

The checks here focus on correctness of the requested workflow. No implementation, real inference, distribution signing or broad security scan was performed as part of this design review.
