# Scratchpad, permissions, and composer contract

The existing app stores project settings, composer drafts, immutable one-off Requests, and sample Run attempts in `App.jsx`. `scheduling.js` owns request creation, timing, cancellation, and restoration. `Runs.jsx` owns execution views; `Schedule.jsx` owns pending intent. Preserve that distinction.

## How it works

Put Message / One-off in the bottom composer action bar, adjacent to attachments/model controls. Keep the text area first. When One-off is active, reveal timing immediately above the bottom bar, below the text; do not move the mode switch back to the top. At narrow widths give timing its own row, retain a visible send action, and collapse model metadata before shrinking text.

Scratchpad holds unscheduled notes. A configurable morning triage gathers unprocessed note revisions, proposes work for the lead, and links each source note to its resulting Request. The lead owns ordering and eventual execution. Scratchpad triage must not create a Run, Radar mark, provider acknowledgment, or promised start time. In this local preview, configure the intended schedule and provide a clearly labeled **Preview triage** action; show “Not connected” beside automation state. Merely passing a scheduled time does not execute code.

## Recommended minimal shape

Keep Scratchpad beside the conversation as a secondary view, sharing its existing main surface. Use a quick-entry textarea and compact saved-note list, with an optional note inspector. Avoid another dashboard, calendar, or primary navigation hierarchy. A saved note is one proposed task; do not split prose heuristically and claim an agent understood it.

* Note: `{id, text, revision, createdAt, updatedAt, triagedRevision, requestId}`. New notes start revision 1. Editing a processed note increments revision and makes that revision eligible again; the old Request remains immutable. Removing a note never deletes its Request.
* Schedule: `{enabled, cadence:'daily'|'weekdays', localTime:'08:00', timeZone:IANA, lastSlot:null}`. Default daily at 08:00 in the captured browser zone. Configurable time, day preset, zone, and pause. Display human wording and derived cron text together; no arbitrary cron parser is needed for this first version. The eventual supervisor, not the browser, owns durable cron execution and restart catch-up policy.
* Triage records: `{id, source:'manual-preview'|'scheduled', slot, at, noteRevisions, requestIds}`. Preview triage consumes only revisions not already triaged. Each generated Request uses `policy:'lead'` and `origin:{kind:'scratchpad',noteId,noteRevision,triageId}`. Use `(noteId,revision)` as the idempotence identity; repeated clicks/reload cannot duplicate Requests. Capture settings/context at triage, with that capture time explicitly shown.

Permissions should complement current settings: keep `access` as the read-only/worktree edit ceiling, retain `mode` as plan/ask/auto workflow, and add `permissions:{commands:'deny'|'ask'|'allow',network:'deny'|'ask'|'allow'}`. Default both to ask. Deny wins; plan-only never performs actions; auto never overrides a denied capability. Configure project permissions in a dedicated Settings category and show their scope from a compact bottom composer control. New Requests and retries explicitly based on current defaults deep-copy permissions. Existing runs/requests lacking this field show “Not captured”; never merge live defaults into old snapshots. Original-basis retries preserve missing/old permissions rather than widening them silently.

## Two viable structures

1. **Recommended:** add pure `scratchpad.js` domain helpers, `Scratchpad.jsx` view, and `permissions.js` normalization/labels. `App.jsx` owns persisted state and cross-object submission; `data.js` defaults, `scheduling.js` origin/permission validation, `Runs.jsx`/`Schedule.jsx` snapshot display, `styles.css` compact layout. Focused tests cover idempotence, restoration, immutable snapshots, and next occurrence.
2. A unified reducer for notes, triage batches, Requests, and Runs gives atomic transitions and durable service migration, but rewrites working state ownership. Save that for connecting the supervisor; the current bounded feature does not justify it.

No actual scheduler, agent understanding, or security enforcement has been checked or implemented by this read-only proposal. The preview must state those limits plainly.
