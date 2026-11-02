# Local alpha verification

The native application completed one bounded **real Codex** integration trial on November 1, 2026 (November 2 UTC). The result is a local, reviewed fixture change. Nothing was pushed, merged, published or deployed. The broader factory roadmap remains unfinished.

## Actual provider trial

The installed Codex CLI was `0.159.3`, using its existing supported authentication. Settings were `gpt-6.1-sol`, low effort, worktree edits, Commands Ask, Network Deny and Integrations Deny. Each turn had a 120-second deadline and an observed 4,000-output-token stop threshold; one worker attempt and one open review were allowed. That threshold is checked when usage arrives, not a prepaid token or cost cap.

Through the bundled native app, we registered a disposable repository and persistent objective, saved permissions/model/limits, sent steering, and submitted one one-off while paused. Resuming dispatch produced one justified Task. The worker waited for a durable, attempt-scoped command approval before starting. The lead planned the work, the worker changed the file, and a bounded read-only lead wake summarized the verified result. No additional work was generated.

| Record | Observed identity |
| --- | --- |
| Project | `da851f34-1c8e-4fe7-82a4-2d2200beec28` |
| Request | `9fafec80-97e2-49ee-8180-4427d128dbec` |
| Worker Run | `59bbbce4-a506-4b29-b4cf-4589446eac43` |
| Review | `d148b1e7-8bb0-44cc-818d-9ff945a8568f` |
| Base revision | `65b9475bab006db94901ce21445cbaefa109f2e2` |
| Tested revision | `39576124ac6bd9d81ac0b1cfafb62c8d084c704a` |

The only changed file was `value.txt`, from `pending\n` to `ready\n`. The supervisor created a detached checkpoint commit, independently invoked `/usr/bin/python3 check.py` from the worktree root with a five-second timeout through the provider's command sandbox, and observed exit **0** with `value.txt is ready`. macOS Python also emitted sandbox-denied temporary/cache diagnostics, retained in the check evidence; these did not change the observed exit code. The check source SHA-256 stayed `d39f3a427d4e57e5aa18cf83359b3ed7c013cd7e51b74d4fe66072e301a469e3`. The parent repository remained at `pending\n`; the attempt worktree was clean at the exact tested revision.

The native Review desk showed the real diff, pinned revision, current/clean source, actual check output and explicit limitations. Local acceptance was recorded against that revision. This neither merges the change nor marks it delivered. The project was paused, both client and daemon were stopped and restarted, and the native app reopened the same accepted Review. Request, Task, Run, Review and lead-wake JSON hashes were unchanged; event sequence stayed 38 and counts stayed one each. No duplicate inference or execution was launched.

Sanitized evidence:

- [Actual result, IDs, authorization and check output](evidence/real-integration-result.json)
- [Restart and unchanged-record comparison](evidence/real-restart-result.json)
- [Native real diff](evidence/native-real-review-changes.png)
- [Native worker history](evidence/native-real-run-history.png)
- [Accepted Review after restart](evidence/native-real-review-restarted.png)
- [Source hashes for the implementation exercised](evidence/trial-source-manifest.json)

The fixture remains in `/Users/dylanmitic/Documents/Codex/2026-10-31/go/work/real-alpha-trial-26518c52`; its private daemon state is excluded from the portable handoff. Do not copy its `connection.json`, SQLite database or provider account/session state into source or an archive.

## Native control and responsive checks

Before actual inference, an external-process deterministic provider substitute exercised the complete native flow. It created real isolated Git worktrees and real verification exits; its model responses were fixtures. A slow second attempt remained alive after closing the desktop client and reopened with the same Run, process, thread and turn IDs. The native Interrupt action reached a matching terminal outcome, and the process was absent afterward. Pausing and restarting the daemon preserved both attempts and local review acceptance. These control proofs used the substitute, not a second paid Codex trial.

- [Client alive after closing desktop](evidence/client-independence-after-close.json)
- [Observed interrupt outcome](evidence/native-control-result.json)
- [Fake-provider daemon restart](evidence/daemon-restart-result.json)
- [Desktop review](evidence/native-fake-review-desktop.png), [394px review](evidence/native-fake-review-phone.png), [final 394px composer](evidence/native-fake-oneoff-phone-final.png)
- [Installed-provider OS sandbox smoke](evidence/sandbox-smoke-result.json): no inference; inside-worktree write allowed, sibling write and owned loopback request denied.

The phone inspection found a full-SHA overflow in the conversation review card. The final card abbreviates that hash while keeping the full revision in Review Evidence. The final phone capture has no horizontal overflow and retains bottom controls with borderless timing. The earlier `native-fake-conversation-phone.png` is a before-fix record.

## Commands and checks

From the repository root, `cargo test --locked` passed **67** Rust integration tests: 14 lead, 13 provider, 13 supervisor, 4 service, 13 store and 10 worktree. One installed-provider smoke test is ignored by default and was run separately without inference. `cargo fmt --all --check` passed for both Rust crates.

From `apps/desktop`, `pnpm build` passed and `node --test --test-reporter=dot tests/*.test.mjs` passed **55** tests. From `apps/desktop/src-tauri`, `cargo test` passed the native bridge security test. From `apps/desktop`, `cargo tauri build --debug --bundles app` produced and launched the unsigned local `.app` used above. All frontend checks preserve the existing Sites build contract.

Actual daemon startup for the trial, from the repository root:

```sh
cargo run --bin factoryd -- \
  --data-dir /Users/dylanmitic/Documents/Codex/2026-10-31/go/work/real-alpha-trial-26518c52/state \
  --provider-executable /Users/dylanmitic/.local/bin/codex
```

Actual native startup used the same data directory:

```sh
FACTORY_DATA_DIR=/Users/dylanmitic/Documents/Codex/2026-10-31/go/work/real-alpha-trial-26518c52/state \
  '/Users/dylanmitic/workspace/software-factory-prototype/apps/desktop/src-tauri/target/debug/bundle/macos/Software Factory.app/Contents/MacOS/software-factory-desktop'
```

The trial is paused and its daemon is stopped. The launched app now uses empty default user state, ready for a real repository; see [the observed empty state](evidence/user-ready-result.json) and [native screen](evidence/native-ready-for-project.png). For a fresh project, use the default-state startup in [START-HERE](../../START-HERE.md). Metadata discovery does not start inference; resuming eligible intent or sending steering on an unpaused configured project can start bounded model work.

## Limits

This is an unsigned local alpha with one project per data directory, one Codex lead and one worker. It requires an awake host and a separately running daemon. Unsupported restrictions block execution; unresolved process control fences later dispatch. The legacy sandbox constrains writes/network and does not isolate host reads or credentials. Provider-reported usage is observed accounting; price/cost is unknown. Scratchpad is an autosaved client document with a saved morning plan, not a running recurring triage service. Rendered artifact capture, file editing/attachments, TUI, other providers, GitHub merge, deployment and hosted continuity remain later milestones.
