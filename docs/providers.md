# Provider integration

Status: Codex app-server adapter implemented and exercised in the native local alpha. Deterministic transport/control tests preceded one bounded real Codex fixture trial using `gpt-6.1-sol` at low effort: the supervisor observed `/usr/bin/python3 check.py` exit 0 at revision `39576124ac6bd9d81ac0b1cfafb62c8d084c704a`, and the local Review survived daemon restart. See the [runtime checkpoint](runtime/IMPLEMENTATION.md) and [trial record](runtime/INTEGRATION-TRIAL.md). The October 31 metadata probes and broader provider contracts below remain useful history; Claude, Cursor and Gemini factory adapters are not implemented.

The alpha captures immutable requested/effective settings and separate execution authorization. It queues steering for the lead, performs independent checks with sandboxed `command/exec`, and disables native goal continuation, integrations and agent command networking. Installed legacy controls restrict writes/network, not host reads. Output usage is an observed stop threshold, not a hard prepaid token/cost cap. Unsupported restrictions fail closed; uncertain turn or stop outcomes remain `recovery_required` and fence successors.

## Common boundary

Keep the supervisor independent of provider-specific transports. Each adapter receives a working directory, bounded task prompt, optional model/effort, optional saved session ID, permission policy and deadline. It emits normalized activity plus the original provider method/event for diagnostics.

Persist provider IDs immediately. Use the supervisor's terminal run states: `Succeeded`, `Failed`, `Interrupted` and `Cancelled`. `NeedsApproval` is nonterminal waiting with a durable scoped approval record. If waiting must end, observe the executor's stopped terminal outcome and retain the unresolved approval separately. A timeout or interrupt with uncertain control remains `RecoveryRequired`, rather than claiming a stopped executor. A process exiting or a stream ending does not establish successful completion. Verification remains an evidence gate before a task moves from `Running` to `Review`; the alpha commits verified success and the Review together.

Capabilities must be explicit: resume, live steering, approval callbacks, structured final output, usage reporting and sandboxing. If live steering is unavailable, save input for the next bounded turn and show that state. Do not silently claim equivalent behavior.

Invoke executables with argument arrays and write prompts over stdin or protocol messages. Do not concatenate task text into a shell command. Use each vendor's documented login or API-key flow; keep credentials out of event payloads and application records.

## Codex: app-server first

Start `codex app-server --listen stdio://`. The adapter owns the child process. Exchange newline-delimited JSON-RPC messages over stdin/stdout; drain stderr separately with bounded retention.

The lifecycle is:

1. Send `initialize` with stable client name, title and version. Await its response, then send `initialized`.
2. Call `thread/start`, or `thread/resume` with the saved `threadId`. Persist the returned ID.
3. Call `turn/start`. Persist its `turnId`; stream item, plan, tool and usage notifications.
4. Steer using `turn/steer` and `expectedTurnId`. Interrupt with `turn/interrupt`.
5. Treat `turn/completed` as terminal only after inspecting `turn.status`. Completed, failed and interrupted are different outcomes.

Server-initiated approval requests require a decision; never leave the reader blocked indefinitely. `thread/goal/set`, `get` and `clear` provide persisted per-thread goals. Keep the larger factory goal in factory state. Initially run bounded explicit task turns; enable active native goals only after their continuation and stop behavior is tested. Generate or inspect schemas for the installed version before depending on optional fields. [Official Codex app-server documentation](https://learn.chatgpt.com/docs/app-server)

Initial messages:

```json
{"id":1,"method":"initialize","params":{"clientInfo":{"name":"software_factory","title":"Software Factory","version":"0.1.0"}}}
{"method":"initialized","params":{}}
{"id":2,"method":"thread/start","params":{"cwd":"/absolute/worktree","sandbox":"workspace-write","approvalPolicy":"never"}}
```

The first two messages must be sequenced around the successful initialization response. Model and effort selection must follow the installed schema and current account access; a listed model alone does not verify an inference entitlement.

For simpler finite jobs, `codex exec --json` emits JSONL activity, `--output-schema` requests a schema-shaped final answer, and `codex exec resume <SESSION_ID>` continues an existing session. Saved CLI authentication is reused by default. Use exec when interactive control is unnecessary; app-server provides the richer lifecycle the factory needs. [Official non-interactive documentation](https://learn.chatgpt.com/docs/non-interactive-mode)

October 31 metadata probes: `codex-cli 0.159.3`; handshake succeeded; `model/list` returned eight entries; `thread/goal/get` recognized the method and returned the expected missing-thread error. Those probes performed no inference. The CLI exposes workspace-write sandboxing and schema generation. Its generated stable schema confirms `thread/start` and `thread/resume` accept `sandbox` and `approvalPolicy`; `turn/start` accepts `effort`; `turn/steer` requires `expectedTurnId`. Text input's `text_elements` field is optional. Runtime permissions and version checks remain necessary when invoking an actual task.

## Native goals: verified persistence and remaining uncertainty

The generated local schema defines `ThreadGoalStatus` as `active`, `paused`, `blocked`, `usageLimited`, `budgetLimited` or `complete`. A goal stores objective, timestamps, tokens used, time used and an optional token budget. `ThreadGoalSetParams` requires only `threadId`; status, objective and budget are optional. These are wire shapes, not proof of scheduling behavior.

A metadata-only probe on October 31 created an isolated read-only thread, set a **paused** goal with a one-token budget, read it, restarted app-server, read it before resuming, resumed the thread and read it again. Goal objects were identical across all three reads. The exact observed result was:

```json
{
  "codex_version": "0.159.3",
  "goal_status_before": "paused",
  "goal_status_after_process_restart": "paused",
  "goal_status_after_thread_resume": "paused",
  "tokenBudget": 1,
  "tokensUsed": 0,
  "timeUsedSeconds": 0,
  "createdAt": 1793501172,
  "updatedAt": 1793501172,
  "goal_objects_identical": true,
  "resumed_thread_status": "idle",
  "thread_turns_list_count": 0,
  "turn_or_item_notifications": [],
  "active_goal_tested": false
}
```

This establishes paused-state durability without a model turn in the observed lifecycle. It does **not** establish that setting an active goal, or resuming one, cannot start generation. The probe used a child-only `CODEX_HOME` override for scratch storage; that conflicted with this session's environment-variable instruction and is not a recommended reproduction procedure. No existing user threads were edited. Further probes need a separately supported isolation mechanism.

Official guidance describes goals as thread-scoped completion contracts. Continuation occurs at safe idle boundaries; pending work/input, plan-only work, interruption, budget limits and a continuation without tool use can prevent another turn. The app-server reference documents goal state methods, but does not establish a guarantee that a custom client receives or owns every autonomous continuation. [Official guide to Goals](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex)

Design inference: native goals can help complete one bounded task; they cannot replace factory-wide prioritization, cross-provider task assignment, review queues, recurring deployment observations or global budget enforcement. Keep one factory supervisor in charge of those decisions. Do not run both an assumed native continuation loop and an independent task continuation loop until ownership is demonstrated. The first adapter should execute explicit bounded turns and retain goal persistence as a capability rather than silently enabling autonomy.

## Minimal Rust adapter design

The implemented adapter lives in `src/provider.rs`; no generic plugin framework is required. A concrete `CodexSession` owns the child process, stdin/stdout readers, pending RPC IDs and bounded framing; stderr is drained and discarded. The sketch below preserves the original sequencing idea rather than the exact current signature.

Use two steps so the supervisor can persist the provider thread before inference:

```rust
let mut session = CodexSession::connect(config).await?;
store.save_provider_thread(run_id, session.thread_id())?;
let outcome = session.run(prompt, controls, on_event).await?;
```

Current `CodexOptions` carries the executable, working directory, settings, optional existing thread ID and planner mode; `run` additionally receives an output schema, controls and a fallible event sink. Controls support interruption, while product steering queues for the lead. The sink persists a returned turn ID before acknowledging it to clients. Do not retry a turn start after losing its response without reconciling the saved thread.

The adapter initializes once, starts/resumes the thread, then concurrently reads protocol messages, controls and its deadline. Match responses by RPC ID, since notifications can arrive between responses. Pass through unknown notification methods as diagnostics. A malformed required response, unexpected EOF or missing terminal turn never establishes success. After inference may have begun, the alpha preserves uncertain control as `RecoveryRequired` rather than pretending the executor stopped.

Steering acknowledgments identify the accepted input; the supervisor retains its own applied/evidence state. Stop using native `turn/interrupt`, allow a short grace period, then terminate and reap the owned process group if necessary. Preserve the supervisor's stop reason so cancellation, timeout and provider interruption remain distinct.

Choose demonstrated sandbox/approval controls from the project's effective permission ceilings; workspace writes require authorized worktree access. If a request needs a person, persist its scope and enter nonterminal `NeedsApproval` while the adapter retains ownership. If interactive resolution is unavailable or a waiting deadline expires, interrupt and observe a stopped terminal outcome, retaining the unresolved approval/blocker separately. Never grant extra permissions or leave the protocol reader blocked indefinitely. Add interactive approval responses only when the client can present and durably record them.

Provider outcome and supervisor outcome remain separate: raw Codex `completed` maps to `Succeeded`, `failed` to `Failed`, and `interrupted` to `Interrupted` unless a recorded cancellation/timeout overrides it. Unresolved approval remains a separate blocker, not a terminal success or an excuse to duplicate execution. The supervisor then performs task verification.

## Claude

Headless entry point: `claude -p` with `--output-format stream-json --verbose`; add `--include-partial-messages` when token deltas are useful. Persist the result's session ID for resume. The SDK is preferable when native input/approval callbacks become necessary. Interrupt cleanly before stopping a process; SIGTERM can leave its turn unfinished. [Headless operation](https://code.claude.com/docs/en/headless)

The CLI documents `--max-turns` and `--max-budget-usd`; the spend cap is an estimate and can be exceeded. A factory-wide allowance must still be enforced across attempts and resumed sessions. [CLI reference](https://code.claude.com/docs/en/cli-reference)

Use Claude Code's documented account login or API configuration. Installation and authenticated inference were not tested here. [Authentication](https://code.claude.com/docs/en/authentication)

## Gemini

Headless entry point: `gemini -p <prompt> --output-format stream-json`. Events include initialization, messages, tool calls/results, errors and a final result with statistics. Preserve the session ID and distinguish a nonzero exit from a final successful result. [Headless reference](https://geminicli.com/docs/cli/headless/)

The CLI supports `--resume` with a session ID. [Configuration reference](https://geminicli.com/docs/reference/configuration/)

Headless runs can reuse cached supported credentials. Without them, configure a supported API-key or Vertex AI authentication path. Installation and authenticated inference were not tested here. [Authentication](https://geminicli.com/docs/get-started/authentication/)

## Cursor

Headless entry point: `agent -p --output-format stream-json`. Select the workspace and sandbox explicitly. Documented headless file modification uses `--force`; that flag grants broad automatic action and should be tied to the factory's chosen agency/environment, never added merely to suppress an unexpected prompt. [Headless CLI](https://cursor.com/docs/cli/headless)

Supported account login and API-key authentication are available. [Authentication](https://cursor.com/docs/cli/reference/authentication)

Local CLI version `2026.10.22-27dcf5f` supports `--print`, `--output-format stream-json`, `--resume`, `--model`, `--worktree` and `--sandbox enabled`. Its help reports a default text output format; older documentation describes a different default, so the adapter must always set the format explicitly. The initial October 31 probes did not perform authenticated Cursor inference.

Update, November 1: at the user's request, the signed-in Cursor CLI completed a real read-only design task with explicit model `claude-opus-5-5-high`; initialization identified the effective model as **Claude Opus 5.5 300K High**. Claude successfully read the current mockups and references, returned a design brief, and completed a Radar refinement after resuming the same session. Both calls emitted a successful terminal result and exited zero. The invocation used plan mode and requested sandboxing; no product code was written, no MCP approvals were granted, and no external integration actions were requested. This verifies that specific account-backed Claude-through-Cursor design workflow, not a completed factory adapter, native Claude CLI, or unattended coding/recovery behavior. [Design provenance](design/claude-redesign/provenance.json)

## Adapter acceptance

Test deterministic stream parsing before paid inference: partial lines, unknown fields, out-of-order notifications, server requests, abrupt EOF and a terminal failure. Test stderr flooding and bounded buffering. A terminal failure must never normalize to success.

Then run one small real task per available authenticated provider. Verify IDs survive restart, output is streamed, cancellation stops descendants, steering is accurately acknowledged, and timeouts return a clear terminal outcome. Provider availability and login status belong in the interface; unsupported providers must never appear as successful simulated runs.
