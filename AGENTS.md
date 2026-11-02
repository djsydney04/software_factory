# Software Factory agent instructions

Read [START-HERE.md](START-HERE.md) before editing. The active local-alpha scope is [GOAL.md](docs/runtime/GOAL.md); inspect [the checkpoint](docs/runtime/IMPLEMENTATION.md), [the agent prompt](docs/handoff/AGENT-PROMPT.md), and [build plan](docs/handoff/BUILD-PLAN.md) before continuing. Apply `apps/desktop/AGENTS.md` to frontend work.

This checkout contains a Rust/SQLite core, authenticated local `factoryd` service, Codex app-server adapter, managed Git worktrees, and a thin Tauri client over the approved React UI. Browser-only Vite remains a separate **design preview** with fixtures and local simulations. Native live mode uses daemon records and never falls back to fixtures. Implementation does not by itself prove the goal's complete workflow; use the checkpoint and recorded integration evidence for acceptance status. TUI, additional providers, GitHub/deployment writes, hosted continuity and recurring Scratchpad triage remain later work.

The subsequent [repository integration](docs/integrations.md) adds explicit human-requested reads of GitHub.com/GitLab.com repositories, open PRs/MRs and selected-request CI through existing `gh`/`glab` authentication. Keep these separate from agent execution and from local Review certification. No provider writes or interactive login are implemented by this integration.

## Source precedence

1. Current explicit user direction.
2. The active goal, these instructions, applicable module instructions, and current interaction contracts under `docs/design/compact-preview/`.
3. Current source, runtime checkpoint/evidence, and latest QA screenshots indexed in `docs/handoff/DESIGN-INDEX.md`.
4. Architecture/harness/provider documentation, amended by `docs/handoff/BUILD-PLAN.md` and the implemented runtime's documented limitations.
5. Historical mocks, consultations and archived prompts, which are context rather than a new implementation assignment.

If available on this machine, follow `/Users/dylanmitic/workspace/agent-skills/guidance/skill-routing.md`. The handoff must still work elsewhere without that private path or any optional skill/plugin. Keep changes small, read connected code before editing, and validate actual behavior rather than adding abstractions or implementation-mirroring tests.

## Invariants

- One human-facing lead conversation; workers report through that lead.
- Request is intent, Task is planned work, and Run is one execution attempt. Do not fabricate a Run, acknowledgment, check or completion because a message was saved or a scheduled time arrived.
- Capture immutable requested/effective model, permission and context snapshots. New defaults never rewrite prior evidence. Preserve source identity when migrating the local preview.
- Enforce project edit/command/network/integration ceilings in the real runtime. A next-send choice can narrow them. A worktree is not a security sandbox. Unsupported restrictions must fail visibly and closed.
- State changes and their events commit together. Reconcile uncertain execution on restart before launching another attempt; preserve existing worktrees and unrelated dirty work.
- Checks and review decisions bind to the exact Run/attempt/revision. A changed head invalidates old green evidence. Preview actions are never real GitHub actions.
- Repository integration reads are bounded and manually requested. Keep provider credentials CLI-managed, raw CLI output private, and public-provider hosts fixed. Remote CI must show its observed revision, time and gaps; it never certifies a local Review or grants an agent integration access.
- Preserve the 168px rail, 40px header, restrained 14/13px typography, bottom composer controls, blank Scratchpad, active-only Sundial, and Reviews inbox → full-width desk. See the desktop instructions for the complete current direction.
- Keep provider credentials out of browser assets, source, logs, event payloads and handoff files. Use supported authentication mechanisms; do not copy local account/session state into the project.
- Keep `connection.json`, databases, provider sessions, managed worktrees and private runtime state out of source handoffs. The connection credential is native-only and rotates on daemon restart.
- A command approval is scoped to one attempt and its independent checks. Network and integrations are narrowed to Deny for alpha execution. Legacy provider controls restrict writes/network, not host read roots; never claim credential-read isolation or a hard prepaid token cap.
- Preserve recovery-required records and their worktrees. An uncertain provider outcome fences further execution; clearing the database or launching a replacement is not reconciliation.

## Working agreement

For setup and checks, use [getting started](docs/getting-started.md). Existing frontend dependencies and build-template files are intentional. Read module instructions before changing them. New runtime commands must be documented only after they exist and have been exercised. Metadata discovery and fake-provider tests are not paid inference. Do not repeat the goal's single real fixture trial merely to refresh documentation; inspect its evidence and existing authorization first.

Preserve existing uncommitted and untracked work. The handoff ZIP includes files absent from the recorded Git base commit. Repository snapshots are not permission to publish, merge, deploy, expose the local daemon publicly, or expand an account's access. Continue authorized local implementation and reversible validation without unnecessary approval stops.
