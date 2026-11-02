# Local alpha implementation checkpoint

Follow-up: read-only GitHub/GitLab repository connections and remote Reviews are now implemented. See [repository integration verification](REPOSITORY-INTEGRATIONS.md) and [setup](../integrations.md). The original local-alpha goal/trial below is complete; this later work does not repeat its paid trial. The initial alpha is committed and pushed at `8e3fbd7`; the original handoff's base/snapshot statements describe its packaging history.

The active goal is a usable native application, not a store-only milestone. Preserve the approved React interface. The authoritative scope is [GOAL.md](GOAL.md), with [the build plan](../handoff/BUILD-PLAN.md) and [current design contracts](../design/compact-preview/README.md).

## Completion predicate

One registered local repository and persistent objective → one Codex lead conversation → a justified bounded task → one authorized worker in an isolated worktree → supervisor-observed checks and exact revision → real local Review packet. The native client must support settings, steering, requests, history, pause and observed interruption. Client closure and daemon restart preserve records and cannot cause blind duplicate execution. No seeded activity counts as evidence.

## Settled implementation

- One Rust core crate with focused modules; SQLite state and events commit together.
- An independent authenticated Axum daemon owns execution and scheduling. A thin Tauri bridge keeps its credential outside JavaScript.
- Request, Task and Run are separate; requested/effective settings and context are immutable evidence.
- Codex app-server is the first provider. Deterministic fake-provider tests precede one bounded real fixture trial. No push, PR, merge, deployment or recurring-job installation.
- Keep the compact warm-paper UI and existing design preview. Live mode never falls back to example records.

## Work sequence

1. [x] Inspect the full handoff, working tree, installed toolchains and baseline.
2. [x] Durable model/store; reopen, rollback, idempotency and claim tests.
3. [x] Authenticated loopback service, exclusive ownership and typed native client.
4. [x] Capability-verified provider transport and deterministic failure/control tests.
5. [x] Bounded lead/supervisor/worktree/check/review workflow and recovery, including durable read-only lead wakeups.
6. [x] Connect and launch the current interface through Tauri; desktop/phone inspection.
7. [x] One real disposable fixture trial after fake tests pass; record exact IDs, SHA, commands and limitations.
8. [x] Startup/usage instructions, final verification and remaining roadmap.

## Verified baseline (2026-11-01)

Before runtime edits, `pnpm build` in `apps/desktop` passed, followed by `node --test --test-reporter=dot tests/*.test.mjs`: 45 passing tests. Existing protected Sites files remain intact. Baseline Git HEAD: `4c8938e90f5f10e8d275c7a0fa64ce03f41cf7e4`; current handoff/frontend files include uncommitted work and must be preserved.

Installed: Rust/Cargo 1.99.1, Tauri CLI 2.12.4, Xcode 26.6, Node 24.22.1, pnpm 11.25.0, Codex 0.159.3. Codex metadata probes used existing authentication and performed no inference. Installed schemas, not assumed field names, govern the adapter. A turn succeeds only from its matching terminal event; EOF and interrupt acknowledgments do not establish success or stop. Installed turn/start has no hard token/cost cap; deadlines and observed accounting must be labeled accurately.

## Current work and next action

The Rust core, SQLite schema 2, authenticated daemon, provider adapter, managed worktrees, safe supervisor checkpoint commits, exact-revision checks, atomic completion/review packets, Tauri bridge and live React mode are implemented. A run-scoped command approval records actual execution authorization separately from its immutable original ceiling. New steering waits for the lead rather than bypassing it into a worker.

The final Rust suite passed **67 tests** (14 lead, 13 provider, 13 supervisor, 4 service, 13 store, 10 worktree), with one installed-provider smoke ignored by default and separately passed without inference. Frontend build and **55 tests** passed. The native bridge security test, both Rust formatting checks and the bundled `.app` build passed.

The native application demonstrated registration, supported model selection, steering, one-off submission, scoped command approval, real Git/check evidence and local acceptance. A deterministic external-process substitute first proved client-independent execution, reopening the same active identity, observed interruption/reaping, pause and daemon restart. Desktop and 394px reviews/composer were inspected. The full-SHA phone overflow was corrected and its final capture has no horizontal overflow.

The **single authorized actual Codex integration trial passed**, using existing supported authentication, `gpt-6.1-sol`/low, 120-second turns, an observed 4,000-output-token threshold, one worker attempt and one review. Only `value.txt` changed from pending to ready. The supervisor checkpointed and independently observed `/usr/bin/python3 check.py` exit 0 on exact revision `39576124ac6bd9d81ac0b1cfafb62c8d084c704a`. The parent repository remained unchanged. Native local acceptance and paused client/daemon restart retained the same records and event sequence without duplicate launch. Read [INTEGRATION-TRIAL.md](INTEGRATION-TRIAL.md) for commands, IDs, evidence and limits.

The local-alpha finish line is complete. Operator/onboarding and continuation documentation describe the implemented source and the later roadmap. The current portable source/design handoff includes sanitized trial evidence and excludes private daemon/provider state. Its manifest and ZIP are verified during packaging.

The native application is launched at **Start a local project**, connected to a daemon using default `~/Library/Application Support/SoftwareFactory` state with zero projects, requests, runs or reviews. It is ready for the user's repository and objective. The completed real trial remains preserved and paused in task workspace `work/real-alpha-trial-26518c52/state`; its daemon/client are stopped. The pre-existing browser design preview remains on `127.0.0.1:5173`. No external work was pushed, published, merged or deployed. All source remains uncommitted, and the full handoff is needed in addition to the recorded base commit.

Next operator action: enter a project name, actual repository root and persistent objective in the native app; choose supported model/limits and project permissions before submitting work. [Getting started](../getting-started.md) provides exact startup and usage commands. No further inference is needed to validate this completed goal. Later milestones require a new scoped assignment.

## Known capability limits

Legacy Codex sandbox policies bound writes/network, not host read roots. Explicit per-ID MCP disabling and enumerated user-skill disabling require readback; empty configuration objects do not erase merged configuration. `Ask` has a durable attempt-scoped command approval; agent command networking and integrations are explicitly narrowed to Deny in this alpha. The supervisor checkpoints through sanitized Git instead of broadening worker access to shared Git metadata. Unsupported submodule/gitlink repositories fail closed. Uncertain provider or detached-process outcomes remain in recovery and fence successors. No claim of a hard prepaid spend cap, full credential-read isolation, or unattended work during host sleep is made.
