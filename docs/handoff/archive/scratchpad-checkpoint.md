# Scratchpad and permissions feature

User goal: put Message/One-off at the bottom, add explicit permissions, and capture Scratchpad notes for a configurable morning agent review before the lead schedules work.

- Blocking first steps. Read current App/localStorage/request boundaries and the shared routing guide. Get independent domain and Claude-through-Cursor design proposals. Keep the existing local-preview execution boundary.
- Independent workstreams. One feature owner implements coupled App, state, new UI, and focused tests. Parent owns design synthesis, docs, browser verification, and final review. A fresh reviewer examines the resulting source after the owner's release.
- Shared mutable state. App remains the single writer of the existing stored aggregate. Notes and review configuration are explicit additional fields in that same persisted object. Generated requests and note provenance update together; no second writer or timer writes execution records.
- Smallest safe decomposition. One implementation owner prevents overlap across permission capture, settings, request promotion, retry, and restore. Separate pure permissions and Scratchpad domain modules provide meaningful test boundaries without replacing the app's state architecture.

## Selected shape

Keep access as the existing edit ceiling, mode as the workflow, and add explicit command/network/integration permissions. Existing execution snapshots with absent permissions remain visibly uncaptured. New sends and generated requests capture resolved permission values. Mode never grants denied capabilities.

Scratchpad notes have stable identity, text, revision, timestamps, and an inbox/sent/archived lifecycle. Morning review defaults to 09:00 in the browser's IANA zone and is configurable. Cron is derived from supported cadence/time configuration; show it as a saved plan while runtime is disconnected. A clearly labeled Preview triage action promotes inbox notes to lead-policy requests and records note/revision provenance idempotently. No semantic agent output, active cron, fake acknowledgement, Run, or task is invented.

Use a quiet native Message type select at the bottom beside Send. Model and Permissions stay in the bottom toolbar. Optional one-off timing appears below the writing area and above this footer. Scratchpad is a simple secondary capture destination, with a jot editor/list and compact review configuration. Keep it outside Radar.

## Verification promises

Drafts and old local data survive restore. Note edits are retained, repeated triage does not duplicate a request, archived and already sent notes do not re-enter triage, and settings changes do not mutate submitted requests. Invalid cron configuration is rejected. Browser checks cover capture, configuration, permissions, promotion/deep links, reload, mode/keyboard behavior, and compact widths. No OS permission, provider, or desktop automation changes are part of this UI feature.

## Completed evidence

Bottom native type control, requested permissions, morning-configurable Scratchpad, and linked manual promotion are implemented. Parent browser checks covered all promised journeys, including cancelled-origin retention and project-versus-composer snapshot separation. All 19 focused tests and the final production build pass. Twenty responsive geometry cases pass at five widths; paired desktop/phone captures and the final report are in `outputs/software-factory/docs/design/compact-preview/scratchpad-screenshots/` and `apps/desktop/design-qa.md`. Both P2 review findings were fixed and final independent source review reports no actionable P1/P2 issue. Four protected template hashes are unchanged, whitespace and 58 documentation links pass. QA tabs are closed; original user data and Reviews tab are preserved. Live cron, agent interpretation, scheduling, dispatch, and permission enforcement remain future runtime work.
