# Independent Scratchpad and permissions review

What this change does: Moves Message/One-off to the composer footer, adds permission ceilings and next-send narrowing, and persists Scratchpad notes plus a configurable review plan. Preview triage creates lead-policy Requests with note provenance; it does not create execution records or run a cron.

## Must fix

1. **Blank keyboard save discards an open edit (P2).** `apps/desktop/src/Scratchpad.jsx:27` and `:60`: edit a note, clear its text, then press Cmd/Ctrl+Enter. The shortcut calls `finishEdit(note, true)` even though Save is disabled; the function deletes the draft before `editNote` rejects blank input. Guard saving blank text before removing the edit. Retain the empty draft and focus so the user can continue editing.

2. **Hourly review can reject an invisible daily-time field (P2).** `apps/desktop/src/scratchpad.js:13`, called by `Scratchpad.jsx:44`: configure daily review, clear Review time, switch to Every hour, then Save. The time field disappears but validation still rejects the now-empty time. Validate time only for daily cadence, or normalize an invalid hidden time when changing cadence. Hourly and quarter-hour cron plans must not depend on an inapplicable daily time field.

## Contract clarification sent to owner

`App.jsx:160` stores ordinary Message sends without settings/context, then clears the override. The Next-send permissions UI and contract imply captured permissions for all new sends, but only One-offs currently capture them. Either capture the resolved permission/model/context snapshot on the saved user message or explicitly limit the control's scope. This is consequential if those saved messages later become lead-agent input; the preview itself dispatches nothing.

## Verification scope

Read permissions, Scratchpad, scheduling, Schedule, App submission and persistence, run retry/settings callers, and focused tests. Promotion is atomic within the stored aggregate, de-duplicates existing note/revision provenance including cancelled Requests, and preserves independent snapshots. Legacy missing permissions stay uncaptured. No P1 found.

Meaningful remaining coverage: the actual Message submit path and RetryForm original/default permission capture are not covered by the pure-module tests. Verify those through the UI rather than adding implementation-shaped assertions.

Verdict: fix 1 and 2 first; settle the Message capture contract. Browser/runtime checks are owned by the parent; this review made no source changes.
