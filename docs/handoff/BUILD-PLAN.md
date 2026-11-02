# First usable local factory: implementation and acceptance

The target is **one repository → one Codex-backed lead → one bounded worker → supervisor-observed checks → a real local Review packet**. The Rust runtime, daemon and native client are now implemented around the approved React interface. This plan retains the acceptance gates and later roadmap; consult [the checkpoint](../runtime/IMPLEMENTATION.md) for what actually passed and the next unfinished step. Implementation or a successful build alone does not complete the [active goal](../runtime/GOAL.md).

Start with [START-HERE](../../START-HERE.md), [getting started](../getting-started.md), [desktop instructions](../../apps/desktop/AGENTS.md), and [the design index](DESIGN-INDEX.md). The [architecture](../architecture.md), [provider notes](../providers.md), and [broader acceptance](../acceptance.md) include later intended behavior; current source and runtime evidence distinguish connected features from proposals.

## Current inventory

| Area | Implemented source | Boundary |
| --- | --- | --- |
| Durable model and SQLite | `src/model.rs`, `src/store.rs` | Schema 2, one project per data directory |
| Authenticated local service/client | `src/service.rs`, `src/client.rs`, `src/bin/factoryd.rs` | Literal loopback, native-only credential, one owner |
| Codex app-server adapter | `src/provider.rs` | Installed schema/capability readback, existing authentication |
| Planning, supervision and lead wakes | `src/runtime.rs`, `src/lead.rs` | Bounded lead, one worker, no unbounded goal generation |
| Managed worktrees/checkpoint | `src/worktree.rs` | No submodules, sanitized local Git, no publishing |
| Native and live React client | `apps/desktop/src-tauri`, `apps/desktop/src/LiveApp.jsx` | Separate from browser-only fixtures |

The static Sites worker remains asset delivery, not execution. Scratchpad's blank document and morning review plan remain client-local; recurring triage is not connected. TUI, other providers, hosted execution, file editing/attachments, GitHub and deployments remain later work.

## 1. Durable identity and state

SQLite transactions commit state, events and command outcomes together. Keep Request, Task and Run distinct:

- **Request:** immutable submitted intent, policy/earliest start, source, model, permissions and context.
- **Task:** justified planned work with acceptance criteria and executable independent checks.
- **Run:** one execution attempt with a stable ID, attempt number, captured context/settings, executor fence, provider IDs, worktree/base/head and terminal evidence.

New defaults never rewrite prior snapshots. Actual execution authorization is recorded separately from the original ceiling and cites scoped approvals. Do not import sample Green Room history as real work. Browser preview state and native drafts use separate namespaces; authoritative runtime records live in SQLite.

**Gate:** reopen preserves objective, steering, requests, revisions and ordered events; injected failure rolls back state and event; duplicate command IDs replay one result; conflicting payloads fail; concurrent claims cannot create two active attempts. See `tests/store.rs` and `tests/lead.rs`.

## 2. Independent service and native trust

The daemon owns scheduling, provider processes, policy and evidence. It holds an exclusive lock per data directory and exposes typed snapshot/event/command operations on an ephemeral literal `127.0.0.1` address. Project endpoints require the random owner-only native credential; Host and browser Origin checks reject direct browser access. Reads do not mutate state.

The Tauri bridge reads the connection in Rust and passes no token or arbitrary destination to JavaScript. The trusted main window uses explicit commands, ACL and CSP. The bridge does not start the daemon. Reopening the client must not change execution ownership or launch work again.

**Gate:** unauthenticated/wrong-origin/invalid-host access fails; second-owner startup fails; uncertain command responses retain their command ID; closing the native client leaves an awake-host worker owned by the daemon. See `tests/service.rs`, the native bridge test and recorded native evidence. No execution-through-sleep promise is made.

## 3. Capability-verified provider and permissions

Codex app-server uses the observed installed protocol, supported existing authentication and discovered model/effort catalog. Thread and turn IDs are saved before continuing; only the matching terminal outcome can establish completion. Partial/interleaved frames, stderr flooding, EOF, unknown events, timeouts and approval events are not success.

Project read-only/worktree access and Commands/Network/Integrations ceilings resolve at dispatch. Commands Ask produces a durable attempt-scoped approval covering worker commands and declared supervisor checks. This alpha narrows agent command networking and integrations to Deny, records the limitation and blocks unsupported expansion. Explicit merged MCP/skill disablement requires provider readback.

Legacy sandbox controls bound writes and command networking but permit host reads. A worktree is not credential isolation. Output-token limits are observed stop thresholds; the installed protocol does not offer a hard prepaid token/cost cap. Missing accounting stays unknown.

**Gate:** deterministic protocol fixtures prove terminal failure, nonterminal waiting, deadline/interrupt handling and lost responses without false success. Forced process stop or detached-descendant uncertainty remains recovery-required and fences successors. The separate installed-provider sandbox smoke verifies actual inside/outside write and loopback-network behavior without inference. See `tests/provider.rs` and runtime evidence.

## 4. Bounded lead, schedule and worker

One shared lead receives durable steering and eligible one-off requests. It either returns one validated plan with acceptance/checks or a reason for waiting. New steering and completed work receive bounded read-only lead wakes; they do not fabricate tasks or requests. Unchanged idle state does not repeatedly invoke planning.

The supervisor owns scheduling, concurrency, limits, policy and dispatch. Scheduled time is an earliest start, not a fabricated Run. Pause stops future work; cancellation of pending intent is separate from interrupting an active attempt. Recovery-required planning/wakes fence the shared lead until reconciled.

Each coding attempt has a managed linked worktree. The worker leaves intended edits; the supervisor makes a sanitized local checkpoint without granting worker access to shared Git metadata, then independently checks its exact revision. Hooks, signing, filters and other Git execution paths are disabled for these operations. Parent dirty work and uncertain source are preserved.

**Gate:** one Request becomes a justified Task and one observed Run or an explicit wait; queue limits and pause prevent new claims; failures do not poll into repeated retries; attempts retain immutable snapshots; checks cannot escape the managed cwd. See `tests/runtime.rs`, `tests/lead.rs` and `tests/worktree.rs`.

## 5. Real UI and source-bound Review

Native `LiveApp` uses service records and explicit disconnected/loading/error states, never seeded successes. Preserve the 168px rail, 40px header, one lead Conversation, quiet activity, bottom composer controls, borderless timing, blank Scratchpad, project permission editor, Graph/Board/Sundial and separate large Reviews desk.

A successful Run and its Review packet commit atomically with actual check argv, exit codes, revision and diff. The service independently observes current managed worktree identity, clean state, HEAD and latest attempt. Historical evidence remains visible but cannot certify changed or unobserved source. Local decisions are revision-bound; they neither merge nor mark coding work delivered. Optional screenshot gaps are disclosed.

**Gate:** actual native registration, observed settings, steering, one-off, scoped approval, history and local review work through the approved UI. Desktop and phone views remain usable. Source changes block current decisions, and reconnect retains the same identities. Use frontend tests, native bridge tests and screenshots; do not treat browser simulations as proof.

## 6. Recovery and the single real fixture

Deterministic fake-provider gates precede inference. Restart after claim/start/terminal output must not blindly launch a replacement; stale writers are fenced and uncertain ownership preserved. An interrupt receipt is only intent until a matching terminal/observed process outcome or explicit unresolved state exists. There is no general recovery-resolution UI in this alpha.

The active goal authorized **one bounded actual Codex fixture trial** in a disposable local repository, using existing supported authentication and explicit limits. Inspect the checkpoint and trial evidence before doing anything billable. Do not repeat a completed or uncertain trial just to update documentation. The trial must retain actual source identity, provider IDs, tested SHA, check exit code, usage when available and sanitized evidence. No push, PR, merge, deploy, access broadening or recurring-job installation.

**Gate:** the native app proves the complete real cycle, while fake-provider/native controls prove client independence, observed interruption, pause and daemon restart without duplicated attempts. Record the precise commands, evidence and remaining manual service-start step. A model catalog, OS smoke, successful bundle or substitute-provider run alone does not satisfy this gate.

## Subsequent milestones

Preserve the broader product direction after the local slice: recurring Scratchpad agent triage with configurable morning scheduling, time-zone/DST/missed-slot/deduplication rules and exact-source completion marks; additional verified providers; Ratatui parity; GitHub review/merge; deployment observation; stronger isolation; always-on hosting; and evaluated harness improvements.

Latest [one-off](../design/compact-preview/oneoff-interaction-contract.md), [Scratchpad/permissions](../design/compact-preview/scratchpad-permissions-contract.md) and [Reviews](../design/compact-preview/reviews-interaction-contract.md) contracts supersede historical layout assumptions. Retain [all design materials](DESIGN-INDEX.md); implement the current compact-preview direction, not archived queue/Split mocks.
