# Software Factory — entry point

Continue this project and its approved interface. The repository now includes the local-alpha runtime: a Rust/SQLite core, independent authenticated Axum daemon, Codex app-server adapter, managed worktrees, and Tauri desktop client. The ordinary browser app remains a separate design preview.

- **Use the native alpha:** follow [getting started](docs/getting-started.md) for the tested build, service launch, registration and first request.
- **Connect a provider repository:** follow [GitHub/GitLab setup](docs/integrations.md) for existing CLI authentication, read-only pull/merge requests and CI.
- **Continue implementation:** read the [active goal](docs/runtime/GOAL.md), [checkpoint](docs/runtime/IMPLEMENTATION.md), [copyable agent prompt](docs/handoff/AGENT-PROMPT.md), and [build plan](docs/handoff/BUILD-PLAN.md).
- **Inspect the design:** use the [design index](docs/handoff/DESIGN-INDEX.md) for current desktop/phone screens, all mocks and superseded directions.

The local alpha is committed and pushed on `main` at `8e3fbd7`; a current clone includes its runtime and design source. The original frozen handoff's recorded base predates that implementation and remains historical evidence. Preserve any current working-tree changes, including the subsequent repository-integration increment, and use the checkpoint for its verification status.

## Connected and deferred

| Native live mode | Current boundary |
| --- | --- |
| One registered Git repository, persistent objective and revisioned steering | One project per data directory; local awake host |
| Observed Codex model/effort choices, immutable request/attempt settings | Existing supported Codex authentication required |
| One-off requests, bounded lead planning and one worker | No unbounded autonomous task generation |
| Isolated linked worktree, sanitized supervisor checkpoint, independent checks | Worktrees separate edits; they do not isolate host reads |
| Scoped command approval, pause, interruption and recovery fencing | Network/integrations disabled for alpha execution; uncertain control remains explicit |
| Real Runs/Progress and exact-revision local Reviews | Local acceptance is not merge or delivery |
| Read-only GitHub.com/GitLab.com connection and repository Reviews | Existing provider CLI authentication; manual reads; remote CI does not certify local Reviews |
| Blank Scratchpad and saved configurable 09:00 review plan | Client-local document/plan; no recurring triage or installed cron |

Files/context editing, attachments, additional providers, TUI, repository writes, deployments and hosted continuity remain unavailable or later milestones. Native live mode states these gaps explicitly. Browser preview interactions remain simulations and cannot supply runtime evidence. `apps/desktop/worker/index.js` serves static assets; it is not an agent executor.

## Read in this order

1. [Root instructions](AGENTS.md), then [desktop instructions](apps/desktop/AGENTS.md). Preserve the compact warm-paper design, one lead conversation, bottom composer controls, blank Scratchpad, active-only Sundial and large separate Reviews desk.
2. [GOAL](docs/runtime/GOAL.md), [implementation checkpoint](docs/runtime/IMPLEMENTATION.md), and runtime evidence for actual acceptance status. The goal's original description of an unconnected preview records the starting point, not the current source inventory.
3. [Getting started](docs/getting-started.md) and [native bridge notes](apps/desktop/src-tauri/README.md) for commands and operating boundaries.
4. [Build plan](docs/handoff/BUILD-PLAN.md), [design index](docs/handoff/DESIGN-INDEX.md), and the source/tests for the area being changed.
5. Current [design contract](docs/design/compact-preview/README.md), [runs](docs/design/compact-preview/interaction-contract.md), [one-offs](docs/design/compact-preview/oneoff-interaction-contract.md), [Scratchpad/permissions](docs/design/compact-preview/scratchpad-permissions-contract.md), and [Reviews](docs/design/compact-preview/reviews-interaction-contract.md).
6. [Architecture](docs/architecture.md), [harness](docs/harness.md), [providers](docs/providers.md), and [full acceptance](docs/acceptance.md) for the broader intended product. Proposed later features are not completed merely because they appear there.

The [original objective and five visual references](docs/handoff/source/README.md), [historical prompts](docs/handoff/archive/README.md), and [handoff contents](docs/handoff/README.md) retain the design history. They do not supersede current instructions or prove real execution.

## Verify the current source

Run these from the repository root with the documented toolchain on `PATH`:

```sh
cargo fmt --all --check
cargo test
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
```

Then, in `apps/desktop`:

```sh
pnpm build
node --test tests/*.test.mjs
```

These deterministic checks do not start a paid inference trial. The installed-provider sandbox smoke test is ignored by the default Rust suite and has separate evidence. The checkpoint records executed checks, native UI observations and the outstanding finish-line work; do not infer integration success from a passing build.
