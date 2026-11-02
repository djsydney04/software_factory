# Paste this into the continuing agent

The local-alpha finish line has now passed; see [the actual trial](../runtime/INTEGRATION-TRIAL.md). The prompt below preserves the build/maintenance contract. A new agent should use the current user assignment and checkpoint to choose the next milestone, and must not repeat the completed paid trial or rebuild finished components.

Use the full supplied repository/snapshot, not only its remote URL or screenshots. The canonical finished local-alpha goal is [GOAL.md](../runtime/GOAL.md); its original preview description records the starting point. The current source already includes the runtime. Paste this prompt from the repository root:

---

Continue Software Factory toward the finish line in `docs/runtime/GOAL.md`, using this full source/design handoff. Preserve the approved interface and all unrelated uncommitted work. Do not restart the design or rebuild components that already exist.

Read `START-HERE.md`, root `AGENTS.md`, `apps/desktop/AGENTS.md`, `docs/runtime/GOAL.md`, `docs/runtime/IMPLEMENTATION.md`, `docs/handoff/BUILD-PLAN.md`, and `docs/handoff/DESIGN-INDEX.md`. Inspect the current code, tests, working tree, runtime evidence and any recorded integration trial before editing. The snapshot includes untracked files absent from the recorded Git base commit. Use the checkpoint to establish what has actually passed and the next unfinished acceptance gate.

The current implementation uses one Rust core crate with focused modules, SQLite transactions, an independent authenticated loopback Axum daemon, Codex app-server, managed linked Git worktrees, and a thin Tauri shell over the existing React frontend. Native live mode reads daemon records; browser-only Vite remains a separate design preview. Keep those data and evidence boundaries explicit.

The local-alpha outcome is **one repository, one persistent objective, one Codex lead conversation, one bounded worker, and a real local Review packet**. Continue the unfinished work needed to prove that outcome through the native app:

- Preserve Request as intent, Task as justified planned work, and Run as one stable execution attempt. Capture immutable requested/effective settings, context, permissions, provider IDs, worktree identity and tested revision.
- Keep state and events transactional and commands idempotent. Reconcile uncertain starts and stops before successors; preserve recovery-required records instead of blindly retrying external execution.
- Verify installed provider protocol/capabilities before changing the adapter. Use deterministic fake-provider tests first for failure, approval, interruption, concurrent claims and recovery. A model list or provider's claimed checks is not execution evidence.
- Enforce project edit/command/network/integration ceilings and narrower next-send choices. Unsupported operations remain visibly unavailable. In this alpha, execution networking/integrations are disabled and Commands Ask is a durable attempt-scoped grant for the worker and its independent checks. Worktrees and the legacy sandbox do not isolate host reads.
- Keep the lead bounded: new steering, eligible requests and completed work produce one justified action or a quiet wait. Preserve deadline, attempt, review-queue and observed-usage limits. Never describe an observed threshold as a hard prepaid spend cap.
- Have the supervisor checkpoint intended work safely and independently run checks on the exact revision. Reviews must show the actual diff, command/exit-code evidence and limitations, and reject stale source. Local acceptance is not merge or delivery.
- Prove settings, steering, one-off submission, actual history, pause, interruption, client closure/reopen and daemon restart through the real native client. Inspect changed UI at desktop and phone widths.

The goal authorized **one bounded actual Codex fixture trial using existing supported authentication**, after fake-provider checks, with explicit configured limits and one worker. Inspect the checkpoint and recorded trial first: do not repeat a completed or uncertain trial merely to refresh evidence. If its authorized trial remains unstarted, complete the independent gates and use a disposable local repository. Preserve exact IDs, SHA, actual check results, usage when observed and sanitized evidence. Do not publish, push, create/merge a PR, deploy, grant broader system access or install recurring jobs. Never copy credentials into prompts, source, browser assets, logs or reports.

Preserve these product decisions:

- One human-facing lead conversation; workers report through it.
- Compact warm-paper workbench: 168px rail, 40px header, restrained 14/13px typography and clear hierarchy.
- Bottom composer controls with borderless timing, one receipt per send, quiet routine activity, visible approvals and preserved reading position.
- Blank autosaved Scratchpad, with its configurable morning plan in Settings. Real recurring triage remains deferred.
- Graph, Board and original Sundial Radar for active work. Changes, diffs and reviews stay outside Radar.
- Compact Reviews inbox opening a large full-width Preview/Changes/Evidence desk, with feedback bound to exact source.
- Explicitly unavailable attachments/file editing, TUI, additional providers, hosted continuity, GitHub/deployments and recurring triage until implemented.

Use the documented commands and focused meaningful checks. Reuse the running preview where applicable; do not stop another project's server. Validate actual persistence, permission and lifecycle risks instead of mirroring implementation details in tests.

Update `docs/runtime/IMPLEMENTATION.md` at meaningful stopping boundaries with actual commands/results, real versus simulated behavior, blockers and the next exact action. Finish by launching the implemented app and providing exact startup/usage instructions, connected capabilities, evidence and remaining limitations. If an external input blocks progress, complete independent work and name the concrete missing input. Do not substitute fabricated success or a passing build for the real workflow.

---

This prompt does not start an agent itself. Give it and the complete source handoff to the agent that will continue the project. Use the canonical [goal](../runtime/GOAL.md) when creating or continuing the local-alpha goal; do not silently expand it into the later roadmap.
