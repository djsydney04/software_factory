# Software Factory visual direction

Status: approved compact frontend implemented in both the browser design preview and the native local alpha. Browser React/Vite interactions use fixtures and local simulations. Tauri live mode now reads real Rust/SQLite daemon records and supports a bounded Codex lead/worker/check/Review workflow; it never falls back to fixtures. See the [runtime checkpoint](../runtime/IMPLEMENTATION.md) and [integration trial](../runtime/INTEGRATION-TRIAL.md) for connected capabilities and evidence. The earlier compact pass recorded 35 passing responsive view and width combinations; the [QA record](../../apps/desktop/design-qa.md) owns later preview refinement results, with native desktop/phone captures under [runtime evidence](../runtime/evidence/).

The latest Reviews request replaces its queue/Split layout with a compact inbox and a full-width review desk. Scratchpad remains a blank document with completed source passages highlighted and its review controls in Settings. Preserve the more compact workspace, bottom composer type control, explicit requested permissions, and configurable morning review. Retain additional run views, scoped model selection, configuration, file settings, and corrected proportions. Claude through Cursor CLI implemented the original compact preview from the earlier redesign and refined Sundial source; subsequent local refinements extend it.

[Run the preview](../getting-started.md) · [Current handoff](compact-preview/README.md) · [Interaction contract](compact-preview/interaction-contract.md) · [Scratchpad and permissions](compact-preview/scratchpad-permissions-contract.md) · [Reviews contract](compact-preview/reviews-interaction-contract.md).

The hierarchy below records the approved preview interactions. Live mode reuses its proportions and surfaces, but has local review acceptance/changes requests rather than simulated merge. It exposes actual attempt authorization and source freshness. File editing, attachments, rendered run previews and recurring Scratchpad triage remain explicitly unavailable in the local alpha; the blank Scratchpad and saved configurable 09:00 plan remain local client features.

## Current hierarchy

- **Conversation:** one main-agent thread and one composer. Message / One-off type, model, and Permissions controls sit at the bottom. Results link to Reviews without embedding the review workspace.
- **Scratchpad:** one continuous autosaved document with a minimal title and save state. Exact source passages highlight green only when their latest linked Run succeeds; submitted or queued intent stays unhighlighted. Original note records remain internal provenance. **Settings → Scratchpad** owns the daily 09:00 review plan, cadence, time, zone, pause, derived cron, and manual Preview triage. Triage saves lead-policy Requests with source-range links and project snapshots; no background agent or cron runs.
- **Runs:** List, Timeline, and Schedule; selected-run History, Activity, Files, Checks, Artifacts, and immutable Settings. Saved requests remain distinct from execution attempts.
- **Progress:** Graph, Board and Radar show work and ownership. Radar contains only Plan, Build, Check and Waiting, with no changes, PRs, previews, diffs or merge controls.
- **Reviews:** a compact searchable inbox opens a full-width desk with Preview, Changes, and Evidence. No persistent queue or Split view. Evidence stays bound to the exact Run/attempt/revision; feedback drafts and local decisions persist. Simulated merge requires a successful latest task attempt and current passing checks. Actions make no GitHub call.
- **Files and Settings:** secondary utilities for context inputs, illustrative edits, access scope, requested command/network/integration permissions, and future-run defaults. Next-send permission overrides can narrow project ceilings; legacy snapshots remain Not captured. Preview values grant no real OS or provider capabilities.

A project switcher replaces the large wordmark and repeated project list. The compact desktop shell uses 168px navigation, a 40px header, 16px workspace gutters, 14px body text, and 13px metadata, with some 12px labels. Review the seven views at 1440, 1280, 1024, 768, and 390 CSS px. The [QA record](../../apps/desktop/design-qa.md) owns rendered geometry and interaction results.

The Sundial radar uses engraved paper-like rings and tapering delegation shadows. Angle shows current stage; radius shows time in stage. Only active work and retained blocked inputs appear. Reviews and completed changes stay outside the radar. Browser-preview data is illustrative; native live mode derives work and lead state from daemon records.

## Source and history

- [Frontend execution provenance](compact-preview/provenance.json), [tokens](compact-preview/tokens.json), and [layout declarations](compact-preview/layout-budgets.json).
- [Reviews overhaul and implementation choices](compact-preview/reviews-overhaul-brief.md), [session provenance](compact-preview/reviews-overhaul-provenance.json), and [interaction contract](compact-preview/reviews-interaction-contract.md).
- [Blank-document consultation and implementation choices](compact-preview/document-claude-brief.md) with [session provenance](compact-preview/document-provenance.json).
- [Earlier raster screens](claude-redesign/README.md) and [Claude's original brief](claude-redesign/claude-design-brief.md), with the initial Radar proposal superseded by [its refinement](claude-redesign/radar-refinement.md).
- [Raster provenance](claude-redesign/provenance.json) and [render prompts](claude-redesign/render-prompts.md).
- [Earlier notes](pre-claude-notes.md), superseded by the current direction. The earlier `compact-*.png` images put reviews inside monitoring views and must not be used as the build target.

Design and frontend: Claude Opus via Cursor CLI, followed by parent review fixes. Earlier raster rendering: built-in Image Gen. The Green Room review example uses a labeled example Collection screenshot; its presence does not establish a real PR or deployment.

## Additional visual explorations

The remote history preserves [Soft Charcoal](soft-charcoal/README.md), [Charcoal Desk](charcoal-desk/README.md) and [Operating Desk](operating-desk/README.md), including their prompts, images and execution provenance. Soft Charcoal explores a workspace overview, project-scoped destinations and model pickers, softer dark surfaces and run drilldown. These proposals are comparison material; this alpha retains the compact-preview interface and current interaction contracts above. The [reusable design suite](../../design-skills/README.md) is also retained.
