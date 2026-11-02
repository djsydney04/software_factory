# Use the local alpha

Software Factory has two entry points: the **native alpha** connects to the independent local daemon and actual Codex execution; the **browser preview** demonstrates the approved design with fixtures. Use native mode for real work. [START-HERE](../START-HERE.md) indexes the source, design and implementation handoff.

The [runtime checkpoint](runtime/IMPLEMENTATION.md) records executed verification and the goal's current acceptance status. A successful build, model list or fake-provider run is not proof of real inference.

## Prerequisites

The exercised macOS toolchain is Rust/Cargo 1.99.1, Tauri CLI 2.12.4, Xcode 26.6, Node 24.22.1, pnpm 11.25.0, and Codex CLI 0.159.3. Put Cargo, Node, pnpm and the Tauri CLI on `PATH`. The root has `Cargo.toml`; frontend commands run in `apps/desktop`. No `factory` CLI or TUI exists.

Use an already authenticated supported Codex installation. The tested executable is `$HOME/.local/bin/codex`; replace the provider path below if yours differs. Do not copy Codex credentials or session files into this repository. The daemon discovers account/model metadata without inference; sending eligible steering or a request can start model turns and consume account usage.

Choose a local Git repository with a committed base. The alpha rejects submodule/gitlink repositories. It preserves unrelated parent changes and creates managed linked worktrees for attempts. Local checkpoint commits remain separate from the parent branch and are never pushed automatically.

## Build and launch the native app

From the repository root, install the existing frontend dependencies:

```sh
cd apps/desktop
pnpm install --frozen-lockfile
```

In a terminal at the repository root, start the independent daemon:

```sh
cargo run --bin factoryd -- --provider-executable "$HOME/.local/bin/codex"
```

It prints an ephemeral `127.0.0.1` listening address. Keep this terminal running. The native client discovers the address through a private connection file; do not paste its credential into the browser or a prompt. A second daemon for the same data directory fails its ownership lock.

In another terminal at `apps/desktop`, build and launch the packaged local app:

```sh
cargo tauri build --debug --bundles app
"./src-tauri/target/debug/bundle/macos/Software Factory.app/Contents/MacOS/software-factory-desktop"
```

The build invokes `pnpm build` and bundles those frontend assets. This is an unsigned local debug application, not an installed or notarized release. The packaged app needs the daemon but does not need Vite. It does not start the daemon itself. Closing its window leaves the independent service running on an awake host.

On macOS the default state directory is `~/Library/Application Support/SoftwareFactory`. It holds the database, native connection credential and managed worktrees. One project is supported per data directory. For a separate fixture or project, set the **same** `FACTORY_DATA_DIR` environment variable on both the daemon and desktop executable, or use `--data-dir` on the daemon and matching `FACTORY_DATA_DIR` on the client. Reuse the same directory after restart. Never copy its private state into a source handoff.

For frontend development, start `pnpm dev --host 127.0.0.1` in `apps/desktop`, then run `cargo tauri dev --no-watch` there in another terminal. This uses the configured Vite URL `http://127.0.0.1:5173`; the daemon remains separate. Reuse an existing server for this checkout rather than stopping another project's server to obtain the port.

## Connect and send real work

1. In the native app's **Start a local project** form, enter a name, the absolute Git repository root, and a persistent objective; choose **Connect project**.
2. In **Settings → Models & execution**, select a model and effort from the daemon's observed catalog, then save defaults. Set the turn deadline, observed output-token threshold, maximum attempts and open-review cap before submitting work. No model is selected by default.
3. In **Settings → Permissions**, save the edit ceiling and Commands/Network/Integrations rules. Ask is the default. The alpha narrows command networking and integrations to Deny for actual execution, and records that restriction separately from the original snapshot. **Commands → Ask** produces an approval for one attempt and its independent checks, not a prompt before every command.
4. Send **Message** steering in Conversation. The daemon persists it before a bounded lead acknowledgment. To prepare intent without starting turns, use **Pause future dispatch** first; resume when ready. Pause does not interrupt an already active attempt.
5. Choose **One-off** at the bottom of the same composer and describe one small change with an executable check. Keep **Lead decides**, choose **Next available**, or set a future earliest start with its displayed IANA time zone. The timing control stays in the bottom action row.
6. Follow the request in **Runs → Schedule**. The lead either creates a justified Task or records why it is waiting. Only an actual execution claim creates a Run. A saved due time alone is not execution.
7. If an attempt needs command approval, inspect its worktree and checks, then use **Allow once** or deny it. Next-send permissions can narrow project ceilings; they cannot silently broaden them.
8. Inspect actual attempt state and history in Runs, Graph, Board or Sundial. When independent checks succeed on the recorded checkpoint, open its Review packet.

The goal and steering are captured as immutable context. New defaults and messages affect future planning/attempts and do not rewrite prior snapshots. The lead uses bounded turns for new steering, eligible requests and completed work, then waits quietly; this alpha does not continuously invent new work from an objective.

Turn deadlines are enforced by the supervisor; output-token thresholds are enforced when the provider reports usage. The installed protocol offers no hard pre-inference token/cost cap. Missing usage or cost stays unknown.

## Review and control

Reviews opens a compact inbox and then a full-width **Preview / Changes / Evidence** desk. Inspect the real diff, exact Run/attempt/SHA, supervisor check argv and exit codes, and stated limitations. Optional screenshot gaps remain visible. Changes in the source head, dirty state or latest attempt make old evidence ineligible for a current decision.

**Accept locally** and **Request changes** persist feedback on that revision. They do not push, merge, create a PR, mark work delivered, or automatically launch a successor. GitHub/GitLab and deployment writes are not connected.

Use the header's **Pause future dispatch** to stop new claims and lead work. Use a Run's **Interrupt** to request a stop. A saved interrupt request is not a stopped process: the app reports the observed terminal outcome or an explicit unresolved state. Cancelling pending scheduled intent prevents future dispatch and does not interrupt an existing attempt.

Close and reopen the client against the same daemon/data directory to recover the same records. To restart the daemon, stop its own terminal with Ctrl+C and run the same command again. Its private connection credential rotates; reconnect the client. Interrupted or uncertain external execution can remain **Recovery required / Needs reconciliation**, fencing further work rather than blindly launching a duplicate. There is no general recovery-resolution UI in this alpha. Preserve its database, provider IDs and worktree for inspection; do not delete state to force another attempt.

## Connect GitHub or GitLab

With the native project registered, open **Settings → Integrations** to detect its supported `origin` and connect using existing provider CLI authentication. **Reviews → Repository** then reads open pull/merge requests and selected-request CI on demand; **Reviews → Local** retains the supervisor's separate review packets. Follow [repository integration setup](integrations.md) for `gh`/`glab` prerequisites, authentication, supported hosts and CI limitations. Connecting does not grant the lead or worker integration permissions.

## Scratchpad and unavailable features

Scratchpad is a blank autosaved document in this native client's local storage. **Settings → Scratchpad** saves its configurable review plan, defaulting to daily 09:00 in the captured browser time zone. No recurring agent triage or cron is installed. Live Scratchpad does not fabricate completed passages from unrelated runs.

Native drafts, review feedback drafts and UI preferences are client-local; authoritative project settings, steering, requests, attempts, approvals and local decisions live in SQLite. Keep the window open and copy unfinished text if local draft storage reports an error.

Attachments, arbitrary file editing/custom context selection, TUI, additional providers, hosted continuity, repository/deployment writes and recurring Scratchpad triage remain later work. Worktree isolation and the current provider sandbox do not restrict host read roots or guarantee credential-read isolation. Work does not continue through laptop sleep.

## Open the browser design preview

In `apps/desktop`:

```sh
pnpm dev --host 127.0.0.1
```

Open the URL Vite prints, normally [http://127.0.0.1:5173](http://127.0.0.1:5173). This browser-only view uses fixtures and a separate localStorage namespace, needs no provider credentials, and cannot execute agents or enforce OS permissions. Preview triage, file edits, run activity and merge actions remain simulations. `worker/index.js` is a static Sites handler.

The [design index](handoff/DESIGN-INDEX.md), [current contracts](design/compact-preview/README.md) and [frontend QA record](../apps/desktop/design-qa.md) preserve mocks and design evidence. They do not replace runtime verification.

## Verify source and diagnose setup

From the repository root:

```sh
cargo fmt --all --check
cargo test
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --check
```

In `apps/desktop`, build before the full frontend suite because Sites tests inspect generated output:

```sh
pnpm build
node --test tests/*.test.mjs
```

The default Rust suite uses deterministic provider fixtures; its installed-Codex sandbox smoke is ignored and separately authorized. These commands do not repeat a paid integration trial. Preserve the protected Sites files and both existing frontend lockfiles.

| Symptom | Next check |
| --- | --- |
| Native app says local service unavailable | Confirm `factoryd` is running, then check daemon/client `FACTORY_DATA_DIR` match and reconnect. |
| No supported model or account metadata | Check the configured Codex executable and its existing supported authentication; do not put credentials in the UI. |
| Request waits | Read its reason and inspect pause, model selection, earliest start, approvals, review cap and recovery state. |
| Command response is uncertain | Use **Retry saved action**, which retains the command ID. Do not submit a duplicate request. |
| Native dev window cannot load | Confirm Vite serves this checkout at the exact configured `127.0.0.1:5173` URL. |
| Review is historical or source needs attention | Inspect its recorded SHA against the preserved worktree; old green checks cannot certify changed source. |

For the active acceptance scope and next exact implementation action, use [GOAL](runtime/GOAL.md), [the checkpoint](runtime/IMPLEMENTATION.md), and [the build plan](handoff/BUILD-PLAN.md).

Additional [Soft Charcoal](design/soft-charcoal/README.md), [Charcoal Desk](design/charcoal-desk/README.md) and [Operating Desk](design/operating-desk/README.md) mockups are preserved for comparison. The native application described here follows the compact-preview contracts.
