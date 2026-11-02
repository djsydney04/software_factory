# Compact desktop preview

Status: clickable React/Vite design preview implemented, November 1, 2026, including one-off requests, requested permissions, and a blank-document Scratchpad with its configurable morning plan in Settings, plus a compact Reviews inbox and full-width review desk. The preview uses localStorage sample state; a Rust engine, TUI, Tauri native build, and live providers/GitHub remain planned. Validation commands and scoped results are in the [run instructions](../../getting-started.md) and [QA record](../../../apps/desktop/design-qa.md).

[Run instructions](../../getting-started.md) · [Local preview](http://127.0.0.1:5173).

## Current direction and provenance

The latest Reviews request replaces its queue/Split layout with a compact inbox and a full-width desk. Scratchpad remains a blank continuous document, completed agent work highlights its exact source text, and its review settings live in Settings. Preserve bottom composer type controls, explicit permissions, and the configurable morning review plan. Retain the compact proportions, run views, scoped model/configuration controls, one main-agent Conversation, and separate Progress and Reviews.

Claude Opus 5.5 authored the preview through Cursor CLI. Requested model: `claude-opus-5-5-high`; session: `5fba86e2-5eeb-4a5a-8ec4-191d290c58c5`. Auto-review and sandbox were enabled. Its terminal result was success after 1282 seconds, followed by parent review fixes. See the [provenance record](provenance.json); authoring success is separate from frontend and runtime verification.

The one-off refinement also used Claude through Cursor CLI in plan mode. See its [bounded design brief](oneoff-claude-brief.md) and [session provenance](oneoff-provenance.json). That consultation informed the local implementation; it is not evidence of live agent orchestration.

The earlier note-based Scratchpad refinement used a bounded Claude consultation through Cursor CLI; its [brief](scratchpad-claude-brief.md) and [provenance](scratchpad-provenance.json) remain history. A later [blank-document consultation](document-claude-brief.md) and [session provenance](document-provenance.json) record the current direction and local implementation choices. The [interaction contract](scratchpad-permissions-contract.md) distinguishes the saved preview from the future scheduler and agent runtime.

The Reviews overhaul used another read-only Claude consultation through Cursor CLI. Its [brief](reviews-overhaul-brief.md), [provenance](reviews-overhaul-provenance.json), and [interaction contract](reviews-interaction-contract.md) record the selected inbox/desk shape and the implementation owner’s simpler navigation, image scaling, and local evidence rules.

Use the [current screen and mock index](../../handoff/DESIGN-INDEX.md) to find the latest implemented references. Earlier raster sources are [Conversation](../claude-redesign/conversation.png), [Radar](../claude-redesign/radar.png), and [Reviews](../claude-redesign/reviews.png), with the authoritative [Sundial refinement](../claude-redesign/radar-refinement.md). The compact geometry and current interaction contracts supersede those older screenshots' layout details.

## Layout and surfaces

The compact desktop shell uses a 168px navigation rail, 40px header, 16px workspace padding, 14px body text, and 13px metadata, with some 12px labels. Responsive QA covers 1440, 1280, 1024, 768, and 390 CSS px. Secondary panes collapse before the primary work surface; preserve the selected run and drafts across navigation.

Runs offers List, Timeline, and Schedule. Run detail provides History plus Activity, Files, Checks, Artifacts, and Settings. Schedule holds requests with search, status filters, selected details, rescheduling, and cancellation. Files & context holds inputs; Settings → Permissions holds edit access and command/network/integration ceilings. Reviews owns an inbox and a full-width desk for Preview, Changes, Evidence, and local review actions. Its current desk replaces the earlier permanent queue and Split layout. Sundial shows active Plan, Build, Check, and Waiting work only; reviews, completed changes, and requests without an active attempt stay outside the dial.

## One-off requests

Conversation has a native **Message / One-off** type select beside Send at the bottom. One-off reveals a borderless **When** select in that bottom action section, below model and Permissions: **Lead decides**, **Next available**, or **Schedule at…**. Timing wraps above Type / Send on narrow screens. Only Schedule at reveals a local datetime input, with the resolved IANA zone and offset. A future time is an earliest-start constraint; Next available waits for current work and dependencies.

Routine goal/assignment updates collapse behind a native disclosure; approvals and review outcomes remain visible. Own sends have one receipt rather than a duplicate success notice. New updates preserve older reading position and offer Latest updates; near-bottom activity and own sends follow the end, respecting reduced motion. The Permissions dialog separates saved project rules from next-send restrictions and links directly to Settings → Permissions.

A request stores intent separately from a Run. Sending captures requested model settings, permissions, and file context, links the receipt to Schedule, and returns the composer to Message. Rescheduling changes timing only; cancellation preserves the record. Drafts and requests survive navigation and reload in this browser. Once a scheduled timestamp passes, **Due · awaiting lead** is derived without dispatching work. Both Message and One-off leave execution attempts unchanged in this preview.

## Scratchpad and permissions

Scratchpad is one continuous autosaved document, with no note cards, filters, or capture controls. Its title, local save state, and completion evidence stay quiet around the writing surface. Exact source passages highlight green only when their request's latest linked Run succeeds. Untouched ranges shift with surrounding edits; overlapping or ambiguous source anchors lose their highlight rather than certify changed text. Queued, cancelled, and merely submitted intent never counts as done. Completion details link to the supporting Run.

**Settings → Scratchpad** (`#/settings/scratchpad`) owns the review plan and manual **Preview triage**. The plan defaults to daily at **09:00** in the browser's IANA zone, with configurable cadence, time, zone, pause, and derived cron. Blank lines separate preview passages; triage saves new unlinked text as **Lead decides** Requests using current project settings and context. It preserves document text and captures immutable source provenance. Submitted requests do not change when the document is edited, and cancellation does not replay the original source. Legacy visible notes, unfinished edits, and drafts migrate once; original records remain provenance. No cron, agent interpretation, lead scheduling, or dispatch runs in this preview.

Permissions separate Agent edits from Commands, Network, and Integrations. Project defaults set ceilings; next-send overrides can narrow them. New messages, requests, and retries explicitly based on current defaults capture resolved settings. Legacy snapshots remain **Not captured**. Preview values grant no real OS or provider access.

## Configuration and run identity

| Scope | Contract |
| --- | --- |
| Project defaults | Apply to future submissions. Changing them leaves existing runs intact. |
| Next send override | Applies to one submission; show inherited versus overridden values and a way to use the project default. |
| Submitted request | Immutable requested settings and file context. Rescheduling leaves this snapshot intact. |
| Selected run settings | Immutable effective snapshot, including provider/model, effort, access, and context revision. |
| Retry | Creates a new stable run ID for the same task. Start from the prior snapshot; choosing current defaults is explicit. |

A run is one execution attempt; **Attempt N** is its display label, not another stored entity. History changes every detail tab to the selected run together. Sample models, checks, artifacts, settings changes, and execution controls must remain identifiable as preview behavior.

## Reviews

`#/reviews` is a compact searchable inbox with status filtering and linked rows. Selecting one opens its desk at `#/reviews/RV-24`; Back restores the inbox context and row focus. Previous/Next traverse known reviews without automatically advancing after a decision. Historical merged strings remain read-only history.

The desk separates Preview, Changes, and Evidence into one work surface at a time. Readable-width screenshots scroll vertically; native 100% zoom pans within the image scroller. Only Changes contains a file picker. Evidence shows the exact pinned Run, attempt, revision, current and stale checks, artifact states, limitations, and local decision history. Missing imagery and source records have explicit recovery states.

Feedback drafts persist per review. Request changes and Simulate merge capture local source-bound decisions; confirmation rechecks unchanged source and eligibility. Merge simulation requires a Ready review, a succeeded latest task attempt, well-formed evidence, and all current-revision checks passing. Missing optional imagery alone does not block text-only work. These actions never call GitHub or dispatch an agent. See the [Reviews contract](reviews-interaction-contract.md) for the complete boundary.

## Review handoff

Use the [run interaction contract](interaction-contract.md), [one-off contract](oneoff-interaction-contract.md), [Scratchpad and permissions contract](scratchpad-permissions-contract.md), [Reviews contract](reviews-interaction-contract.md), [semantic tokens and contrast pairs](tokens.json), and [layout declarations](layout-budgets.json). Declared arithmetic and color checks do not prove the rendered layout, keyboard behavior, or actual computed contrast.

The [desktop QA record](../../../apps/desktop/design-qa.md) records the build, browser, responsive, and interaction results and their limits. The preceding compact-preview pass covers 35 view/width combinations and 85 rendered color pairs. Later refinements have their own scoped evidence in that record.
