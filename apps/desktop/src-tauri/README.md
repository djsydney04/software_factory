# Native desktop shell

This Tauri application wraps the approved React interface and connects its live
mode to the independent `factoryd` service. Closing the client does not stop the
daemon or its worker on an awake host. The bridge starts neither the daemon nor
Vite and installs no recurring job.

For the full operator procedure, use [getting started](../../../docs/getting-started.md).
The [runtime checkpoint](../../../docs/runtime/IMPLEMENTATION.md) records actual
native and provider verification. The ordinary browser Vite page remains a
separate fixture-based design preview; native mode never falls back to it.

## Build and launch

Use the tested macOS prerequisites documented in getting started: Cargo/Rust,
Tauri CLI, Xcode, Node and pnpm on `PATH`, plus an already authenticated supported
Codex installation. From the repository root, keep this running in its own
terminal:

```sh
cargo run --bin factoryd -- --provider-executable "$HOME/.local/bin/codex"
```

In `apps/desktop`, after `pnpm install --frozen-lockfile`:

```sh
cargo tauri build --debug --bundles app
"./src-tauri/target/debug/bundle/macos/Software Factory.app/Contents/MacOS/software-factory-desktop"
```

The explicit bundle command builds the frontend and creates an unsigned local
debug `.app`. Packaged assets do not need Vite. For frontend development instead,
run `pnpm dev --host 127.0.0.1` and `cargo tauri dev --no-watch` in separate
terminals at `apps/desktop`; the configured development URL is exactly
`http://127.0.0.1:5173`. Both paths still need the independent daemon.

The macOS default data directory is
`~/Library/Application Support/SoftwareFactory`. For a custom directory, set the
same `FACTORY_DATA_DIR` on the daemon and desktop executable, or pair daemon
`--data-dir` with matching client `FACTORY_DATA_DIR`. The native client reads the
connection on each invocation, so a daemon restart can rotate its credential
without sending it through JavaScript. Keep this private directory and
`connection.json` out of handoff archives.

## Native bridge

The trusted main window uses the public Tauri global API. This example pauses a
registered project; it includes the project identity required by the service:

```js
const { invoke } = window.__TAURI__.core;
const snapshot = await invoke("factory_snapshot");
const project = snapshot.projects[0];
if (!project) throw new Error("Connect a project before changing its state.");

await invoke("factory_capabilities");
await invoke("factory_events", { after: 0, limit: 100 });
await invoke("factory_command", {
  command: {
    commandId: crypto.randomUUID(),
    command: {
      type: "set_paused",
      projectId: project.id,
      paused: true,
    },
  },
});
```

The service validates the command envelope. Retain a command ID when reconciling
an uncertain response; a new ID represents a new deliberate action. Live React
stores its pending command identity before transport and exposes **Retry saved
action**. Rejections and disconnected-service errors must not become successful
mutations or fall back to sample data.

The same authorized native bridge exposes `factory_repository_detect`,
`factory_repository_changes` and `factory_repository_change` for explicit
read-only project integration requests. Connect/disconnect use durable
`factory_command` envelopes. The daemon uses the existing provider CLI's
authentication; provider tokens never pass through JavaScript. See
[GitHub/GitLab setup](../../../docs/integrations.md) for the UI procedure,
prerequisites and CI limits. These reads do not change agent execution policy
or certify a local Review.

Rust reads the owner-only connection file through `client::default_data_dir()`
and uses the authenticated loopback client. Neither credentials nor arbitrary
destinations are passed through JavaScript. No shell, file-system or HTTP plugin
API is exposed. Event page size is bounded to 1–500.

Commands require the `main` window and its trusted URL. Bundled navigation uses
`tauri://localhost`; debug builds additionally accept the exact configured
`http://127.0.0.1:5173` origin. Remote/confusable navigation and new embedded
windows are blocked. Project HTML is not trusted application UI; external
project-preview opening is not implemented by this bridge. The separate
`factory_open_repository_url` command validates plain HTTPS links on
`github.com`/`gitlab.com` before opening the system browser on macOS; it never
embeds provider content. Other destinations, credentials, queries and fragments
are rejected. The config has an
explicit command capability and separate CSP for packaged assets and Vite's
debug refresh requirements.

## Runtime behavior and boundaries

The daemon owns durable project state, bounded lead planning/wakes, one worker,
scoped approvals, independent checks and local Review evidence. Network and
integrations are narrowed to Deny for alpha execution. Original ceilings and
actual authorization remain separately recorded. Local review acceptance does
not push, merge or mark work delivered.

Pause prevents future dispatch; interrupt saves a stop request and waits for an
observed outcome or an explicit unresolved state. Recovery-required records
fence new work and must not be erased to force a replacement. There is no general
recovery-resolution UI yet. Worktree/sandbox limits do not isolate host reads,
and output-token thresholds are observed rather than hard prepaid caps.
Scratchpad/its review plan remain client-local with no cron or recurring triage.

## Checks

From the repository root:

```sh
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --check
```

The focused native checks cover trusted-window and provider-link validation. They do not alone
prove the rendered end-to-end workflow; use the checkpoint and recorded native
evidence for that result. The default Rust tests and frontend checks do not
repeat the goal's single paid Codex trial.
