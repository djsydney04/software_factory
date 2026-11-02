# Handoff verification

The current snapshot contains the implemented local alpha and the complete design history. [INTEGRATION-TRIAL.md](../runtime/INTEGRATION-TRIAL.md) owns runtime/native execution evidence and its limits. The recorded Git base does not contain the full uncommitted source; use the full archive.

## Current verification

- Rust core: `cargo test --locked` passed 67 tests; one installed-provider smoke is ignored by default and separately passed without inference.
- Frontend: `pnpm build` and all 55 Node tests passed.
- Native bridge security test, root/native formatting checks and unsigned debug app bundle passed.
- Native desktop and 394px Review/composer inspection passed, with the SHA overflow fixed and final capture retained.
- One bounded actual Codex fixture produced a one-file diff, supervisor check exit 0 on `39576124ac6bd9d81ac0b1cfafb62c8d084c704a`, and an accepted local Review. Client/daemon restart retained identical records without another execution.
- Deterministic-provider native controls separately proved client-independent active execution, observed interruption, pause and reconnect.

No push, PR, merge, deploy or recurring-job installation occurred. No second actual inference trial was run for packaging. See [the runtime record](../runtime/INTEGRATION-TRIAL.md) for precise commands, IDs, evidence and capability limits.

## Package checks

The current [manifest](manifest.json) inventories source, tests, Cargo/frontend lockfiles, public assets, all design materials and sanitized runtime evidence. Packaging excludes Git history, every `target`/`node_modules`/`dist`/generated cache directory, environment files, private daemon databases/connection files, Codex accounts/sessions and raw model streams. ZIP integrity and each included file's size and SHA-256 are checked against the manifest. The manifest excludes itself from its own inventory.

The initial design-only handoff is preserved as [the earlier verification](archive/design-handoff-verification.md) and [its frozen manifest](archive/design-handoff-manifest.json). Its clean source-copy install/build and 45-test results describe the earlier preview baseline, not the current runtime. The earlier archive is retained separately in the task outputs.

Follow [START-HERE](../../START-HERE.md) and [getting started](../getting-started.md) to build and use the current source. The native app is an unsigned local alpha requiring a separately running daemon and an awake host. Broader hosted/provider/TUI/GitHub/triage features remain later milestones.
