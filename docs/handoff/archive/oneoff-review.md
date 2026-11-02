# One-off feature review

What this change does: Adds one-off intent to the lead conversation and a Schedule view for saved requests. It keeps requests separate from execution attempts and captures the requested settings and context when sent.

Scope: Read-only source review of the new frontend feature, at single-user browser-local preview load. No browser, provider, or backend verification was performed by this reviewer.

## Must fix

1. **P2 — Saved scheduling labels omit the year** (`apps/desktop/src/scheduling.js:34`).
   - What this is: `policyLabel` supplies the requested start date in the conversation receipt, Schedule table, and selected request detail.
   - Problem: Scheduling at November 1, 2027 or November 1, 2028 at 10:00 in America/Los_Angeles produces the identical label `Nov 1, 10:00 AM GMT-7`. The only full year in the detail is the captured-at timestamp, which is a different event. A saved request cannot be reviewed unambiguously.
   - Fix: Pass `true` for the formatter's full argument for an `at` policy, or show the full scheduled date in the selected detail and a year-aware compact table label.
   - If we skip it: A user can mistake the requested year when checking future work.

2. **P2 — Returning after cancellation can lose keyboard focus** (`apps/desktop/src/Schedule.jsx:30`).
   - What this is: Back to Schedule attempts to restore focus to the previously selected request row while preserving filters.
   - Problem: Filter to Waiting, select a request, cancel it, then press Back to Schedule. The cancelled row no longer matches the filter, so the selector has no target; the focused Back button is removed with the detail. A Due filter or status search has the same problem. This follows directly from the filter and focus code; root owns runtime confirmation.
   - Fix: If the prior row is absent, focus the Schedule heading or active filter using a ref/fallback target.
   - If we skip it: Keyboard users lose their place and restart navigation from outside the list.

## Verified source behavior

- Submission does not create tasks or execution attempts and does not add unstarted intent to Radar.
- Model/effort validation happens again at submit; timing is validated at submit, including DST gaps and resolved offsets.
- Requested model, source, permissions, budget, concurrency, context roots, attachments, and ignore configuration are copied when submitted.
- Rescheduling and cancellation preserve request identity and captured settings/context. Cancelled intent cannot be rescheduled.
- Past scheduled timestamps derive Due without mutating intent or starting execution.
- A stored absolute timestamp and IANA zone keep the requested instant stable across later browser zone changes.
- Message/One-off toggle preserves the draft and controls; successful send resets to Message.
- Requests have stable deep-link IDs and Schedule filters remain separate from execution filters.

## Test evidence and limits

Ran `node --test tests/oneoff.test.mjs` using the absolute bundled Node binary: all six tests passed. They meaningfully cover date validation, a real America/Los_Angeles DST gap/overlap, capture immutability, cancel/reschedule identity, derived Due, and restore rejection/copying. They do not verify the React submit wiring, browser-local serialization, duplicate-submit handling through the composer, draft restoration, routing, focus return, or storage-failure notices; those need root's browser journey checks. No extra broad test suite is recommended for this preview.

Verdict: Fix 1 and 2 first.

Not checked: Rendered layout, keyboard runtime, browser persistence, or real orchestration. No real scheduler is part of this frontend preview.

## Resolution check

Both findings are resolved in the final source. Scheduled `policyLabel` now requests the full year, with assertions distinguishing 2027 and 2028. Schedule return focus now falls back to its focusable heading when the original row is absent. Root separately confirmed the filtered cancellation/Back journey focuses that heading in the browser.

The final narrow changes also return focus to the composer after a successful send, check new identifiers against saved requests and messages, remove redundant narrow table columns, and give selected requests the available workspace below 1180px. These retain the same request identity and scope. No remaining P1/P2 finding in this bounded source check.

Final verdict: Ship this frontend preview. Root reports the final build and all ten tests pass, with 25 rendered geometry cases and no overflow; this reviewer did not rerun that broader verification.
