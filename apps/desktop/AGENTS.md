# Desktop client instructions

This frontend has two explicit modes. Browser-only Vite is the design preview with fixtures and local simulations. The Tauri native client uses the independent Rust/SQLite `factoryd` service through typed native commands and never falls back to fixtures. Read the root [agent instructions](../../AGENTS.md), [runtime checkpoint](../../docs/runtime/IMPLEMENTATION.md) and [integration trial](../../docs/runtime/INTEGRATION-TRIAL.md) before changing connected behavior. Preserve the approved visual direction in both modes.

The native local alpha supports repository/objective registration, observed Codex model/effort selection, saved project ceilings, steering, one-off requests, bounded execution, scoped approvals, history, pause/interruption and source-bound local Reviews. `Request`, `Task` and `Run` remain distinct. Uncertain provider/control outcomes stay visibly `recovery_required`; never relabel them stopped or fabricate replacement execution. Local review acceptance does not merge or mark delivery Done. Files, attachments, rendered run previews, other providers and recurring Scratchpad triage remain unavailable rather than simulated in live mode. Scratchpad itself remains a blank local autosaved document with a configurable saved morning plan.

Project permissions must be enforced by the runtime, not only UI selections. Keep immutable requested/effective snapshots and separate attempt-scoped execution authorization. The alpha narrows agent command networking and integrations to Deny; legacy provider controls do not isolate host reads. The output-token limit is an observed threshold, not a prepaid cap. Keep daemon credentials native-only and out of frontend storage, assets and event text.

## Design preview workflow

Run the local server yourself and open the preview in the browser available to this environment. Do not give the user server-start instructions when you can run it.

Before making substantial visual changes, use the Product Design plugin's `get-context` skill when the visual source is unclear or no longer matches the current goal. When the user gives durable prototype-specific design feedback, preferences, or decisions, record them in `AGENTS.md`.

When implementing from a selected generated mock, treat that image as the source of truth for layout, component anatomy, density, spacing, color, typography, visible content, and hierarchy.

Build app UI in `src/`. Keep `.openai/hosting.json`, `worker/index.js`, `scripts/prepare-sites-build.mjs`, and `tests/sites-worker.test.mjs` intact so the same local prototype can be handed to Sites. Before a Sites handoff, run `npm run build` and `npm run test:sites`; the build must leave `dist/client/index.html`, `dist/server/index.js`, and `dist/.openai/hosting.json`.

## User direction — November 1, 2026
Compact warm-paper workbench; one lead conversation. Clear top-level hierarchy, 168px sidebar, 40px header, 14px body / 13px metadata. More Run views, scoped model picker, project configuration, file/context settings. Reviews is a separate large workspace; no changes/diffs/PRs in Radar. Radar uses the original Sundial concept (active work/stage age only). Prototype data/actions are explicitly examples. Preserve this direction while adding interactions.

One-off requests are sent to the same lead, which will orchestrate them around ongoing work. Use Message / One-off in the existing composer and Runs → Schedule for saved requests. Requests are intent, separate from execution attempts; submitting or reaching a scheduled time must never fabricate a Run or an agent acknowledgment. Default to Lead decides, with Next available and a future earliest start as optional policies. Preserve drafts and immutable submitted model/context snapshots. This original refinement did not imply additional navigation or recurrence; the November 2026 Scratchpad direction below explicitly adds both a destination and a review schedule plan.


## Scratchpad and permissions direction — November 2026

The Message / One-off control belongs at the bottom of the composer beside Send. Use quiet native type and timing selects with transparent borders and backgrounds. One-off timing and its help belong in the bottom action row with Type / Send, below the model and Permissions row, leaving the writing area clear. On narrow screens timing wraps above Type / Send within that bottom section.

Scratchpad is one blank, continuous, autosaved document after Conversation. Keep its surface free of note cards, filters, capture buttons, and schedule controls. Highlight exact passages completed by the agent in quiet green, with a non-color completion summary and links to the matching attempts. Submitted/queued/cancelled intent is not completed work. Editing source text invalidates overlapping marks; prior requests keep their original snapshots. Preserve existing notes, unfinished edits, and drafts during conversion; keep legacy records for provenance.

Settings → Scratchpad owns the review plan and local Preview triage. Default to every morning at 09:00 in the browser IANA zone, configurable by cadence, time, zone, and pause. The preview saves that plan but does not install cron or call an agent. Document passages produce lead-policy Requests with immutable source provenance; the lead owns eventual scheduling. Keep generated intent out of execution Runs and Radar until real dispatch exists. Completion highlights require the latest explicitly linked attempt to succeed, rather than deriving completion from a queued request.

Permissions remain separate from context inputs and execution mode. Preserve read-only/worktree access and add Commands, Network, and Integrations with Ask defaults. Next-send overrides can narrow project ceilings; project denials and read-only access cannot be broadened there. New requests and retries based on current defaults capture resolved permissions. Historical snapshots without permissions display Not captured, and retries based on those snapshots preserve that absence. Browser-preview values never grant real OS or provider capabilities; native execution additionally requires daemon/provider enforcement.

Make project permission scope explicit in Settings → Permissions, with descriptions and Save project permissions. The composer Permissions dialog shows saved project rules, separates next-send restrictions, and links directly to the project editor. Changing defaults affects future sends, preserving existing snapshots.

Keep conversation activity quiet: routine goal/assignment details may collapse, while approvals and review outcomes stay visible. Use one local receipt beneath the user's send instead of a second success notice. Follow new messages only near the bottom or after an own send; preserve reading position otherwise and offer Latest updates. Respect reduced motion and never invent an agent response in this disconnected preview.

## Reviews overhaul — November 2026

The user requested a total overhaul of Reviews. The current implementation separates a compact inbox from a full-width selected review desk, with stable review-ID routes. Preserve the large inspection area: no permanent queue and no default split. Preview, Changes, and Evidence have separate modes; screenshots use readable width and their own scroll region, while code and file selection stay in Changes. Back restores inbox context and row focus; Previous/Next preserves each review's view choices.

Pin review source to its run, attempt, and revision. Stale checks never certify new work. Browser-preview simulated merge requires current passing checks, a matching succeeded source, and no newer task attempt; confirmation rechecks these conditions. Native local acceptance and changes requests bind to the observed current source and never call GitHub. Missing optional screenshots remain explicit gaps without blocking text-only changes. Missing sources and unknown review IDs recover safely. Keep feedback drafts and local decision history across navigation/reload. No review action mutates execution Runs or appears in Radar.
