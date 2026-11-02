# One-off request contract

The lead remains the single conversation partner. A **request** is user intent; a **Run** remains one execution attempt for a task. Submitting intent must not fabricate a running or queued provider attempt.

## Smallest surface

Add a compact composer switch: **Message · One-off**. Message preserves the lead conversation. One-off adds one adjacent **When** control: **Lead decides** (default), **Next available**, **Schedule at…**. Reveal a native local datetime field only for Schedule at. Keep the existing model/settings/context controls; skip priority because there is no ordering contract to make it meaningful.

Send stays **Send to Factory**. The receipt appears in the same conversation, links to the request, and says “Saved locally for the lead” in this preview. No extra top-level destination or second chat. Runs gains **Schedule** alongside List/Timeline, showing requests rather than mixing them into the attempt table. A compact “New one-off” button opens/focuses the conversation composer with its draft intact.

Schedule rows show title, policy or resolved time, status, and Cancel. Selecting a row opens a request detail with original text, attachments, immutable submitted settings, requested policy, and eventual linked tasks/runs. Do not place requests without active attempts in Radar.

## State and scope

Persist a separate `requests` array: `id`, `text`, `createdAt` (epoch milliseconds), `policy` (`lead`, `next`, `at`), optional `scheduledAt` (epoch milliseconds), `timeZone` (captured IANA zone), `status` (`pending`, `scheduled`, `cancelled`), `cancelledAt`, frozen `settings/context` snapshot, and linked task/run IDs when actually present. Keep request IDs distinct from run IDs. Do not create an empty task solely to represent an unplanned request.

Lead decides means pending planning; Next available means the lead should fit it after current capacity/dependencies, not interrupt existing work. Schedule at is an earliest start constraint, not a promised exact start. These are requested policies, not claims that this prototype has scheduled a worker. Display “Due · awaiting lead” as a derived label when a saved scheduled timestamp passes. No timer should launch an attempt or mutate status in this frontend.

Changing project defaults later leaves the submitted snapshot unchanged. When a real orchestrator eventually creates tasks/attempts, store the originating request ID and a separately captured effective execution snapshot; report any resolution that differs from submitted intent. Cancelling a pending request prevents future dispatch; it never cancels already-created runs implicitly. Hide or disable cancellation after real dispatch and link to each run’s own action.

## Validation and persistence

Reject blank text, unavailable model combinations, missing/invalid datetime, or a scheduled time not in the future at submission. Convert local input to an absolute timestamp, reject nonexistent DST times by local-component round-trip, and show the resolved date, offset, and zone before Send; ambiguous times must expose the resolved offset. Persist original zone so travel/reload cannot silently reinterpret intent. Preserve draft, policy, datetime and overrides across navigation/reload. Prevent double submit; announce receipt/error via existing Notice. If storage fails, show that state rather than claiming the request survives reload.

Touchpoints: `App.jsx` composer/store/submit/receipt, `Runs.jsx` Schedule/list/detail, `data.js` pure validation/restore helpers, existing CSS/tabs. Keep real scheduling separate from fixed sample `NOW`/minute-based attempts.

## Delivery scope

This pass extends the frontend preview. The future supervisor owns planning, capacity, dependencies, provider execution, recovery, and durable dispatch. No browser timer claims to run an agent.
