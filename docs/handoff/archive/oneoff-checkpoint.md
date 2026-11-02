# One-off feature checkpoint

The user wants bounded one-off requests sent through the main agent and scheduled into ongoing project work. This pass improves the existing frontend preview.

- Blocking first steps. Read the existing composer, run identity, persistence, and the shared routing guide. Separate request intent from execution attempts. Claude via Cursor and an independent contract reviewer agree on that boundary.
- Independent workstreams. One code owner updates the coupled composer, scheduling UI, state helpers, and focused tests. Root owns docs and rendered QA. A fresh reviewer checks the finished code.
- Shared mutable state. Existing app source has one writer. Preview localStorage remains browser-local state, not a scheduler service.
- Smallest safe decomposition. One implementation owner is best because submission, captured settings, persistence, and navigation must change together. No backend or new dependency.

Data shape. OneOffRequest stores id, text, createdAt, policy lead/next/at, optional scheduledAt epoch, captured IANA zone, status pending/scheduled/cancelled, and immutable requested settings/context. Run still means one actual execution attempt. Requests never enter Radar or observed run Timeline.

Alternatives. Extending Run with scheduled status would be fewer edits, but would falsely create an attempt before dispatch and break existing evidence rules. A separate request array keeps intent and execution distinct. Keep this small collection in app state and one focused scheduling module.

Verification. Test date validation, DST round trips, immutable captures, derived due state, and cancellation/rescheduling. Browser-test the full submit-to-Schedule journey, refresh, model-scope changes, no extra worker attempt, and narrow controls. Keep sample data honest.

Delivery. Verified local preview only. No external PR, commit, push, deployment, production agent execution, or automation is requested.
