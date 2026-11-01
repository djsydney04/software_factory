# Start here

This repository preserves the Software Factory architecture, provider research and visual design work. It does not yet contain application source code, a build command or a running factory. The latest visual direction is proposed, not approved.

## Open the project

```sh
git clone https://github.com/djsydney04/software_factory.git
cd software_factory
```

Read these in order:

1. [Current designs](design/charcoal-desk/README.md): Conversation, Progress and Reviews, with the Sundial radar.
2. [Architecture](architecture.md): one Rust crate, durable SQLite state and a shared daemon for both clients.
3. [Provider integration](providers.md): Codex app-server first, including the limits of the completed probes.
4. [Acceptance criteria](acceptance.md): the observable behavior required before calling this a working alpha.

Earlier screenshots remain in the design folder for history. Use the `charcoal-desk` set as the current proposal; the previous screens that embed reviews in monitoring are superseded.

## First implementation slice

Start with durable local state, which does not depend on selecting the final visual design:

- Create one Rust crate with `model` and `store` modules.
- Implement factory creation and reading with a repository path, objective and context revision.
- Add persisted steering; update its revision and append the matching event in the same transaction.
- Prove that closing and reopening the database preserves the objective, steering, revision and ordered events. Prove that a failed write leaves neither a partial state update nor a stray event.

Keep this first slice limited to storage. Do not label stored sample records as running agents. The architecture's larger type inventory is a proposed contract, not a requirement to scaffold every module immediately.

## Next milestones

| Milestone | Evidence required |
| --- | --- |
| Local daemon | Authenticated loopback API exposes durable state; unauthenticated access fails. |
| One real worker | Codex changes a disposable repository in an isolated worktree; the supervisor captures the terminal outcome and checks the exact commit. |
| Recovery and control | Restart, pause, queued steering and cancellation preserve state without duplicate execution. |
| Clients | TUI and desktop share Conversation / Progress / Reviews and display the same daemon records. |
| Review | A selected change shows its diff, observed checks and limitations; stale evidence is explicit. |

The full [acceptance criteria](acceptance.md) remain authoritative. Saving a goal, completing a provider handshake, or rendering mockups does not satisfy the working-alpha milestone.

## Existing evidence

The provider notes record metadata probes, including paused-goal persistence, and Claude's design session through Cursor. These establish only the capabilities explicitly observed there. There has been no end-to-end worker run, crash-recovery test, daemon, desktop build or release.
